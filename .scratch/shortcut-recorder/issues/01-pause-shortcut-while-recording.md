# 01: Pause the shortcut while recording

**What to build:** While the Settings shortcut field has focus, the app's shortcut stops working, so pressing it records instead of starting a rewrite. The user can record Ctrl+Shift+Space again after changing it. When recording ends without a save (the field loses focus, Esc, or Settings is hidden or closed), the saved shortcut works again. Rust also brings the saved shortcut back when the Settings window is hidden or destroyed, so a lost frontend message can't leave the app without one. See `.scratch/shortcut-recorder/spec.md` (Pause and resume, Changing while paused).

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] Focusing the shortcut field pauses the saved shortcut. Pressing it then reaches the field and starts no rewrite.
- [x] Leaving the field, Esc, or hiding or closing Settings without a save brings the saved shortcut back.
- [x] Resuming when the shortcut is already registered changes nothing.
- [x] Changing the shortcut while it's paused registers and saves the new one. If saving fails, the new one is dropped and resuming brings back the old one.
- [x] Pause, resume and change are serialized by the existing shortcut lock.
- [x] Rust tests use the fake hotkeys and cover: pause frees the shortcut; resume brings it back (twice is harmless); recording the paused shortcut again saves it and leaves it working; change while paused, including a failed save.
- [x] Rows added to the manual test checklist: record the current shortcut again; close Settings mid-recording and the shortcut still works.
- [x] `pnpm check` passes.
