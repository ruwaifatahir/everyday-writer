Labels: ready-for-agent

# Shortcut recorder

## Problem Statement

Changing the shortcut in Settings is confusing and sometimes impossible.

- Once the user picks another shortcut, they can't record the default (Ctrl+Shift+Space) again. While a shortcut is active, Windows sends its key press to the app, which starts a rewrite instead of letting the field record it. There is no "Reset to default" either, so the only way back is to type the text by hand.
- Combinations with Win never record. Windows takes most Win combinations (Win+K opens Cast) before the field sees them.
- Shift with a single key does nothing, because a modifier other than Shift is required, but nothing tells the user that.
- The field also takes free typing, so it behaves like a text box. No standard shortcut field does this: Windows' own hot key control, PowerToys, Flow Launcher, ShareX, Chrome and Firefox extension shortcuts, VS Code and the macOS KeyboardShortcuts library all only record key presses.
- A new shortcut needs an extra "Use shortcut" click before it applies.

## Solution

The shortcut field only records key presses. Clicking it starts recording, the app's own shortcut pauses so it can be recorded again, and a valid combination saves the moment it's pressed. Anything that can't be used gets a clear one-line reason, and the field keeps recording. A "Reset to default" button brings back Ctrl+Shift+Space in one click.

The rules are simple: a shortcut needs Ctrl or Alt, may add Shift, and can't use Win. Ctrl+C and Ctrl+V are refused, because the app presses them to copy the selection and paste the result. Ctrl+Alt combinations are allowed, with a warning that they can clash with AltGr on some keyboards.

## User Stories

1. As a user, I want to click the shortcut field and press a combination, so that I can change the shortcut without typing anything.
2. As a user, I want the field to say "Press a shortcut…" while it's recording, so that I know it's waiting for keys.
3. As a user, I want the modifiers I'm holding to show live (for example "Ctrl + Shift + …"), so that I can see what I'm about to set.
4. As a user, I want a valid combination to save the moment I press it, so that I don't need an extra click.
5. As a user, I want "Shortcut saved" shown after saving, so that I know it worked.
6. As a screen reader user, I want the result announced, so that I know it saved without looking.
7. As a user, I want to record the shortcut that's active right now, so that I can set Ctrl+Shift+Space again after changing it.
8. As a user, I want the app's shortcut to stop working while I'm recording, so that pressing it doesn't start a rewrite.
9. As a user, I want the old shortcut working again as soon as I leave the field without saving, so that I'm never left without one.
10. As a user, I want the old shortcut back when I close Settings in the middle of recording, so that closing never breaks the app.
11. As a user, I want Esc to cancel recording and keep the old shortcut, so that I can back out.
12. As a keyboard user, I want Tab to move to the next field instead of being recorded, so that I can still get around Settings.
13. As a user, I want typed letters and spaces to be ignored, so that the field can't end up holding text that isn't a shortcut.
14. As a user, I want "Include Ctrl or Alt" when I press Shift+K or a key on its own, so that I know why it wasn't taken.
15. As a user, I want a clear message when I press a Win combination, so that I know Win can't be used.
16. As a user, I want a clear message when I press Ctrl+C or Ctrl+V, so that I know the app needs those to copy and paste.
17. As a user, I want the field to keep recording after a refused combination, so that I can try again straight away.
18. As a user, I want a warning when I pick a Ctrl+Alt combination, so that I know it can clash with AltGr on some keyboards.
19. As a user, I want "Another app already uses this shortcut" if Windows refuses a combination, so that I pick a different one, and I want my old shortcut kept.
20. As a user, I want a hint under the field ("Nothing happens? Another app may be using that shortcut."), so that I understand why some presses don't record.
21. As a user, I want a "Reset to default" button, so that I can get Ctrl+Shift+Space back in one click.
22. As a user, I want "Reset to default" hidden when the shortcut already is the default, so that the section stays tidy.
23. As a user, I want "Reset to default" to tell me if Ctrl+Shift+Space is taken by another app, and keep my current shortcut, so that it never leaves me without one.
24. As a user, I want to never be left with no shortcut, so that the app always stays usable.
25. As an existing user with a Win shortcut saved, I want it to keep working after the update, so that my setup doesn't break.
26. As an existing user, I want my old shortcut to keep showing in the field until I change it, so that I know what's set.
27. As a user, I want the warning that the shortcut failed at startup to stay, so that I know to pick a new one.
28. As a user with a different keyboard layout, I want the shortcut recorded by physical key, so that it works the same whatever language I type in.

