# 03: Background app skeleton

**What to build:** The app starts as a quiet background app with an icon in the Windows notification area. Launching it a second time doesn't start a second copy. All Tauri plugins the PRD will need are added and registered, but have no behavior yet: `global-shortcut`, `clipboard-manager`, `single-instance` (registered first) and `autostart`, plus the `tray-icon` feature. Each window's capability grants only the permissions it needs. Add one empty, documented Rust module per job: `shortcut`, `capture`, `llm`, `replace`, `tray`, `settings`. `tracing` logging is set up under the rule that selected text, clipboard contents, prompts, LLM responses and API keys are never logged.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 01: Runnable empty app

**Status:** ready-for-agent

- [ ] Plugins added through `tauri add` and registered; single instance registered first
- [ ] Capabilities are least-privilege, with no wildcard or unused permissions
- [ ] Tray icon shows when the app runs; no main window shows on startup
- [ ] Launching the app a second time doesn't start a second process
- [ ] Modules `shortcut`, `capture`, `llm`, `replace`, `tray`, `settings` exist, each with a doc comment stating its job and its privacy rule where one applies
- [ ] `tracing` set up; the no-text-logging rule is written in the logging setup's doc comment
- [ ] tokio (if added) uses only the features needed, not `full`
- [ ] Release binary size recorded in the PR description, for comparison later
