//! The live settings: what the settings window edits and what the rest of
//! the app reads through [`SettingsView`].
//!
//! Privacy: keys pass through to the [`KeyStore`] and are never kept,
//! returned or logged here.

use std::sync::{Arc, PoisonError, RwLock};

use super::{
    ApiKey, KeyError, KeyStore, ModelSettings, ProviderKind, Settings, SettingsError, SettingsFile,
};

/// What the rest of the app reads from the settings. A port: the rewrite
/// session depends on this trait, and tests supply a fake.
///
/// Later settings (the active instruction set, the shortcut) are added here
/// as methods, so readers never see the settings file or the keyring.
pub trait SettingsView: Send + Sync {
    /// The chosen provider and model.
    fn model_settings(&self) -> ModelSettings;

    /// Whether a rewrite can be sent: a model is set, the provider has a key
    /// (or doesn't need one) and a Custom provider has an `http(s)` base URL.
    fn is_provider_ready(&self) -> bool;
}

/// How the settings were found at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Startup {
    /// No settings file existed. Defaults were saved, so this happens once.
    FirstLaunch,
    /// The file couldn't be read. Defaults are in use; the file is left as
    /// is until the user saves.
    Damaged,
    /// The saved settings were loaded.
    Loaded,
}

/// A key couldn't be saved.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SaveKeyError {
    /// The key is empty or only spaces. Use [`AppSettings::clear_key`] to
    /// remove a key.
    #[error("the key is blank")]
    Blank,
    /// The credential store failed.
    #[error(transparent)]
    Store(#[from] KeyError),
}

/// The live settings, backed by the settings file and the key store.
pub struct AppSettings {
    file: SettingsFile,
    keys: Arc<dyn KeyStore>,
    current: RwLock<Settings>,
}

impl AppSettings {
    /// Loads the settings from `file`, or defaults if there are none yet.
    /// On first launch the defaults are saved right away, so only the very
    /// first start reports [`Startup::FirstLaunch`].
    pub fn open(file: SettingsFile, keys: Arc<dyn KeyStore>) -> (Self, Startup) {
        let (current, startup) = match file.load() {
            Ok(Some(settings)) => (settings, Startup::Loaded),
            Ok(None) => {
                let defaults = Settings::default();
                if file.save(&defaults).is_err() {
                    tracing::warn!("the default settings couldn't be saved");
                }
                (defaults, Startup::FirstLaunch)
            }
            Err(_) => {
                tracing::warn!("the settings file is damaged; using defaults");
                (Settings::default(), Startup::Damaged)
            }
        };
        let app = Self {
            file,
            keys,
            current: RwLock::new(current),
        };
        (app, startup)
    }

    /// A copy of the current settings.
    pub fn settings(&self) -> Settings {
        self.current
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Saves `settings` to the file, then makes them current.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] if the file can't be written. The current
    /// settings stay unchanged then.
    pub fn update(&self, settings: Settings) -> Result<(), SettingsError> {
        self.file.save(&settings)?;
        *self.current.write().unwrap_or_else(PoisonError::into_inner) = settings;
        Ok(())
    }

    /// Whether `provider` has a saved key. A failing store counts as no key.
    pub fn is_key_saved(&self, provider: ProviderKind) -> bool {
        self.keys.has_key(provider).unwrap_or(false)
    }

    /// Saves `key` for `provider`, replacing any earlier one. Surrounding
    /// spaces are trimmed, as they are never part of a real key.
    ///
    /// # Errors
    ///
    /// Returns [`SaveKeyError::Blank`] for an empty key and
    /// [`SaveKeyError::Store`] if the store can't be written.
    pub fn set_key(&self, provider: ProviderKind, key: &ApiKey) -> Result<(), SaveKeyError> {
        let trimmed = key.expose_secret().trim();
        if trimmed.is_empty() {
            return Err(SaveKeyError::Blank);
        }
        self.keys
            .set_key(provider, &ApiKey::new(trimmed.to_owned()))?;
        Ok(())
    }

    /// Removes `provider`'s key. Succeeds if there was none.
    ///
    /// # Errors
    ///
    /// Returns [`KeyError`] if the store can't be written.
    pub fn clear_key(&self, provider: ProviderKind) -> Result<(), KeyError> {
        self.keys.delete_key(provider)
    }
}

