//! Text capture: reads the selection in the foreground app.
//!
//! It copies the selection with a synthetic Ctrl+C and reads it from the
//! clipboard, the approach Microsoft `PowerToys` uses:
//!
//! 1. Save the clipboard.
//! 2. Release modifier keys still held from the shortcut.
//! 3. Send Ctrl+C as virtual-key codes.
//! 4. Wait up to [`COPY_WAIT`] for the clipboard to change. No change means
//!    nothing was selected.
//! 5. Read the text, then restore the clipboard.
//!
//! Privacy rule: the selection and the clipboard contents are user data.
//! Never log them, and restore the user's clipboard after capture.

use std::thread;
use std::time::{Duration, Instant};

use crate::clipboard;
use crate::session::{BridgeError, Captured};
use crate::win32::{self, SourceWindow};

/// How long the foreground app gets to copy the selection.
const COPY_WAIT: Duration = Duration::from_millis(500);

/// How often the clipboard is checked while waiting.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Captures the selection in the foreground app. Blocks for up to about
/// half a second, so call it off the async runtime.
///
/// Returns `Ok(None)` when nothing is selected.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be used or Windows
/// refuses the synthetic key presses.
pub fn capture() -> Result<Option<Captured<SourceWindow>>, BridgeError> {
    let Some(window) = SourceWindow::foreground() else {
        return Ok(None);
    };
    let saved = clipboard::save()?;
    let before = clipboard::sequence_number();
    if !win32::release_held_modifiers() || !win32::send_copy() {
        return Err(BridgeError);
    }
    if !wait_for_change(before) {
        // Nothing was copied, so the clipboard is untouched.
        return Ok(None);
    }
    let selection = clipboard::read_text();
    let restored = clipboard::restore(&saved);
    let selection = selection?;
    restored?;
    Ok(selection.map(|selection| Captured { selection, window }))
}

/// Waits until the clipboard's sequence number differs from `before`.
/// Returns `false` if it doesn't within [`COPY_WAIT`].
fn wait_for_change(before: Option<std::num::NonZeroU32>) -> bool {
    let deadline = Instant::now() + COPY_WAIT;
    loop {
        if clipboard::sequence_number() != before {
            // The app may still be adding formats; give it a moment.
            thread::sleep(POLL_INTERVAL);
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(POLL_INTERVAL);
    }
}
