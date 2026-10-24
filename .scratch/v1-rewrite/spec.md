Labels: ready-for-agent

# v1 rewrite

## Problem Statement

People make typos and grammar mistakes in everyday writing: emails, chats, docs and posts. Quick messages often come out messy or hard to read. Today, fixing them means copying the text into a chat assistant, typing the same instructions again ("fix grammar, make it readable, no hyphens") and pasting the result back.

The base project already runs as an empty background Windows app with a tray icon, a Quit item, keyring storage for API keys and a provider-agnostic rewrite interface. It does nothing yet. Nothing reacts to a shortcut, nothing captures a selection, there is no popup and no settings screen.

## Solution

The user selects text in any app and presses one shortcut (default `Ctrl+Shift+Space`). The app captures the selection and opens a small popup near the mouse that shows it is working. It then sends the selection to the AI provider the user chose, together with the active instruction set. The popup shows the result exactly as it came back, with **Replace** (Enter), **Copy** and **✕** (Esc). Replace puts the result into the original app in place of the selection. The user's clipboard is put back the way it was.

The user sets up the app in a small settings window. There they pick a provider (OpenAI, Anthropic, Gemini, OpenRouter, Ollama or Custom) and a model, add a key, change the shortcut, manage their instruction sets and turn start-with-Windows on or off. The app starts with four starter templates, so it works with no writing needed. The tray menu lets the user switch the active instruction set, open Settings or quit.

## User Stories

### Rewriting

1. As a user, I want to select text in any app and press one shortcut, so that I can rewrite it without leaving what I'm doing.
2. As a user, I want the popup to open right away near my mouse, so that I know the shortcut worked.
3. As a user, I want the popup to always be fully on the screen my mouse is on, so that it's never cut off at an edge or split across monitors.
4. As a user, I want a small animation while the AI works, so that I know the only wait is the AI.
5. As a user, I want the result shown exactly as the AI returned it, so that what I see is what gets inserted.
6. As a user, I want the same three buttons every time (Replace, Copy, ✕), so that the popup is predictable.
7. As a user, I want the buttons to stay the same even when the result matches my selection, so that the popup never second-guesses the AI.
8. As a user, I want to press Enter to Replace, so that the common case needs no mouse.
9. As a user, I want to press Esc to close the popup, so that I can dismiss it quickly.
10. As a user, I want Replace to put the result in place of my selection in the original app, so that I don't have to paste it myself.
11. As a user, I want my original app to come back to the front with focus after Replace, so that I can keep typing.
12. As a user, I want Copy to put the result on my clipboard and close the popup, so that I can paste it where Replace can't reach.
13. As a user, I want the result copied, with a short note, if the original app closed before I pressed Replace, so that I don't lose the result.
14. As a user, I want Replace to do nothing harmful where text can't be edited (like a web page), so that I can use Copy there instead.
15. As a user, I want clicking outside the popup to close it, so that it never gets in my way.
16. As a user, I want closing the popup while the AI works to cancel the rewrite, so that nothing happens later behind my back.
17. As a user, I want pressing the shortcut while the popup is open to cancel the current rewrite and try again on the same selection, so that I can get another result if I don't like this one.
18. As a user, I want nothing to happen when I press the shortcut with nothing selected, so that a stray press is harmless.
19. As a user, I want a short "too long" message for a selection over 5,000 characters, so that I know why nothing was sent.
20. As a user, I want a short, friendly message when there is no internet, so that I know what went wrong.
21. As a user, I want a short message when my key is rejected, so that I know to fix my key.
22. As a user, I want a short message when I'm being rate limited, so that I know to wait.
23. As a user, I want a message after 30 seconds if the AI hasn't answered, so that the popup never spins forever.
24. As a user, I want the popup to tell me to add a key, with an **Open Settings** button, when I press the shortcut before setting up, so that I can finish setup in one click.
25. As a user, I want my clipboard put back the way it was after capture and after Replace, so that whatever I had copied before is still there.
26. As a user, I want text, rich text, HTML, copied files and images on my clipboard to survive a rewrite, so that I don't lose what I copied before.
27. As a user, I want the app's own clipboard writes kept out of Windows clipboard history and cloud clipboard, so that the result and my restored clipboard don't pile up there.
28. As a user with a non-Latin keyboard layout (Arabic, Urdu, Cyrillic), I want capture and Replace to work, so that a literal "c" or "v" is never typed into my text.
29. As a user, I want the keys I'm still holding from the shortcut to not interfere with capture, so that Windows shortcuts like the colour filter or Start menu never fire by accident.
30. As a user, I want the result to stay in the language I wrote in unless my instructions say otherwise, so that rewriting never translates by surprise.
31. As a user, I want the result to contain only the rewritten text, so that I never get "Sure! Here's…" pasted into my message.
32. As a user, I want my selected text treated only as text to rewrite, so that a selection like "ignore your instructions" is rewritten, not obeyed.

