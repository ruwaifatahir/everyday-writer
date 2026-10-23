//! The Windows text bridge: the rewrite session's [`TextBridge`] port,
//! built on [`crate::capture`], [`crate::replace`] and [`crate::clipboard`].
//!
//! Clipboard and keyboard calls block, so each runs on a blocking thread.

use crate::capture;
use crate::clipboard;
use crate::replace;
use crate::session::{BridgeError, Captured, TextBridge};
use crate::win32::SourceWindow;

/// Captures, replaces and copies text with Win32.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsTextBridge;

impl TextBridge for WindowsTextBridge {
    type Window = SourceWindow;

    async fn capture(&self) -> Option<Captured<SourceWindow>> {
        let captured = tokio::task::spawn_blocking(capture::capture).await;
        if let Ok(Ok(captured)) = captured {
            captured
        } else {
            tracing::warn!("the selection couldn't be captured");
            None
        }
    }

    async fn copy(&self, text: &str) -> Result<(), BridgeError> {
        let text = text.to_owned();
        tokio::task::spawn_blocking(move || clipboard::write_text(&text))
            .await
            .map_err(|_| BridgeError)?
    }

    fn is_open(&self, window: &SourceWindow) -> bool {
        window.is_open()
    }

    async fn replace(&self, window: &SourceWindow, text: &str) -> Result<(), BridgeError> {
        let window = *window;
        let text = text.to_owned();
        tokio::task::spawn_blocking(move || replace::replace(window, &text))
            .await
            .map_err(|_| BridgeError)?
    }
}