impl SettingsView for AppSettings {
    fn model_settings(&self) -> ModelSettings {
        self.settings().model_settings()
    }

    fn is_provider_ready(&self) -> bool {
        let settings = self.settings();
        if settings.model.trim().is_empty() {
            return false;
        }
        match settings.provider {
            ProviderKind::Custom => is_http_url(&settings.custom_base_url),
            provider => self.is_key_saved(provider),
        }
    }
}

/// Whether `url` starts with `http://` or `https://` and has something after.
fn is_http_url(url: &str) -> bool {
    let url = url.trim();
    ["https://", "http://"].iter().any(|scheme| {
        url.strip_prefix(scheme)
            .is_some_and(|rest| !rest.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::{AppSettings, Startup};
    use crate::settings::{
        ApiKey, KeyError, KeyStore, Provider, ProviderKind, Settings, SettingsFile, SettingsView,
    };

    /// In-memory keys. `broken` makes every call fail like a locked keyring.
    #[derive(Default)]
    struct FakeKeys {
        keys: Mutex<HashMap<ProviderKind, String>>,
        broken: bool,
    }

    impl KeyStore for FakeKeys {
        fn has_key(&self, provider: ProviderKind) -> Result<bool, KeyError> {
            if self.broken {
                return Err(KeyError);
            }
            Ok(self.keys.lock().unwrap().contains_key(&provider))
        }

        fn set_key(&self, provider: ProviderKind, key: &ApiKey) -> Result<(), KeyError> {
            if self.broken {
                return Err(KeyError);
            }
            self.keys
                .lock()
                .unwrap()
                .insert(provider, key.expose_secret().to_owned());
            Ok(())
        }

        fn delete_key(&self, provider: ProviderKind) -> Result<(), KeyError> {
            if self.broken {
                return Err(KeyError);
            }
            self.keys.lock().unwrap().remove(&provider);
            Ok(())
        }
    }

    fn temp_file() -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "everyday-writer-view-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("settings.json")
    }

    fn open(path: &PathBuf, keys: Arc<FakeKeys>) -> (AppSettings, Startup) {
        AppSettings::open(SettingsFile::new(path.clone()), keys)
    }

    fn settings(provider: ProviderKind, model: &str) -> Settings {
        Settings {
            provider,
            model: model.to_owned(),
            ..Settings::default()
        }
    }

    fn key(secret: &str) -> ApiKey {
        ApiKey::new(secret.to_owned())
    }

    #[test]
    fn the_first_launch_is_reported_once() {
        let path = temp_file();

        let (_, first) = open(&path, Arc::default());
        let (_, second) = open(&path, Arc::default());

        assert_eq!(first, Startup::FirstLaunch);
        assert_eq!(second, Startup::Loaded);
    }

    #[test]
    fn provider_model_and_custom_base_url_survive_a_restart() {
        let path = temp_file();
        let saved = Settings {
            provider: ProviderKind::Custom,
            model: "local-model".to_owned(),
            custom_base_url: "http://localhost:1234/v1".to_owned(),
        };
        let (app, _) = open(&path, Arc::default());
        app.update(saved.clone()).unwrap();

        let (restarted, startup) = open(&path, Arc::default());

        assert_eq!(startup, Startup::Loaded);
        assert_eq!(restarted.settings(), saved);
    }

    #[test]
    fn the_settings_file_never_contains_a_key() {
        let path = temp_file();
        let keys = Arc::new(FakeKeys::default());
        let (app, _) = open(&path, Arc::clone(&keys));

        app.set_key(ProviderKind::OpenAi, &key("sk-SECRET-123"))
            .unwrap();
        app.update(settings(ProviderKind::OpenAi, "gpt-test"))
            .unwrap();
        let (restarted, _) = open(&path, keys);

        let file = std::fs::read_to_string(&path).unwrap();
        assert!(!file.contains("sk-SECRET-123"));
        assert_eq!(
            restarted.settings(),
            settings(ProviderKind::OpenAi, "gpt-test")
        );
        assert!(restarted.is_key_saved(ProviderKind::OpenAi));
    }

    #[test]
    fn each_provider_keeps_its_own_key() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.set_key(ProviderKind::OpenAi, &key("k-openai")).unwrap();
        app.update(settings(ProviderKind::Gemini, "gemini-test"))
            .unwrap();
        app.set_key(ProviderKind::Gemini, &key("k-gemini")).unwrap();

        app.update(settings(ProviderKind::OpenAi, "gpt-test"))
            .unwrap();

        assert!(app.is_key_saved(ProviderKind::OpenAi));
        assert!(app.is_key_saved(ProviderKind::Gemini));
        assert!(!app.is_key_saved(ProviderKind::Anthropic));
    }

    #[test]
    fn a_cleared_key_is_gone_for_that_provider_only() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.set_key(ProviderKind::OpenAi, &key("k-openai")).unwrap();
        app.set_key(ProviderKind::Gemini, &key("k-gemini")).unwrap();

        app.clear_key(ProviderKind::OpenAi).unwrap();

        assert!(!app.is_key_saved(ProviderKind::OpenAi));
        assert!(app.is_key_saved(ProviderKind::Gemini));
    }

    #[test]
    fn the_view_reports_the_chosen_provider_and_model() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.update(Settings {
            provider: ProviderKind::Custom,
            model: " local-model ".to_owned(),
            custom_base_url: "http://localhost:1234/v1".to_owned(),
        })
        .unwrap();

        let current = app.model_settings();

        assert_eq!(
            current.provider,
            Provider::Custom {
                base_url: "http://localhost:1234/v1".to_owned()
            }
        );
        assert_eq!(current.model, "local-model");
    }

    #[test]
    fn a_provider_with_a_model_and_a_key_is_ready() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.update(settings(ProviderKind::Anthropic, "claude-test"))
            .unwrap();
        app.set_key(ProviderKind::Anthropic, &key("k")).unwrap();

        assert!(app.is_provider_ready());
    }

    #[test]
    fn a_provider_that_needs_a_key_is_not_ready_without_one() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.set_key(ProviderKind::Gemini, &key("k")).unwrap();

        app.update(settings(ProviderKind::OpenAi, "gpt-test"))
            .unwrap();

        assert!(!app.is_provider_ready());
    }

    #[test]
    fn a_provider_is_not_ready_without_a_model() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.set_key(ProviderKind::OpenAi, &key("k")).unwrap();

        app.update(settings(ProviderKind::OpenAi, "  ")).unwrap();

        assert!(!app.is_provider_ready());
    }

    #[test]
    fn custom_is_ready_without_a_key_once_it_has_a_base_url() {
        let (app, _) = open(&temp_file(), Arc::default());
        let mut custom = settings(ProviderKind::Custom, "local-model");
        app.update(custom.clone()).unwrap();
        let without_url = app.is_provider_ready();

        custom.custom_base_url = "http://localhost:1234/v1".to_owned();
        app.update(custom).unwrap();

        assert!(!without_url);
        assert!(app.is_provider_ready());
    }

    #[test]
    fn custom_is_not_ready_with_a_base_url_that_is_not_http() {
        let (app, _) = open(&temp_file(), Arc::default());

        app.update(Settings {
            provider: ProviderKind::Custom,
            model: "local-model".to_owned(),
            custom_base_url: "localhost:1234".to_owned(),
        })
        .unwrap();

        assert!(!app.is_provider_ready());
    }

    #[test]
    fn a_broken_keyring_means_not_ready() {
        let keys = Arc::new(FakeKeys {
            broken: true,
            ..FakeKeys::default()
        });
        let (app, _) = open(&temp_file(), keys);

        app.update(settings(ProviderKind::OpenAi, "gpt-test"))
            .unwrap();

        assert!(!app.is_provider_ready());
        assert!(!app.is_key_saved(ProviderKind::OpenAi));
    }

    #[test]
    fn a_blank_key_is_refused() {
        let keys = Arc::new(FakeKeys::default());
        let (app, _) = open(&temp_file(), Arc::clone(&keys));

        let result = app.set_key(ProviderKind::OpenAi, &key("   "));

        assert!(result.is_err());
        assert!(!app.is_key_saved(ProviderKind::OpenAi));
    }

    #[test]
    fn a_damaged_file_is_reported_and_loads_defaults() {
        let path = temp_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{ not json").unwrap();

        let (app, startup) = open(&path, Arc::default());

        assert_eq!(startup, Startup::Damaged);
        assert_eq!(app.settings(), Settings::default());
    }
}
