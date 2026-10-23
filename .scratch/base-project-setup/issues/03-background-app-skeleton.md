# 03: Background app skeleton

**What to build:** The app starts as a quiet background app with an icon in the Windows notification area. Launching it a second time doesn't start a second copy. All Tauri plugins the PRD will need are added and registered, but have no behavior yet: `global-shortcut`, `clipboard-manager`, `single-instance` (registered first) and `autostart`, plus the `tray-icon` feature. Each window's capability grants only the permissions it needs. Add one empty, documented Rust module per job: `shortcut`, `capture`, `llm`, `replace`, `tray`, `settings`. `tracing` logging is set up under the rule that selected text, clipboard contents, prompts, LLM responses and API keys are never logged.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 01: Runnable empty app

**Status:** resolved

- [x] Plugins added through `tauri add` and registered; single instance registered first
- [x] Capabilities are least-privilege, with no wildcard or unused permissions
- [x] Tray icon shows when the app runs; no main window shows on startup
- [x] Launching the app a second time doesn't start a second process
- [x] Modules `shortcut`, `capture`, `llm`, `replace`, `tray`, `settings` exist, each with a doc comment stating its job and its privacy rule where one applies
- [x] `tracing` set up; the no-text-logging rule is written in the logging setup's doc comment
- [x] tokio (if added) uses only the features needed, not `full`
- [x] Release binary size recorded in the PR description, for comparison later

## Comments

**Release sizes**, for later comparison. Built with `pnpm tauri build` on 2026-10-23 using Rust 1.99.1 and the release profile (`opt-level = "s"`, LTO, `removeUnusedCommands`):

- `everyday-writer.exe`: 2,203,136 bytes (2.10 MiB)
- `everyday-writer_0.1.0_x64-setup.exe` (NSIS): 942,460 bytes (920.4 KiB)

**Checked on the release exe:**

- The tray icon is registered, with the tooltip "everyday-writer". Windows 11 puts new icons in the "Show hidden icons" overflow.
- No app window opens on startup. The process owns only hidden helper windows: single-instance's message window and tao's event target.
- A second launch exits with code 0, and exactly one process keeps running.

**Notes:**

- `tauri add` put each plugin first in the builder chain, so I added single-instance last to make it end up first. It also wrote `capabilities/desktop.json`, which granted `global-shortcut:default` and `autostart:default` to a `main` window that doesn't exist; I deleted it. It installed three npm guest packages too. I removed them because all plugin calls run in Rust, which needs no capability.
- `capabilities/popup.json` grants nothing yet. The popup ticket will add only what the popup's JS calls.
- tokio isn't added because nothing is async yet. Ticket 04 adds it with only the features `genai` needs.
- The scaffold's `build.rs` and `main.rs` fail `missing_docs` and pedantic clippy, because they lack crate docs and trailing semicolons. I left them for ticket 02, which owns the lint setup.
