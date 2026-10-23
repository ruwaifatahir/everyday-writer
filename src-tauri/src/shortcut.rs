//! Global shortcut: registers the rewrite shortcut and reports each press.
//!
//! The `global-shortcut` plugin is registered in [`crate::run`].

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// The rewrite shortcut until it can be changed in Settings.
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+Space";

/// The shortcut couldn't be registered. Holds no detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ShortcutError {
    /// The text isn't a valid shortcut.
    #[error("the shortcut is not valid")]
    Invalid,
    /// Windows refused it, almost always because another app uses it.
    #[error("the shortcut is used by another app")]
    Taken,
}

/// Registers `shortcut` and calls `on_press` each time it goes down.
/// `on_press` runs on the main thread, so it must return quickly.
///
/// # Errors
///
/// Returns [`ShortcutError::Invalid`] if `shortcut` can't be parsed and
/// [`ShortcutError::Taken`] if Windows refuses it.
pub fn register(
    app: &AppHandle,
    shortcut: &str,
    on_press: impl Fn(&AppHandle) + Send + Sync + 'static,
) -> Result<(), ShortcutError> {
    let shortcut: Shortcut = shortcut.parse().map_err(|_| ShortcutError::Invalid)?;
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                on_press(app);
            }
        })
        .map_err(|_| ShortcutError::Taken)
}
