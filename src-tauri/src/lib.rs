//! everyday-writer: rewrites text selected in any app.
//!
//! The app runs in the background with only a tray icon. It opens the
//! settings window on its own only on first launch, and a second launch
//! exits right away.

pub mod capture;
pub mod llm;
pub mod logging;
pub mod replace;
pub mod settings;
pub mod settings_window;
pub mod shortcut;
pub mod tray;

use std::sync::Arc;

use tauri::{App, Manager, RunEvent};

use settings::{AppSettings, KeyringApiKeys, SettingsFile, Startup};

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
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            settings_window::get_settings,
            settings_window::save_settings,
            settings_window::is_key_saved,
            settings_window::save_api_key,
            settings_window::clear_api_key,
        ])
        .setup(|app| {
            let startup = load_settings(app)?;
            tray::init(app)?;
            if opens_settings_at_startup(startup) && settings_window::open(app.handle()).is_err() {
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
/// `Arc<AppSettings>`.
fn load_settings(app: &App) -> tauri::Result<Startup> {
    let path = app.path().app_data_dir()?.join("settings.json");
    let keys = Arc::new(KeyringApiKeys::new(app.config().identifier.clone()));
    let (settings, startup) = AppSettings::open(SettingsFile::new(path), keys);
    app.manage(Arc::new(settings));
    Ok(startup)
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
}
