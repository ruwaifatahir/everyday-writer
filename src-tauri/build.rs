//! Tauri build script.
//!
//! Listing the app's commands makes Tauri generate one permission per
//! command, so each window's capability grants only the commands it needs.

fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_settings",
            "save_settings",
            "is_key_saved",
            "save_api_key",
            "clear_api_key",
            "get_instruction_sets",
            "add_instruction_set",
            "edit_instruction_set",
            "delete_instruction_set",
            "set_active_instruction_set",
            "restore_starter_templates",
            "popup_state",
            "popup_replace",
            "popup_copy",
            "popup_close",
            "popup_open_settings",
        ]),
    ))
    .expect("failed to run the Tauri build script");
}
