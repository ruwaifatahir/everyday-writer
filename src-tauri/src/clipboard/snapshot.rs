//! Saving and restoring everything the user had on the clipboard, and
//! writing text to it, behind the [`ClipboardAccess`] seam so the rules are
//! testable without Windows.
//!
//! - Every memory-based format is saved and restored in the order Windows
//!   listed it: text, rich text, HTML, file lists, DIB and PNG images and
//!   registered formats.
//! - GDI-handle formats and formats Windows synthesizes are skipped. This is
//!   an accepted gap.
//! - Every write is tagged to stay out of clipboard history, the cloud
//!   clipboard and clipboard managers.
//!
//! Privacy: clipboard contents are user data. Never log them.

use std::thread;
use std::time::Duration;

use crate::session::BridgeError;

/// An open clipboard, seen as numbered formats holding bytes.
pub trait ClipboardAccess {
    /// Every format on the clipboard, in the order Windows lists them.
    fn formats(&self) -> Vec<u32>;
    /// Reads one format's bytes, or `None` if it can't be read.
    fn read(&self, format: u32) -> Option<Vec<u8>>;
    /// Removes everything from the clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError`] if Windows refuses.
    fn empty(&mut self) -> Result<(), BridgeError>;
    /// Adds one format without removing the others.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError`] if Windows refuses.
    fn write(&mut self, format: u32, data: &[u8]) -> Result<(), BridgeError>;
    /// The name of a registered format, or `None` for a standard one.
    fn format_name(&self, format: u32) -> Option<String>;
    /// The number of the registered format called `name`, registering it
    /// if needed.
    fn register(&self, name: &str) -> Option<u32>;
}

/// Everything the user had on the clipboard. No `Debug`: it holds user data.
pub struct Snapshot(Vec<(u32, Vec<u8>)>);

impl Snapshot {
    /// Reads every format worth keeping from `clipboard`.
    pub fn take(clipboard: &impl ClipboardAccess) -> Self {
        let formats = clipboard.formats();
        let saved = formats
            .iter()
            .enumerate()
            .filter(|(position, format)| {
                !skips(**format, &formats[..*position], &formats)
                    && !is_unwanted_registered(clipboard, **format)
            })
            .filter_map(|(_, format)| clipboard.read(*format).map(|data| (*format, data)))
            .collect();
        Self(saved)
    }

    /// Replaces the clipboard's contents with the saved formats, kept out of
    /// clipboard history. A format Windows refuses is left out; the rest
    /// still go back.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError`] if the clipboard can't be emptied.
    pub fn restore(&self, clipboard: &mut impl ClipboardAccess) -> Result<(), BridgeError> {
        clipboard.empty()?;
        for (format, data) in &self.0 {
            let _ = clipboard.write(*format, data);
        }
        tag_private(clipboard);
        Ok(())
    }
}

/// Replaces the clipboard's contents with `text` as Unicode text, kept out
/// of clipboard history.
///
/// # Errors
///
/// Returns [`BridgeError`] if the clipboard can't be written.
pub fn write_text(clipboard: &mut impl ClipboardAccess, text: &str) -> Result<(), BridgeError> {
    let bytes: Vec<u8> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect();
    clipboard.empty()?;
    clipboard.write(CF_UNICODETEXT, &bytes)?;
    tag_private(clipboard);
    Ok(())
}

/// Runs `attempt` up to `attempts` times, pausing between tries, until it
/// succeeds. Opening and closing the clipboard fail now and then while
/// another app (a clipboard manager, Remote Desktop) holds it.
///
/// # Errors
///
/// Returns the last attempt's error.
pub fn retry<T, E>(
    attempts: usize,
    pause: Duration,
    mut attempt: impl FnMut() -> Result<T, E>,
) -> Result<T, E> {
    let mut left = attempts;
    loop {
        left = left.saturating_sub(1);
        match attempt() {
            Err(_) if left > 0 => thread::sleep(pause),
            outcome => return outcome,
        }
    }
}

