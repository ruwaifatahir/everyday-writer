//! Tray icon: the app's only visible presence, in the Windows notification area.

use tauri::App;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

/// Menu item ID of the "Quit" entry.
const QUIT_ID: &str = "quit";

/// Builds the tray icon with its menu.
///
/// # Errors
///
/// Returns an error if the app has no default window icon or the tray menu or
/// icon cannot be created.
pub fn init(app: &App) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or(tauri::Error::InvalidIcon(std::io::Error::other(
            "missing default window icon",
        )))?;
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("everyday-writer")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == QUIT_ID {
                app.exit(0);
            }
        })
        .build(app)?;
    Ok(())
}
