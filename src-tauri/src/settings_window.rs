//! The settings window and the commands it calls.
//!
//! Privacy: a key comes in through [`save_api_key`] and goes straight to the
//! keyring. No command ever sends a key back; the window only learns whether
//! one is saved.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use tauri::window::Color;
use tauri::{
    AppHandle, Emitter, Manager, State, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

use crate::autostart::{self, TauriAutostart};
use crate::llm::{self, Connection};
use crate::settings::{
    ApiKey, ApiKeySource, AppSettings, KeyError, KeyringApiKeys, Provider, ProviderKind,
    SaveKeyError, Settings,
};
use crate::shortcut::{self, Hotkeys, ShortcutError, TauriHotkeys};

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
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
        if let Some(section) = section {
            window.emit_to(LABEL, SCROLL_EVENT, section.id())?;
        }
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App(page(section).into()))
        .title(format!("{} Settings", app.package_info().name))
        .inner_size(440.0, 720.0)
        .min_inner_size(380.0, 560.0)
        .theme(Some(Theme::Light))
        .background_color(BACKGROUND)
        .maximizable(false)
        .center()
        .focused(true)
        .build()?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if resumes_shortcut(event) {
            resume_saved_shortcut_later(&handle);
        }
    });
    #[cfg(windows)]
    if let Ok(hwnd) = window.hwnd() {
        crate::win32::color_title_bar(hwnd, TITLE_BAR, TITLE_TEXT);
    }
    Ok(())
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
    /// Windows refused the new shortcut. The previous one is kept.
    #[error("This shortcut is used by another app. Pick a new one.")]
    ShortcutTaken,
    /// The new shortcut isn't a usable combination.
    #[error("Use Ctrl, Alt or Win together with one other key.")]
    ShortcutInvalid,
    /// Windows refused to change its startup list.
    #[error("Couldn't change the Windows startup setting. Try again.")]
    Autostart,
    /// A key test found no key to send.
    #[error("Enter an API key first.")]
    NoKeyToTest,
    /// A key test had no model to check.
    #[error("Choose a model first.")]
    NoModel,
    /// A key test had no usable Custom base URL.
    #[error("Enter a base URL starting with http:// or https://.")]
    InvalidBaseUrl,
    /// The provider rejected the key.
    #[error("{} rejected this API key. Check it and try again.", .0.display_subject())]
    KeyRejected(ProviderKind),
    /// The provider is rate limiting; the key itself may be fine.
    #[error("{} is busy. Wait a moment and try again.", .0.display_subject())]
    RateLimited(ProviderKind),
    /// The provider couldn't be reached.
    #[error("Couldn't reach {}. Check your connection and try again.", .0.display_name())]
    Unreachable(ProviderKind),
    /// The provider answered with another error.
    #[error("{} returned an error. Try again.", .0.display_subject())]
    ProviderFailed(ProviderKind),
}

impl CommandError {
    /// The key test's error for `error`, naming `provider` as the popup does.
    fn from_rewrite(error: llm::RewriteError, provider: ProviderKind) -> Self {
        use llm::{InvalidSetting, RewriteError};
        match error {
            RewriteError::MissingApiKey => Self::NoKeyToTest,
            RewriteError::KeyStore => Self::KeyStore,
            RewriteError::InvalidSettings(InvalidSetting::Model) => Self::NoModel,
            RewriteError::InvalidSettings(InvalidSetting::BaseUrl) => Self::InvalidBaseUrl,
            RewriteError::Unauthorized => Self::KeyRejected(provider),
            RewriteError::RateLimited => Self::RateLimited(provider),
            RewriteError::Network => Self::Unreachable(provider),
            RewriteError::Provider { .. } | RewriteError::EmptyResponse | RewriteError::Other => {
                Self::ProviderFailed(provider)
            }
        }
    }
}

