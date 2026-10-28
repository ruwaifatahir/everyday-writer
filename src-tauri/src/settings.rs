//! Settings: the user's provider, model, shortcut and instruction sets, plus
//! keyring access for the API key.
//!
//! Privacy rule: the API key lives only in the OS keyring. Never log it or
//! write it to a settings file.

mod instruction_sets;
mod store;
mod view;

use std::collections::HashMap;

use keyring_core::Entry;
use serde::{Deserialize, Serialize};

use crate::updates::Remembered;

pub use instruction_sets::{
    InstructionSet, InstructionSetChoice, InstructionSetChoices, InstructionSetError,
    InstructionSetId, InstructionSets, StarterTemplate,
};
pub use store::{SettingsError, SettingsFile};
pub use view::{AppSettings, ChangeError, ProviderReadiness, SaveKeyError, SettingsView, Startup};

/// Which provider the user picked, without its details. Keys are stored per
/// kind, so each provider keeps its own key.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// `OpenAI`.
    OpenAi,
    /// Anthropic (Claude).
    Anthropic,
    /// Google Gemini. The default: it has a free tier, so a new user can
    /// start without paying.
    #[default]
    Gemini,
    /// `OpenRouter`.
    OpenRouter,
    /// Ollama on this computer.
    Ollama,
    /// Any endpoint that speaks the `OpenAI` chat completions API.
    Custom,
}

impl ProviderKind {
    /// Every provider, in the order Settings lists them.
    pub const ALL: [Self; 6] = [
        Self::OpenAi,
        Self::Anthropic,
        Self::Gemini,
        Self::OpenRouter,
        Self::Ollama,
        Self::Custom,
    ];

    /// The account name of this provider's API key in the keyring.
    pub fn key_name(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::OpenRouter => "openrouter",
            Self::Ollama => "ollama",
            Self::Custom => "custom",
        }
    }

    /// The provider's name inside a sentence, as in "Couldn't reach Gemini".
    /// Matches `PROVIDER_NAMES` in `src/settings/main.ts`, except Custom,
    /// which reads as "your server".
    pub fn display_name(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::Gemini => "Gemini",
            Self::OpenRouter => "OpenRouter",
            Self::Ollama => "Ollama",
            Self::Custom => "your server",
        }
    }

    /// The provider's name at the start of a sentence, as in "Gemini is
    /// busy" or "Your server is busy".
    pub fn display_subject(self) -> &'static str {
        match self {
            Self::Custom => "Your server",
            other => other.display_name(),
        }
    }

    /// Whether a rewrite needs a saved key. Ollama and Custom work without
    /// one, so local servers work. A blank key there is sent as an empty
    /// key, never looked up anywhere else (ADR 0001 amendment).
    pub fn requires_key(self) -> bool {
        !matches!(self, Self::Ollama | Self::Custom)
    }
}

/// What the settings file holds. Never an API key: those live only in the
/// keyring. Missing fields load as their defaults, so files written by older
/// versions still load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// The chosen provider.
    pub provider: ProviderKind,
    /// The model name exactly as the provider expects it. Missing from a
    /// file it loads empty, not as the fresh-install model, which belongs
    /// to the default provider and may not match the saved one.
    #[serde(default)]
    pub model: String,
    /// The Custom provider's base URL. Kept while another provider is
    /// chosen, so switching back doesn't lose it.
    pub custom_base_url: String,
    /// The instruction sets and the active one. A file without them (first
    /// launch, or from before instruction sets) gets the starter templates.
    pub instruction_sets: InstructionSets,
    /// The rewrite shortcut as the user chose it, for example
    /// `Ctrl+Shift+Space`.
    pub shortcut: String,
    /// Whether the app starts when the user signs in to Windows.
    pub start_with_windows: bool,
    /// Whether the shortcut starts the rewrite at once. Off, it opens the
    /// popup on Ready and nothing is sent until the user presses Rewrite.
    /// A file from before this setting loads with it on.
    pub rewrite_on_open: bool,
    /// What the update controller keeps between runs, stored as top-level
    /// fields of the file.
    #[serde(flatten)]
    pub updates: Remembered,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: ProviderKind::default(),
            model: crate::llm::first_model(ProviderKind::default())
                .unwrap_or_default()
                .to_owned(),
            custom_base_url: String::new(),
            shortcut: crate::shortcut::DEFAULT_SHORTCUT.to_owned(),
            start_with_windows: true,
            rewrite_on_open: true,
            updates: Remembered::default(),
            instruction_sets: InstructionSets::default(),
        }
    }
}

impl Settings {
    /// The provider and model a rewrite uses.
    pub fn model_settings(&self) -> ModelSettings {
        ModelSettings {
            provider: Provider::from_kind(self.provider, &self.custom_base_url),
            model: self.model.trim().to_owned(),
        }
    }
}

