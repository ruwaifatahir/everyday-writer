//! The settings window and the commands it calls.
//!
//! Privacy: a key comes in through [`save_api_key`] and goes straight to the
//! keyring. No command ever sends a key back; the window only learns whether
//! one is saved.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::settings::{ApiKey, AppSettings, ProviderKind, SaveKeyError, Settings};

/// Label of the settings window. The `settings` capability is granted to it.
pub const LABEL: &str = "settings";

/// Opens the settings window, or brings it to the front if it's open.
///
/// # Errors
///
/// Returns an error if the window can't be created or shown.
pub fn open(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) {
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("settings.html".into()))
        .title("everyday-writer settings")
        .inner_size(440.0, 600.0)
        .min_inner_size(380.0, 560.0)
        .maximizable(false)
        .center()
        .focused(true)
        .build()?;
    Ok(())
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
}

impl Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

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

/// Saves the provider, model and Custom base URL.
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
        .update(new_settings)
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
