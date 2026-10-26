//! The settings window and the commands it calls.
//!
//! Privacy: a key comes in through [`save_api_key`] and goes straight to the
//! keyring. No command ever sends a key back; the window only learns whether
//! one is saved.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

mod reveal;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tauri::window::Color;
use tauri::{
    AppHandle, Emitter, Manager, State, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

use crate::autostart::{self, TauriAutostart};
use crate::llm;
use crate::settings::{ApiKey, AppSettings, ProviderKind, SaveKeyError, Settings};
use crate::shortcut::{self, Hotkeys, TauriHotkeys};
use reveal::Reveal;

/// Label of the settings window. The `settings` capability is granted to it.
pub const LABEL: &str = "settings";

/// The page background (`--cream` in `src/styles/tokens.css`), painted natively
/// so the window never flashes white before the page loads.
const BACKGROUND: Color = Color(0xff, 0xff, 0xeb, 0xff);

/// The title bar matches the page: `--cream` behind `--ink` text.
const TITLE_BAR: u32 = 0xff_ff_eb;
const TITLE_TEXT: u32 = 0x1a_1a_1a;

/// Asks an open settings window to scroll to a section. The payload is the
/// section's id in `settings.html`.
const SCROLL_EVENT: &str = "settings-scroll-to";

/// A section the settings window can open scrolled to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Version and updates.
    Updates,
}

impl Section {
    /// The section's element id in `settings.html`.
    const fn id(self) -> &'static str {
        match self {
            Self::Updates => "updates",
        }
    }
}

/// The page to load: the settings page, with the section as its fragment so
/// a new window starts there.
fn page(section: Option<Section>) -> String {
    match section {
        Some(section) => format!("settings.html#{}", section.id()),
        None => "settings.html".to_owned(),
    }
}

/// Opens the settings window scrolled to `section`, or brings an open one to
/// the front and scrolls it there. `None` leaves an open window where it is.
///
/// # Errors
///
/// Returns an error if the window can't be created or shown.
pub fn open_at(app: &AppHandle, section: Option<Section>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) {
        // One still being drawn shows itself once ready; showing it now
        // would show it blank.
        if !reveal_state().is_hidden() {
            window.unminimize()?;
            window.show()?;
            window.set_focus()?;
        }
        if let Some(section) = section {
            window.emit_to(LABEL, SCROLL_EVENT, section.id())?;
        }
        return Ok(());
    }
    // Hidden until its page is drawn (see `settings_ready`), so it never
    // shows an empty frame. Counted as hidden before it's built, so even a
    // page that is ready at once is shown.
    let opening = reveal_state().open();
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App(page(section).into()))
        .title(format!("{} Settings", app.package_info().name))
        .inner_size(650.0, 720.0)
        .min_inner_size(380.0, 560.0)
        .theme(Some(Theme::Light))
        .background_color(BACKGROUND)
        .maximizable(false)
        .center()
        .visible(false)
        .focused(true)
        .build();
    let window = built.inspect_err(|_| reveal_state().close(opening))?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            reveal_state().close(opening);
        }
        track_focus(&handle, event);
    });
    #[cfg(windows)]
    if let Ok(hwnd) = window.hwnd() {
        crate::win32::color_title_bar(hwnd, TITLE_BAR, TITLE_TEXT);
    }
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(READY_TIMEOUT).await;
        if reveal_state().time_out(opening) {
            tracing::warn!("the settings page didn't get ready in time; showing it anyway");
            show_window(&handle);
        }
    });
    Ok(())
}

/// How long a new settings window waits for its page before it shows anyway,
/// so Settings never seems broken.
const READY_TIMEOUT: Duration = Duration::from_secs(2);

/// Whether the settings window is still hidden, waiting for its page.
static REVEAL: Mutex<Reveal> = Mutex::new(Reveal::new());

fn reveal_state() -> MutexGuard<'static, Reveal> {
    REVEAL.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Shows the settings window in front, with keyboard focus.
fn show_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    if window.show().and_then(|()| window.set_focus()).is_err() {
        tracing::warn!("the settings window could not be shown");
    }
}

