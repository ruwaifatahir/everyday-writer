//! everyday-writer: rewrites text selected in any app.
//!
//! The app runs in the background with only a tray icon. It opens no window
//! on startup, and a second launch exits right away.

pub mod capture;
pub mod llm;
pub mod logging;
pub mod replace;
pub mod settings;
pub mod shortcut;
pub mod tray;

use tauri::RunEvent;

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
        .setup(|app| {
            tray::init(app)?;
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
    use super::keeps_running_on_exit_request;

    #[test]
    fn closing_the_last_window_keeps_the_app_running() {
        assert!(keeps_running_on_exit_request(None));
    }

    #[test]
    fn an_explicit_exit_quits_the_app() {
        assert!(!keeps_running_on_exit_request(Some(0)));
    }
}
