# 03: Check for updates by hand

**What to build:** The tray menu gets **Check for updates** (between "Instruction set" and "Settings"). Clicking it opens Settings at Updates, which shows "Checking…", then "You're up to date" or "Version 1.1.0 is available". The Updates section also has a **Check for updates** button. A failed manual check shows "Couldn't check for updates". A manual check never shows a Windows notification. See the spec, user stories 14–20.

This ticket introduces the **update controller** from the spec (events in, ports out, `UpdatesState` rendered to Settings and the tray label), with the update source, updates view and memory ports. Later tickets extend it.

**Blocked by:** 01, 02

**Status:** ready-for-agent

- [ ] The update controller owns the check rules and is driven only through events and ports; the real update source wraps the Tauri updater with a timeout
- [ ] `UpdatesState` (`Idle`, `Checking`, `UpToDate`, `Available { version }`, `Failed { kind }`; later `Downloading`) is serialized for the Settings page like `PopupState`, and the page listens for its changes
- [ ] Tray: "Check for updates" normally, "Update to {version}" when an update is known; both only open Settings at Updates. The tray menu model tests cover the new item, its position and `action()`
- [ ] Controller tests with fake ports cover: up to date, available, failed check, and a manual check never notifying
- [ ] Logs hold only a non-identifying error kind, never URLs with user data or response bodies
- [ ] The check sends no identifiers (no version or machine data in the URL or headers beyond defaults)
- [ ] Verified by hand against a test endpoint serving a newer version and against no network
- [ ] `pnpm check` passes
