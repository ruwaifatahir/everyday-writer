//! The live settings: what the settings window edits and what the rest of
//! the app reads through [`SettingsView`].
//!
//! Privacy: keys pass through to the [`KeyStore`] and are never kept,
//! returned or logged here.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

use super::{
    ApiKey, InstructionSetChoices, InstructionSetError, InstructionSetId, InstructionSets,
    KeyError, KeyStore, ModelSettings, ProviderKind, Settings, SettingsError, SettingsFile,
};

/// What the rest of the app reads from the settings. A port: the rewrite
/// session depends on this trait, and tests supply a fake.
///
/// Later settings (the active instruction set, the shortcut) are added here
/// as methods, so readers never see the settings file or the keyring.
pub trait SettingsView: Send + Sync {
    /// The chosen provider and model.
    fn model_settings(&self) -> ModelSettings;

    /// Whether a rewrite can be sent and, if not, the first thing missing.
    fn provider_readiness(&self) -> ProviderReadiness;

    /// The active instruction set's instructions, which every rewrite
    /// follows.
    fn active_instructions(&self) -> String;

    /// Every instruction set's id and name, in order, and the active id.
    fn instruction_set_choices(&self) -> InstructionSetChoices;

    /// Makes the set with `id` the Active Instruction Set and saves it.
    /// Everyone listening for instruction set changes (the tray, the
    /// settings window) hears about it.
    ///
    /// # Errors
    ///
    /// Returns [`ChangeError`] if no set has `id` or the file can't be
    /// written. The active set stays as it was then.
    fn set_active_instruction_set(&self, id: InstructionSetId) -> Result<(), ChangeError>;
}

impl<T: SettingsView + ?Sized> SettingsView for Arc<T> {
    fn model_settings(&self) -> ModelSettings {
        (**self).model_settings()
    }

    fn provider_readiness(&self) -> ProviderReadiness {
        (**self).provider_readiness()
    }

    fn active_instructions(&self) -> String {
        (**self).active_instructions()
    }

    fn instruction_set_choices(&self) -> InstructionSetChoices {
        (**self).instruction_set_choices()
    }

    fn set_active_instruction_set(&self, id: InstructionSetId) -> Result<(), ChangeError> {
        (**self).set_active_instruction_set(id)
    }
}

/// Whether the chosen provider can take a rewrite, or what it still needs.
/// When several things are missing, the key is reported first, then the
/// model, then the Custom base URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderReadiness {
    /// A rewrite can be sent.
    Ready,
    /// The provider needs a key and none is saved (or the store failed).
    MissingKey,
    /// No model is set.
    NoModel,
    /// Custom is chosen without an `http(s)` base URL.
    NoCustomUrl,
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