### Instruction sets

33. As a new user, I want four starter templates installed (Fix & make readable, Fix mistakes only, Professional, Shorter), so that I can start with no writing.
34. As a new user, I want "Fix & make readable" active by default, so that the first rewrite does what most people want.
35. As a user, I want to save several named instruction sets, so that I can rewrite chats, emails and work messages differently.
36. As a user, I want exactly one active instruction set, so that the shortcut always knows what to do.
37. As a user, I want to edit any instruction set, including the starters, so that the app writes like me.
38. As a user, I want to write a new instruction set from scratch, so that I can use the app for any purpose (for example, Roman Urdu in, clean English out).
39. As a user, I want to delete an instruction set, so that my list stays short.
40. As a user, I want the last remaining instruction set to be undeletable, so that the shortcut always has something to use.
41. As a user, I want deleting the active set to make the first remaining set active, so that the app keeps working.
42. As a user, I want a **Restore default templates** button, so that I can get the starters back.
43. As a user, I want Restore to ask me first, so that I don't reset my edits by accident.
44. As a user, I want Restore to bring back deleted starters and reset edited ones to the original, so that I end up with the real starters.
45. As a user, I want Restore to never touch instruction sets I wrote myself, so that my own work is safe.
46. As a user, I want the active set to stay active if Restore resets it, so that my choice doesn't change under me.
47. As a user, I want instruction sets to keep their order, so that "first remaining set" is predictable.

### Setup and settings

48. As a new user, I want Settings to open on their own the first time I start the app, so that I can set it up right away.
49. As a user, I want to pick my provider from OpenAI, Anthropic (Claude), Google Gemini, OpenRouter, Ollama and Custom, so that I can use the AI I already pay for or run locally.
50. As a user, I want Custom to accept any OpenAI-compatible base URL (for example Vercel AI Gateway or LM Studio), so that I can use any compatible service.
51. As a user, I want one Custom provider at a time, so that settings stay simple.
52. As a user, I want to pick a model from the provider's own list when it can be fetched, so that I don't have to know model names.
53. As a user, I want to type any model name, so that I can use a model the list doesn't show or when the list can't be fetched.
54. As a user, I want to enter my key for the chosen provider, so that requests go out under my account.
55. As a user, I want the key to be optional for Ollama and Custom, so that local AI servers work.
56. As a user, I want each provider's key kept separately, so that switching provider and back doesn't lose a key.
57. As a user, I want my key stored in the Windows Credential Manager, never in a plain file, so that it stays safe.
58. As a user, I want the key field to never show my saved key back in plain text, so that someone looking at my screen can't read it.
59. As a user, I want all instruction sets to use the same provider and model, so that there is one setup.
60. As a user, I want to change the shortcut, so that it doesn't clash with my other apps.
61. As a user, I want Settings to tell me when another app already uses the shortcut and keep my old one, so that I'm never left without a working shortcut.
62. As a user, I want a tray notification and Settings to open if my shortcut can't be registered at startup, so that I find out right away instead of wondering why nothing works.
63. As a user, I want the app to start with Windows by default, so that it's always ready.
64. As a user, I want to turn start-with-Windows off, so that I control what runs at login.
65. As a user, I want Settings to hold only the provider, model, key, shortcut, instruction sets and the start-with-Windows switch, so that there's nothing to wade through.
66. As a user, I want my settings kept after a restart, so that I set up once.

### Tray

