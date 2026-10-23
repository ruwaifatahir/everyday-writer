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
        ]),
    ))
    .expect("failed to run the Tauri build script");
}