impl From<shortcut::ChangeError> for CommandError {
    fn from(error: shortcut::ChangeError) -> Self {
        match error {
            shortcut::ChangeError::Shortcut(ShortcutError::Taken) => Self::ShortcutTaken,
            shortcut::ChangeError::Shortcut(ShortcutError::Invalid) => Self::ShortcutInvalid,
            shortcut::ChangeError::Save => Self::SaveSettings,
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

/// Checks that `provider` accepts a key, at `custom_base_url` for Custom,
/// without spending tokens (see [`llm::check_connection`]), and whether it
/// offers `model`. Uses `key` if one was typed, otherwise the saved key
/// where [`Settings::may_send_saved_key`] allows.
///
/// # Errors
///
/// Returns a [`CommandError`] saying why the provider couldn't be used.
#[tauri::command]
pub async fn test_connection(
    app: AppHandle,
    provider: ProviderKind,
    custom_base_url: String,
    key: Option<String>,
    model: String,
) -> Result<Connection, CommandError> {
    let target = Provider::from_kind(provider, &custom_base_url);
    let model = Some(model.as_str());
    let checked = match key.filter(|key| !key.trim().is_empty()) {
        Some(key) => {
            llm::check_connection(&target, Arc::new(TypedKey(ApiKey::new(key))), model).await
        }
        None if app
            .state::<Arc<AppSettings>>()
            .settings()
            .may_send_saved_key(&target) =>
        {
            let keys = app.state::<KeyringApiKeys>().inner().clone();
            llm::check_connection(&target, Arc::new(keys), model).await
        }
        None => llm::check_connection(&target, Arc::new(NoKey), model).await,
    };
    checked.map_err(|error| CommandError::from_rewrite(error, provider))
}

/// A key source with no keys, for endpoints the saved key must not reach.
struct NoKey;

impl ApiKeySource for NoKey {
    fn api_key(&self, _provider: &Provider) -> Result<Option<ApiKey>, KeyError> {
        Ok(None)
    }
}

/// The key typed in Settings and not saved yet, for [`test_connection`].
struct TypedKey(ApiKey);

impl ApiKeySource for TypedKey {
    fn api_key(&self, _provider: &Provider) -> Result<Option<ApiKey>, KeyError> {
        Ok(Some(self.0.clone()))
    }
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

/// Serializes shortcut changes, pauses and resumes, so two quick changes
/// can't both read the same previous shortcut and a resume can't bring back
/// a shortcut that a change is replacing.
static SHORTCUT_CHANGE: Mutex<()> = Mutex::new(());

/// Pauses the saved shortcut while Settings records a new one, so pressing it
/// reaches the field instead of starting a rewrite.
#[tauri::command(async)]
pub fn pause_shortcut(settings: State<'_, Arc<AppSettings>>, hotkeys: State<'_, TauriHotkeys>) {
    let _changing = SHORTCUT_CHANGE
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    shortcut::pause(hotkeys.inner(), &settings.settings().shortcut);
}

/// Brings back the saved shortcut when recording ends without a save. Does
/// nothing if it already works.
#[tauri::command(async)]
pub fn resume_shortcut(app: AppHandle) {
    resume_saved_shortcut(&app);
}

/// Whether a settings window `event` ends any recording, so the saved shortcut
/// must come back even if the window's own resume never arrives. Hiding or
/// minimizing the window takes its focus, so losing focus covers those too.
/// Regaining focus focuses the field again, which pauses again.
const fn resumes_shortcut(event: &WindowEvent) -> bool {
    matches!(event, WindowEvent::Focused(false) | WindowEvent::Destroyed)
}

/// [`resume_saved_shortcut`] off the main thread: the plugin runs each call
/// on the main thread and blocks until it's done, so window events, which run
/// there, must not wait for it.
fn resume_saved_shortcut_later(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || resume_saved_shortcut(&app));
}

/// [`resume_shortcut`] for Rust callers, such as the settings window's own
/// events.
fn resume_saved_shortcut(app: &AppHandle) {
    let _changing = SHORTCUT_CHANGE
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let saved = app.state::<Arc<AppSettings>>().settings().shortcut;
    let Some(hotkeys) = app.try_state::<TauriHotkeys>() else {
        return;
    };
    if shortcut::resume(hotkeys.inner(), &saved).is_err() {
        tracing::warn!("the paused shortcut couldn't be registered again");
    }
}

/// Changes the rewrite shortcut. The new one works right away and is saved;
/// if Windows refuses it, the previous one keeps working.
///
/// # Errors
///
/// Returns [`CommandError::ShortcutTaken`] if another app holds it,
/// [`CommandError::ShortcutInvalid`] if it isn't a usable combination and
/// [`CommandError::SaveSettings`] if it can't be saved.
#[tauri::command(async)]
pub fn set_shortcut(
    settings: State<'_, Arc<AppSettings>>,
    hotkeys: State<'_, TauriHotkeys>,
    shortcut: String,
) -> Result<(), CommandError> {
    let _changing = SHORTCUT_CHANGE
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let current = settings.settings().shortcut;
    shortcut::change(hotkeys.inner(), &current, &shortcut, |new| {
        settings.modify(|s| new.clone_into(&mut s.shortcut))
    })?;
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

/// Whether the saved shortcut is the default, so Settings can offer to
/// reset it.
#[tauri::command(async)]
pub fn is_default_shortcut(settings: State<'_, Arc<AppSettings>>) -> bool {
    shortcut::is_default(&settings.settings().shortcut)
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
    use super::{CommandError, Section, page};
    use crate::llm::RewriteError;
    use crate::settings::ProviderKind;

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

    #[test]
    fn a_key_test_error_names_the_provider() {
        let error = CommandError::from_rewrite(RewriteError::Network, ProviderKind::Custom);

        assert_eq!(
            error.to_string(),
            "Couldn't reach your server. Check your connection and try again."
        );
    }
}
