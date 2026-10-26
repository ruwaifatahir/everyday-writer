# 03: Save and Discard only on change

**What to build:** The provider card has no buttons at rest. As soon as the user changes the provider, model, Custom base URL or types a key, **Discard** and **Save** slide in at the bottom right of the card. Save keeps the change, shows "Saved." for about two seconds, and the buttons go away. Discard puts back the last saved provider, model and base URL, empties a typed key, and the buttons go away. No "Unsaved changes" text shows, but screen readers hear it. See the spec, user stories 1 to 26.

**Blocked by:** 02

**Status:** resolved

- [x] With no unsaved changes the card's foot is hidden; editing a field back to its saved value hides it again (same unsaved-changes rule as today)
- [x] With changes, the foot shows Discard (secondary) then Save (primary), right-aligned, rising in with the window's existing animation and without motion under reduced motion
- [x] No visible "Unsaved changes" text; a visually hidden live region announces "Unsaved changes" when the buttons appear
- [x] Save success: buttons hide, "Saved." shows in the foot for about two seconds, then the foot goes away
- [x] Save failure (such as Credential Manager unavailable): the error shows in the foot left of the buttons, and the buttons stay
- [x] An empty model or bad base URL still shows its error under the field on Save, and the buttons stay
- [x] Discard restores the last saved provider, model and base URL, re-renders the provider-dependent parts (base URL field, Ollama hint, model hint, key field), empties the key field, clears field errors and hides the foot; it never touches the key store
- [x] Enter saves; Esc keeps its current behaviour and never discards
- [x] The close-window and Update now "Save your changes?" prompts still work (checklist S12 to S16)
- [x] The design notes describe the save bar; the manual checklist gains rows for rest state, Save, Discard, errors, the screen reader announcement and reduced motion; the CHANGELOG's Unreleased section describes Save and Discard
- [x] `pnpm check` passes

## Comments

**2026-10-26:** Done in `c4d389b`, refined in `3fa9af2` after review. "Key removed." is announced through the hidden live region rather than shown, and a failed remove shows as a field error under the key field. If settings fail to load, the save bar shows Save without Discard. Enter with nothing changed does nothing. Not yet run by hand: Narrator, reduced motion, the real Credential Manager, checklist S12 to S16.
