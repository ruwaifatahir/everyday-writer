# 04: Reset to default

**What to build:** A "Reset to default" button in the shortcut section sets Ctrl+Shift+Space in one click. It shows only when the saved shortcut isn't the default. It changes the shortcut through the existing change command, so if another app holds Ctrl+Shift+Space, the user sees the taken message and keeps the current shortcut. See `.scratch/shortcut-recorder/spec.md` (Reset to default).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The button shows when the saved shortcut differs from the default, compared as parsed shortcuts rather than strings, and hides once the default is set.
- [ ] Clicking it makes Ctrl+Shift+Space work and saves it, and the old shortcut stops working.
- [ ] If the default is taken, the taken message shows and the current shortcut keeps working.
- [ ] A Rust test covers changing back to the default, including when it's taken.
- [ ] A row is added to the manual test checklist.
- [ ] Copy follows the better-writing skill.
- [ ] `pnpm check` passes.
