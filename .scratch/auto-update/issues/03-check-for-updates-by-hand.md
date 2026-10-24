# 03: Check for updates by hand

**What to build:** The tray menu gets **Check for updates** (between "Instruction set" and "Settings"). Clicking it opens Settings at Updates, which shows "Checking…", then "You're up to date" or "Version 1.1.0 is available". The Updates section also has a **Check for updates** button. A failed manual check shows "Couldn't check for updates". A manual check never shows a Windows notification. See the spec, user stories 14–20.

This ticket introduces the **update controller** from the spec (events in, ports out, `UpdatesState` rendered to Settings and the tray label), with the update source, updates view and memory ports. Later tickets extend it.

**Blocked by:** 01, 02

**Status:** resolved

- [ ] The update controller owns the check rules and is driven only through events and ports; the real update source wraps the Tauri updater with a timeout
- [ ] `UpdatesState` (`Idle`, `Checking`, `UpToDate`, `Available { version }`, `Failed { kind }`; later `Downloading`) is serialized for the Settings page like `PopupState`, and the page listens for its changes
- [ ] Tray: "Check for updates" normally, "Update to {version}" when an update is known; both only open Settings at Updates. The tray menu model tests cover the new item, its position and `action()`
- [ ] Controller tests with fake ports cover: up to date, available, failed check, and a manual check never notifying
- [ ] Logs hold only a non-identifying error kind, never URLs with user data or response bodies
- [ ] The check sends no identifiers (no version or machine data in the URL or headers beyond defaults)
- [ ] Verified by hand against a test endpoint serving a newer version and against no network
- [ ] `pnpm check` passes

## Comments

**Implementer (ticket/03-check-for-updates-by-hand):**

- Controller: `src-tauri/src/updates.rs` (`UpdateController<U, N, V>`, `UpdateEvent`, `UpdatesState`, ports `UpdateSource` / `Notifier` / `UpdatesView`), tests in `updates/tests.rs`. Internal `Phase<H>` holds the `AvailableUpdate { version, handle }`, so 04 can install from `Phase::Available` without a second check. To extend: add an `UpdateEvent` variant + match arm in `handle`, a `Phase` variant + `shown()` arm, and new generic params (busy gate, memory) on `UpdateController` / `new` / `AppUpdates`.
- `UpdatesState` is tagged by `kind`, so the failed variant is `Failed { step: FailedStep }` (`{ "kind": "failed", "step": "check" }`), not `Failed { kind }`. Only `FailedStep::Check` exists; 04 adds `Install` (and `Downloading`).
- `Notifier` has only `update_available`; the field is `expect(dead_code)` outside tests until 06 calls it. 08 adds `updated_to`.
- No memory port or settings fields yet: nothing in 03 reads or writes them, and 06/08 own those fields.
- Wiring: `src-tauri/src/live_updates.rs` holds the real ports (`TauriUpdateSource`: `app.updater()` + `check()` under a 30 s `tokio::time::timeout`, errors mapped to a fixed `CheckError` kind before logging), `send(app, event)`, and the commands `get_updates_state` / `check_for_updates` (build.rs, handler list, settings capability). `TauriUpdatesView` keeps the state, rebuilds the tray (`tray::show_updates`) and emits `updates-state` to the settings window.
- Tray: two ids, so the click needs no state: `check-for-updates` -> `TrayAction::CheckForUpdates` (open at Updates + `CheckRequested`), `show-update` ("Update to X") -> `TrayAction::ShowUpdate` (open at Updates only). `tray_menu(sets, updates)` now takes the updates state.
- Page: `#updates-check` button and `#updates-status` line in the Updates section; `updates.ts` renders the state (button disabled while checking). An event that arrives while the getter runs wins over the getter's answer.
- Open: "Verified by hand against a test endpoint serving a newer version and against no network" is not done. A debug-only endpoint override is not added; to test, serve a `latest.json` via `plugins.updater.endpoints` in a local config override. Manual checklist rows for Updates are not added yet.
