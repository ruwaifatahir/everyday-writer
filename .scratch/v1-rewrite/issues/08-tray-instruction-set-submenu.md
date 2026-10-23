# 08: Tray Instruction set submenu

**What to build:** Right-clicking the tray icon shows an **Instruction set** submenu, followed by Settings and Quit. The submenu lists every set in order, with a check by the active one. Clicking a set makes it active. The submenu rebuilds whenever sets are added, renamed, deleted, restored or switched in Settings.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 07 (Instruction sets)

**Status:** resolved

- [ ] The submenu lists every set, with a check by the active one
- [ ] Clicking a set makes it active; the next rewrite uses it, and Settings shows it as active
- [ ] Changes made in Settings show up in the tray without restarting the app
- [ ] Menu order: Instruction set, Settings, Quit

## Comments

Merged in 3f4bea9. `pnpm check` passes. Still to verify in the running app:

- The submenu lists every set, with a check by the active one
- Clicking a set makes it active; the next rewrite uses it, and Settings shows it as active
- Changes made in Settings show up in the tray without restarting the app
- Menu order: Instruction set, Settings, Quit
- Re-clicking the active set keeps the check
- `&` in set names shows as typed
