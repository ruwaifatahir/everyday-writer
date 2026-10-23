use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use super::*;

const CF_TEXT: u32 = 1;
const CF_BITMAP: u32 = 2;
const CF_OEMTEXT: u32 = 7;
const CF_DIB: u32 = 8;
const CF_UNICODETEXT: u32 = 13;
const CF_HDROP: u32 = 15;
const CF_LOCALE: u32 = 16;
const CF_DIBV5: u32 = 17;
const RICH_TEXT: u32 = 0xC0A1;
const HTML: u32 = 0xC0A2;
const PNG: u32 = 0xC0A3;

/// An in-memory clipboard. Formats keep the order they were written in, as
/// Windows lists them.
#[derive(Default)]
struct FakeClipboard {
    contents: Vec<(u32, Vec<u8>)>,
    /// Every format read, in order.
    reads: RefCell<Vec<u32>>,
    /// Registered format names.
    names: RefCell<HashMap<String, u32>>,
}

impl FakeClipboard {
    fn holding(contents: &[(u32, &[u8])]) -> Self {
        Self {
            contents: contents.iter().map(|(f, d)| (*f, d.to_vec())).collect(),
            ..Self::default()
        }
    }

    fn formats_in_order(&self) -> Vec<u32> {
        self.contents.iter().map(|(format, _)| *format).collect()
    }

    /// The formats on the clipboard other than the exclusion tags.
    fn untagged_formats(&self) -> Vec<u32> {
        let names = self.names.borrow();
        let tags: Vec<u32> = TAGS
            .iter()
            .filter_map(|tag| names.get(*tag))
            .copied()
            .collect();
        self.formats_in_order()
            .into_iter()
            .filter(|format| !tags.contains(format))
            .collect()
    }

    fn data(&self, format: u32) -> Option<&[u8]> {
        self.contents
            .iter()
            .find(|(f, _)| *f == format)
            .map(|(_, data)| data.as_slice())
    }
}

impl ClipboardAccess for FakeClipboard {
    fn formats(&self) -> Vec<u32> {
        self.formats_in_order()
    }

    fn read(&self, format: u32) -> Option<Vec<u8>> {
        self.reads.borrow_mut().push(format);
        self.data(format).map(<[u8]>::to_vec)
    }

    fn empty(&mut self) -> Result<(), BridgeError> {
        self.contents.clear();
        Ok(())
    }

    fn write(&mut self, format: u32, data: &[u8]) -> Result<(), BridgeError> {
        self.contents.push((format, data.to_vec()));
        Ok(())
    }

    fn format_name(&self, format: u32) -> Option<String> {
        self.names
            .borrow()
            .iter()
            .find(|(_, id)| **id == format)
            .map(|(name, _)| name.clone())
    }

    fn register(&self, name: &str) -> Option<u32> {
        let mut names = self.names.borrow_mut();
        let next = 0xC100 + u32::try_from(names.len()).unwrap();
        Some(*names.entry(name.to_owned()).or_insert(next))
    }
}

#[test]
fn restore_puts_back_rich_text_html_and_files_in_their_original_order() {
    let mut clipboard = FakeClipboard::holding(&[
        (RICH_TEXT, b"{\\rtf1 hi}"),
        (HTML, b"<b>hi</b>"),
        (CF_HDROP, b"files"),
        (CF_UNICODETEXT, b"h\0i\0\0\0"),
    ]);
    let snapshot = Snapshot::take(&clipboard);
    clipboard.contents = vec![(CF_TEXT, b"the result".to_vec())];

    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(
        clipboard.untagged_formats(),
        [RICH_TEXT, HTML, CF_HDROP, CF_UNICODETEXT]
    );
    assert_eq!(clipboard.data(RICH_TEXT), Some(&b"{\\rtf1 hi}"[..]));
    assert_eq!(clipboard.data(CF_HDROP), Some(&b"files"[..]));
}

#[test]
fn gdi_handle_formats_are_never_read_or_restored() {
    // Bitmap, metafile picture, palette, enhanced metafile, owner display,
    // the display formats, private handles and GDI objects.
    let handles = [
        2, 3, 9, 14, 0x80, 0x82, 0x83, 0x8E, 0x200, 0x2FF, 0x300, 0x3FF,
    ];
    let mut contents: Vec<(u32, &[u8])> = handles.iter().map(|f| (*f, &b"handle"[..])).collect();
    contents.push((CF_HDROP, b"files"));
    let mut clipboard = FakeClipboard::holding(&contents);

    let snapshot = Snapshot::take(&clipboard);
    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(*clipboard.reads.borrow(), [CF_HDROP]);
    assert_eq!(clipboard.untagged_formats(), [CF_HDROP]);
}

