//! Tray icon: the app's only visible presence, in the Windows notification area.
//!
//! Its menu holds the "Instruction set" submenu, the update item, Settings
//! and Quit. The menu is rebuilt after every instruction set change, wherever
//! it was made, and whenever the updates state changes.

pub mod menu;

use std::sync::Arc;

use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager, Wry};

use crate::live_updates::{self, TauriUpdatesView};
use crate::settings::{AppSettings, InstructionSetId, InstructionSets};
use crate::settings_window::{self, Section};
use crate::updates::{UpdateEvent, UpdatesState};
use menu::{TrayAction, TrayItem};

/// ID of the app's one tray icon.
const TRAY_ID: &str = "main";

/// Builds the tray icon with its menu, and keeps the menu in step with the
/// instruction sets and the updates state. Call after the settings and the
/// updates view are managed.
///
/// # Errors
///
/// Returns an error if the app has no default window icon or the tray menu or
/// icon cannot be created.
pub fn init(app: &App) -> tauri::Result<()> {
    let settings = Arc::clone(app.state::<Arc<AppSettings>>().inner());
    let menu = build_menu(
        app.handle(),
        &settings.settings().instruction_sets,
        &current_updates(app.handle()),
    )?;
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
        .on_menu_event(|app, event| match menu::action(event.id.as_ref()) {
            Some(TrayAction::Activate(id)) => activate_later(app, id),
            Some(TrayAction::CheckForUpdates) => {
                settings_window::open_at_later(app, Some(Section::Updates));
                live_updates::send(app, UpdateEvent::CheckRequested);
            }
            Some(TrayAction::ShowUpdate) => {
                settings_window::open_at_later(app, Some(Section::Updates));
            }
            Some(TrayAction::OpenSettings) => settings_window::open_at_later(app, None),
            Some(TrayAction::Quit) => app.exit(0),
            None => {}
        })
        .build(app)?;

    let handle = app.handle().clone();
    settings.on_instruction_sets_change(move |sets| refresh(&handle, sets));
    let handle = app.handle().clone();
    app.state::<Arc<TauriUpdatesView>>()
        .on_change(move |updates| show_updates(&handle, updates));
    Ok(())
}

/// Makes the set active from a blocking task: the menu event runs inside
/// the event loop, and the change saves the settings file and rebuilds this
/// very menu. On success the change listener rebuilds the menu.
fn activate_later(app: &AppHandle, id: InstructionSetId) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let settings = app.state::<Arc<AppSettings>>();
        if settings
            .change_instruction_sets(|sets| sets.set_active(id))
            .is_err()
        {
            tracing::warn!("the instruction set picked in the tray couldn't be made active");
            // Windows already moved the check mark; put it back.
            refresh(&app, &settings.settings().instruction_sets);
        }
    });
}

/// Rebuilds the tray menu for a new updates state.
fn show_updates(app: &AppHandle, updates: &UpdatesState) {
    let sets = app.state::<Arc<AppSettings>>().settings().instruction_sets;
    rebuild(app, &sets, updates);
}

/// Replaces the tray menu with one built from `sets`.
fn refresh(app: &AppHandle, sets: &InstructionSets) {
    rebuild(app, sets, &current_updates(app));
}

/// The updates state shown now, or idle before the updates view exists.
fn current_updates(app: &AppHandle) -> UpdatesState {
    app.try_state::<Arc<TauriUpdatesView>>()
        .map_or(UpdatesState::Idle, |view| view.state())
}

/// Replaces the tray menu with one built from `sets` and `updates`.
fn rebuild(app: &AppHandle, sets: &InstructionSets, updates: &UpdatesState) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let rebuilt = build_menu(app, sets, updates).and_then(|menu| tray.set_menu(Some(menu)));
    if rebuilt.is_err() {
        tracing::warn!("the tray menu couldn't be rebuilt");
    }
}

/// Turns the menu model from [`menu::tray_menu`] into a real menu.
fn build_menu(
    app: &AppHandle,
    sets: &InstructionSets,
    updates: &UpdatesState,
) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    for item in menu::tray_menu(sets, updates) {
        match item {
            TrayItem::Submenu { id, label, sets } => {
                let items = sets
                    .into_iter()
                    .map(|set| {
                        CheckMenuItem::with_id(
                            app,
                            set.id,
                            set.label,
                            true,
                            set.checked,
                            None::<&str>,
                        )
                    })
                    .collect::<tauri::Result<Vec<_>>>()?;
                let refs: Vec<&dyn IsMenuItem<Wry>> = items
                    .iter()
                    .map(|item| item as &dyn IsMenuItem<Wry>)
                    .collect();
                menu.append(&Submenu::with_id_and_items(app, id, label, true, &refs)?)?;
            }
            TrayItem::Item { id, label } => {
                menu.append(&MenuItem::with_id(app, id, label, true, None::<&str>)?)?;
            }
            TrayItem::Disabled { label } => {
                menu.append(&MenuItem::new(app, label, false, None::<&str>)?)?;
            }
            TrayItem::Separator => menu.append(&PredefinedMenuItem::separator(app)?)?,
        }
    }
    Ok(menu)
}