/// Registered formats that keep a write out of clipboard history, the
/// cloud clipboard and clipboard managers. Each gets a DWORD 0.
const PRIVATE_TAGS: [&str; 3] = [
    "ExcludeClipboardContentFromMonitorProcessing",
    "CanIncludeInClipboardHistory",
    "CanUploadToCloudClipboard",
];

/// Registered OLE formats that point at the copying app's live data object.
/// Restored, they would point at a stale one, so they are skipped; rich
/// text survives through "Rich Text Format" and "HTML Format".
const OLE_FORMATS: [&str; 2] = ["DataObject", "Ole Private Data"];

/// Formats numbered from here on are registered by name.
const FIRST_REGISTERED: u32 = 0xC000;

/// Marks the clipboard's contents as private. Best effort: a failed tag
/// only means the contents may show up in history.
fn tag_private(clipboard: &mut impl ClipboardAccess) {
    // Windows ignores an empty block, so the DWORD 0 must be spelled out.
    let zero = 0_u32.to_ne_bytes();
    for name in PRIVATE_TAGS {
        if let Some(format) = clipboard.register(name) {
            let _ = clipboard.write(format, &zero);
        }
    }
}

/// Whether `format` is a registered OLE format or an old private tag, which
/// [`Snapshot::restore`] adds again itself.
fn is_unwanted_registered(clipboard: &impl ClipboardAccess, format: u32) -> bool {
    format >= FIRST_REGISTERED
        && clipboard.format_name(format).is_some_and(|name| {
            OLE_FORMATS.contains(&name.as_str()) || PRIVATE_TAGS.contains(&name.as_str())
        })
}

/// Formats whose data is a GDI handle rather than a block of memory:
/// `CF_BITMAP`, `CF_METAFILEPICT`, `CF_PALETTE`, `CF_ENHMETAFILE`,
/// `CF_OWNERDISPLAY`, `CF_DSPBITMAP`, `CF_DSPMETAFILEPICT` and
/// `CF_DSPENHMETAFILE`. Reading them as bytes is meaningless, so they are
/// skipped. Images still survive through the DIB and PNG formats.
const HANDLE_FORMATS: [u32; 8] = [2, 3, 9, 14, 0x80, 0x82, 0x83, 0x8E];

/// `CF_PRIVATEFIRST` through `CF_GDIOBJLAST`: app-defined handles and GDI
/// objects.
const PRIVATE_HANDLE_FORMATS: std::ops::RangeInclusive<u32> = 0x200..=0x3FF;

const CF_TEXT: u32 = 1;
const CF_OEMTEXT: u32 = 7;
const CF_DIB: u32 = 8;
const CF_UNICODETEXT: u32 = 13;
const CF_DIBV5: u32 = 17;

/// Whether `format` is left out of the snapshot. `earlier` holds the
/// formats Windows listed before it, `all` every format listed.
fn skips(format: u32, earlier: &[u32], all: &[u32]) -> bool {
    is_handle(format) || is_synthesized(format, earlier, all)
}

/// Whether `format` holds a handle that can't be copied as bytes.
fn is_handle(format: u32) -> bool {
    HANDLE_FORMATS.contains(&format) || PRIVATE_HANDLE_FORMATS.contains(&format)
}

/// Whether Windows makes `format` up from another format it will get back,
/// so restoring it too would only duplicate data.
///
/// - ANSI and OEM text come from Unicode text.
/// - The two DIB formats come from each other. The original is listed
///   first, so the one listed second is the synthesized copy.
fn is_synthesized(format: u32, earlier: &[u32], all: &[u32]) -> bool {
    match format {
        CF_TEXT | CF_OEMTEXT => all.contains(&CF_UNICODETEXT),
        CF_DIB => earlier.contains(&CF_DIBV5),
        CF_DIBV5 => earlier.contains(&CF_DIB),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
