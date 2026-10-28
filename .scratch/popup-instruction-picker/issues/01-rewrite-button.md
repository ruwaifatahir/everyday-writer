# 01: Rewrite button

**What to build:** The popup gets a **Rewrite** button that runs a new rewrite of the same selection with the Active Instruction Set, without the user going back to select the text again. It sits next to Copy and Replace when a result is shown, and next to the message when a rewrite failed (provider refused it, busy, network error, timed out, sent back nothing). Messages where a retry can't help (nothing to rewrite, too long, key or model or base URL missing, key rejected) keep only their current action. Pressing the shortcut while the popup is open now acts exactly like Rewrite, with no capture, and is ignored while a rewrite is running. See the spec, user stories 11 to 17 and 31 to 33.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The session has a Rewrite event: from a result or a failed-rewrite message it shows working, then the outcome, using the same selection and the Active Instruction Set's current instructions
- [ ] Rewrite while working, while replacing, or with the popup hidden does nothing
- [ ] Failed-rewrite messages tell the popup that Rewrite is offered; the other messages don't
- [ ] The shortcut while the popup is open behaves like Rewrite (no capture), and does nothing while working
- [ ] The popup shows Rewrite with the result (secondary, beside Copy and Replace) and with a failed-rewrite message (primary, with Enter); Enter on that message runs Rewrite
- [ ] Rewrite is disabled while working; Enter on a result still replaces
- [ ] A popup command feeds the Rewrite event, with its capability entry
- [ ] Session tests cover: rewrite from a result, rewrite from a failed-rewrite message, rewrite ignored while working, no Rewrite on a not-ready or too-long message, the shortcut while open acting as Rewrite; existing shortcut-while-open tests are adjusted
- [ ] The manual test checklist covers the button, Enter on a failed-rewrite message, and the shortcut while open
- [ ] The CHANGELOG's Unreleased section mentions the Rewrite button
- [ ] `pnpm check` passes
