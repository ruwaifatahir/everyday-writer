//! Safe wrappers over the few Win32 calls the text bridge needs: the
//! foreground window, held modifier keys and synthetic key presses.
//!
//! The only module with `unsafe`. Each wrapper allows it locally and
//! documents every block with a `// SAFETY:` comment.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY, VK_C, VK_CONTROL, VK_LCONTROL,
    VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetForegroundWindow, GetWindowThreadProcessId, IsWindow,
};

/// The top-level window a selection came from. Stored as plain numbers,
/// since `HWND` can't cross threads; the process ID guards against Windows
/// reusing the handle for another window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceWindow {
    handle: usize,
    process_id: u32,
}

impl SourceWindow {
    /// The top-level window that has keyboard focus, if any.
    #[expect(unsafe_code, reason = "Win32 FFI: GetForegroundWindow, GetAncestor")]
    pub fn foreground() -> Option<Self> {
        // SAFETY: GetForegroundWindow takes no arguments and returns a
        // window handle or null; it has no preconditions.
        let focused = unsafe { GetForegroundWindow() };
        if focused.is_invalid() {
            return None;
        }
        // SAFETY: GetAncestor accepts any handle value; an invalid or
        // destroyed one makes it return null, which is checked below.
        let root = unsafe { GetAncestor(focused, GA_ROOT) };
        let window = if root.is_invalid() { focused } else { root };
        let process_id = process_id(window)?;
        Some(Self {
            handle: window.0.addr(),
            process_id,
        })
    }

    /// Whether the window still exists and still belongs to the same
    /// process.
    #[expect(unsafe_code, reason = "Win32 FFI: IsWindow")]
    pub fn is_open(self) -> bool {
        let window = self.hwnd();
        // SAFETY: IsWindow accepts any handle value and only reports
        // whether it names an existing window.
        let exists = unsafe { IsWindow(Some(window)) }.as_bool();
        exists && process_id(window) == Some(self.process_id)
    }

    fn hwnd(self) -> HWND {
        // A window handle is an opaque number, never dereferenced.
        HWND(std::ptr::without_provenance_mut(self.handle))
    }
}

/// The ID of the process that owns `window`, or `None` if the window is
/// gone.
#[expect(unsafe_code, reason = "Win32 FFI: GetWindowThreadProcessId")]
fn process_id(window: HWND) -> Option<u32> {
    let mut process_id = 0_u32;
    // SAFETY: `process_id` is a live, writable u32 for the whole call, and
    // an invalid handle makes the call return 0 without writing.
    let thread_id = unsafe { GetWindowThreadProcessId(window, Some(&raw mut process_id)) };
    (thread_id != 0).then_some(process_id)
}

/// Modifier keys that can still be held from the shortcut.
const MODIFIERS: [VIRTUAL_KEY; 8] = [
    VK_LCONTROL,
    VK_RCONTROL,
    VK_LSHIFT,
    VK_RSHIFT,
    VK_LMENU,
    VK_RMENU,
    VK_LWIN,
    VK_RWIN,
];

/// Keys whose release alone has an effect: Alt focuses a menu bar and Win
/// opens Start.
const RELEASE_ACTS_ALONE: [VIRTUAL_KEY; 4] = [VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN];

/// A virtual-key code no key is assigned to. Pressing it before releasing
/// Alt or Win makes Windows treat them as part of a combination, so their
/// release does nothing (the `PowerToys` approach).
const UNASSIGNED_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xFF);

/// Whether the physical key `key` is down right now.
#[expect(unsafe_code, reason = "Win32 FFI: GetAsyncKeyState")]
fn is_held(key: VIRTUAL_KEY) -> bool {
    // SAFETY: GetAsyncKeyState takes any virtual-key code by value and has
    // no other preconditions.
    let state = unsafe { GetAsyncKeyState(i32::from(key.0)) };
    state < 0
}

/// Releases every modifier key still held from the shortcut, so the next
/// synthetic key press isn't read as, say, Ctrl+Win+C (the colour filter).
///
/// Returns `false` if Windows refused the input.
pub fn release_held_modifiers() -> bool {
    let held: Vec<VIRTUAL_KEY> = MODIFIERS.into_iter().filter(|key| is_held(*key)).collect();
    if held.is_empty() {
        return true;
    }
    let mut inputs = Vec::with_capacity(held.len() + 2);
    if held.iter().any(|key| RELEASE_ACTS_ALONE.contains(key)) {
        inputs.push(key_input(UNASSIGNED_KEY, false));
        inputs.push(key_input(UNASSIGNED_KEY, true));
    }
    inputs.extend(held.into_iter().map(|key| key_input(key, true)));
    send(&inputs)
}

/// Presses Ctrl+C as virtual-key codes, so it works on every keyboard
/// layout. Returns `false` if Windows refused the input.
pub fn send_copy() -> bool {
    send(&[
        key_input(VK_CONTROL, false),
        key_input(VK_C, false),
        key_input(VK_C, true),
        key_input(VK_CONTROL, true),
    ])
}

/// One key going down or up, by virtual-key code.
fn key_input(key: VIRTUAL_KEY, up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    if [VK_RCONTROL, VK_RMENU, VK_LWIN, VK_RWIN].contains(&key) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Sends `inputs` in one call, so the user's own typing can't land in
/// between. Returns `false` unless Windows accepted all of them.
#[expect(unsafe_code, reason = "Win32 FFI: SendInput")]
fn send(inputs: &[INPUT]) -> bool {
    let Ok(size) = i32::try_from(size_of::<INPUT>()) else {
        return false;
    };
    // SAFETY: `inputs` is a slice of fully initialised keyboard INPUTs whose
    // union member matches `INPUT_KEYBOARD`, and `size` is the size of one
    // element, as SendInput requires.
    let sent = unsafe { SendInput(inputs, size) };
    usize::try_from(sent).is_ok_and(|sent| sent == inputs.len())
}
