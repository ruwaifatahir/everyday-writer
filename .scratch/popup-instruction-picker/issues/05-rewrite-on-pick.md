# 05: Picking a set rewrites at once

**What to build:** Picking a set in the popup's dropdown starts a rewrite with it straight away, the same as pressing Rewrite after the pick, so one pick gets the user the new result. A rewrite in progress is cancelled first and its late answer is ignored. On a closed dropdown the plain ↑ and ↓ keys do nothing: a keyboard user opens the list with Alt+↓, moves with the arrows and picks with Enter, so every pick sends exactly one rewrite. Changes made in the tray or Settings while the popup is open still don't rewrite: they clear the popup to Ready as before. This replaces ticket 02's rule that a pick never starts a rewrite. See the spec, user stories 6, 7, 10 and 19.

**Blocked by:** 01, 02, 03, 04 (all resolved)

**Status:** resolved

- [x] A pick in the popup makes the set active and then runs a rewrite exactly as the Rewrite event would: working, then the result or message, using the picked set's instructions
- [x] A pick while a rewrite is running cancels it and starts a new one with the picked set; the old rewrite's late answer is never shown
- [x] A pick on the too-long message keeps that message and never calls the rewriter; a pick on the not-set-up message checks the provider again, as Rewrite does
- [x] Picking the set that is already active still changes nothing; a pick that can't be saved still puts the dropdown back and sends nothing
- [x] A set change from the tray or Settings (switch, delete the active set, edit its instructions) still clears to Ready and never calls the rewriter
- [x] The popup's own pick, announced back by the settings listener, never starts a second rewrite or clears the one just started
- [x] Ready stays for Rewrite on open off and for changes from outside the popup; a pick from Ready runs the rewrite
- [x] On the closed dropdown, plain ↑/↓ are blocked (no `change`, no pick); Alt+↓ still opens the list, and arrows plus Enter in the open list pick once
- [x] Enter on the closed dropdown keeps its current meaning: Rewrite on Ready and on a failed rewrite, Replace on a result
- [x] Session tests replace "pick never calls the rewriter" with: pick runs a rewrite with the new instructions, pick while working cancels and reruns with a late answer ignored, pick on too-long sends nothing, the announced-back pick starts no second rewrite, an outside change still sends nothing
- [x] DESIGN.md, the manual test checklist (including Alt+↓, arrows, Enter in the real popup sending one rewrite, and plain ↑/↓ doing nothing) and the CHANGELOG's Unreleased line for the dropdown describe the new behaviour
- [x] `pnpm check` passes

## Comments

**2026-10-28:** Decided in a grilling session after tickets 01 to 04 were merged. Chromium on Windows fires `change` on every plain arrow press on a closed `select` (checked: three presses gave three changes), which would send one rewrite per press. With the list opened by Alt+↓, arrows only move the highlight and Enter fires a single `change`, and Enter there doesn't reach the page's keydown handler (checked in Chrome; confirm in the WebView2 popup by hand). So blocking plain arrows on the closed list removes the need for any delay. `stage` isn't pushed until this ticket is in.

**2026-10-28:** Done in `e2fe0f1`, `c01b6bf`, `e09bd10` and `7f2bfa4`. The closed dropdown lets through only keys that keep the set: Tab, Esc, Enter, the keys that open the list (Alt+↓, Alt+↑, F4, Space), and modifier or Ctrl/Win combinations. Arrows, Home, End, Page Up/Down and letters are blocked, because each one fires `change` (checked in Chrome; Ctrl combinations checked too and change nothing). Enter on the dropdown with a result shown now replaces, which it didn't do before. Before saving a pick, the session records the sets as they will be after it, so the announcement coming back never clears the new rewrite, in any order (tested in three orders).
