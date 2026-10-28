# 03: Dropdown stays in sync with Settings and the tray

**What to build:** While the popup is open, changes to the instruction sets made elsewhere show up in it straight away. Adding, renaming or deleting a set updates the dropdown's list and leaves the popup's state alone. Switching the Active Instruction Set from the tray or Settings, deleting the active set, or editing the active set's instructions is treated like a pick in the popup: a rewrite in progress is cancelled, a result is cleared, and the popup shows Ready. See the spec, user stories 34 to 37.

**Blocked by:** 02 (Instruction set dropdown in the popup)

**Status:** ready-for-agent

- [ ] Every change to the instruction sets (add, edit, rename, delete, set active, restore starter templates), from Settings or the tray, tells the session
- [ ] If the Active Instruction Set or its instructions changed, the session cancels a rewrite in progress, clears a result or failed-rewrite message, and shows Ready
- [ ] Otherwise it only re-renders the current state with the new list
- [ ] Messages where a rewrite can't help (too long, not set up) are not turned into Ready
- [ ] With the popup hidden, a change does nothing visible
- [ ] Session tests cover: a rename keeps the result, a tray switch clears the result, editing the active set's instructions clears it, deleting the active set clears it and shows the new active set, a change while hidden does nothing
- [ ] The manual test checklist covers switching from the tray and editing a set in Settings while the popup is open
- [ ] `pnpm check` passes
