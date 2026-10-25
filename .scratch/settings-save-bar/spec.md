Labels: ready-for-agent

# Settings: Save and Discard only when something changed

## Problem Statement

The provider card in Settings feels cluttered and inconsistent:

- The API key row crams a sentence ("Saved in Windows Credential Manager") and two buttons (Test key, Change key) into one box. To replace a key the user first has to click Change key to swap the box for a field.
- Test key is a feature the user doesn't want. It adds a button, a status line and a whole backend check for something a rewrite already tells them.
- The Save button always sits at the foot of the provider card, even when nothing has changed. Every other section of the window (Shortcut and startup, Instruction Sets, Updates) applies changes right away, so a permanent Save in one card makes the user wonder what is and isn't saved.
- An "Unsaved changes" line repeats what the user already knows.

## Solution

The provider card has no buttons at rest. As soon as the user changes the provider, model, base URL or types a key, **Save** and **Discard** slide in at the foot of the card. Save keeps the change, shows "Saved." briefly, and the buttons go away. Discard puts back what was last saved, and the buttons go away. So the user can always change their mind before saving.

The API key is one plain password field. Its placeholder says whether a key is saved ("Key saved. Type a new one to replace it.") or asks for one. Under it there is only **Remove key**, on the right, when a key is saved.

Test key is gone, with everything behind it.

## User Stories

1. As a user, I want the provider card to show no buttons when nothing has changed, so that the window looks calm and I know everything is saved.
2. As a user, I want Save and Discard to appear the moment I change the provider, so that I know the change isn't kept yet.
3. As a user, I want Save and Discard to appear the moment I change the model, so that I know the change isn't kept yet.
4. As a user, I want Save and Discard to appear the moment I change the Custom base URL, so that I know the change isn't kept yet.
5. As a user, I want Save and Discard to appear the moment I type in the key field, so that I know the key isn't stored yet.
6. As a user, I want Save and Discard to disappear again if I edit a field back to its saved value, so that the buttons only show for a real change.
7. As a user, I want no "Unsaved changes" text next to the buttons, so that the card doesn't say the same thing twice.
8. As a screen reader user, I want to hear "Unsaved changes" when the buttons appear, so that I learn about them without seeing them.
9. As a user, I want Save and Discard at the bottom right of the provider card, so that they sit next to the fields they act on.
10. As a user, I want Save and Discard to slide in with the window's usual small rise, so that their appearance feels calm and not jumpy.
11. As a user who turned on reduced motion, I want the buttons to appear without movement, so that the window respects my setting.
12. As a user, I want Save to store my provider, model, base URL and typed key, so that the next rewrite uses them.
13. As a user, I want Save to hide the buttons and show "Saved." for about two seconds in their place, so that I know it worked.
14. As a user, I want the key field emptied after Save, with the "Key saved" placeholder, so that the key is never shown again.
15. As a user, I want Discard to put back the last saved provider, model and base URL, so that I can change my mind.
16. As a user, I want Discard to clear a key I typed but didn't save, so that the saved key stays as it was.
17. As a user, I want Discard to hide the buttons, so that the card is back at rest.
18. As a user who switched provider and then clicked Discard, I want my old provider and its old model back, so that a mistaken pick costs nothing.
19. As a user, I want an empty model or a bad base URL to show its error under the field when I click Save, so that I know what to fix.
20. As a user, I want Save and Discard to stay visible after a field error, so that I can fix the field and save, or discard.
21. As a user, I want an error like "Couldn't reach Windows Credential Manager" to show next to the buttons when saving fails, so that I see it where I clicked.
22. As a user, I want the buttons to stay after a failed save, so that I can try again or discard.
23. As a keyboard user, I want Enter in the card to save, so that I don't have to reach for Save.
24. As a keyboard user, I want Esc to keep doing what it does today and never discard, so that a stray key press can't throw away my edits.
25. As a user closing Settings with unsaved changes, I want the existing "Save your changes?" prompt, so that I don't lose an edit by accident.
26. As a user clicking Update now with unsaved changes, I want the existing save prompt before the update, so that the restart doesn't lose my edit.
27. As a user with a key saved, I want the key field to say "Key saved. Type a new one to replace it.", so that I know a key exists without seeing it.
28. As a user with no key saved, I want the key field to say "Paste your API key", so that I know what to do.
29. As a Custom provider user with no key saved, I want the field to say "Paste your API key (optional)", so that I know my server may not need one.
30. As a user, I want to replace my key by typing straight into the field, so that I don't need a Change key step first.
31. As a user, I want nothing under the key field except Remove key, so that the card stays short.
32. As a user with a key saved, I want Remove key on the right under the field, so that I can delete my key.
33. As a user with no key saved, I want no Remove key, so that I don't see an action that does nothing.
34. As a user, I want Remove key to change to "Remove permanently" on the first click, so that I can't delete my key by accident.
35. As a user, I want Remove key to take effect on the second click without waiting for Save, so that removing a key is one clear action.
36. As a user, I want removing a key not to touch my other unsaved edits, so that removing a key and editing the model stay separate.
37. As a user switching provider, I want the key field placeholder and Remove key to follow the new provider's saved key, so that I always see that provider's state.
38. As an Ollama user, I want the key field to stay hidden as today, so that I'm not asked for a key Ollama doesn't use.
39. As a user, I want no Test key button anywhere, so that the card has fewer controls.
40. As a user with a wrong key or model, I want the popup's existing error to tell me on the first rewrite, so that I still find out what's wrong.
41. As a maintainer, I want the connection check code deleted, so that there is no unused code left behind.
42. As a maintainer, I want the CHANGELOG, design notes and manual test checklist to match the new card, so that the docs don't describe buttons that no longer exist.