## Implementation Decisions

- **Shortcut module (Rust): new rules for a new choice.** Picking a new shortcut requires Ctrl or Alt, allows Shift, refuses Win, and refuses Ctrl+C and Ctrl+V. These rules are kept apart from loading a saved shortcut. Loading still accepts any combination the old rules allowed, Win included, so old setups keep working (story 25).
- **Refusals carry a reason.** The error for a refused choice tells the cases apart: missing Ctrl or Alt, Win used, needed for copy and paste, not a shortcut at all, taken by another app, couldn't be saved. Settings shows a message for each, in the app's voice. Rust is the only judge of what's allowed, and the field just reports what was pressed.
- **Pause and resume (new commands).** Settings asks to pause the shortcut when recording starts, and to resume it when recording ends without a save: blur, Esc, Settings hidden or closed. Pausing unregisters the saved shortcut. Resuming registers the saved shortcut again and does nothing if it's already registered. As a safety net, the Rust side also resumes when the Settings window is hidden or destroyed, so a lost frontend message can't leave the app without a shortcut.
- **Changing while paused.** Changing the shortcut keeps its order: register the new one, save it, then release the old one. When the old one is paused, releasing it does nothing. If saving fails, the new one is dropped and resuming brings back the old one. The existing lock still covers every change, pause and resume, so they can't interleave.
- **Reset to default.** It is a change to the default shortcut through the same command. No separate command is needed.
- **Recorder (Settings window).**
  - The field is read-only to typing. It is labelled as a shortcut field for assistive tech, and the status line is a live region.
  - On keydown it ignores lone modifiers, Tab and auto-repeat. Esc cancels. Every other key is a press: the field builds the combination from the physical key code plus the held modifiers and sends it to be saved straight away.
  - The "Use shortcut" button and Enter-to-apply are removed.
  - "Reset to default" shows only when the saved shortcut differs from the default.
  - The Ctrl+Alt warning is decided in the frontend after a successful save, because it's only a message.
- **Copy.** All strings follow the better-writing skill and the app's one voice (see recent copy commits).
- **Docs.**
  - A new ADR records the recorder rules (record-only, Ctrl or Alt required, no Win, Ctrl+C and Ctrl+V refused, pause while recording) and why.
  - `docs/PRD.md` and `docs/manual-test-checklist.md` are updated to match.
  - The comment at the top of the Settings shortcut code that justifies typing is removed.

## Testing Decisions

- A good test drives the shortcut module only through its public functions and the existing fake hotkeys port. It checks what the user would notice: which shortcut works and what got saved. It doesn't check how that's done.
- **Shortcut module (the one automated seam).** Add tests for:
  - The new rules: Ctrl or Alt required, Win refused, Ctrl+C and Ctrl+V refused, Ctrl+Alt allowed.
  - Each refusal reason.
  - A saved Win shortcut still loading.
  - Pausing frees the shortcut, and resuming brings it back (twice is harmless).
  - Recording the paused shortcut again saves it and leaves it working.
  - Changing while paused, including when saving fails.
  - Reset to default, including when the default is taken.
- **Prior art.** The existing change tests use an in-memory fake that holds "taken" combinations and the registered set. Extend that fake and follow the same test naming.
- **Recorder.** The project has no frontend test setup, and this spec doesn't add one. The recorder, pause on focus, resume on blur/close, Esc, Tab, live modifiers, messages and screen reader announcement are covered by new rows in the manual test checklist.

## Out of Scope

- A low-level keyboard hook to catch combinations Windows or other apps take first (the PowerToys approach).
- Naming which app holds a taken combination.
- More than one shortcut, chords, single-modifier shortcuts, or left/right modifier distinction.
- A blocklist beyond Ctrl+C and Ctrl+V.
- Adding a frontend test framework.

## Further Notes

- Diagnosis behind this spec: a key log in the Settings field showed Ctrl+Shift+K and Ctrl+Alt+K arrive in full. Ctrl+Shift+Space never arrived, because the app's own registered shortcut caught it and ran a capture (its injected keys showed with an empty code). For Win+K only the Win key arrived.
- Once typing is removed, a combination another app has registered can't be recorded at all, since Windows never delivers it to the field. That is why the "Nothing happens?" hint exists. The "taken" message will rarely show, but it stays for the cases Windows does refuse.
- The `global-hotkey` crate rebuilds shortcut strings in its own form, so compare parsed shortcuts, never strings.
