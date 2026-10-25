# 02: One key field

**What to build:** The API key is one plain password field, always shown (except for Ollama). Its placeholder tells the user whether a key is saved: "Key saved. Type a new one to replace it.", "Paste your API key", or, for Custom with no key, "Paste your API key (optional)". The user replaces a key by typing straight into the field and saving. Under the field there is only **Remove key**, on the right, when the provider has a saved key. See the spec, user stories 14, 27 to 38.

**Blocked by:** 01

**Status:** resolved

- [x] The saved-key box, Change key and Cancel are gone, with the "replacing a key" state behind them
- [x] One password field shows for every provider except Ollama, with the three placeholders above, matching the provider's saved-key state
- [x] Switching provider updates the placeholder and Remove key to that provider's saved key
- [x] The hint under the key field (where to get a key, and Credential Manager) is removed, along with the per-provider key page table
- [x] Remove key shows on the right under the field only when a key is saved; the first click turns it into "Remove permanently", the second removes the key at once without Save, and the placeholder switches to the no-key wording
- [x] Removing a key doesn't change other unsaved edits in the card
- [x] Saving with a typed key stores it in Credential Manager, empties the field and shows the "Key saved" placeholder; the existing Save button still does this until ticket 03
- [x] The design notes' "Saved key" description matches the new field; the manual checklist's key rows (S4, S5 and any others mentioning Change key) are updated
- [x] `pnpm check` passes

## Comments

**2026-10-26:** Done in `f3460ed`, refined in `3fa9af2` after review. DESIGN.md's "placeholders never instruct" rule now has an exception for the key field. Remove key leaves a typed key alone. Remove key stays hidden until the new provider's key state has loaded.