67. As a user, I want the app to live only in the tray, so that it stays out of sight.
68. As a user, I want an **Instruction set** submenu in the tray menu with a check by the active set, so that I can see which one the shortcut uses.
69. As a user, I want to click a set in that submenu to make it active, so that I can switch without opening Settings.
70. As a user, I want the tray menu to update when I add, rename, delete or restore sets in Settings, so that it always matches.
71. As a user, I want **Settings** and **Quit** in the tray menu, so that I can reach setup or stop the app.
72. As a user, I want a second launch of the app to open Settings instead of starting another copy, so that double-clicking the shortcut on my desktop does something useful.

### Privacy and trust

73. As a user, I want my text sent only to the provider I chose, so that nothing goes anywhere I didn't pick.
74. As a user, I want the provider to never be guessed from the model name or read from environment variables, so that my text and key can't go to the wrong place.
75. As a user, I want the app to never store, log or send my text anywhere else, so that my writing stays private.
76. As a user, I want the app to never log my clipboard, the result or my key, so that log files hold nothing private.
77. As a user, I want no tracking or analytics, so that nothing about my use leaves my machine except the rewrite request.
78. As a user, I want to be told that Windows clipboard history (Win+V) may show the text I rewrite, so that I'm not surprised.

### Performance

79. As a user, I want the popup, Replace and menus to respond instantly, so that the only wait is the AI.
80. As a user, I want the app to use almost no memory, CPU or battery while idle, so that I never notice it running.

### Developer

81. As the developer, I want the code to use the glossary's terms (Rewrite, Selection, Result, Replace, Instruction Set), so that code and docs say the same thing.
82. As the developer, I want the whole rewrite flow behind one rewrite session with fake ports, so that I can test every popup behaviour with no Windows, no network and no key.
83. As the developer, I want instruction set rules as pure logic, so that I can test them directly.
84. As the developer, I want a manual test checklist for capture and Replace in real apps, so that the parts unit tests can't reach are still checked before release.

## Implementation Decisions

### Vocabulary and renames

- The `Corrector` interface, its prompt type and its error type are renamed to match the glossary: **Rewriter**, rewrite prompt, rewrite error. The genai-backed implementation and the fake are renamed with them. Behaviour stays the same.
- "Correct", "fix" and "correction" are no longer used for the feature. "Fix mistakes only" survives only as the name of a starter template.

### Modules

- **Rewrite session (new, deep module).** It owns the whole flow and is the only module that knows the order of steps. Inputs are events: shortcut pressed, Replace, Copy, close (✕, Esc, or click outside), Open Settings. It depends only on four ports:
  - a **text bridge**: capture the selection, Replace into the original window, copy text, and report whether the original window still exists;
  - the **Rewriter**;
  - a **settings view**: the active instruction set's instructions, and whether the chosen provider is ready (a key is set, or the provider allows none);
  - a **popup view**: receives popup states and shows or hides the popup.
- **Text bridge, Windows implementation (new).** It captures and replaces using Win32 and is the only place with `unsafe`/FFI. It sits on the existing capture and replace modules.
- **Instruction set library (new, pure).** It holds an ordered list of instruction sets plus the active one. Operations: add, edit (name and instructions), delete, set active, restore default templates. Each starter template carries a stable identifier, so Restore can find edited or deleted starters. A user-written set has no starter identifier.
- **Settings (extended).** It loads and saves the provider, model, Custom base URL, shortcut, start-with-Windows and the instruction set library as a plain settings file in the app's data folder. It never contains the key. The key stays in the Windows Credential Manager, one entry per provider.
- **Rewriter (extended).** The provider list gains OpenRouter and Ollama. It also gets a model-list call for the settings dropdown.
- **Shortcut (extended).** It registers the user's shortcut, reports a registration failure as a distinct error, and sends a "shortcut pressed" event to the rewrite session.
- **Tray (extended).** It adds the Instruction set submenu, rebuilt whenever the library changes, plus Settings and Quit.
- **Frontend.** Two windows: the popup and settings, in vanilla TypeScript. Both use Tauri commands and events only. The popup gets its state from the session and sends button events back. It has no logic of its own.

### Rewrite session behaviour

- Popup states: hidden, working, result, message. A message is a short text with an optional action (Open Settings).
- **On shortcut pressed while the popup is hidden:**
  1. Capture first.
  2. Nothing selected: stay hidden.
  3. Selection over 5,000 characters: show the "too long" message without calling the Rewriter.
  4. Provider not ready: show the missing-key message with Open Settings, without calling the Rewriter.
  5. Otherwise: show working, then call the Rewriter with the active instruction set and the selection.
