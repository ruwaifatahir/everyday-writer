//! Logging setup for the background app.

use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Crates that send requests to the LLM provider. Their logs can describe
/// requests, so they are capped at `WARN`.
const NETWORK_CRATES: [&str; 4] = ["genai", "reqwest", "hyper", "hyper_util"];

/// Installs the global `tracing` subscriber.
///
/// Privacy rule: log events and fields must never include the selection,
/// clipboard contents, prompts, LLM responses or API keys. Log only what
/// happened (for example "capture failed") and non-content metadata such as
/// lengths or error kinds.
///
/// Everything is capped at `INFO`, so no crate's debug or trace logs, which
/// may include request details, are emitted. The HTTP and LLM crates
/// (`genai`, `reqwest`, `hyper`, `hyper_util`) are capped at `WARN`. The caps
/// limit dependency noise; they don't make text logging safe, so the rule
/// above still applies to every log call in this app.
///
/// # Panics
///
/// Panics if a global subscriber is already installed.
pub fn init() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish()
        .with(level_filter())
        .init();
}

/// `INFO` for everything except the network crates, which get `WARN`.
fn level_filter() -> Targets {
    NETWORK_CRATES.into_iter().fold(
        Targets::new().with_default(Level::INFO),
        |filter, crate_name| filter.with_target(crate_name, Level::WARN),
    )
}

#[cfg(test)]
mod tests {
    use tracing::Level;

    use super::level_filter;

    #[test]
    fn app_logs_at_info_but_not_debug() {
        let filter = level_filter();

        assert!(filter.would_enable("everyday_writer_lib::tray", &Level::INFO));
        assert!(!filter.would_enable("everyday_writer_lib::tray", &Level::DEBUG));
    }

    #[test]
    fn http_and_llm_crates_log_only_warnings_and_errors() {
        let filter = level_filter();

        for target in ["genai", "reqwest", "hyper", "hyper_util::client::legacy"] {
            assert!(!filter.would_enable(target, &Level::INFO), "{target}");
            assert!(filter.would_enable(target, &Level::WARN), "{target}");
        }
    }
}