/// The settings page is drawn: its main form, every section, its fonts, and
/// scrolled to the section it opened at. Shows the window, unless the
/// timeout already did. The page can't show or focus windows itself.
#[tauri::command]
pub fn settings_ready(app: AppHandle) {
    if reveal_state().ready() {
        show_window(&app);
    }
}

/// [`open_at`] from a spawned task. Use this from callbacks that run inside
/// the event loop (the single-instance callback, menu events, notification
/// clicks): building a window there deadlocks on Windows.
pub fn open_at_later(app: &AppHandle, section: Option<Section>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if open_at(&app, section).is_err() {
            tracing::warn!("the settings window could not open");
        }
    });
}

/// A command failed. Serialized as a short message the window shows as is.
/// Never carries a key or the settings file's contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The settings file couldn't be written.
    #[error("Couldn't save your settings. Check that the app data folder is writable.")]
    SaveSettings,
    /// The Windows Credential Manager failed.
    #[error("Couldn't reach Windows Credential Manager. Try again.")]
    KeyStore,
    /// An empty key was submitted.
    #[error("Enter an API key first.")]
    BlankKey,
    /// Windows refused to change its startup list.
    #[error("Couldn't change the Windows startup setting. Try again.")]
    Autostart,
}

/// Why [`set_shortcut`] didn't change the shortcut. The previous one still
/// works. Unlike [`CommandError`], it reaches the window as a reason, such as
/// `"noCtrlOrAlt"`, and Settings words it: Rust judges, the page explains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SetShortcutError {
    /// The keys don't make a shortcut.
    NotAShortcut,
    /// Neither Ctrl nor Alt is held.
    NoCtrlOrAlt,
    /// Win is held.
    Win,
    /// It is Ctrl+C or Ctrl+V, which the app presses itself.
    CopyPaste,
    /// Windows refused it, almost always because another app uses it.
    Taken,
    /// The settings file couldn't be written.
    Save,
}

impl From<shortcut::ChangeError> for SetShortcutError {
    fn from(error: shortcut::ChangeError) -> Self {
        use shortcut::{ChangeError, Refusal};
        match error {
            ChangeError::Refused(Refusal::NotAShortcut) => Self::NotAShortcut,
            ChangeError::Refused(Refusal::NoCtrlOrAlt) => Self::NoCtrlOrAlt,
            ChangeError::Refused(Refusal::Win) => Self::Win,
            ChangeError::Refused(Refusal::CopyPaste) => Self::CopyPaste,
            ChangeError::Taken => Self::Taken,
            ChangeError::Save => Self::Save,
        }
    }
}

impl From<autostart::ChangeError> for CommandError {
    fn from(error: autostart::ChangeError) -> Self {
        match error {
            autostart::ChangeError::Autostart(_) => Self::Autostart,
            autostart::ChangeError::Save => Self::SaveSettings,
        }
    }
}

serialize_as_message!(CommandError);

impl From<SaveKeyError> for CommandError {
    fn from(error: SaveKeyError) -> Self {
        match error {
            SaveKeyError::Blank => Self::BlankKey,
            SaveKeyError::Store(_) => Self::KeyStore,
        }
    }
}

/// The current settings. Holds no key.
#[tauri::command(async)]
pub fn get_settings(settings: State<'_, Arc<AppSettings>>) -> Settings {
    settings.settings()
}

/// Saves the provider, model and Custom base URL. Other settings in
/// `new_settings` are ignored: the shortcut and start-with-Windows have their
/// own commands, because they must reach Windows first.
///
/// # Errors
///
/// Returns [`CommandError::SaveSettings`] if the file can't be written.
#[tauri::command(async)]
pub fn save_settings(
    settings: State<'_, Arc<AppSettings>>,
    new_settings: Settings,
) -> Result<(), CommandError> {
    settings
        .save_form(new_settings)
        .map_err(|_| CommandError::SaveSettings)
}

/// Whether `provider` has a saved key. The key itself never leaves Rust.
#[tauri::command(async)]
pub fn is_key_saved(settings: State<'_, Arc<AppSettings>>, provider: ProviderKind) -> bool {
    settings.is_key_saved(provider)
}