- **On shortcut pressed while the popup is open:** cancel any rewrite in progress and start a new one on the same selection, with the current active instruction set. Nothing is captured again.
- **Timeout:** a rewrite that hasn't answered after 30 seconds is cancelled and shows the "took too long" message. 30 seconds is a constant, not a setting.
- **Close in any state:** cancels any rewrite in progress and hides the popup. A cancelled rewrite never updates the popup later.
- **Result:** shown as-is, with Replace, Copy and ✕, whether or not it differs from the selection.
- **Replace:** hide the popup. If the original window still exists, Replace into it. If it's gone, copy the result and show the "original window closed, copied instead" message.
- **Copy:** copy the result and hide the popup.
- **Rewrite errors:** each maps to one short, friendly message (no internet, key rejected, rate limited, provider error, empty response, other). The messages carry no request or response content.
- Every popup always offers the same three buttons. Read-only targets are not detected: nothing can detect a failed paste reliably, and no comparable tool tries.

### Text bridge, Windows implementation

- **Capture**, following the Microsoft PowerToys approach:
  1. Save the clipboard.
  2. Release any modifier keys still held from the shortcut, so Ctrl+C isn't sent as Ctrl+Win+C.
  3. Send Ctrl+C with virtual-key codes, never letters, so it works on every keyboard layout.
  4. Wait up to about 500 ms for the clipboard sequence number to change. No change means nothing was selected.
  5. Read the text, then restore the clipboard.
- **Replace:**
  1. Bring the original window back to the foreground. The window handle was recorded at capture.
  2. Write the result to the clipboard, marked to stay out of history and cloud.
  3. Send Ctrl+V with virtual-key codes, then restore the clipboard about 300 ms later.
- **Clipboard snapshot:** every memory-based format (text, rich text, HTML, file lists, DIB/PNG images, and registered formats) is saved and restored. GDI-handle formats are skipped (bitmap handles, metafiles, palettes, and formats Windows synthesizes). This is a known, accepted gap.
- **Clipboard history and cloud:** every clipboard write the app makes is tagged with the exclusion format, so it stays out of Win+V history and cloud clipboard. The target app's own copy of the selection can't be excluded. The PRD's privacy section says so.
- Opening and flushing the clipboard is retried a few times, because it fails now and then when another app holds it.
- Candidate crates: `clipboard-win` (format enumeration, raw get/set, sequence number), `windows` (SendInput, key state, foreground window), and `arboard` only if it covers a need more simply. Each comes with minimal features. `enigo` isn't needed if SendInput is called directly.

### Providers (per ADR 0001 and its amendment)

- **Provider choices:**

  | Provider | Adapter | Endpoint |
  |---|---|---|
  | OpenAI | genai OpenAI (Chat Completions) | OpenAI's API |
  | Anthropic | genai Anthropic | Anthropic's API |
  | Gemini | genai Gemini | Google's Gemini API |
  | OpenRouter | genai OpenRouter | OpenRouter's API |
  | Ollama | genai Ollama (native API) | Local Ollama by default |
  | Custom | genai OpenAI (Chat Completions) | The user's base URL |

- Each call still builds an explicit service target: adapter, endpoint and key. There is no genai auth resolver, no environment-variable fallback and no model-name inference.
- The key is required for OpenAI, Anthropic, Gemini and OpenRouter. It's optional for Ollama and Custom. A blank key for Custom is sent as an empty key, never treated as "look elsewhere".
- **Model list:** a Rewriter call fetches the provider's model names with the same explicit target. If it fails, the dropdown falls back to a short built-in suggested list per provider. The user can always type a name instead.

### Instruction set rules

- The list is ordered, and new sets go at the end.
- Exactly one set is active. Deleting the active set makes the first remaining set active.
- The last set can't be deleted, so the delete operation refuses.
- **Restore default templates:** every starter missing from the list is re-added at the end. Every starter still present gets its original name and instructions back, in place. User-written sets are untouched. The active set stays the same. The confirm dialog lives in the settings UI, not in the library.
- **Starter templates:** Fix & make readable (active by default), Fix mistakes only (no rewording), Professional (emails and work messages), Shorter (same message, fewer words). Their exact instruction text is a prompt-design task: use the `prompt-optimizer` skill. Every starter keeps the result in the language the user wrote in.

