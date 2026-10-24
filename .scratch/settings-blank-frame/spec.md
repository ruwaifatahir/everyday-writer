Labels: ready-for-agent

# Settings opens without a blank frame

## Problem Statement

When Settings opens, the user first sees an empty cream-coloured window, and only then does the content appear. It happens on every open, not just at app start:

- When Settings opens as the app starts (first run, a damaged settings file, a shortcut that can't be registered, or a second launch), the window appears about 0.45 s after launch and stays empty until about 1.1 s.
- When Settings is opened while the app is already running (tray menu, popup button, update notification), the window appears at once and stays empty for about 0.65 s.

These numbers come from a release build on the developer's machine, checked with screenshots. In a dev build it feels like 1–2 s.

The app is meant to feel instant, and an empty window makes it look slow and unfinished. The page already fades in to hide its half-drawn state, but the window itself is on screen before the page has anything to show.

## Solution

Settings stays hidden until its page is fully drawn, then appears in one step with its content, at the section it was opened for, in front of other windows. If the page never gets ready, the window shows anyway after 2 seconds so Settings never seems broken.

The page also loads faster: the work it does before it can draw runs in parallel where it can, which shortens the wait before the window appears.

## User Stories

1. As a user, I want Settings to appear with its content already drawn, so that I never see an empty window.
2. As a user, I want Settings to look finished the moment it appears, so that the app feels fast and polished.
3. As a first-time user, I want the Settings window that opens on first run to appear fully drawn, so that my first impression of the app is a finished screen.
4. As a user whose saved shortcut can't be registered, I want Settings to open fully drawn at start-up, so that I can fix the shortcut without a broken-looking window.
5. As a user with a damaged settings file, I want Settings to open fully drawn at start-up, so that the recovery screen looks trustworthy.
6. As a user who launches the app a second time, I want the running app's Settings to appear fully drawn, so that relaunching feels like opening Settings.
7. As a user, I want Settings opened from the tray menu to appear fully drawn, so that every way in looks the same.
8. As a user, I want Settings opened from the popup's settings button to appear fully drawn, so that going from a rewrite to Settings feels smooth.
9. As a user who clicks an update notification, I want Settings to appear already scrolled to Updates, so that the window doesn't jump after it appears.
10. As a user who opens Settings at a specific section from the tray, I want it to appear already at that section, so that I don't watch it scroll.
11. As a user, I want Settings to appear in front of other windows with keyboard focus, so that I can start using it right away.
12. As a user, I want Settings to appear as quickly as possible after I ask for it, so that the extra wait for a drawn window stays short.
13. As a user, I want the text in Settings to appear in its real fonts from the first frame, so that nothing reflows or swaps after it appears.
14. As a user, I want Settings to show up even if something goes wrong while it loads, so that I'm never left with a Settings window that never opens.
15. As a user whose Settings failed to get ready, I want the window to appear within 2 seconds, so that I can still use or close it.
16. As a user who opens Settings when it's already open, I want the existing window brought to the front immediately, so that nothing changes from today.
17. As a user who closes Settings quickly after opening it, I want it to stay closed, so that a delayed timer never brings it back.
18. As a screen reader user, I want Settings to receive focus when it appears, so that my screen reader starts reading the window.
19. As a user, I want the popup to keep working exactly as it does today, so that this change doesn't affect rewrites.

## Implementation Decisions

- **Hidden until ready.** The Settings window is built hidden. This happens in the one function every open path goes through, so the tray menu, second launch, first run, a failed shortcut at start-up, the popup's settings button and the update notification are all covered by one change.
- **Already-open window unchanged.** If Settings is already open, it is brought to the front at once, as today.
- **Ready signal owned by Rust.** The page tells the backend it's ready through a new command (working name `settings_ready`). Rust then shows and focuses the window. The page is not given permission to show or focus windows itself.
- **Safety timeout.** Rust shows the window after 2 seconds if no ready signal has arrived.
- **Reveal rule.** The window is revealed at most once: on the ready signal or the timeout, whichever comes first. The later of the two does nothing. If the window has been closed, the timeout does nothing. This rule is kept small and separate from window handling so it can be unit-tested.
- **What "ready" means.** The page signals ready once all of these are true:
  - the main form (provider, model, API key, shortcut) is drawn;
  - the Instruction Sets and Updates sections are drawn;
  - the fonts have finished loading;
  - the page has scrolled to the requested section, if there is one.

  If drawing Instruction Sets and Updates turns out to add a noticeable delay, "ready" falls back to "main form drawn and scrolled to the requested section".
- **Parallel loading.** Fetching the Suggested Models and fetching the saved settings run at the same time. The saved-key check and the shortcut check depend on the saved settings, so they start once those arrive, and run in parallel with each other.
- **No fade-in for Settings.** The opacity fade-in that hid the half-drawn page is removed for Settings, since the window now appears already drawn. The popup keeps its fade-in.
- **Fonts unchanged.** Fonts keep `font-display: block`. The page waits for its fonts to load before signalling ready, so the window never shows a fallback font. The fonts are small local files, about 44 KB in total.
- **Focus.** Revealing the window both shows and focuses it. If Windows refuses to bring it to the front for one of the open paths, that path gets its own fix.
- **Popup untouched.** The popup is already built hidden at start-up and only shown after its Result is drawn, so it has no blank frame.

## Testing Decisions

- A good test checks what the user sees or the rule's outcome, not how it's wired. "No blank frame appears" and "the window is revealed once" are good; "this function was called" is not.
- **Main proof: the real app.** Run a screenshot test against a release build:
  - cold: Settings opens as the app starts;
  - warm: Settings is opened while the app has been running for a while.

  The test captures the Settings window every few tens of milliseconds and checks the frames. Pass means no frame shows the window empty. Record before and after numbers for time from request to drawn window. The same method was used to measure the problem: before the fix, the window was empty for about 0.65 s in both cases.
- **Manual check of every open path.** Tray menu, second launch, first run, failed shortcut at start-up, popup settings button and update notification. Each one should appear drawn, in front and focused, and the update notification should land on the Updates section.
- **Unit test: the reveal rule.** Cases:
  - the ready signal reveals the window;
  - the timeout reveals it if no ready signal came;
  - whichever arrives second does nothing;
  - the timeout does nothing after the window was closed.

  Prior art is the existing unit tests in the Settings window module that cover which page and section a window opens at.
- **No TypeScript tests.** The repo has none, and the page's start-up depends on the live backend.

## Out of Scope

- Lowering WebView2's memory use, whether through its low-memory mode or by making the popup native. That is a separate decision.
- Moving off Tauri or WebView2.
- Destroying the Settings window more cleanly on close. Memory stayed about 19 MB higher after closing it; that is worth a separate look.
- Changing the popup.
- Speeding up the backend calls themselves, such as reading the saved key from Windows Credential Manager.
- Dev-mode start-up time (`pnpm tauri dev`).

## Further Notes

- **Baseline measurements**, from a release build of v1.2.0 on the developer's machine:

  | Case | Window appears | Content drawn |
  |---|---|---|
  | Cold (Settings opens as the app starts) | about 0.45 s | about 1.1 s |
  | Warm (app already running) | about 0.07 s | about 0.73 s |
- **Measuring blank frames.** Counting colours in a window capture is not enough. A just-created window that hasn't painted yet shows whatever is behind it, which looks like content. Check the captured frames themselves, or confirm the window is drawn first and then look for content.
- **Microsoft's WebView2 performance guidance** recommends against WebView2 for simple dialogs. This spec stays on WebView2; that trade-off belongs to the separate memory decision.