/// The LLM provider the user picked. Never inferred from the model name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// `OpenAI`.
    OpenAi,
    /// Anthropic (Claude).
    Anthropic,
    /// Google Gemini.
    Gemini,
    /// `OpenRouter`.
    OpenRouter,
    /// Ollama on this computer, at its default address.
    Ollama,
    /// Any endpoint that speaks the `OpenAI` chat completions API.
    Custom {
        /// Base URL, for example `https://api.example.com/v1/`.
        base_url: String,
    },
}

impl Provider {
    /// The provider `kind`, with `custom_base_url` (trimmed) for Custom. The
    /// URL is ignored for every other kind.
    pub fn from_kind(kind: ProviderKind, custom_base_url: &str) -> Self {
        match kind {
            ProviderKind::OpenAi => Self::OpenAi,
            ProviderKind::Anthropic => Self::Anthropic,
            ProviderKind::Gemini => Self::Gemini,
            ProviderKind::OpenRouter => Self::OpenRouter,
            ProviderKind::Ollama => Self::Ollama,
            ProviderKind::Custom => Self::Custom {
                base_url: custom_base_url.trim().to_owned(),
            },
        }
    }

    /// The account name of this provider's API key in the keyring.
    pub fn key_name(&self) -> &'static str {
        self.kind().key_name()
    }

    /// Whether a rewrite needs a saved key. See [`ProviderKind::requires_key`].
    pub fn requires_key(&self) -> bool {
        self.kind().requires_key()
    }

    /// Which provider this is, without its details.
    pub fn kind(&self) -> ProviderKind {
        match self {
            Self::OpenAi => ProviderKind::OpenAi,
            Self::Anthropic => ProviderKind::Anthropic,
            Self::Gemini => ProviderKind::Gemini,
            Self::OpenRouter => ProviderKind::OpenRouter,
            Self::Ollama => ProviderKind::Ollama,
            Self::Custom { .. } => ProviderKind::Custom,
        }
    }
}

/// The provider and model every instruction set uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSettings {
    /// The chosen provider.
    pub provider: Provider,
    /// The model name exactly as the provider expects it.
    pub model: String,
}

/// The credential store failed. Holds no detail: keyring errors can carry
/// the raw secret bytes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the credential store failed")]
pub struct KeyError;

impl From<keyring_core::Error> for KeyError {
    fn from(_: keyring_core::Error) -> Self {
        Self
    }
}

/// Where API keys come from. Lets tests supply keys without the OS keyring.
pub trait ApiKeySource: Send + Sync {
    /// The saved API key for `provider`, or `None` if there is none.
    ///
    /// # Errors
    ///
    /// Returns [`KeyError`] if the store can't be read.
    fn api_key(&self, provider: &Provider) -> Result<Option<ApiKey>, KeyError>;
}

/// An API key. Its `Debug` output is redacted, and it has no `Display` or
/// `Serialize`, so it can't end up in a log or a settings file by accident.
/// Read the secret only with [`ApiKey::expose_secret`], where a provider
/// needs it.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Wraps `key`.
    pub fn new(key: String) -> Self {
        Self(key)
    }

    /// The raw key. Call only to hand it to the provider or the keyring.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

/// Installs the Windows Credential Manager as the keyring store.
///
/// Call once at startup, before any [`KeyringApiKeys`] access.
///
/// # Errors
///
/// Returns [`KeyError`] if the store can't be created.
#[cfg(windows)]
pub fn init_keyring() -> Result<(), KeyError> {
    keyring_core::set_default_store(windows_native_keyring_store::Store::new()?);
    Ok(())
}

/// API keys in the OS keyring, one entry per provider.
#[derive(Debug, Clone)]
pub struct KeyringApiKeys {
    service: String,
}

impl KeyringApiKeys {
    /// Keys stored under `service`. Pass the app identifier from the Tauri
    /// config so the name has one source.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    /// The one way to address `provider`'s credential, so get, set and delete
    /// always reach the same entry. `persistence = Local` keeps it on this
    /// machine (not roaming).
    fn entry(&self, provider: ProviderKind) -> Result<Entry, KeyError> {
        let modifiers = HashMap::from([("persistence", "Local")]);
        Ok(Entry::new_with_modifiers(
            &self.service,
            provider.key_name(),
            &modifiers,
        )?)
    }
}

/// Saves, checks and removes API keys, one per provider. Lets tests use an
/// in-memory store instead of the OS keyring.
pub trait KeyStore: Send + Sync {
    /// Whether `provider` has a saved key. Never returns the key itself.
    ///
    /// # Errors
    ///
    /// Returns [`KeyError`] if the store can't be read.
    fn has_key(&self, provider: ProviderKind) -> Result<bool, KeyError>;

    /// Saves `key` for `provider`, replacing any earlier one.
    ///
    /// # Errors
    ///
    /// Returns [`KeyError`] if the store can't be written.
    fn set_key(&self, provider: ProviderKind, key: &ApiKey) -> Result<(), KeyError>;

    /// Deletes `provider`'s key. Succeeds if there was none.
    ///
    /// # Errors
    ///
    /// Returns [`KeyError`] if the store can't be written.
    fn delete_key(&self, provider: ProviderKind) -> Result<(), KeyError>;
}

