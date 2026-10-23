# 07: Instruction sets

**What to build:** The user manages instruction sets in Settings, and every rewrite uses the active set.

- **Starter templates:** on first launch the app installs Fix & make readable (active), Fix mistakes only, Professional and Shorter.
- **Managing sets:** the user can add a set, edit its name and instructions, delete it, and make it the active set.
- **Restore default templates:** a button that asks to confirm, then re-adds missing starters at the end and resets edited ones in place. User-written sets are never touched, and the active set stays active.
- **Deleting:** the last set can't be deleted. Deleting the active set makes the first remaining set active.

The rules live in a pure instruction set library, saved with the settings. Each starter keeps a stable identifier so Restore can find it. The starter wording is designed with the `prompt-optimizer` skill, and every starter keeps the result in the language the user wrote in.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 02 (Minimal settings), 03 (Tracer bullet: shortcut, popup, Copy)

**Status:** resolved

- [x] Library tests cover add, edit, set active, delete the active set, refuse to delete the last set, and Restore (missing starter re-added, edited starter reset, user set untouched, active set unchanged)
- [x] The rewrite session uses the active set's instructions, and the built-in placeholder from 03 is gone
- [ ] Instruction sets and the active set survive a restart
- [ ] Restore asks for confirmation in the settings UI
- [x] Each starter's wording is reviewed against the spec's description of it, and the result stays in the user's language
- [ ] The instruction set UI is modern, minimal and clean (`impeccable` skill)

## Comments

Merged into feat/v1-rewrite at 54e5c96. `pnpm check` passed.

Still to verify manually:
- Instruction sets and active set survive a restart in the real app
- Restore confirm dialog in a real window
- Visual look of the instruction set UI (impeccable)
- Model-quality evals of the starter wording
