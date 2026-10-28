Labels: needs-triage

# Settings reopens instantly

## Problem Statement

Settings opens more slowly than in other desktop apps when opened from the tray. Each open builds a new WebView2 window from scratch (`settings_window.rs`, `WebviewWindowBuilder`): new window, page load (HTML, CSS, JS, fonts), settings fetched from Rust, then shown once drawn. Closing destroys the window, so every open pays the full cost again.

The popup doesn't have this problem: it is created hidden at startup and only ever hidden, never destroyed (`popup.rs`, `create`).

Switching to a native stack (C++/Qt, as in Beeftext or Telegram Desktop) was considered and rejected: it means a full rewrite, a likely bigger binary, LGPL issues, and no gain for this problem. The slowness comes from the window's lifecycle, not from Tauri.

## Measurements

Taken 2026-10-28 on the installed v1.5.0 release build, opening Settings through a second launch (which routes to Settings), timed until the Settings window is visible.

Open time, 12 runs:

- Usually 335–410 ms. The second launch itself takes 70–150 ms of that, so a tray click is about 260–330 ms.
- Outliers: 1.1 s, 1.7 s, 3.5 s, and one at 10 s. These are the moments that feel slow.

Memory of the whole process tree (the app plus its WebView2 processes):

| State | Private | Working set |
|---|---|---|
| Idle, popup hidden | 162 MB | 181 MB |
| Settings open | 215 MB | 272 MB |
| After closing Settings | 172 MB | 192 MB |

- Settings adds one renderer process: about 30 MB private and 67 MB working set while visible, plus about 20 MB briefly in the GPU process.
- The hidden popup's renderer shows what a hidden Settings would cost: 30 MB private, but only about 8 MB working set, because Windows trims hidden windows.
- Estimated cost of keeping Settings alive while hidden: about 8 MB RAM, about 30 MB reserved, no CPU while idle.

Microsoft's WebView2 performance guidance supports this: reuse one environment, set `MemoryUsageTargetLevel = Low` on inactive WebViews, or call `TrySuspendAsync()` to pause the renderer when it won't be used for a while.
(https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/performance)

## Proposed Solution

Close hides Settings instead of destroying it. Don't pre-create it at startup. The first open costs the same as today; every later open just shows the existing window.

Optional: pre-create it hidden at startup so even the first open is instant. This costs the memory above from the moment the app starts.

## Things to handle

- **Fresh data on every show.** The page stays loaded, so it must reload saved settings (and the API key status) each time it's shown, and drop unsaved edits from the last session. Decide how this fits with the save bar's Discard flow.
- **Destroy-based cleanup.** Code that runs on `WindowEvent::Destroyed` has to run on hide too: the shortcut pause (`SHORTCUT_PAUSE.close()`) and the reveal state (`reveal_state().close(opening)`).
- **Reveal logic.** The hidden-until-ready flow and the 2-second `READY_TIMEOUT` only apply to the first build. A reopen should just show and focus the window.
- **Opening at a section.** Reopening at a section (for example Updates from the notification) must scroll before the window shows, so it doesn't jump.
- **Exit handling.** Check that `lib.rs`'s exit-on-last-window rule still behaves correctly when Settings is hidden rather than closed.
- **Optional memory trim.** While hidden, consider lowering the WebView2 memory target or suspending the renderer, if Tauri/wry exposes it.

## Verification

- Repeat the measurement above (open time, process-tree memory) on a release build, before and after.
- Expect a reopen to be near-instant, with no 1–10 s outliers.
- Expect idle memory with Settings hidden to be roughly 8 MB above the idle baseline.
