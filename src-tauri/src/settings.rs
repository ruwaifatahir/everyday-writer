//! Settings: the user's provider, model, shortcut and instruction sets, plus
//! keyring access for the API key.
//!
//! Privacy rule: the API key lives only in the OS keyring. Never log it or
//! write it to a settings file.

use std::collections::HashMap;

use keyring_core::Entry;

/// The LLM provider the user picked. Never inferred from the model name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// `OpenAI`.
    OpenAi,
    /// Anthropic (Claude).
    Anthropic,
    /// Google Gemini.
    Gemini,
    /// Any endpoint that speaks the `OpenAI` chat completions API.
    Custom {
        /// Base URL, for example `https://api.example.com/v1/`.
        base_url: String,
    },
}

impl Provider {
    /// The account name of this provider's API key in the keyring.
    pub fn key_name(&self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Custom { .. } => "custom",
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

    /// Saves `key` for `provider`, on this machine only (not roaming).
    ///
    /// # Errors
    ///
    /// Returns [`KeyError`] if the store can't be written.
    pub fn set_api_key(&self, provider: &Provider, key: &ApiKey) -> Result<(), KeyError> {
        self.entry(provider)?.set_password(key.expose_secret())?;
        Ok(())
    }

    /// Deletes the key for `provider`. Succeeds if there was none.
    ///
    /// # Errors
    ///
    /// Returns [`KeyError`] if the store can't be written.
    pub fn delete_api_key(&self, provider: &Provider) -> Result<(), KeyError> {
        match self.entry(provider)?.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// The one way to address `provider`'s credential, so get, set and delete
    /// always reach the same entry. `persistence = Local` keeps it on this
    /// machine (not roaming).
    fn entry(&self, provider: &Provider) -> Result<Entry, KeyError> {
        let modifiers = HashMap::from([("persistence", "Local")]);
        Ok(Entry::new_with_modifiers(
            &self.service,
            provider.key_name(),
            &modifiers,
        )?)
    }
}

impl ApiKeySource for KeyringApiKeys {
    fn api_key(&self, provider: &Provider) -> Result<Option<ApiKey>, KeyError> {
        match self.entry(provider)?.get_password() {
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

    use super::{ApiKey, ApiKeySource, KeyringApiKeys, Provider};

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

        keys.set_api_key(&Provider::Gemini, &ApiKey::new("k-1".to_owned()))
            .unwrap();
        let saved = keys.api_key(&Provider::Gemini).unwrap();
        keys.delete_api_key(&Provider::Gemini).unwrap();
        let after_delete = keys.api_key(&Provider::Gemini).unwrap();
        keyring_core::unset_default_store();

        assert_eq!(saved.as_ref().map(ApiKey::expose_secret), Some("k-1"));
        assert!(after_delete.is_none());
        let expected: Spec = (
            "test.service".to_owned(),
            "gemini".to_owned(),
            vec![("persistence".to_owned(), "Local".to_owned())],
        );
        let built = built.lock().unwrap();
        assert_eq!(built.len(), 4);
        assert!(built.iter().all(|spec| *spec == expected));
    }
}
