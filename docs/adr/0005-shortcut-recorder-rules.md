# The shortcut field only records, and a new shortcut needs Ctrl or Alt

The shortcut field in Settings only records key presses, as Windows' hot key control, PowerToys, VS Code and browser extension settings do, because typing let the field hold text that wasn't a shortcut. While it records, the saved shortcut pauses, or Windows would send its press to the app and start a rewrite instead of letting the field record it. A new shortcut needs Ctrl or Alt and may add Shift. Win is refused because Windows takes most Win combinations (Win + K opens Cast) before the field sees them. Ctrl + C and Ctrl + V are refused because the app presses them to copy the selection and paste the result. Ctrl + Alt is allowed with a warning, since on keyboards with AltGr it also types characters.

## Considered Options

- **A low-level keyboard hook to catch Win and combinations other apps hold** (the PowerToys approach): rejected. It is heavy, runs on every key press in every app, and the app is meant to stay small.
- **Refuse Ctrl + Alt outright:** rejected. Many layouts have no AltGr, and those users lose a whole family of free combinations; a warning is enough.

## Consequences

- The rules apply only when picking a shortcut. A saved shortcut still loads by the older rules (Ctrl, Alt or Win), so a Win shortcut from an earlier version keeps working and showing until the user changes it, but it can't be picked again.
- Rust judges what is allowed: it refuses with a reason (not a shortcut, no Ctrl or Alt, Win, copy and paste, taken, couldn't be saved) and Settings words it. The one thing Settings decides is the AltGr advice after a Ctrl + Alt shortcut saves, since it is only a message and changes nothing.
- The pause holds only while Settings asks for it and its window has focus. Rust brings the shortcut back when the window loses focus or closes and pauses it again when focus returns to a field that still records, so a pause or resume that reaches Rust late can't leave the app without its shortcut. If another app took the shortcut while it was paused, a tray notification says so.
- A combination another app holds never reaches the field, so it can't be recorded at all. The hint under the field says so.
