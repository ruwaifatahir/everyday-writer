# 04: Replace

**What to build:** Replace (the button or Enter) puts the result into the original app in place of the selection:

1. The popup hides.
2. The original window, recorded at capture, comes back to the front.
3. The result is pasted with Ctrl+V, sent as virtual-key codes.
4. The clipboard is restored about 300 ms later.

If the original window has closed, the result is copied instead and the popup says "Original window closed, copied instead". On a read-only target Replace does nothing, and the app doesn't try to detect that.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 03 (Tracer bullet: shortcut, popup, Copy)

**Status:** resolved

- [ ] Replace works in Notepad, Word, a Chrome text field, Slack (or another Electron app) and Windows Terminal
- [x] Enter triggers Replace
- [ ] Focus returns to the original window, so the user can keep typing
- [ ] If the original window has closed, the result is copied and the message is shown
- [ ] Text on the user's clipboard is the same after Replace as before
- [x] Session tests cover Replace into an existing window and into a closed window

## Comments

Merged in f0428e3. Still to verify by hand in the real app (unticked above):
- Replace in Notepad, Word, Chrome text field, Slack/Electron, Windows Terminal
- Focus returns to the original window
- Closed-window fallback shows "Original window closed, copied instead"
- Clipboard unchanged after Replace
