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

## Comments

**Implementer (ticket/05-unsaved-changes-prompt):**

- Dirty state was missing, so it's new: `main.ts` keeps `savedForm` (provider, model, base URL) from `init()` and every successful `save()`. `hasUnsavedChanges(saved, form, typedKey)` in `src/settings/unsaved-changes.ts` compares trimmed values; a typed key counts (except for Ollama, which never saves one). If loading the settings failed, only a typed key counts.
- Scope decision: only the provider form counts. The Shortcut and startup group applies right away; an open instruction-set editor is not asked about (it saves on its own, and Save in the prompt couldn't save it). Revisit if that loses edits in practice.
- `save()` now resolves `true`/`false`. Save in the prompt updates only on `true`; on failure the usual error shows (validation focuses the field, otherwise `#status` is scrolled into view).
- `updates.ts` gained `beforeUpdateNow(confirm)`; Update now awaits it and calls `update_now` only on `true`. Cancel re-enables and refocuses the button.
- Prompt: `<dialog id="unsaved-dialog">` reusing `.confirm` styles; Cancel / Discard (quiet) / Save (primary, autofocus). Esc leaves `returnValue` empty, read as Cancel (`choiceFrom`).
- No automated tests (no frontend test framework); the pure functions were checked with a throwaway script. Manual cases S12–S16 added.