#[test]
fn ansi_text_is_left_for_windows_to_synthesize_from_unicode_text() {
    let mut clipboard = FakeClipboard::holding(&[
        (CF_UNICODETEXT, b"h\0i\0\0\0"),
        (CF_LOCALE, b"\x09\x04\0\0"),
        (CF_TEXT, b"hi\0"),
        (CF_OEMTEXT, b"hi\0"),
    ]);

    let snapshot = Snapshot::take(&clipboard);
    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(clipboard.untagged_formats(), [CF_UNICODETEXT, CF_LOCALE]);
}

#[test]
fn ansi_text_is_kept_when_there_is_no_unicode_text() {
    let mut clipboard = FakeClipboard::holding(&[(CF_TEXT, b"hi\0")]);

    let snapshot = Snapshot::take(&clipboard);
    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(clipboard.untagged_formats(), [CF_TEXT]);
}

#[test]
fn an_image_keeps_its_png_and_the_first_dib_listed() {
    let mut clipboard = FakeClipboard::holding(&[
        (PNG, b"png"),
        (CF_DIBV5, b"dibv5"),
        (CF_BITMAP, b"handle"),
        (CF_DIB, b"dib"),
    ]);

    let snapshot = Snapshot::take(&clipboard);
    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(clipboard.untagged_formats(), [PNG, CF_DIBV5]);
    assert_eq!(clipboard.data(CF_DIBV5), Some(&b"dibv5"[..]));
}

#[test]
fn a_restored_clipboard_is_kept_out_of_history_and_cloud_clipboard() {
    let mut clipboard = FakeClipboard::holding(&[(CF_HDROP, b"files")]);

    let snapshot = Snapshot::take(&clipboard);
    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(clipboard.untagged_formats(), [CF_HDROP]);
    assert_private(&clipboard);
}

#[test]
fn an_empty_clipboard_is_restored_empty_but_still_tagged() {
    let mut clipboard = FakeClipboard::default();
    let snapshot = Snapshot::take(&clipboard);
    clipboard.contents = vec![(CF_UNICODETEXT, b"r\0\0\0".to_vec())];

    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(clipboard.untagged_formats(), Vec::<u32>::new());
    assert_private(&clipboard);
}

#[test]
fn ole_objects_and_old_tags_are_not_restored() {
    let mut clipboard = FakeClipboard::default();
    let data_object = clipboard.register("DataObject").unwrap();
    let ole_private = clipboard.register("Ole Private Data").unwrap();
    let history_tag = clipboard.register(CAN_INCLUDE_IN_HISTORY).unwrap();
    clipboard.contents = vec![
        (data_object, b"stale pointer".to_vec()),
        (RICH_TEXT, b"{\\rtf1 hi}".to_vec()),
        (ole_private, b"stale".to_vec()),
        (history_tag, 0_u32.to_ne_bytes().to_vec()),
    ];

    let snapshot = Snapshot::take(&clipboard);
    snapshot.restore(&mut clipboard).unwrap();

    assert_eq!(clipboard.untagged_formats(), [RICH_TEXT]);
    assert_private(&clipboard);
}

#[test]
fn a_clipboard_briefly_held_by_another_app_is_retried() {
    let mut tries = 0;

    let opened = retry(5, Duration::ZERO, || {
        tries += 1;
        if tries < 3 {
            Err(BridgeError)
        } else {
            Ok(tries)
        }
    });

    assert_eq!(opened, Ok(3));
}

#[test]
fn a_clipboard_held_for_too_long_gives_up_after_the_last_attempt() {
    let mut tries = 0;

    let opened: Result<(), BridgeError> = retry(5, Duration::ZERO, || {
        tries += 1;
        Err(BridgeError)
    });

    assert_eq!(opened, Err(BridgeError));
    assert_eq!(tries, 5);
}

#[test]
fn written_text_is_unicode_and_kept_out_of_history() {
    let mut clipboard = FakeClipboard::holding(&[(CF_HDROP, b"files")]);

    write_text(&mut clipboard, "hé").unwrap();

    assert_eq!(clipboard.untagged_formats(), [CF_UNICODETEXT]);
    assert_eq!(
        clipboard.data(CF_UNICODETEXT),
        Some(&[b'h', 0, 0xE9, 0, 0, 0][..])
    );
    assert_private(&clipboard);
}

const CAN_INCLUDE_IN_HISTORY: &str = "CanIncludeInClipboardHistory";
const TAGS: [&str; 3] = [
    "ExcludeClipboardContentFromMonitorProcessing",
    CAN_INCLUDE_IN_HISTORY,
    "CanUploadToCloudClipboard",
];

/// Asserts that each exclusion tag is on the clipboard exactly once, as a
/// DWORD 0.
fn assert_private(clipboard: &FakeClipboard) {
    for tag in TAGS {
        let format = clipboard.names.borrow()[tag];
        let copies: Vec<&Vec<u8>> = clipboard
            .contents
            .iter()
            .filter(|(f, _)| *f == format)
            .map(|(_, data)| data)
            .collect();
        assert_eq!(copies, [&vec![0_u8; 4]], "{tag}");
    }
}
