# 02: Record-only field that saves on press

**What to build:** The shortcut field only records key presses, like Windows' own hot key control and PowerToys. While recording it says "Press a shortcut…" and shows the held modifiers live ("Ctrl + Shift + …"). A combination saves the moment it's pressed, with no "Use shortcut" button, and "Shortcut saved" is announced through a live region. Tab moves to the next field and Esc cancels. Typed letters, spaces and auto-repeat are ignored. A hint under the field says "Nothing happens? Another app may be using that shortcut." See `.scratch/shortcut-recorder/spec.md` (Recorder).

**Blocked by:** 01 (Pause the shortcut while recording)

**Status:** resolved

- [x] The field can't be typed into, and typed letters or spaces change nothing.
- [x] Focusing the field shows "Press a shortcut…", and held modifiers show live.
- [x] A combination is built from the physical key code plus the held modifiers, and is sent to be saved straight away.
- [x] Lone modifiers, Tab and auto-repeat aren't recorded. Tab moves focus on. Esc cancels and keeps the old shortcut.
- [x] The "Use shortcut" button and Enter-to-apply are gone.
- [x] A successful save shows "Shortcut saved" in a live region. An error shows its message, and the field keeps recording.
- [x] The field is labelled as a shortcut field for assistive tech.
- [x] The "Nothing happens?" hint is shown under the field.
- [x] The startup warning ("This shortcut isn't working…") still shows when the saved shortcut failed at startup.
- [x] The code comment that justifies typing is removed.
- [x] `docs/PRD.md` and the manual test checklist describe the record-only field.
- [x] Copy follows the better-writing skill.
- [x] `pnpm check` passes.

## Comments

**Implementer (ticket/02-record-only-field):**

- Page: `#shortcut` is `readonly` with `aria-roledescription="shortcut field"`; hint "Click the field, then press the keys. Nothing happens? Another app may be using that shortcut." Focus means recording, so `#shortcut:focus` draws the ring after a click too (a read-only field doesn't match `:focus-visible` then). `data-prompt` greys "Press a shortcut…".
- `launch.ts`: every keydown but Tab (and IME composition) is cancelled. Esc blurs (blur resumes). Modifier keydown/keyup render "Ctrl + Shift + …". Any other key, unless it repeats, has an empty code (injected) or a save is in flight, is sent as `Ctrl+Alt+Shift+Super+<key>` with the key code's name (`K`, `7`, `Minus`, `ArrowUp`). Rust judges it, so K on its own or Shift+K reach `set_shortcut` and come back as the existing Invalid message. Saved shortcuts display with " + " and Win for Super.
- Pause, resume and save share one queue (`inTurn`). After a save (or a refusal) while the field still has focus, the shortcut is paused again in the same turn, which closes ticket 01's handoff. The status is emptied before each save so a repeated message is announced again. `aria-invalid` from the startup warning is cleared only by a successful save.
- Reset to default (ticket 04's handoff): when the click leaves focus on the body (the button was disabled while saving, or hid), focus goes back to the button, or to **Start with Windows** once the button hides.
- Rust: only a test, `a_shortcut_recorded_by_physical_key_is_valid`, pinning that the key code names the field sends parse.
- For ticket 03: map the new refusal reasons in `applyShortcut`'s catch (it shows the command's string as is via `errorMessage`). The Ctrl+Alt warning fits after `show(... "Shortcut saved." ...)`, using `wanted`. The hint dropped "Use Ctrl, Alt or Win"; add the new rule to it if wanted. `heldModifiers` still sends `Super` for Win so Rust can refuse it with its reason.
- Open: checklist rows L1, L9 and L12 to L21 aren't verified by hand.
