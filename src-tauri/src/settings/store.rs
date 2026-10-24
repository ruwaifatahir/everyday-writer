//! The settings file: plain JSON in the app's data folder.
//!
//! Privacy: the file never holds an API key. [`Settings`] has no key field,
//! so there is nothing to write.

use std::io::ErrorKind;
use std::path::PathBuf;

use super::Settings;

/// Reading or writing the settings file failed. Holds no detail: parse
/// errors can quote the file's contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SettingsError {
    /// The file or its folder couldn't be read or written.
    #[error("the settings file couldn't be read or written")]
    Io,
    /// The file exists but isn't valid settings.
    #[error("the settings file is damaged")]
    Invalid,
}

impl From<std::io::Error> for SettingsError {
    fn from(_: std::io::Error) -> Self {
        Self::Io
    }
}

/// The settings file at one path.
#[derive(Debug, Clone)]
pub struct SettingsFile {
    path: PathBuf,
}

impl SettingsFile {
    /// The settings file at `path`. Nothing is read or written yet.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The saved settings, or `None` if no file exists yet (first launch).
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError::Io`] if the file can't be read and
    /// [`SettingsError::Invalid`] if it isn't valid settings.
    pub fn load(&self) -> Result<Option<Settings>, SettingsError> {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| SettingsError::Invalid)
    }

    /// Writes `settings`, creating the folder if needed. Writes a temporary
    /// file first and renames it over the old one, so a crash mid-write never
    /// leaves a half-written file.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError::Io`] if the file can't be written.
    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_vec_pretty(settings).map_err(|_| SettingsError::Io)?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{SettingsError, SettingsFile};
    use crate::settings::{ProviderKind, Settings};
    use crate::updates::Remembered;

    /// A fresh, empty folder per test, so tests never share a file.
    fn temp_dir() -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "everyday-writer-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn custom_settings() -> Settings {
        Settings {
            provider: ProviderKind::Custom,
            model: "local-model".to_owned(),
            custom_base_url: "http://localhost:1234/v1".to_owned(),
            ..Settings::default()
        }
    }

    #[test]
    fn saved_settings_load_back_unchanged() {
        let file = SettingsFile::new(temp_dir().join("settings.json"));

        file.save(&custom_settings()).unwrap();
        let loaded = file.load().unwrap();

        assert_eq!(loaded, Some(custom_settings()));
    }

    #[test]
    fn a_missing_file_means_first_launch() {
        let file = SettingsFile::new(temp_dir().join("settings.json"));

        assert_eq!(file.load(), Ok(None));
    }

    #[test]
    fn a_damaged_file_is_reported_as_invalid() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, b"{ not json").unwrap();

        assert_eq!(SettingsFile::new(path).load(), Err(SettingsError::Invalid));
    }

    #[test]
    fn missing_fields_load_as_defaults() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, br#"{ "provider": "gemini" }"#).unwrap();

        let loaded = SettingsFile::new(path).load().unwrap().unwrap();

        assert_eq!(loaded.provider, ProviderKind::Gemini);
        assert_eq!(loaded.model, "");
    }

    #[test]
    fn a_file_without_a_shortcut_loads_the_default_shortcut_and_starts_with_windows() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, br#"{ "provider": "gemini" }"#).unwrap();

        let loaded = SettingsFile::new(path).load().unwrap().unwrap();

        assert_eq!(loaded.shortcut, "Ctrl+Shift+Space");
        assert!(loaded.start_with_windows);
    }

    #[test]
    fn a_file_from_before_updates_checks_automatically_and_remembers_no_notice() {
        let dir = temp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(
            &path,
            br#"{ "provider": "gemini", "startWithWindows": false }"#,
        )
        .unwrap();

        let loaded = SettingsFile::new(path).load().unwrap().unwrap();

        assert!(loaded.updates.auto_check);
        assert_eq!(loaded.updates.last_notified_version, None);
        assert_eq!(loaded.updates.last_run_version, None);
    }

    #[test]
    fn the_update_memory_survives_a_save() {
        let path = temp_dir().join("settings.json");
        let file = SettingsFile::new(path.clone());
        let saved = Settings {
            updates: Remembered {
                auto_check: false,
                last_notified_version: Some("1.1.0".to_owned()),
                last_run_version: Some("1.0.0".to_owned()),
            },
            ..custom_settings()
        };

        file.save(&saved).unwrap();

        assert_eq!(file.load().unwrap(), Some(saved));
        // Kept as top-level fields, as in files written before.
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(json["autoCheck"], false);
        assert_eq!(json["lastNotifiedVersion"], "1.1.0");
        assert_eq!(json["lastRunVersion"], "1.0.0");
    }

    #[test]
    fn the_shortcut_and_start_with_windows_survive_a_save() {
        let file = SettingsFile::new(temp_dir().join("settings.json"));
        let saved = Settings {
            shortcut: "Alt+Shift+R".to_owned(),
            start_with_windows: false,
            ..custom_settings()
        };

        file.save(&saved).unwrap();

        assert_eq!(file.load().unwrap(), Some(saved));
    }
}