### App shell

- **First launch:** no settings file means Settings opens and the starter templates are installed.
- **Second launch:** a second launch opens Settings in the running app.
- **Start with Windows:** on by default through the autostart plugin.
- **Shortcut registration failure:** in Settings, the error is shown and the previous shortcut is kept. At startup, a tray notification is shown and Settings opens. Conflicts from apps that use low-level keyboard hooks can't be detected. That's accepted.

## Testing Decisions

- Good tests check external behaviour through a module's public interface, not its internals. Test names use the glossary's terms. No test calls a real LLM, the real clipboard or the real keyring.
- **Rewrite session (main seam).** Tests drive it with events and use a fake text bridge, fake Rewriter, fake settings view and recording popup view. They assert the sequence of popup states and the calls made to the text bridge. The behaviours to cover:
  - nothing selected leaves the popup hidden;
  - a selection over 5,000 characters shows "too long" and never calls the Rewriter;
  - a missing key shows the message with Open Settings and never calls the Rewriter;
  - a successful rewrite goes from working to result;
  - an unchanged result still shows as a result;
  - Replace into an existing window;
  - Replace into a closed window copies the result and shows the message;
  - Copy;
  - closing while working cancels, and a late answer is ignored;
  - the shortcut while open re-runs on the same selection without capturing;
  - the timeout after 30 seconds, using a controllable clock or paused tokio time;
  - each rewrite error maps to its message.
- **Instruction set library.** Direct tests: add, edit, set active, delete the active set, refuse to delete the last set, and Restore in each case (missing starter, edited starter, user set untouched, active unchanged).
- **Rewriter.** The existing prompt tests stay, renamed. New tests cover how each provider maps to an adapter and endpoint (including OpenRouter, Ollama and a Custom URL) and the optional-key rules (blank key refused for OpenAI, Anthropic, Gemini and OpenRouter; accepted for Ollama and Custom).
- **Settings.** A round-trip test saves and loads the settings file, and asserts that the file never contains a key.
- **Prior art:**
  - the prompt tests and the fake rewriter tests in the LLM module;
  - the recording keyring store test in the settings module;
  - the exit-request test in the app entry point.
  - Async tests use tokio's test macro, already a dev dependency.
- **Manual test checklist** (run before each release, because the Windows text bridge, tray and shortcut aren't unit-tested):
  - capture and Replace in Notepad, Word, Chrome (a text field and a read-only page), Slack or another Electron app, and Windows Terminal;
  - with an Urdu or Arabic keyboard layout;
  - with an image and with copied files on the clipboard before the rewrite;
  - Win+V history after a rewrite;
  - the shortcut taken by another app;
  - the popup near each edge of each monitor.
- **Correction quality** (do the starters rewrite well across models?) is evaluated separately with `promptfoo-evaluation`. Prompt-injection behaviour (selections that look like commands) is tested with `ai-system-testing`. Neither is part of the unit test suite.

## Out of Scope

- Rewriting automatically as you type.
- Editing the result inside the popup.
- Highlighting what changed.
- A saved history of past rewrites.
- A different provider or model for each instruction set.
- More than one Custom provider.
- Detecting read-only targets or failed pastes.
- Capturing through UI Automation.
- Streaming the result as it's written.
- A signed installer.
- Mac, Linux and mobile.

## Further Notes

- In-app update notices were out of scope for v1 (1.0.0 has no updater). They shipped later; see `.scratch/auto-update/spec.md`.
- Decisions come from the grilling session on `docs/PRD.md`. The PRD, `GLOSSARY.md` and ADR 0001 (with its amendment) were updated to match.
- The capture and Replace approach follows research on Microsoft PowerToys Advanced Paste, WritingTools, Espanso and AutoHotkey. None of them detects read-only targets. Most save only the text on the clipboard, so a full multi-format snapshot goes further than they do.
- The empty-key behaviour for Custom against LM Studio, and LM Studio's default port, haven't been verified against a running server. Check them during implementation.
- Popup and settings visual design should use the `impeccable` skill. Win32 code needs the `unsafe-checker` review and local `#[expect(unsafe_code)]` plus `// SAFETY:` comments, as the crate's lint setup requires.
