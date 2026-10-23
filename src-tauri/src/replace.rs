//! Text replacement: puts the result in place of the original
//! selection in the window it came from.
//!
//! 1. Bring the original window back to the front.
//! 2. Save the clipboard and write the result to it, kept out of history.
//! 3. Send Ctrl+V as virtual-key codes.
//! 4. About [`PASTE_WAIT`] later, restore the clipboard.
//!
//! Privacy rule: the result and the clipboard contents are user data.
//! Never log them, and restore the user's clipboard after replacing.

use std::thread;
use std::time::Duration;

use crate::clipboard;
use crate::session::BridgeError;
use crate::win32::{self, SourceWindow};

/// How long the target app gets to read the clipboard after Ctrl+V. Some
/// apps (Chrome, Electron) read it asynchronously.
const PASTE_WAIT: Duration = Duration::from_millis(300);

/// Pastes `text` into `window` in place of its selection. Blocks for about
/// a third of a second, so call it off the async runtime.
///
/// # Errors
///
/// Returns [`BridgeError`] if the window can't be brought to the front, the
/// clipboard can't be used or Windows refuses the key presses. Nothing was
/// pasted then.
pub fn replace(window: SourceWindow, text: &str) -> Result<(), BridgeError> {
    // Never paste into whatever app happens to be in front instead.
    if !window.bring_to_front() {
        return Err(BridgeError);
    }
    let saved = clipboard::save()?;
    if clipboard::write_text(text).is_err() {
        // A failed write may have emptied the clipboard: put the user's back.
        if clipboard::restore(&saved).is_err() {
            tracing::warn!("the clipboard couldn't be restored after a failed write");
        }
        return Err(BridgeError);
    }
    let written = clipboard::sequence_number();
    let pasted = win32::release_held_modifiers() && win32::send_paste();
    if pasted {
        thread::sleep(PASTE_WAIT);
    }
    // If the user copied something meanwhile, keep it rather than restore.
    if clipboard::sequence_number() == written {
        clipboard::restore(&saved)?;
    }
    if pasted { Ok(()) } else { Err(BridgeError) }
}
