//! everyday-writer: rewrites text selected in any app.
//!
//! The app runs in the background with only a tray icon. It opens the
//! settings window on its own on first launch and when the shortcut can't be
//! registered. A second launch opens Settings in the running app and exits.

/// Serializes a command error as its `Display` message, which the window
/// shows as is. Defined before the modules so they can all use it.
macro_rules! serialize_as_message {
    ($error:ty) => {
        impl serde::Serialize for $error {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }
    };
}

pub mod autostart;
pub mod capture;
pub mod clipboard;
pub mod instruction_set_commands;
pub mod live_rewriter;
pub mod live_updates;
pub mod llm;
pub mod logging;
pub mod popup;
pub mod replace;
pub mod session;
pub mod settings;
pub mod settings_window;
pub mod shortcut;
pub mod text_bridge;
pub mod tray;
pub mod updates;
pub mod win32;

use std::sync::Arc;

use tauri::{App, AppHandle, Manager, RunEvent};
use tauri_plugin_notification::NotificationExt;

use autostart::{Autostart, TauriAutostart};
use live_rewriter::LiveRewriter;
use popup::{AppSession, TauriPopup};
use session::{Event, RewriteSession};
use settings::{AppSettings, KeyringApiKeys, SettingsFile, Startup};
use shortcut::{Hotkeys, ShortcutError, TauriHotkeys};
use text_bridge::WindowsTextBridge;

/// Starts the background app.
///
/// If the Tauri runtime fails to start, logs a content-free error and exits
/// the process with code 1.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();
    #[cfg(windows)]
    if settings::init_keyring().is_err() {
        tracing::warn!("Windows Credential Manager is unavailable; API keys can't be read");
    }

    let built = tauri::Builder::default()
        // Single instance must be registered first, as its docs require.
        // A second launch exits and opens Settings in this one.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            settings_window::open_at_later(app, None);
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        // Endpoint, public key and install mode come from `plugins.updater` in tauri.conf.json.
        // Used from Rust only: no window is granted an `updater:*` permission.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings_window::get_settings,
            settings_window::save_settings,
            settings_window::is_key_saved,
            settings_window::save_api_key,
            settings_window::clear_api_key,
            settings_window::suggested_models,
            settings_window::test_connection,
            settings_window::set_shortcut,
            settings_window::is_shortcut_working,
            settings_window::set_start_with_windows,
            live_updates::app_version,
            live_updates::get_updates_state,
            live_updates::check_for_updates,
            live_updates::set_auto_check,
            live_updates::update_now,
            instruction_set_commands::get_instruction_sets,
            instruction_set_commands::add_instruction_set,
            instruction_set_commands::edit_instruction_set,
            instruction_set_commands::delete_instruction_set,
            instruction_set_commands::set_active_instruction_set,
            instruction_set_commands::restore_starter_templates,
            popup::popup_state,
            popup::popup_replace,
            popup::popup_copy,
            popup::popup_close,
            popup::popup_open_settings,
        ])
        .setup(|app| {
            let startup = load_settings(app)?;
            instruction_set_commands::announce_changes(app);
            start_rewrite_session(app)?;
            live_updates::start(app.handle());
            tray::init(app)?;
            apply_start_with_windows(app);
            let registered = register_saved_shortcut(app);
            if let Err(error) = registered {
                notify_shortcut_failed(app.handle(), error);
            }
            if (opens_settings_at_startup(startup) || registered.is_err())
                && settings_window::open_at(app.handle(), None).is_err()
            {
                tracing::warn!("the settings window could not open");
            }
            tracing::info!("background app started");
            Ok(())
        })
        .build(tauri::generate_context!());
    let Ok(app) = built else {
        // The error isn't logged: only a fixed, content-free message.
        tracing::error!("the app could not start");
        std::process::exit(1);
    };

    app.run(|_app, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event
            && keeps_running_on_exit_request(code)
        {
            api.prevent_exit();
        }
    });
}

/// Loads the settings from the app's data folder and keys from the
/// Credential Manager, and hands them to Tauri's state as
/// `Arc<AppSettings>` and the app's one [`KeyringApiKeys`].
fn load_settings(app: &App) -> tauri::Result<Startup> {
    let path = app.path().app_data_dir()?.join("settings.json");
    let keys = KeyringApiKeys::new(app.config().identifier.clone());
    let (settings, startup) = AppSettings::open(SettingsFile::new(path), Arc::new(keys.clone()));
    app.manage(Arc::new(settings));
    app.manage(keys);
    Ok(startup)
}

