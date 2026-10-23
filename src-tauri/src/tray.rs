//! Tray icon: the app's only visible presence, in the Windows notification area.

use tauri::App;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;

use crate::settings_window;

/// ID of the app's one tray icon.
const TRAY_ID: &str = "main";

/// Menu item ID of the "Settings" entry.
const SETTINGS_ID: &str = "settings";

/// Menu item ID of the "Quit" entry.
const QUIT_ID: &str = "quit";

/// Builds the tray icon with its menu.
///
/// # Errors
///
/// Returns an error if the app has no default window icon or the tray menu or
/// icon cannot be created.
pub fn init(app: &App) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &separator, &quit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or(tauri::Error::InvalidIcon(std::io::Error::other(
            "missing default window icon",
        )))?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("everyday-writer")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            SETTINGS_ID => {
                if settings_window::open(app).is_err() {
                    tracing::warn!("the settings window could not open");
                }
            }
            QUIT_ID => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
