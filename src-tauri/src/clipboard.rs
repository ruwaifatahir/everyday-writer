//! The Windows clipboard: read and write text, save and restore everything
//! the user had copied, and watch for changes.
//!
//! The rules (which formats are kept, restore order, privacy tags) live in
//! [`snapshot`]; this module only connects them to the real clipboard.
//! Every write is tagged to stay out of Windows clipboard history and the
//! cloud clipboard.
//!
//! Privacy: clipboard contents are user data. Never log them.

pub mod snapshot;

use std::num::NonZeroU32;
use std::time::Duration;

use clipboard_win::formats::CF_UNICODETEXT;
use clipboard_win::raw;

use crate::session::BridgeError;
use snapshot::{ClipboardAccess, Snapshot, retry};

/// How often opening and closing the clipboard is tried while another app
/// holds it.
const ATTEMPTS: usize = 10;

/// The pause between two tries.
const RETRY_PAUSE: Duration = Duration::from_millis(10);

/// The clipboard's change counter. It changes whenever anything is copied.
pub fn sequence_number() -> Option<NonZeroU32> {
    raw::seq_num()
}

/// Saves everything on the clipboard that can be put back.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be opened.
pub fn save() -> Result<Snapshot, BridgeError> {
    let clipboard = OpenClipboard::open()?;
    let snapshot = Snapshot::take(&clipboard);
    clipboard.close()?;
    Ok(snapshot)
}

/// Puts the saved clipboard back, kept out of clipboard history.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be written.
pub fn restore(snapshot: &Snapshot) -> Result<(), BridgeError> {
    let mut clipboard = OpenClipboard::open()?;
    snapshot.restore(&mut clipboard)?;
    clipboard.close()
}

/// The clipboard's text, or `None` if it holds none.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be opened or read.
pub fn read_text() -> Result<Option<String>, BridgeError> {
    let clipboard = OpenClipboard::open()?;
    if !raw::is_format_avail(CF_UNICODETEXT) {
        return clipboard.close().map(|()| None);
    }
    let mut bytes = Vec::new();
    let read = raw::get_string(&mut bytes);
    clipboard.close()?;
    read.map_err(|_| BridgeError)?;
    String::from_utf8(bytes).map(Some).map_err(|_| BridgeError)
}

/// Replaces the clipboard's contents with `text`, kept out of clipboard
/// history.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be written.
pub fn write_text(text: &str) -> Result<(), BridgeError> {
    let mut clipboard = OpenClipboard::open()?;
    snapshot::write_text(&mut clipboard, text)?;
    clipboard.close()
}

/// The clipboard, opened by this thread. Call [`OpenClipboard::close`] to
/// hand the changes to Windows; dropping it closes without retrying.
struct OpenClipboard {
    open: bool,
}

impl OpenClipboard {
    /// Opens the clipboard, retrying while another app holds it.
    fn open() -> Result<Self, BridgeError> {
        retry(ATTEMPTS, RETRY_PAUSE, || {
            raw::open().map_err(|_| BridgeError)
        })?;
        Ok(Self { open: true })
    }

    /// Closes the clipboard, which publishes what was written, retrying if
    /// Windows refuses.
    fn close(mut self) -> Result<(), BridgeError> {
        self.open = false;
        retry(ATTEMPTS, RETRY_PAUSE, || {
            raw::close().map_err(|_| BridgeError)
        })
    }
}

impl Drop for OpenClipboard {
    fn drop(&mut self) {
        if self.open {
            let _ = raw::close();
        }
    }
}

impl ClipboardAccess for OpenClipboard {
    fn formats(&self) -> Vec<u32> {
        raw::EnumFormats::new().collect()
    }

    fn read(&self, format: u32) -> Option<Vec<u8>> {
        let mut data = Vec::new();
        let read = raw::get_vec(format, &mut data).ok()?;
        (read > 0).then_some(data)
    }

    fn empty(&mut self) -> Result<(), BridgeError> {
        raw::empty().map_err(|_| BridgeError)
    }

    fn write(&mut self, format: u32, data: &[u8]) -> Result<(), BridgeError> {
        raw::set_without_clear(format, data).map_err(|_| BridgeError)
    }

    fn format_name(&self, format: u32) -> Option<String> {
        raw::format_name_big(format)
    }

    fn register(&self, name: &str) -> Option<u32> {
        raw::register_format(name).map(NonZeroU32::get)
    }
}