/// Wires the rewrite session to Windows, the AI provider, the settings and
/// the popup, and creates the hidden popup.
fn start_rewrite_session(app: &App) -> tauri::Result<()> {
    let settings = Arc::clone(app.state::<Arc<AppSettings>>().inner());
    let keys = app.state::<KeyringApiKeys>().inner().clone();
    let popup = Arc::new(TauriPopup::new(app.handle().clone()));
    app.manage(Arc::clone(&popup));
    let session: AppSession = RewriteSession::new(
        WindowsTextBridge,
        LiveRewriter::new(Arc::clone(&settings), keys),
        settings,
        popup,
    );
    app.manage(Arc::new(session));
    popup::create(app.handle())?;
    Ok(())
}

/// Registers the saved shortcut and hands the global shortcuts to Tauri's
/// state for the settings window.
fn register_saved_shortcut(app: &App) -> Result<(), ShortcutError> {
    let saved = app.state::<Arc<AppSettings>>().settings().shortcut;
    let hotkeys = TauriHotkeys::new(app.handle().clone(), |app| {
        popup::send(app, Event::ShortcutPressed);
    });
    let registered = shortcut::parse(&saved).and_then(|parsed| hotkeys.register(parsed));
    app.manage(hotkeys);
    if registered.is_err() {
        tracing::warn!("the rewrite shortcut couldn't be registered");
    }
    registered
}

/// Tells the user, with a tray notification, that the shortcut doesn't work.
fn notify_shortcut_failed(app: &AppHandle, error: ShortcutError) {
    let body = match error {
        ShortcutError::Taken => "Your shortcut is used by another app. Pick a new one in Settings.",
        ShortcutError::Invalid => "Your saved shortcut isn't valid. Pick a new one in Settings.",
    };
    let shown = app
        .notification()
        .builder()
        .title(&app.package_info().name)
        .body(body)
        .show();
    if shown.is_err() {
        tracing::warn!("the shortcut notification couldn't be shown");
    }
}

/// Makes the Windows startup entry match the saved setting. On first launch
/// the setting is on, so the app starts with Windows from then on.
fn apply_start_with_windows(app: &App) {
    let enabled = app
        .state::<Arc<AppSettings>>()
        .settings()
        .start_with_windows;
    if TauriAutostart(app.handle().clone()).set(enabled).is_err() {
        tracing::warn!("the Windows startup entry couldn't be updated");
    }
}

/// Settings open on their own only when there is nothing usable to start
/// from: on first launch, or when the settings file is damaged.
fn opens_settings_at_startup(startup: Startup) -> bool {
    startup != Startup::Loaded
}

/// Decides whether an exit request should be ignored.
///
/// Tauri asks to exit with `code: None` when the last window closes; the
/// background app must keep running then. An explicit `exit` (the tray's
/// "Quit") carries `Some(code)` and must quit.
fn keeps_running_on_exit_request(code: Option<i32>) -> bool {
    code.is_none()
}

#[cfg(test)]
mod tests {
    use super::{keeps_running_on_exit_request, opens_settings_at_startup};
    use crate::settings::Startup;

    #[test]
    fn settings_open_on_their_own_on_first_launch() {
        assert!(opens_settings_at_startup(Startup::FirstLaunch));
    }

    #[test]
    fn settings_open_on_their_own_when_the_file_is_damaged() {
        assert!(opens_settings_at_startup(Startup::Damaged));
    }

    #[test]
    fn settings_stay_closed_on_a_normal_launch() {
        assert!(!opens_settings_at_startup(Startup::Loaded));
    }

    #[test]
    fn closing_the_last_window_keeps_the_app_running() {
        assert!(keeps_running_on_exit_request(None));
    }

    #[test]
    fn an_explicit_exit_quits_the_app() {
        assert!(!keeps_running_on_exit_request(Some(0)));
    }

    /// ADR 0002: what every installed copy bakes in. Changing any of this after a
    /// release strands or weakens existing installs, so it fails loudly here.
    #[test]
    fn updates_come_signed_from_the_static_github_file_and_install_per_user() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let updater = &config["plugins"]["updater"];

        // A plain GET of a fixed URL: no `{{current_version}}` or other template data.
        assert_eq!(
            updater["endpoints"],
            serde_json::json!([
                "https://github.com/ruwaifatahir/everyday-writer/releases/latest/download/latest.json"
            ])
        );
        assert!(
            updater["pubkey"]
                .as_str()
                .is_some_and(|key| !key.is_empty()),
            "the public key is missing"
        );
        assert_eq!(updater["requireSignedVersion"], true);
        assert_eq!(updater["windows"]["installMode"], "passive");
        assert_eq!(config["bundle"]["createUpdaterArtifacts"], true);
        // Per-user NSIS install (the default) means no admin prompt on update.
        let install_mode = &config["bundle"]["windows"]["nsis"]["installMode"];
        assert!(
            install_mode.is_null() || install_mode == "currentUser",
            "the installer must stay per-user"
        );
    }
}