/// An instruction set change wasn't made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ChangeError {
    /// The change breaks an instruction set rule.
    #[error(transparent)]
    Refused(#[from] InstructionSetError),
    /// The settings file couldn't be written.
    #[error(transparent)]
    Save(#[from] SettingsError),
}

/// Hears about every instruction set change.
type Listener = Box<dyn Fn(&InstructionSets) + Send + Sync>;

/// The live settings, backed by the settings file and the key store.
pub struct AppSettings {
    file: SettingsFile,
    keys: Arc<dyn KeyStore>,
    current: RwLock<Settings>,
    listeners: RwLock<Vec<Listener>>,
    /// Whether the file was damaged at startup and nothing has been saved
    /// over it since. Changes the app makes by itself leave it alone then.
    damaged: AtomicBool,
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
            listeners: RwLock::default(),
            damaged: AtomicBool::new(startup == Startup::Damaged),
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

    /// Saves `settings` to the file, then makes them current. The
    /// instruction sets in `settings` are ignored: they change only through
    /// [`AppSettings::change_instruction_sets`], so saving the provider can
    /// never undo a change made meanwhile (from the tray, say).
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] if the file can't be written. The current
    /// settings stay unchanged then.
    pub fn update(&self, mut settings: Settings) -> Result<(), SettingsError> {
        let mut current = self.current.write().unwrap_or_else(PoisonError::into_inner);
        settings.instruction_sets = current.instruction_sets.clone();
        self.file.save(&settings)?;
        self.damaged.store(false, Ordering::Relaxed);
        *current = settings;
        Ok(())
    }

    /// Saves what the settings form edits: the provider, model and Custom
    /// base URL from `form`. Everything else in `form` is ignored, so the
    /// form can't undo the shortcut, start-with-Windows, the instruction
    /// sets or what the app remembers about updates.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] if the file can't be written. The current
    /// settings stay unchanged then.
    pub fn save_form(&self, form: Settings) -> Result<(), SettingsError> {
        self.modify(|current| {
            current.provider = form.provider;
            current.model = form.model;
            current.custom_base_url = form.custom_base_url;
        })
    }

    /// Applies `change`, something the app remembers by itself (such as the
    /// last update it announced), and saves it. While the file is damaged
    /// and the user hasn't saved over it, the change is kept in memory only,
    /// so the app never replaces a file the user might still repair. A
    /// failed save is logged and the change is still kept in memory.
    pub fn remember(&self, change: impl FnOnce(&mut Settings)) {
        let mut current = self.current.write().unwrap_or_else(PoisonError::into_inner);
        let mut changed = current.clone();
        change(&mut changed);
        if !self.damaged.load(Ordering::Relaxed) && self.file.save(&changed).is_err() {
            tracing::warn!("the settings file couldn't be written");
        }
        *current = changed;
    }

    /// Applies `change` to a copy of the current settings, saves it, then
    /// makes it current. Use it to change some settings without touching the
    /// rest.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] if the file can't be written. The current
    /// settings stay unchanged then.
    ///
    /// Instruction sets change only through
    /// [`AppSettings::change_instruction_sets`], which also tells listeners.
    pub fn modify(&self, change: impl FnOnce(&mut Settings)) -> Result<(), SettingsError> {
        self.try_modify(|settings| {
            change(settings);
            Ok::<_, SettingsError>(())
        })
    }

    /// The one save path: applies `change` to a copy of the current
    /// settings, saves it and makes it current, all under the write lock so
    /// concurrent changes never undo each other. If `change` fails, nothing
    /// is saved.
    fn try_modify<T, E: From<SettingsError>>(
        &self,
        change: impl FnOnce(&mut Settings) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut current = self.current.write().unwrap_or_else(PoisonError::into_inner);
        let mut changed = current.clone();
        let value = change(&mut changed)?;
        self.file.save(&changed)?;
        self.damaged.store(false, Ordering::Relaxed);
        *current = changed;
        Ok(value)
    }

    /// Applies `change` to the instruction sets and saves them. Every
    /// listener added with [`AppSettings::on_instruction_sets_change`] then
    /// hears about it. If `change` fails, nothing it did is kept.
    ///
    /// # Errors
    ///
    /// Returns [`ChangeError::Refused`] if `change` breaks an instruction set
    /// rule and [`ChangeError::Save`] if the file can't be written. The
    /// instruction sets stay unchanged then.
    pub fn change_instruction_sets<T>(
        &self,
        change: impl FnOnce(&mut InstructionSets) -> Result<T, InstructionSetError>,
    ) -> Result<T, ChangeError> {
        let (value, sets) = self.try_modify(|settings| {
            let value = change(&mut settings.instruction_sets)?;
            Ok::<_, ChangeError>((value, settings.instruction_sets.clone()))
        })?;
        // Outside the lock, so a listener can read the settings.
        for listener in self
            .listeners
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
        {
            listener(&sets);
        }
        Ok(value)
    }

    /// Calls `listener` with the new instruction sets after every change
    /// made through [`AppSettings::change_instruction_sets`]. The tray uses
    /// it to rebuild its menu, and the settings window to stay in step.
    pub fn on_instruction_sets_change(
        &self,
        listener: impl Fn(&InstructionSets) + Send + Sync + 'static,
    ) {
        self.listeners
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Box::new(listener));
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

    fn provider_readiness(&self) -> ProviderReadiness {
        let settings = self.settings();
        if settings.provider.requires_key() && !self.is_key_saved(settings.provider) {
            ProviderReadiness::MissingKey
        } else if settings.model.trim().is_empty() {
            ProviderReadiness::NoModel
        } else if settings.provider == ProviderKind::Custom
            && !is_http_url(&settings.custom_base_url)
        {
            ProviderReadiness::NoCustomUrl
        } else {
            ProviderReadiness::Ready
        }
    }

    fn active_instructions(&self) -> String {
        self.current
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .instruction_sets
            .active()
            .instructions
            .clone()
    }

    fn instruction_set_choices(&self) -> InstructionSetChoices {
        self.current
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .instruction_sets
            .choices()
    }

    /// Goes through [`AppSettings::change_instruction_sets`], the same path
    /// as the tray and the settings window, so both hear about it.
    fn set_active_instruction_set(&self, id: InstructionSetId) -> Result<(), ChangeError> {
        self.change_instruction_sets(|sets| sets.set_active(id))
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
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::{AppSettings, ChangeError, ProviderReadiness, Startup};
    use crate::settings::{
        ApiKey, InstructionSetError, InstructionSetId, InstructionSets, KeyError, KeyStore,
        Provider, ProviderKind, Settings, SettingsFile, SettingsView, StarterTemplate,
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

    fn open(path: &Path, keys: Arc<FakeKeys>) -> (AppSettings, Startup) {
        AppSettings::open(SettingsFile::new(path), keys)
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
            ..Settings::default()
        };
        let (app, _) = open(&path, Arc::default());
        app.update(saved.clone()).unwrap();

        let (restarted, startup) = open(&path, Arc::default());

        assert_eq!(startup, Startup::Loaded);
        assert_eq!(restarted.settings(), saved);
    }

    #[test]
    fn a_change_to_one_setting_keeps_the_others_and_survives_a_restart() {
        let path = temp_file();
        let (app, _) = open(&path, Arc::default());
        app.update(settings(ProviderKind::Gemini, "gemini-test"))
            .unwrap();

        app.modify(|s| s.shortcut = "Alt+Shift+R".to_owned())
            .unwrap();
        let (restarted, _) = open(&path, Arc::default());

        let expected = Settings {
            shortcut: "Alt+Shift+R".to_owned(),
            ..settings(ProviderKind::Gemini, "gemini-test")
        };
        assert_eq!(app.settings(), expected);
        assert_eq!(restarted.settings(), expected);
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
            ..Settings::default()
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

        assert_eq!(app.provider_readiness(), ProviderReadiness::Ready);
    }

    #[test]
    fn a_provider_that_needs_a_key_is_not_ready_without_one() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.set_key(ProviderKind::Gemini, &key("k")).unwrap();

        app.update(settings(ProviderKind::OpenAi, "gpt-test"))
            .unwrap();

        assert_eq!(app.provider_readiness(), ProviderReadiness::MissingKey);
    }

    #[test]
    fn a_provider_is_not_ready_without_a_model() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.set_key(ProviderKind::OpenAi, &key("k")).unwrap();

        app.update(settings(ProviderKind::OpenAi, "  ")).unwrap();

        assert_eq!(app.provider_readiness(), ProviderReadiness::NoModel);
    }

    #[test]
    fn a_missing_key_is_reported_before_a_missing_model() {
        let (app, _) = open(&temp_file(), Arc::default());

        app.update(settings(ProviderKind::OpenAi, "")).unwrap();

        assert_eq!(app.provider_readiness(), ProviderReadiness::MissingKey);
    }

    #[test]
    fn custom_without_a_model_reports_the_model_before_the_base_url() {
        let (app, _) = open(&temp_file(), Arc::default());

        app.update(settings(ProviderKind::Custom, " ")).unwrap();

        assert_eq!(app.provider_readiness(), ProviderReadiness::NoModel);
    }

    #[test]
    fn custom_is_ready_without_a_key_once_it_has_a_base_url() {
        let (app, _) = open(&temp_file(), Arc::default());
        let mut custom = settings(ProviderKind::Custom, "local-model");
        app.update(custom.clone()).unwrap();
        let without_url = app.provider_readiness();

        custom.custom_base_url = "http://localhost:1234/v1".to_owned();
        app.update(custom).unwrap();

        assert_eq!(without_url, ProviderReadiness::NoCustomUrl);
        assert_eq!(app.provider_readiness(), ProviderReadiness::Ready);
    }

    #[test]
    fn ollama_is_ready_without_a_key() {
        let (app, _) = open(&temp_file(), Arc::default());

        app.update(settings(ProviderKind::Ollama, "llama3.2"))
            .unwrap();

        assert_eq!(app.provider_readiness(), ProviderReadiness::Ready);
    }

    #[test]
    fn openrouter_is_not_ready_without_a_key() {
        let (app, _) = open(&temp_file(), Arc::default());

        app.update(settings(ProviderKind::OpenRouter, "openai/gpt-4.1-mini"))
            .unwrap();
        let without_key = app.provider_readiness();
        app.set_key(ProviderKind::OpenRouter, &key("k")).unwrap();

        assert_eq!(without_key, ProviderReadiness::MissingKey);
        assert_eq!(app.provider_readiness(), ProviderReadiness::Ready);
    }

    #[test]
    fn custom_is_not_ready_with_a_base_url_that_is_not_http() {
        let (app, _) = open(&temp_file(), Arc::default());

        app.update(Settings {
            provider: ProviderKind::Custom,
            model: "local-model".to_owned(),
            custom_base_url: "localhost:1234".to_owned(),
            ..Settings::default()
        })
        .unwrap();

        assert_eq!(app.provider_readiness(), ProviderReadiness::NoCustomUrl);
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

        assert_eq!(app.provider_readiness(), ProviderReadiness::MissingKey);
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

    fn id_of(app: &AppSettings, name: &str) -> InstructionSetId {
        app.settings()
            .instruction_sets
            .sets()
            .iter()
            .find(|set| set.name == name)
            .unwrap()
            .id
    }

    #[test]
    fn a_rewrite_follows_the_active_instruction_set() {
        let (app, _) = open(&temp_file(), Arc::default());
        let first = app.active_instructions();
        let shorter = id_of(&app, "Shorter");

        app.change_instruction_sets(|sets| sets.set_active(shorter))
            .unwrap();

        assert_eq!(first, StarterTemplate::FixAndMakeReadable.instructions());
        assert_eq!(
            app.active_instructions(),
            StarterTemplate::Shorter.instructions()
        );
    }

    #[test]
    fn instruction_sets_and_the_active_set_survive_a_restart() {
        let path = temp_file();
        let (app, _) = open(&path, Arc::default());
        let mine = app
            .change_instruction_sets(|sets| sets.add("Mine", "My rules."))
            .unwrap();
        app.change_instruction_sets(|sets| sets.set_active(mine))
            .unwrap();

        let (restarted, _) = open(&path, Arc::default());

        assert_eq!(
            restarted.settings().instruction_sets,
            app.settings().instruction_sets
        );
        assert_eq!(restarted.active_instructions(), "My rules.");
    }

    #[test]
    fn a_settings_file_from_before_instruction_sets_loads_the_starter_templates() {
        let path = temp_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, br#"{ "provider": "gemini", "model": "g" }"#).unwrap();

        let (app, _) = open(&path, Arc::default());

        assert_eq!(app.settings().instruction_sets, InstructionSets::default());
    }

    #[test]
    fn saving_the_provider_never_changes_the_instruction_sets() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.change_instruction_sets(|sets| sets.add("Mine", "My rules."))
            .unwrap();
        let before = app.settings().instruction_sets;

        // The window sends what it edits, with the starter templates as
        // the sets.
        app.update(settings(ProviderKind::Gemini, "gemini-test"))
            .unwrap();

        assert_eq!(app.settings().instruction_sets, before);
        assert_eq!(app.settings().provider, ProviderKind::Gemini);
    }

    #[test]
    fn a_refused_change_leaves_the_instruction_sets_as_they_were() {
        let path = temp_file();
        let (app, _) = open(&path, Arc::default());
        let before = app.settings().instruction_sets;

        let result = app.change_instruction_sets(|sets| {
            sets.add("Half done", "x")?;
            sets.add(" ", "blank name")
        });

        assert_eq!(
            result,
            Err(ChangeError::Refused(InstructionSetError::BlankName))
        );
        assert_eq!(app.settings().instruction_sets, before);
        let (restarted, _) = open(&path, Arc::default());
        assert_eq!(restarted.settings().instruction_sets, before);
    }

    #[test]
    fn every_instruction_set_change_is_announced_with_the_new_sets() {
        let (app, _) = open(&temp_file(), Arc::default());
        let heard = Arc::new(Mutex::new(Vec::new()));
        let listener = Arc::clone(&heard);
        app.on_instruction_sets_change(move |sets| {
            listener.lock().unwrap().push(sets.active().name.clone());
        });
        let shorter = id_of(&app, "Shorter");

        app.change_instruction_sets(|sets| sets.set_active(shorter))
            .unwrap();
        app.change_instruction_sets(|sets| {
            sets.restore_starter_templates();
            Ok(())
        })
        .unwrap();
        let _ = app.change_instruction_sets(|sets| sets.add("", "refused"));
        app.update(settings(ProviderKind::Gemini, "gemini-test"))
            .unwrap();

        assert_eq!(*heard.lock().unwrap(), ["Shorter", "Shorter"]);
    }

    #[test]
    fn a_set_made_active_through_the_view_is_saved_and_announced() {
        let path = temp_file();
        let (app, _) = open(&path, Arc::default());
        let heard = Arc::new(Mutex::new(Vec::new()));
        let listener = Arc::clone(&heard);
        app.on_instruction_sets_change(move |sets| {
            listener.lock().unwrap().push(sets.active().name.clone());
        });
        let shorter = id_of(&app, "Shorter");

        app.set_active_instruction_set(shorter).unwrap();

        assert_eq!(*heard.lock().unwrap(), ["Shorter"]);
        assert_eq!(app.instruction_set_choices().active, shorter);
        let (restarted, _) = open(&path, Arc::default());
        assert_eq!(restarted.instruction_set_choices().active, shorter);
    }

    #[test]
    fn the_views_choices_list_every_set_by_name_in_order() {
        let (app, _) = open(&temp_file(), Arc::default());
        app.change_instruction_sets(|sets| sets.add("Mine", "My rules."))
            .unwrap();

        let choices = app.instruction_set_choices();

        let names: Vec<_> = choices.sets.iter().map(|set| set.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Fix & make readable",
                "Fix mistakes only",
                "Professional",
                "Shorter",
                "Mine"
            ]
        );
        assert_eq!(choices.active, id_of(&app, "Fix & make readable"));
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

    #[test]
    fn saving_the_form_keeps_the_update_memory() {
        let path = temp_file();
        let (app, _) = open(&path, Arc::default());
        app.modify(|s| {
            s.updates.auto_check = false;
            s.updates.last_notified_version = Some("1.1.0".to_owned());
        })
        .unwrap();
        app.remember(|s| s.updates.last_run_version = Some("1.0.0".to_owned()));

        // The page sends only what it edits; the rest arrives as defaults.
        app.save_form(settings(ProviderKind::Gemini, "gemini-test"))
            .unwrap();
        let (restarted, _) = open(&path, Arc::default());

        let saved = restarted.settings();
        assert_eq!(saved.provider, ProviderKind::Gemini);
        assert_eq!(saved.model, "gemini-test");
        assert!(!saved.updates.auto_check);
        assert_eq!(
            saved.updates.last_notified_version.as_deref(),
            Some("1.1.0")
        );
        assert_eq!(saved.updates.last_run_version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn something_the_app_remembers_by_itself_is_saved() {
        let path = temp_file();
        let (app, _) = open(&path, Arc::default());

        app.remember(|s| s.updates.last_notified_version = Some("1.1.0".to_owned()));
        let (restarted, _) = open(&path, Arc::default());

        assert_eq!(
            restarted
                .settings()
                .updates
                .last_notified_version
                .as_deref(),
            Some("1.1.0")
        );
    }

    #[test]
    fn something_the_app_remembers_by_itself_never_overwrites_a_damaged_file() {
        let path = temp_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{ not json").unwrap();
        let (app, _) = open(&path, Arc::default());

        app.remember(|s| s.updates.last_notified_version = Some("1.1.0".to_owned()));

        assert_eq!(std::fs::read(&path).unwrap(), b"{ not json");
        assert_eq!(
            app.settings().updates.last_notified_version.as_deref(),
            Some("1.1.0")
        );
    }

    #[test]
    fn once_the_user_saves_over_a_damaged_file_the_app_remembers_again() {
        let path = temp_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{ not json").unwrap();
        let (app, _) = open(&path, Arc::default());
        app.save_form(settings(ProviderKind::Gemini, "gemini-test"))
            .unwrap();

        app.remember(|s| s.updates.last_notified_version = Some("1.1.0".to_owned()));
        let (restarted, _) = open(&path, Arc::default());

        assert_eq!(
            restarted
                .settings()
                .updates
                .last_notified_version
                .as_deref(),
            Some("1.1.0")
        );
    }
}