impl KeyStore for KeyringApiKeys {
    fn has_key(&self, provider: ProviderKind) -> Result<bool, KeyError> {
        match self.entry(provider)?.get_password() {
            Ok(_) => Ok(true),
            Err(keyring_core::Error::NoEntry) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Saves on this machine only (not roaming).
    fn set_key(&self, provider: ProviderKind, key: &ApiKey) -> Result<(), KeyError> {
        self.entry(provider)?.set_password(key.expose_secret())?;
        Ok(())
    }

    fn delete_key(&self, provider: ProviderKind) -> Result<(), KeyError> {
        match self.entry(provider)?.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl ApiKeySource for KeyringApiKeys {
    fn api_key(&self, provider: &Provider) -> Result<Option<ApiKey>, KeyError> {
        match self.entry(provider.kind())?.get_password() {
            Ok(key) => Ok(Some(ApiKey::new(key))),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use keyring_core::api::CredentialStoreApi;
    use keyring_core::{Entry, mock};

    use super::{ApiKey, ApiKeySource, KeyStore, KeyringApiKeys, Provider, ProviderKind};

    #[test]
    fn custom_is_named_your_server_in_messages() {
        let name = ProviderKind::Custom.display_name();
        let subject = ProviderKind::Custom.display_subject();

        assert_eq!((name, subject), ("your server", "Your server"));
    }

    #[test]
    fn a_named_provider_reads_the_same_anywhere_in_a_sentence() {
        let name = ProviderKind::OpenAi.display_name();
        let subject = ProviderKind::OpenAi.display_subject();

        assert_eq!((name, subject), ("OpenAI", "OpenAI"));
    }

    #[test]
    fn custom_is_built_with_its_trimmed_base_url() {
        let provider = Provider::from_kind(ProviderKind::Custom, " http://localhost:1234/v1 ");

        assert_eq!(
            provider,
            Provider::Custom {
                base_url: "http://localhost:1234/v1".to_owned()
            }
        );
    }

    #[test]
    fn debug_output_hides_the_api_key() {
        let key = ApiKey::new("sk-SECRET-123".to_owned());

        let debug = format!("{key:?}");

        assert!(!debug.contains("sk-SECRET-123"));
    }

    #[test]
    fn the_secret_is_reachable_through_expose_secret() {
        let key = ApiKey::new("sk-SECRET-123".to_owned());

        assert_eq!(key.expose_secret(), "sk-SECRET-123");
    }

    /// One `build` call: service, account and modifiers, sorted.
    type Spec = (String, String, Vec<(String, String)>);

    /// Records how each entry is addressed, then hands out an in-memory mock
    /// credential, so tests never touch the OS keyring.
    struct RecordingStore {
        inner: Arc<mock::Store>,
        built: Arc<Mutex<Vec<Spec>>>,
    }

    impl CredentialStoreApi for RecordingStore {
        fn vendor(&self) -> String {
            "recording".to_owned()
        }

        fn id(&self) -> String {
            "recording".to_owned()
        }

        fn build(
            &self,
            service: &str,
            user: &str,
            mods: Option<&HashMap<&str, &str>>,
        ) -> keyring_core::Result<Entry> {
            let mut mods: Vec<(String, String)> = mods
                .into_iter()
                .flatten()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect();
            mods.sort();
            self.built
                .lock()
                .unwrap()
                .push((service.to_owned(), user.to_owned(), mods));
            self.inner.build(service, user, None)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    /// The only test that installs a default store, so it can't race others.
    #[test]
    fn set_get_and_delete_address_the_same_local_credential() {
        let built = Arc::new(Mutex::new(Vec::new()));
        keyring_core::set_default_store(Arc::new(RecordingStore {
            inner: mock::Store::new().unwrap(),
            built: Arc::clone(&built),
        }));
        let keys = KeyringApiKeys::new("test.service");

        keys.set_key(ProviderKind::Gemini, &ApiKey::new("k-1".to_owned()))
            .unwrap();
        let saved = keys.api_key(&Provider::Gemini).unwrap();
        let has_key = keys.has_key(ProviderKind::Gemini).unwrap();
        keys.delete_key(ProviderKind::Gemini).unwrap();
        let after_delete = keys.api_key(&Provider::Gemini).unwrap();
        let has_key_after_delete = keys.has_key(ProviderKind::Gemini).unwrap();
        keyring_core::unset_default_store();

        assert_eq!(saved.as_ref().map(ApiKey::expose_secret), Some("k-1"));
        assert!(has_key);
        assert!(after_delete.is_none());
        assert!(!has_key_after_delete);
        let expected: Spec = (
            "test.service".to_owned(),
            "gemini".to_owned(),
            vec![("persistence".to_owned(), "Local".to_owned())],
        );
        let built = built.lock().unwrap();
        assert_eq!(built.len(), 6);
        assert!(built.iter().all(|spec| *spec == expected));
    }
}
