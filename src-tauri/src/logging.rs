//! Logging setup for the background app.

use tracing::Level;

/// Installs the global `tracing` subscriber.
///
/// Privacy rule: log events and fields must never include selected text,
/// clipboard contents, prompts, LLM responses or API keys. Log only what
/// happened (for example "capture failed") and non-content metadata such as
/// lengths or error kinds.
///
/// The level is capped at `INFO` so dependency debug logs, which may include
/// request details, are never emitted.
///
/// # Panics
///
/// Panics if a global subscriber is already installed.
pub fn init() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
}