## Implementation Decisions

- **Provider card at rest.** The card's foot (status line plus Save) is hidden when the form matches what was last loaded or saved and no key is typed. The existing "has unsaved changes" rule decides this, unchanged: a typed key counts except for Ollama, and any difference in provider, model or base URL counts.
- **Save bar.** When there are unsaved changes, the foot shows **Discard** (secondary button) then **Save** (primary button), right-aligned, with the existing rise animation and no motion under reduced motion. No visible "Unsaved changes" text. A visually hidden live region announces "Unsaved changes" when the buttons appear.
- **After Save.** On success the buttons hide and "Saved." shows in the foot for about two seconds, then the foot goes away. On a failed save the error text shows in the foot to the left of the buttons, and the buttons stay. Field validation errors stay under their fields, as today.
- **Discard.** Restores the last saved provider, model and Custom base URL, re-renders the provider-dependent parts (base URL field, Ollama hint, model hint, key state) and empties the key field. It clears field errors and hides the foot. It doesn't touch the key store.
- **Key field.** The saved-key box and its Test key and Change key buttons go away, along with the Cancel button and the "replacing a key" state. One password field is always shown (except for Ollama). Its placeholder depends on state: "Key saved. Type a new one to replace it." with a key saved, "Paste your API key" with none, and "Paste your API key (optional)" for Custom with none.
- **Key hint removed.** The line under the key field (where to get a key, and that it's kept in Windows Credential Manager) is removed, along with the per-provider key page table, which nothing else uses.
- **Remove key.** Shown under the field on the right only when the current provider has a saved key. Keeps the two-step "Remove permanently" confirmation. It acts at once through the existing remove-key command and doesn't wait for Save. Afterwards the placeholder switches to the no-key wording.
- **Keyboard.** Enter submits the form, which saves, as today. Esc keeps its current behaviour and never discards.
- **Prompts unchanged.** The close-window and update-now "Save your changes?" prompts keep working off the same unsaved-changes rule.
- **Test key removed, front to back.** Remove the test-connection command from the settings window module and its registration in the app's command handler, the frontend API wrapper for it, the connection check in the rewriter module (including the OpenRouter key check and model-listing helpers that only it uses), the Connection type and its re-export, and the key-status line under the key field.
- **Docs.** The CHANGELOG gets an entry for the new Save and Discard behaviour and the removal of Test key. The design notes' "Saved key" component description is replaced to match the new key field and save bar. The manual test checklist's settings rows are updated (see Testing Decisions).
- No new domain terms and no ADR: the change is easy to reverse.

## Testing Decisions

- A good test checks what the user sees and can do, not how the window tracks its state.
- **Main seam: the manual test checklist.** The settings window has no automated frontend tests, and adding a frontend test framework would add dependencies for a small change, against the "keep the app small" rule. Update or add settings rows for:
  - No changes: no Save or Discard. Change provider, model, base URL or type a key: both appear. Edit back to the saved value: both go away.
  - Save: "Saved." for about two seconds, buttons gone, settings kept after restart, key stored in Credential Manager and field emptied.
  - Discard after switching provider and model, and after typing a key: everything back to the last saved values, saved key untouched.
  - Empty model or bad base URL on Save: field error, buttons stay.
  - Key placeholder in its three states (saved, none, Custom with none), and the field hidden for Ollama.
  - Remove key: only shown with a saved key, "Remove permanently" on first click, removed on second without Save, other edits kept.
  - Enter saves, Esc doesn't discard, closing with changes still prompts.
  - Screen reader announces "Unsaved changes" when the buttons appear.
  - Reduced motion: buttons appear without movement.
  - Existing rows that mention Test key, Change key or the key hint are reworded. S5 and S12–S16 should still pass as written or with small wording changes.
- **Rust.** The connection check's unit tests in the rewriter module are removed with the code. `cargo test` and clippy must still pass with no warnings about unused code. No new Rust tests: no Rust behaviour is added.

## Out of Scope

- Making the other sections (Shortcut and startup, Instruction Sets, Updates) use Save and Discard. They keep applying right away.
- Any other way to check a key or model before a rewrite.
- Changing the popup's error messages.
- Showing any part of a saved key.
- The rest of the settings layout, fonts and colours.
- Branching, commits and release: not decided yet.

## Further Notes

- Mockups of the agreed look were made as throwaway HTML outside the repo, using the app's real styles: the card at rest has no foot; after a change the foot shows Discard (outlined) and Save (lavender) on the right.
- The CHANGELOG currently advertises Test key as a feature; the new entry should say plainly that it was removed and that a wrong key or model now shows up on the first rewrite.
