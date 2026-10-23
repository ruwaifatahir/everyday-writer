//! The rewrite session's [`Rewriter`] port in the running app: each rewrite
//! goes to the provider and model saved in the settings at that moment.

use crate::llm::{GenaiRewriter, RewriteError, Rewriter};
use crate::settings::{ApiKeySource, SettingsView};

/// Builds a [`GenaiRewriter`] from the current settings for every rewrite,
/// so a change in Settings applies to the next rewrite.
pub struct LiveRewriter<S, K> {
    settings: S,
    keys: K,
}

impl<S, K> LiveRewriter<S, K> {
    /// Rewrites with the provider and model in `settings` and the key from
    /// `keys`.
    pub fn new(settings: S, keys: K) -> Self {
        Self { settings, keys }
    }
}

impl<S, K> Rewriter for LiveRewriter<S, K>
where
    S: SettingsView,
    K: ApiKeySource + Clone + 'static,
{
    async fn rewrite(&self, instructions: &str, text: &str) -> Result<String, RewriteError> {
        let rewriter = GenaiRewriter::new(self.settings.model_settings(), self.keys.clone())?;
        rewriter.rewrite(instructions, text).await
    }
}
