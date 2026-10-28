# 02: Instruction set dropdown in the popup

**What to build:** The popup's header shows a labelled dropdown of every instruction set, by name, in the same order as Settings and the tray, with the Active Instruction Set selected. Picking a set makes it the Active Instruction Set everywhere (the tray's check mark and Settings follow). Picking never starts a rewrite: it cancels a rewrite in progress (a late answer is ignored), clears any result shown, and shows a new **Ready** view: the selection preview with Rewrite, where Enter means Rewrite and Copy and Replace are not shown. With only one set the dropdown is shown but disabled. The glossary's Active Instruction Set entry is updated. See the spec, user stories 1 to 10, 18 to 21 and 41 to 42.

**Blocked by:** 01 (Rewrite button)

**Status:** resolved

- [x] Every popup state carries the sets (id and name only, never instructions) and the Active Instruction Set's id, and the popup renders the dropdown from it
- [x] The session has a pick-a-set event: it sets the Active Instruction Set, cancels a rewrite in progress, clears a result or failed-rewrite message, and shows Ready; it never calls the rewriter
- [x] Picking the set that is already active changes nothing
- [x] The popup has a Ready state showing the selection preview, Rewrite as the primary action with Enter, and no Copy or Replace
- [x] Rewrite from Ready runs a rewrite with the newly picked set
- [x] Picking a set in the popup updates the tray menu's check mark and the Settings window, through the same path the existing set-active command uses
- [x] The dropdown is a native select with an accessible name, reachable with Tab and changed with the arrow keys; it is disabled when only one set exists
- [x] A popup command feeds the pick-a-set event, with its capability entry
- [x] Session tests cover: pick while working cancels and a late answer is ignored, pick with a result clears it and shows Ready, pick never calls the rewriter, Rewrite after a pick uses the new instructions, every state carries the sets and active id
- [x] GLOSSARY.md: Active Instruction Set reads "The one instruction set a rewrite uses. Changing it in the popup, tray or settings changes it everywhere."
- [x] DESIGN.md and the manual test checklist cover the dropdown and the Ready view
- [x] `pnpm check` passes

## Comments

**2026-10-28:** Done in `868cc68` and `a6a83cb`. The popup stays 400 × 220: the dropdown fits in the existing header. A pick that can't be saved sends the same state again so the dropdown goes back to the real active set. Enter in the dropdown runs Rewrite on Ready. The glossary wording replaces "Exactly one is active at any time." After review (`e5a0e3c`), a pick keeps messages a rewrite can't help (too long, not set up), the same rule as changes from elsewhere.

**2026-10-28:** The rule that a pick never starts a rewrite, and the Ready view after a pick, are replaced by [ticket 05](05-rewrite-on-pick.md): a pick in the popup now rewrites at once.
