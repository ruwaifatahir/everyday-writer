# 02: One key field

**What to build:** The API key is one plain password field, always shown (except for Ollama). Its placeholder tells the user whether a key is saved: "Key saved. Type a new one to replace it.", "Paste your API key", or, for Custom with no key, "Paste your API key (optional)". The user replaces a key by typing straight into the field and saving. Under the field there is only **Remove key**, on the right, when the provider has a saved key. See the spec, user stories 14, 27 to 38.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] The saved-key box, Change key and Cancel are gone, with the "replacing a key" state behind them
- [ ] One password field shows for every provider except Ollama, with the three placeholders above, matching the provider's saved-key state
- [ ] Switching provider updates the placeholder and Remove key to that provider's saved key
- [ ] The hint under the key field (where to get a key, and Credential Manager) is removed, along with the per-provider key page table
- [ ] Remove key shows on the right under the field only when a key is saved; the first click turns it into "Remove permanently", the second removes the key at once without Save, and the placeholder switches to the no-key wording
- [ ] Removing a key doesn't change other unsaved edits in the card
- [ ] Saving with a typed key stores it in Credential Manager, empties the field and shows the "Key saved" placeholder; the existing Save button still does this until ticket 03
- [ ] The design notes' "Saved key" description matches the new field; the manual checklist's key rows (S4, S5 and any others mentioning Change key) are updated
- [ ] `pnpm check` passes
