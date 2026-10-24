# The shortcut field only records, and a new shortcut needs Ctrl or Alt

The shortcut field in Settings only records key presses, as Windows' hot key control, PowerToys, VS Code and browser extension settings do, because typing let the field hold text that wasn't a shortcut. While it records, the saved shortcut pauses, or Windows would send its press to the app and start a rewrite instead of letting the field record it. A new shortcut needs Ctrl or Alt and may add Shift. Win is refused because Windows takes most Win combinations (Win+K opens Cast) before the field sees them. Ctrl+C and Ctrl+V are refused because the app presses them to copy the selection and paste the result. Ctrl+Alt is allowed with a warning, since on keyboards with AltGr it also types characters.

## Considered Options

- **A low-level keyboard hook to catch Win and combinations other apps hold** (the PowerToys approach): rejected. It is heavy, runs on every key press in every app, and the app is meant to stay small.
- **Refuse Ctrl+Alt outright:** rejected. Many layouts have no AltGr, and those users lose a whole family of free combinations; a warning is enough.

## Consequences

- The rules apply only when picking a shortcut. A saved shortcut still loads by the older rules (Ctrl, Alt or Win), so a Win shortcut from an earlier version keeps working and showing until the user changes it, but it can't be picked again.
- Rust is the only judge: it refuses with a reason (not a shortcut, no Ctrl or Alt, Win, copy and paste, taken, couldn't be saved) and Settings only words it.
- A combination another app holds never reaches the field, so it can't be recorded at all. The hint under the field says so.
