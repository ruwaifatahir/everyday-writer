//! The Windows clipboard: read and write text, save and restore what the
//! user had copied, and watch for changes.
//!
//! Every write is tagged to stay out of Windows clipboard history and the
//! cloud clipboard.
//!
//! Privacy: clipboard contents are user data. Never log them.
//!
//! Saving and restoring covers text only for now; other formats (images,
//! files, rich text) are not preserved yet.

use std::num::NonZeroU32;

use clipboard_win::formats::CF_UNICODETEXT;
use clipboard_win::{Clipboard, raw};

use crate::session::BridgeError;

/// How often to retry opening the clipboard when another app holds it.
/// clipboard-win waits 5 ms between attempts.
const OPEN_ATTEMPTS: usize = 10;

/// Formats that keep a write out of clipboard history, the cloud clipboard
/// and clipboard managers. Each gets a DWORD 0.
const PRIVATE_FORMATS: [&str; 3] = [
    "ExcludeClipboardContentFromMonitorProcessing",
    "CanIncludeInClipboardHistory",
    "CanUploadToCloudClipboard",
];

/// The text the user had on the clipboard. No `Debug`: it holds user data.
pub struct TextSnapshot(Option<String>);

/// The clipboard's change counter. It changes whenever anything is copied.
pub fn sequence_number() -> Option<NonZeroU32> {
    raw::seq_num()
}

/// Saves the clipboard's text.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be opened.
pub fn save() -> Result<TextSnapshot, BridgeError> {
    Ok(TextSnapshot(read_text()?))
}

/// Puts the saved text back. A clipboard that held no text is left empty.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be written.
pub fn restore(snapshot: &TextSnapshot) -> Result<(), BridgeError> {
    if let Some(text) = &snapshot.0 {
        return write_text(text);
    }
    let _clipboard = open()?;
    raw::empty().map_err(|_| BridgeError)?;
    tag_private();
    Ok(())
}

/// The clipboard's text, or `None` if it holds none.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be opened or read.
pub fn read_text() -> Result<Option<String>, BridgeError> {
    let _clipboard = open()?;
    if !raw::is_format_avail(CF_UNICODETEXT) {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    raw::get_string(&mut bytes).map_err(|_| BridgeError)?;
    String::from_utf8(bytes).map(Some).map_err(|_| BridgeError)
}

/// Replaces the clipboard's contents with `text`, kept out of clipboard
/// history.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be written.
pub fn write_text(text: &str) -> Result<(), BridgeError> {
    let _clipboard = open()?;
    raw::set_string(text).map_err(|_| BridgeError)?;
    tag_private();
    Ok(())
}

/// Opens the clipboard, retrying while another app holds it. It closes
/// when the returned guard is dropped.
fn open() -> Result<Clipboard, BridgeError> {
    Clipboard::new_attempts(OPEN_ATTEMPTS).map_err(|_| BridgeError)
}

/// Marks the open clipboard's contents as private. Best effort: a failed
/// tag only means the text may show up in history.
fn tag_private() {
    let zero = 0_u32.to_ne_bytes();
    for name in PRIVATE_FORMATS {
        if let Some(format) = raw::register_format(name) {
            let _ = raw::set_without_clear(format.get(), &zero);
        }
    }
}
