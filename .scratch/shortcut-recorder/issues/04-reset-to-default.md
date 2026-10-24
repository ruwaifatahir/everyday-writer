# 04: Reset to default

**What to build:** A "Reset to default" button in the shortcut section sets Ctrl+Shift+Space in one click. It shows only when the saved shortcut isn't the default. It changes the shortcut through the existing change command, so if another app holds Ctrl+Shift+Space, the user sees the taken message and keeps the current shortcut. See `.scratch/shortcut-recorder/spec.md` (Reset to default).

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] The button shows when the saved shortcut differs from the default, compared as parsed shortcuts rather than strings, and hides once the default is set.
- [x] Clicking it makes Ctrl+Shift+Space work and saves it, and the old shortcut stops working.
- [x] If the default is taken, the taken message shows and the current shortcut keeps working.
- [x] A Rust test covers changing back to the default, including when it's taken.
- [x] A row is added to the manual test checklist.
- [x] Copy follows the better-writing skill.
- [x] `pnpm check` passes.

## Comments

**Implementer (ticket/04-reset-to-default):**

- Rust: `shortcut::is_default(text)` compares parsed shortcuts with `DEFAULT_SHORTCUT` (an unparsable one is never the default). Command `is_default_shortcut` reads the saved shortcut (build.rs, handler list, `allow-is-default-shortcut` in the settings capability).
- Tests in `shortcut.rs`: `resetting_to_the_default_brings_it_back_and_saves_it`, `resetting_to_a_default_held_by_another_app_keeps_the_current_shortcut`, `the_default_is_recognised_however_it_is_written`. The two reset tests pass on the existing `change`, as the spec has reset reuse it.
- Page: `#shortcut-reset` "Reset to default" (secondary) beside the field. Shown after load and after each successful change only when `is_default_shortcut` is false; hidden if that call fails. It calls `applyShortcut(DEFAULT_SHORTCUT)`, so success shows "Shortcut changed." and a taken default shows the taken error with the current shortcut kept. `DEFAULT_SHORTCUT` in `api.ts` mirrors the Rust constant.
- For ticket 02: when the reset succeeds the focused button hides and focus falls to the page. Moving focus to the field would start recording once ticket 01 pauses on focus, so it was left as is.
- Open: checklist rows L7 and L8 aren't verified by hand.
