//! The settings window and the commands it calls.
//!
//! Privacy: a key comes in through [`save_api_key`] and goes straight to the
//! keyring. No command ever sends a key back; the window only learns whether
//! one is saved.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

use std::sync::{Arc, Mutex, PoisonError};

use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::autostart::{self, TauriAutostart};
use crate::llm::{self, ModelList};
use crate::settings::{
    ApiKey, ApiKeySource, AppSettings, KeyError, KeyringApiKeys, Provider, ProviderKind,
    SaveKeyError, Settings,
};
use crate::shortcut::{self, Hotkeys, ShortcutError, TauriHotkeys};

/// Label of the settings window. The `settings` capability is granted to it.
pub const LABEL: &str = "settings";

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
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App(page(section).into()))
        .title("everyday-writer settings")
        .inner_size(440.0, 720.0)
        .min_inner_size(380.0, 560.0)
        .maximizable(false)
        .center()
        .focused(true)
        .build()?;
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
    #[error("Enter a key first.")]
    BlankKey,
    /// Windows refused the new shortcut. The previous one is kept.
    #[error("This shortcut is used by another app")]
    ShortcutTaken,
    /// The new shortcut isn't a usable combination.
    #[error("Use Ctrl, Alt or Win together with one other key.")]
    ShortcutInvalid,
    /// Windows refused to change its startup list.
    #[error("Couldn't change the Windows startup setting. Try again.")]
    Autostart,
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

/// The models `provider` offers, for the model dropdown, at
/// `custom_base_url` for Custom. The saved key goes along only where
/// [`Settings::may_send_saved_key`] allows: a Custom URL typed but not saved
/// is asked without a key. Falls back to built-in suggestions when the list
/// can't be fetched; never fails.
#[tauri::command]
pub async fn list_models(
    app: AppHandle,
    provider: ProviderKind,
    custom_base_url: String,
) -> ModelList {
    let target = Provider::from_kind(provider, &custom_base_url);
    let saved = app.state::<Arc<AppSettings>>().settings();
    if saved.may_send_saved_key(&target) {
        let keys = app.state::<KeyringApiKeys>().inner().clone();
        llm::model_list(&target, Arc::new(keys)).await
    } else {
        llm::model_list(&target, Arc::new(NoKey)).await
    }
}

/// A key source with no keys, for endpoints the saved key must not reach.
struct NoKey;

impl ApiKeySource for NoKey {
    fn api_key(&self, _provider: &Provider) -> Result<Option<ApiKey>, KeyError> {
        Ok(None)
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

/// Serializes shortcut changes, so two quick changes can't both read the
/// same previous shortcut.
static SHORTCUT_CHANGE: Mutex<()> = Mutex::new(());

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

        assert_eq!(json, r#""Enter a key first.""#);
    }
}
