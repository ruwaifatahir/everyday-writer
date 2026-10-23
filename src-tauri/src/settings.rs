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
    fn api_key(&self, provider: &Provider) -> Result<Option<String>, KeyError>;
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
    pub fn set_api_key(&self, provider: &Provider, key: &str) -> Result<(), KeyError> {
        let modifiers = HashMap::from([("persistence", "Local")]);
        Entry::new_with_modifiers(&self.service, provider.key_name(), &modifiers)?
            .set_password(key)?;
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

    fn entry(&self, provider: &Provider) -> Result<Entry, KeyError> {
        Ok(Entry::new(&self.service, provider.key_name())?)
    }
}

impl ApiKeySource for KeyringApiKeys {
    fn api_key(&self, provider: &Provider) -> Result<Option<String>, KeyError> {
        match self.entry(provider)?.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
