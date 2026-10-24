# 02: Record-only field that saves on press

**What to build:** The shortcut field only records key presses, like Windows' own hot key control and PowerToys. While recording it says "Press a shortcut…" and shows the held modifiers live ("Ctrl + Shift + …"). A combination saves the moment it's pressed, with no "Use shortcut" button, and "Shortcut saved" is announced through a live region. Tab moves to the next field and Esc cancels. Typed letters, spaces and auto-repeat are ignored. A hint under the field says "Nothing happens? Another app may be using that shortcut." See `.scratch/shortcut-recorder/spec.md` (Recorder).

**Blocked by:** 01 (Pause the shortcut while recording)

**Status:** ready-for-agent

- [ ] The field can't be typed into, and typed letters or spaces change nothing.
- [ ] Focusing the field shows "Press a shortcut…", and held modifiers show live.
- [ ] A combination is built from the physical key code plus the held modifiers, and is sent to be saved straight away.
- [ ] Lone modifiers, Tab and auto-repeat aren't recorded. Tab moves focus on. Esc cancels and keeps the old shortcut.
- [ ] The "Use shortcut" button and Enter-to-apply are gone.
- [ ] A successful save shows "Shortcut saved" in a live region. An error shows its message, and the field keeps recording.
- [ ] The field is labelled as a shortcut field for assistive tech.
- [ ] The "Nothing happens?" hint is shown under the field.
- [ ] The startup warning ("This shortcut isn't working…") still shows when the saved shortcut failed at startup.
- [ ] The code comment that justifies typing is removed.
- [ ] `docs/PRD.md` and the manual test checklist describe the record-only field.
- [ ] Copy follows the better-writing skill.
- [ ] `pnpm check` passes.