/// Saves `key` for `provider` in the Windows Credential Manager.
///
/// # Errors
///
/// Returns [`CommandError::BlankKey`] for an empty key and
/// [`CommandError::KeyStore`] if the Credential Manager fails.
#[tauri::command(async)]
pub fn save_api_key(
    settings: State<'_, Arc<AppSettings>>,
    provider: ProviderKind,
    key: String,
) -> Result<(), CommandError> {
    Ok(settings.set_key(provider, &ApiKey::new(key))?)
}

/// The Suggested Model names for every provider, fastest first, for the
/// model picker. Built in, so this never touches the network.
#[tauri::command]
pub fn suggested_models() -> HashMap<ProviderKind, Vec<&'static str>> {
    ProviderKind::ALL
        .into_iter()
        .map(|provider| {
            let names = llm::suggested_models(provider)
                .iter()
                .map(|model| model.name)
                .collect();
            (provider, names)
        })
        .collect()
}

/// Removes `provider`'s key from the Windows Credential Manager.
///
/// # Errors
///
/// Returns [`CommandError::KeyStore`] if the Credential Manager fails.
#[tauri::command(async)]
pub fn clear_api_key(
    settings: State<'_, Arc<AppSettings>>,
    provider: ProviderKind,
) -> Result<(), CommandError> {
    settings
        .clear_key(provider)
        .map_err(|_| CommandError::KeyStore)
}

/// The saved shortcut's pause while Settings records. It also serializes
/// every shortcut change (see [`shortcut::Pause`]).
static SHORTCUT_PAUSE: shortcut::Pause = shortcut::Pause::new();

/// Pauses the saved shortcut while Settings records a new one, so pressing it
/// reaches the field instead of starting a rewrite. Takes effect only while
/// the window has focus, so a pause that arrives late can't outlast it.
#[tauri::command(async)]
pub fn pause_shortcut(app: AppHandle) {
    request_pause(&app, true);
}

/// Brings back the saved shortcut when recording ends. Does nothing if it
/// already works.
#[tauri::command(async)]
pub fn resume_shortcut(app: AppHandle) {
    request_pause(&app, false);
}

fn request_pause(app: &AppHandle, recording: bool) {
    let Some(hotkeys) = app.try_state::<TauriHotkeys>() else {
        return;
    };
    let settled = SHORTCUT_PAUSE.request(hotkeys.inner(), || saved_shortcut(app), recording);
    report_lost_shortcut(app, settled);
}

/// Tells Rust's side of the pause that the settings window gained or lost
/// focus, or closed, then settles the shortcut to match, so it can't stay
/// paused once the window loses focus or closes, whatever Settings sends.
///
/// Losing focus still counts, not only closing: hiding or minimizing the
/// window has no event of its own but takes its focus. Regaining focus
/// pauses again if the field still records. On Windows, Tauri reports these
/// from the webview's own focus, so they track the page, not the frame.
fn track_focus(app: &AppHandle, event: &WindowEvent) {
    match event {
        WindowEvent::Focused(focused) => SHORTCUT_PAUSE.set_focused(*focused),
        WindowEvent::Destroyed => SHORTCUT_PAUSE.close(),
        _ => return,
    }
    // Off the main thread: the plugin runs each call there and blocks until
    // it's done, so window events, which run there, must not wait for it.
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(hotkeys) = app.try_state::<TauriHotkeys>() else {
            return;
        };
        let settled = SHORTCUT_PAUSE.settle(hotkeys.inner(), || saved_shortcut(&app));
        report_lost_shortcut(&app, settled);
    });
}

/// The saved shortcut, or nothing before the settings are loaded.
fn saved_shortcut(app: &AppHandle) -> String {
    app.try_state::<Arc<AppSettings>>()
        .map(|settings| settings.settings().shortcut)
        .unwrap_or_default()
}

/// If the paused shortcut couldn't come back, says so with a tray
/// notification, as when it fails at startup.
fn report_lost_shortcut(app: &AppHandle, settled: Result<(), shortcut::ShortcutError>) {
    if let Err(error) = settled {
        tracing::warn!("the paused shortcut couldn't be registered again");
        crate::notify_shortcut_failed(app, error);
    }
}

