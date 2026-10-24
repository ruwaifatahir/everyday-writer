# 03: New rules, with a reason for each refusal

**What to build:** A new shortcut must include Ctrl or Alt and may add Shift. Win is refused, and so are Ctrl+C and Ctrl+V, because the app presses them to copy the selection and paste the result. Each refusal tells the user why ("Include Ctrl or Alt", Win can't be used, needed for copy and paste, taken by another app), and the field keeps recording. Ctrl+Alt combinations save, with a warning that they can clash with AltGr on some keyboards. A saved shortcut that breaks the new rules (for example Win+K from an earlier version) still loads and works until the user changes it. See `.scratch/shortcut-recorder/spec.md` (new rules, Refusals carry a reason).

**Blocked by:** 02 (Record-only field that saves on press)

**Status:** ready-for-agent

- [ ] Picking a new shortcut follows these rules: Ctrl or Alt required; Win refused; Ctrl+C and Ctrl+V refused; Ctrl+Alt allowed.
- [ ] Loading a saved shortcut still accepts the old rules (Win included), so it registers at startup and shows in the field.
- [ ] The refusal error tells apart: missing Ctrl or Alt, Win, copy and paste keys, not a shortcut, taken, couldn't be saved.
- [ ] Settings shows a message for each reason, and the field keeps recording.
- [ ] After saving a Ctrl+Alt combination, the AltGr warning shows in place of "Shortcut saved".
- [ ] Rust tests cover each rule, each refusal reason and a saved Win shortcut still loading.
- [ ] A new ADR in `docs/adr/` records the recorder rules (record-only, Ctrl or Alt required, no Win, Ctrl+C and Ctrl+V refused, pause while recording) and why.
- [ ] The PRD and the manual test checklist list the rules.
- [ ] Copy follows the better-writing skill.
- [ ] `pnpm check` passes.
