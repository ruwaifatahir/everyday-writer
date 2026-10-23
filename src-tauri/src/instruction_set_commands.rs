//! The commands the settings window calls to manage instruction sets, and
//! the event that keeps the window in step with changes made elsewhere.
//!
//! Every command returns the whole library after the change, so the window
//! always shows exactly what is saved.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

use std::sync::Arc;

use serde::Serialize;
use tauri::{App, Emitter, Manager, State};

use crate::settings::{
    AppSettings, ChangeError, InstructionSetError, InstructionSetId, InstructionSets,
};
use crate::settings_window;

/// Sent to the settings window, with the new [`InstructionSets`], after
/// every change, including ones made outside the window (the tray).
pub const CHANGED_EVENT: &str = "instruction-sets-changed";

/// An instruction set command failed. Serialized as a short message the
/// window shows as is. Never carries set content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The name was empty.
    #[error("Give the instruction set a name.")]
    BlankName,
    /// The set was deleted meanwhile.
    #[error("That instruction set no longer exists.")]
    NotFound,
    /// The last set can't be deleted.
    #[error("Keep at least one instruction set.")]
    LastSet,
    /// The settings file couldn't be written.
    #[error("Couldn't save your instruction sets. Check that the app data folder is writable.")]
    Save,
}

impl Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl From<ChangeError> for CommandError {
    fn from(error: ChangeError) -> Self {
        match error {
            ChangeError::Refused(InstructionSetError::BlankName) => Self::BlankName,
            ChangeError::Refused(InstructionSetError::NotFound) => Self::NotFound,
            ChangeError::Refused(InstructionSetError::LastSet) => Self::LastSet,
            ChangeError::Save(_) => Self::Save,
        }
    }
}

/// Sends [`CHANGED_EVENT`] to the settings window after every instruction
/// set change. Call once, after the settings are managed.
pub fn announce_changes(app: &App) {
    let handle = app.handle().clone();
    app.state::<Arc<AppSettings>>()
        .on_instruction_sets_change(move |sets| {
            // The window may be closed; then there is no one to tell.
            if handle.get_webview_window(settings_window::LABEL).is_some()
                && handle
                    .emit_to(settings_window::LABEL, CHANGED_EVENT, sets)
                    .is_err()
            {
                tracing::warn!(
                    "the settings window couldn't be told about an instruction set change"
                );
            }
        });
}

/// Applies `change` and returns the library after it.
fn change(
    settings: &AppSettings,
    change: impl FnOnce(&mut InstructionSets) -> Result<(), InstructionSetError>,
) -> Result<InstructionSets, CommandError> {
    settings
        .change_instruction_sets(|sets| {
            change(sets)?;
            Ok(sets.clone())
        })
        .map_err(CommandError::from)
}

/// The instruction sets and the active one.
#[tauri::command(async)]
pub fn get_instruction_sets(settings: State<'_, Arc<AppSettings>>) -> InstructionSets {
    settings.settings().instruction_sets
}

/// Adds a set the user wrote at the end of the list.
///
/// # Errors
///
/// Returns [`CommandError::BlankName`] for an empty name and
/// [`CommandError::Save`] if the file can't be written.
#[tauri::command(async)]
pub fn add_instruction_set(
    settings: State<'_, Arc<AppSettings>>,
    name: String,
    instructions: String,
) -> Result<InstructionSets, CommandError> {
    change(&settings, |sets| sets.add(&name, &instructions).map(drop))
}

/// Changes a set's name and instructions.
///
/// # Errors
///
/// Returns [`CommandError::BlankName`], [`CommandError::NotFound`] or
/// [`CommandError::Save`].
#[tauri::command(async)]
pub fn edit_instruction_set(
    settings: State<'_, Arc<AppSettings>>,
    id: InstructionSetId,
    name: String,
    instructions: String,
) -> Result<InstructionSets, CommandError> {
    change(&settings, |sets| sets.edit(id, &name, &instructions))
}

/// Deletes a set. Deleting the active set makes the first remaining one
/// active.
///
/// # Errors
///
/// Returns [`CommandError::LastSet`], [`CommandError::NotFound`] or
/// [`CommandError::Save`].
#[tauri::command(async)]
pub fn delete_instruction_set(
    settings: State<'_, Arc<AppSettings>>,
    id: InstructionSetId,
) -> Result<InstructionSets, CommandError> {
    change(&settings, |sets| sets.delete(id))
}

/// Makes a set the active one.
///
/// # Errors
///
/// Returns [`CommandError::NotFound`] or [`CommandError::Save`].
#[tauri::command(async)]
pub fn set_active_instruction_set(
    settings: State<'_, Arc<AppSettings>>,
    id: InstructionSetId,
) -> Result<InstructionSets, CommandError> {
    change(&settings, |sets| sets.set_active(id))
}

/// Restore default templates. The window asks the user to confirm first.
///
/// # Errors
///
/// Returns [`CommandError::Save`] if the file can't be written.
#[tauri::command(async)]
pub fn restore_starter_templates(
    settings: State<'_, Arc<AppSettings>>,
) -> Result<InstructionSets, CommandError> {
    change(&settings, |sets| {
        sets.restore_starter_templates();
        Ok(())
    })
}
