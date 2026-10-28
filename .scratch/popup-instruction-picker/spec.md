Labels: ready-for-agent

# Popup: pick the instruction set and rewrite from the popup

## Problem Statement

To rewrite a selection with a different instruction set, the user has to leave the popup: open Settings or the tray menu, switch the Active Instruction Set, go back to the other app, select the text again and press the shortcut again. The popup itself offers no way to change the set.

The popup also has no way to run the rewrite again. When the provider rejects the request, it times out, or the user simply doesn't like the result, the only way to retry is to press the shortcut again.

And every shortcut press starts a rewrite straight away, so a user who wants to pick the set first spends a rewrite (and tokens) on the wrong one.

## Solution

The popup gets an instruction set dropdown and a **Rewrite** button.

- The dropdown lists every instruction set and shows the Active Instruction Set. Picking a set there makes it the Active Instruction Set everywhere, the same as picking it in Settings or the tray.
- Picking a set never starts a rewrite on its own. It cancels a rewrite in progress, clears any result shown, and leaves the popup ready for the user to press Rewrite.
- Rewrite always runs a new rewrite of the same selection with the set picked. It is how the user retries after an error, retries a result they don't like, and runs a rewrite after changing the set.
- Enter replaces when a result is shown, and runs a rewrite otherwise.

A new setting, **Rewrite on open**, decides what the shortcut does:

- **On (the default):** the popup opens and the rewrite starts at once, as today.
- **Off:** the popup opens showing a preview of the selection, the dropdown and Rewrite, and nothing is sent until the user presses Rewrite.

## User Stories

1. As a user, I want to see which instruction set will be used in the popup, so that I know what the rewrite will do before trusting its result.
2. As a user, I want a dropdown in the popup listing all my instruction sets, so that I can switch sets without leaving the popup.
3. As a user, I want the dropdown to list my sets in the same order as Settings and the tray, so that I find them where I expect.
4. As a user, I want picking a set in the popup to make it the Active Instruction Set, so that the next shortcut press uses it too.
5. As a user, I want the tray's check mark and Settings to show the set I picked in the popup, so that the app agrees with itself everywhere.
6. As a user, I want picking a set never to start a rewrite by itself, so that I can browse sets without spending rewrites.
7. As a user, I want picking a set while a rewrite is running to cancel that rewrite, so that I don't wait for a result I no longer want.
8. As a user, I want a cancelled rewrite's late answer never to appear, so that the popup never shows a result from a set I moved away from.
9. As a user, I want picking a set to clear the result shown, so that I can't replace my selection with a result from a set that is no longer picked.
10. As a user, I want the popup to show my selection and the Rewrite button after I pick a set, so that it's clear what will happen next.
11. As a user, I want a Rewrite button in the popup, so that I can run the rewrite again without pressing the shortcut.
12. As a user, I want Rewrite to rewrite the same selection I captured, so that I don't have to go back and select the text again.
13. As a user, I want Rewrite to use the set currently picked, so that it reflects my latest choice.
14. As a user who got a result I don't like, I want to press Rewrite to get another one, so that I can try again with one click.
15. As a user whose rewrite failed (provider refused it, timed out, network error), I want the message to offer Rewrite, so that I can retry straight from the popup.
16. As a user, I want Rewrite to be unavailable while a rewrite is running, so that I can't start two at once.
17. As a user, I want Enter to replace when a result is shown, so that the fast path stays the same as today.
18. As a user, I want Enter to run a rewrite when no result is shown, so that I can drive the popup from the keyboard.
19. As a keyboard user, I want to reach the dropdown with Tab and change it with the arrow keys, so that I never need the mouse.
20. As a screen reader user, I want the dropdown to have a label naming it the instruction set, so that I know what it changes.
21. As a user with only one instruction set, I want the dropdown still shown but disabled, so that the popup layout is the same every time.
22. As a user, I want a Rewrite on open setting in Settings, so that I can choose whether the shortcut rewrites at once.
23. As a new user, I want Rewrite on open to be on by default, so that the app works as before out of the box.
24. As an existing user upgrading, I want Rewrite on open to be on after the update, so that nothing changes until I choose.
25. As a user with Rewrite on open off, I want the shortcut to open the popup without sending anything to the AI, so that I can pick a set first.
26. As a user with Rewrite on open off, I want the popup to show a preview of my selection, so that I can check what was captured before rewriting.
27. As a user with Rewrite on open off, I want Replace and Copy to be unavailable until a result exists, so that I can't act on nothing.
28. As a user with Rewrite on open off, I want the provider checks (missing key, model, base URL) to show when I press Rewrite, so that I learn what's missing at the moment it matters.
29. As a user with Rewrite on open off and nothing selected, I want the popup to stay hidden, as today, so that an empty popup never appears.
30. As a user with Rewrite on open off, I want a selection over the length limit to show the too-long message as today, so that I know it won't be sent.
31. As a user, I want pressing the shortcut while the popup is open to act like pressing Rewrite on the same selection, so that the shortcut I already know also retries.
32. As a user, I want that shortcut press to do nothing while a rewrite is running, the same as Rewrite, so that I can't start two at once.
33. As a user who clicks into another app to select new text, I want the popup to close as today, so that my next shortcut press is a fresh open with the new selection.
34. As a user, I want the dropdown to update while the popup is open when I add, rename or delete a set in Settings, so that it never lists sets that don't exist.
35. As a user, I want the dropdown to follow when I switch sets from the tray while the popup is open, so that it always shows the Active Instruction Set.
36. As a user, I want a set change made outside the popup to be treated like a pick in the popup (cancel, clear the result), so that the result always matches the set shown.
37. As a user, I want editing the instructions of the picked set in Settings to clear a result shown in the popup, so that I never replace with a result from old instructions.
38. As a user, I want Esc, ✕ and clicking outside to still close the popup and cancel any rewrite, so that closing works as it always has.
39. As a user, I want Copy and Replace to keep working as today once a result is shown, so that nothing I rely on breaks.
40. As a user, I want the clipboard restored after capture and replace as today, so that the new buttons never cost me my clipboard.
41. As a user, I want the popup to grow only as much as the dropdown row needs, so that it stays small and out of the way.
42. As a user who turned on reduced motion, I want the popup's state changes to appear without movement, so that the app respects my setting.

