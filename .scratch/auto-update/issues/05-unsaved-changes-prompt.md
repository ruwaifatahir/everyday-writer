# 05: Ask about unsaved Settings changes before updating

**What to build:** If the Settings form has unsaved edits when the user clicks Update now, the page asks "Save your changes before updating?" with **Save**, **Discard** and **Cancel**. Save saves then updates, Discard updates without saving, Cancel leaves everything as it was. With no unsaved edits, Update now goes straight ahead. See the spec, user stories 33–34.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] The prompt appears only when the form has unsaved changes, using the page's existing dirty-state knowledge
- [ ] Save: saves, and only updates if the save succeeded; a failed save shows the usual save error and doesn't update
- [ ] Discard: updates without saving. Cancel: no update, form untouched
- [ ] The prompt matches the existing Settings styles and is keyboard-usable (Esc = Cancel)
- [ ] Added to `docs/manual-test-checklist.md` (Save, Discard and Cancel paths), since the page has no automated tests
- [ ] `pnpm check` passes
