# 03: New rules, with a reason for each refusal

**What to build:** A new shortcut must include Ctrl or Alt and may add Shift. Win is refused, and so are Ctrl+C and Ctrl+V, because the app presses them to copy the selection and paste the result. Each refusal tells the user why ("Include Ctrl or Alt", Win can't be used, needed for copy and paste, taken by another app), and the field keeps recording. Ctrl+Alt combinations save, with a warning that they can clash with AltGr on some keyboards. A saved shortcut that breaks the new rules (for example Win+K from an earlier version) still loads and works until the user changes it. See `.scratch/shortcut-recorder/spec.md` (new rules, Refusals carry a reason).

**Blocked by:** 02 (Record-only field that saves on press)

**Status:** resolved

- [x] Picking a new shortcut follows these rules: Ctrl or Alt required; Win refused; Ctrl+C and Ctrl+V refused; Ctrl+Alt allowed.
- [x] Loading a saved shortcut still accepts the old rules (Win included), so it registers at startup and shows in the field.
- [x] The refusal error tells apart: missing Ctrl or Alt, Win, copy and paste keys, not a shortcut, taken, couldn't be saved.
- [x] Settings shows a message for each reason, and the field keeps recording.
- [x] After saving a Ctrl+Alt combination, the AltGr warning shows in place of "Shortcut saved".
- [x] Rust tests cover each rule, each refusal reason and a saved Win shortcut still loading.
- [x] A new ADR in `docs/adr/` records the recorder rules (record-only, Ctrl or Alt required, no Win, Ctrl+C and Ctrl+V refused, pause while recording) and why.
- [x] The PRD and the manual test checklist list the rules.
- [x] Copy follows the better-writing skill.
- [x] `pnpm check` passes.

## Comments

**Implementer (ticket/03-shortcut-rules):**

- Rust: `shortcut::parse_new` holds the new rules and returns a `Refusal` (`NotAShortcut`, `Win`, `NoCtrlOrAlt`, `CopyPaste`, checked in that order, so Win+K says Win). `change` uses it for the new shortcut and still reads `current` with `parse`. `parse` (loading rules: Ctrl, Alt or Win) is unchanged and still drives startup, `pause`, `resume`, `is_default` and `is_shortcut_working`. `shortcut::ChangeError` is now `Refused(Refusal) | Taken | Save`. Ctrl+C and Ctrl+V are matched as parsed shortcuts with exactly Ctrl, so Ctrl+Shift+C is allowed.
- Command: `set_shortcut` rejects with `SetShortcutError`, serialized as a camelCase reason (`notAShortcut`, `noCtrlOrAlt`, `win`, `copyPaste`, `taken`, `save`), not a message. `CommandError::ShortcutTaken` and `ShortcutInvalid` are gone. `ShortcutRefusal` in `api.ts` mirrors it; a test in `settings_window.rs` pins the names.
- Page: `REFUSALS` in `launch.ts` words each reason; anything else shows "Couldn't save the shortcut. Try again.". A saved shortcut holding Ctrl and Alt shows the AltGr warning in place of "Shortcut saved.". The hint now states the rule.
- Docs: ADR 0005, PRD rules, checklist rows L22 to L25 and updated C4, L2, L8, L15, L18.
- Open: checklist rows L22 to L25 aren't verified by hand.