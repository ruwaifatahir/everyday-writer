# 02: Instruction set dropdown in the popup

**What to build:** The popup's header shows a labelled dropdown of every instruction set, by name, in the same order as Settings and the tray, with the Active Instruction Set selected. Picking a set makes it the Active Instruction Set everywhere (the tray's check mark and Settings follow). Picking never starts a rewrite: it cancels a rewrite in progress (a late answer is ignored), clears any result shown, and shows a new **Ready** view: the selection preview with Rewrite, where Enter means Rewrite and Copy and Replace are not shown. With only one set the dropdown is shown but disabled. The glossary's Active Instruction Set entry is updated. See the spec, user stories 1 to 10, 18 to 21 and 41 to 42.

**Blocked by:** 01 (Rewrite button)

**Status:** ready-for-agent

- [ ] Every popup state carries the sets (id and name only, never instructions) and the Active Instruction Set's id, and the popup renders the dropdown from it
- [ ] The session has a pick-a-set event: it sets the Active Instruction Set, cancels a rewrite in progress, clears a result or failed-rewrite message, and shows Ready; it never calls the rewriter
- [ ] Picking the set that is already active changes nothing
- [ ] The popup has a Ready state showing the selection preview, Rewrite as the primary action with Enter, and no Copy or Replace
- [ ] Rewrite from Ready runs a rewrite with the newly picked set
- [ ] Picking a set in the popup updates the tray menu's check mark and the Settings window, through the same path the existing set-active command uses
- [ ] The dropdown is a native select with an accessible name, reachable with Tab and changed with the arrow keys; it is disabled when only one set exists
- [ ] A popup command feeds the pick-a-set event, with its capability entry
- [ ] Session tests cover: pick while working cancels and a late answer is ignored, pick with a result clears it and shows Ready, pick never calls the rewriter, Rewrite after a pick uses the new instructions, every state carries the sets and active id
- [ ] GLOSSARY.md: Active Instruction Set reads "The one instruction set a rewrite uses. Changing it in the popup, tray or settings changes it everywhere."
- [ ] DESIGN.md and the manual test checklist cover the dropdown and the Ready view
- [ ] `pnpm check` passes
