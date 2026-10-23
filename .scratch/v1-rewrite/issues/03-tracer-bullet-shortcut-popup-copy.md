# 03: Tracer bullet: shortcut, popup, Copy

**What to build:** The first end-to-end rewrite:

1. The user selects text in any app and presses `Ctrl+Shift+Space`.
2. The app captures the selection and opens the popup near the mouse, fully on that monitor.
3. The popup shows a working animation, then the result exactly as it came back, with Replace, Copy and ✕.

Copy puts the result on the clipboard and closes the popup. ✕, Esc and clicking outside close it, and closing while working cancels the rewrite. With nothing selected, no popup appears. Until ticket 07, rewrites use a built-in "Fix & make readable" instruction set. Replace is shown but only starts working in ticket 04.

This ticket builds the **rewrite session**: the one controller that owns the flow and the popup states (hidden, working, result, message). It talks only to four ports: the text bridge, the Rewriter, the settings view and the popup view.

The Windows text bridge handles capture:

1. Save the clipboard.
2. Release any modifier keys still held from the shortcut.
3. Send Ctrl+C using virtual-key codes.
4. Wait up to about 500 ms for the clipboard sequence number to change.
5. Read the text.
6. Restore the clipboard.

Here, saving and restoring may cover text only. Ticket 06 extends it to every format.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 01 (Rename Corrector to Rewriter), 02 (Minimal settings)

**Status:** ready-for-agent

- [ ] Selecting text in Notepad and pressing the shortcut shows the result in the popup
- [ ] Capture works with an Urdu or Arabic keyboard layout (no literal "c" typed)
- [ ] Holding Win or Alt as part of the shortcut never triggers a Windows shortcut during capture
- [ ] No selection: no popup and no sound
- [ ] The popup opens near the mouse and is never cut off at a screen edge or split across monitors
- [ ] Copy, ✕, Esc and click-outside behave as described; closing while working cancels, and a late answer never updates the popup
- [ ] Text on the user's clipboard is the same after capture as before
- [ ] Session tests with fake ports cover: no selection, working then result, an unchanged result still shown as a result, Copy, and close while working cancels
- [ ] Win32 code passes the `unsafe-checker` review, with local `#[expect(unsafe_code)]` and `// SAFETY:` comments
- [ ] Nothing private is logged