/// Changes the rewrite shortcut. The new one works right away and is saved;
/// if Windows refuses it, the previous one keeps working.
///
/// # Errors
///
/// Returns the [`SetShortcutError`] that says why: a rule for new shortcuts
/// is broken, another app holds it, or it can't be saved.
#[tauri::command(async)]
pub fn set_shortcut(
    settings: State<'_, Arc<AppSettings>>,
    hotkeys: State<'_, TauriHotkeys>,
    shortcut: String,
) -> Result<(), SetShortcutError> {
    SHORTCUT_PAUSE.change(
        hotkeys.inner(),
        || settings.settings().shortcut,
        &shortcut,
        |new| settings.modify(|s| new.clone_into(&mut s.shortcut)),
    )?;
    Ok(())
}

/// Whether the saved shortcut is registered, so Settings can warn when it
/// failed at startup.
#[tauri::command(async)]
pub fn is_shortcut_working(
    settings: State<'_, Arc<AppSettings>>,
    hotkeys: State<'_, TauriHotkeys>,
) -> bool {
    shortcut::parse(&settings.settings().shortcut).is_ok_and(|saved| hotkeys.is_registered(saved))
}

/// The default shortcut, for "Reset to default" to set through
/// [`set_shortcut`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultShortcut {
    /// As Rust parses it, such as `Ctrl+Shift+Space`.
    shortcut: &'static str,
    /// Whether the saved shortcut is already the default.
    is_saved: bool,
}

/// The default shortcut and whether it is the saved one, so Settings can
/// offer to reset it.
#[tauri::command(async)]
pub fn default_shortcut(settings: State<'_, Arc<AppSettings>>) -> DefaultShortcut {
    DefaultShortcut {
        shortcut: shortcut::DEFAULT_SHORTCUT,
        is_saved: shortcut::is_default(&settings.settings().shortcut),
    }
}

/// Turns start-with-Windows on or off.
///
/// # Errors
///
/// Returns [`CommandError::Autostart`] if Windows refuses and
/// [`CommandError::SaveSettings`] if the setting can't be saved.
#[tauri::command(async)]
pub fn set_start_with_windows(
    app: AppHandle,
    settings: State<'_, Arc<AppSettings>>,
    enabled: bool,
) -> Result<(), CommandError> {
    autostart::change(&TauriAutostart(app), enabled, |enabled| {
        settings.modify(|s| s.start_with_windows = enabled)
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CommandError, DefaultShortcut, Section, SetShortcutError, page};

    #[test]
    fn the_window_opens_at_the_top_by_default() {
        assert_eq!(page(None), "settings.html");
    }

    #[test]
    fn the_window_can_open_at_the_updates_section() {
        assert_eq!(page(Some(Section::Updates)), "settings.html#updates");
    }

    #[test]
    fn an_error_reaches_the_window_as_its_message() {
        let json = serde_json::to_string(&CommandError::BlankKey).unwrap();

        assert_eq!(json, r#""Enter an API key first.""#);
    }

    /// `DefaultShortcut` in `src/settings/api.ts` mirrors this shape.
    #[test]
    fn the_default_shortcut_reaches_the_window_with_whether_it_is_saved() {
        let default = DefaultShortcut {
            shortcut: crate::shortcut::DEFAULT_SHORTCUT,
            is_saved: true,
        };

        let json = serde_json::to_string(&default).unwrap();

        assert_eq!(json, r#"{"shortcut":"Ctrl+Shift+Space","isSaved":true}"#);
    }

    /// `SetShortcutErrorReason` in `src/settings/api.ts` mirrors these names.
    #[test]
    fn a_refused_shortcut_reaches_the_window_as_its_reason() {
        use crate::shortcut::{ChangeError, Refusal};

        let reasons = [
            (ChangeError::Refused(Refusal::NotAShortcut), "notAShortcut"),
            (ChangeError::Refused(Refusal::NoCtrlOrAlt), "noCtrlOrAlt"),
            (ChangeError::Refused(Refusal::Win), "win"),
            (ChangeError::Refused(Refusal::CopyPaste), "copyPaste"),
            (ChangeError::Taken, "taken"),
            (ChangeError::Save, "save"),
        ];
        for (error, reason) in reasons {
            let json = serde_json::to_string(&SetShortcutError::from(error)).unwrap();

            assert_eq!(json, format!("\"{reason}\""));
        }
    }
}