## Implementation Decisions

- **Rewrite session (the core of the change).** The session's events gain two:
  - `Rewrite`: run a new rewrite of the captured selection with the Active Instruction Set. Ignored while a rewrite is in progress, while replacing, or with nothing captured.
  - `PickInstructionSet(id)`: make that set the Active Instruction Set, cancel a rewrite in progress, clear any result shown, and show the ready view. Never starts a rewrite.
- **A captured selection outlives its rewrite.** Today a selection lives only inside a rewrite run. It now stays with the popup from capture until the popup closes, so Rewrite and PickInstructionSet can act on it.
- **A new popup state, `Ready`.** It carries the selection preview and means "a selection is captured and nothing is running". Shown when Rewrite on open is off, after a set is picked, and after a set change from outside the popup. Enter in `Ready` means Rewrite.
- **Message state gains Rewrite.** Messages about a failed rewrite (provider errors, timeouts, refusals) offer Rewrite next to any existing action. Messages where a rewrite can't help (nothing selected, too long, provider not set up) keep their current actions.
- **The popup learns the instruction sets.** Every popup state carries the list of sets (id and name, in order) and the Active Instruction Set's id, so the dropdown always renders from the state it's given and the popup holds no state of its own. Names only: instructions are never sent to the popup.
- **Shortcut press while open.** Acts exactly like the Rewrite event, with no capture. The popup has focus at that moment, so a capture would copy from the popup itself, and clicking into another app to select new text already closes the popup. The only change from today's "cancel and rerun" is that a press while working is now ignored, like Rewrite.
- **Set changes from outside the popup.** When the instruction sets change in Settings or the tray while the popup is open, the session is told. If the Active Instruction Set changed, or its instructions were edited, it behaves as for PickInstructionSet. Otherwise (a rename or another set added or deleted) it only re-renders the current state with the new list.
- **Settings.** `Settings` gains `rewrite_on_open: bool`, default `true`, so a file from before this change loads with it on. The settings view the session reads gains a way to read it and a way to set the Active Instruction Set. Settings shows it as a switch in the Shortcut and startup section and applies it at once, like the other switches there. It is not in the tray.
- **Popup commands.** The popup gains a command for Rewrite and one for picking a set. Both feed the session's events, like the existing Replace, Copy and Close commands.
- **Popup UI.** A labelled native `select` for the dropdown and a Rewrite button, using the existing tokens and motion from DESIGN.md. The dropdown is disabled while only one set exists. Replace and Copy show only in the result state. Rewrite is disabled while working.
- **Glossary.** The Active Instruction Set's definition changes from "the one the shortcut uses" to "the one a rewrite uses; changing it in the popup, tray or settings changes it everywhere". No new domain terms: picking a set and pressing Rewrite are both described with existing words.
- **No ADR.** The change is easy to reverse.

## Testing Decisions

- **Good tests here** drive the session only through its events and assert only on what the popup is told to show, what the rewriter was asked, and what was copied or replaced. They never look at the session's internal phase.
- **The rewrite session is the one seam.** Its tests already use a fake text bridge, a fake rewriter (including a gated one for in-progress rewrites), fake settings and a recording popup. The new tests are written the same way:
  - With Rewrite on open off, the shortcut shows `Ready` with the selection and the rewriter is never called.
  - Rewrite from `Ready` shows working, then the result, using the active set's instructions.
  - Rewrite from a result runs a new rewrite of the same selection.
  - Rewrite from a failed-rewrite message runs a new rewrite.
  - Rewrite while working does nothing.
  - Picking a set while working cancels the rewrite and a late answer is ignored.
  - Picking a set with a result shown clears it and shows `Ready`, and makes that set active.
  - Picking a set never calls the rewriter.
  - The shortcut while open acts like Rewrite: no capture, ignored while working.
  - A set change from outside while a result is shown clears it. A rename only re-renders.
  - Every state carries the set list and the active id.
  - Existing tests keep passing, adjusted only where shortcut-while-open behaviour changed.
- **Settings store.** One test that a saved file without the new field loads with Rewrite on open on, following the existing store tests.
- **Not automated.** The popup's look, the dropdown's keyboard behaviour and the Settings switch go into the manual test checklist.

## Out of Scope

- Showing a small indicator automatically when text is selected (the Grammarly-style floating button). Discussed and set aside: it needs a global mouse hook, which ADR 0005's reasoning argues against, and UI Automation capture, which the v1 spec ruled out.
- A one-time set pick that doesn't change the Active Instruction Set.
- Typing a custom instruction in the popup.
- Editing the result in the popup, highlighting changes, or streaming the result.
- Keyboard shortcuts to jump straight to a set.
- A Rewrite on open toggle in the tray.

## Further Notes

- The dropdown never shows instructions, only names, and the selection preview follows the existing rule: user data, never logged.
- The prior research (writing tools such as Grammarly, Apple Writing Tools, Chrome Help me write and Notion AI) all offer a retry and a way to change the style next to the result, which is what this spec adds without the selection-triggered indicator.
