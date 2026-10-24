# 06: Automatic checks

**What to build:** The app checks for an update by itself about 10 seconds after it starts, then every 24 hours while it runs. When it finds a version it hasn't notified about yet, it shows one quiet Windows notification ("everyday-writer 1.1.0 is available") and the tray item reads "Update to 1.1.0". Failed automatic checks stay silent. Settings gets a "Check for updates automatically" switch (on by default) that applies immediately. Turning it off stops the schedule but keeps a known update in the tray, and manual checks keep working. See the spec, user stories 1–7, 9–13.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] The controller owns the schedule (10 s after start, then every 24 h) and cancels it cleanly when auto-check is turned off; it restarts when the switch goes back on
- [ ] Notify only when the version is newer than `last_notified_version`, then record it; again for a newer version; never for manual checks
- [ ] Failed automatic checks leave the visible state unchanged and show nothing
- [ ] The new settings-file fields (`auto_check` default on, `last_notified_version`) load from older files via serde defaults, and saving the Settings form doesn't overwrite them; settings store tests cover both
- [ ] Controller tests run under Tokio's paused clock (no real sleeps) and cover: first check at 10 s, the 24 h repeat, no checks when off, notify once per version, notify again for newer, silent failures, keeping `Available` when switched off
- [ ] `pnpm check` passes

## Comments

**Implementer (ticket/06-automatic-checks):**

- Seams tested (chosen without a user round-trip, as an agent): the update controller through `UpdateEvent`s with fake ports under `start_paused`; `SettingsFile` load/save; `AppSettings` (form save, `remember`).
- Controller (`src-tauri/src/updates.rs`): new events `Started` (runs the schedule and never returns; `live_updates::start` sends it on its own task) and `AutoCheckChanged(bool)`. The schedule watches a `tokio::sync::watch<bool>` seeded from memory; off cancels the wait, on starts over (first check 10 s later again). No public `Tick`: the timer is internal. Automatic checks never render `Checking`, render only on a change, and are skipped/ignored while a manual check runs (`Phase::takes_automatic_result`, an exhaustive match, so 04 must decide for `Downloading` etc.). `semver` is a direct dep now (already in the lock).
- Memory port: `UpdateMemory { recall() -> Remembered, remember_notified(version) }`. For 08: add `last_run_version` to `Remembered` and `Settings`, a `remember_run(version)` method, and map it in `live_updates::SettingsMemory`.
- Settings: `autoCheck` (default on), `lastNotifiedVersion`. Form save moved to `AppSettings::save_form` (tested to keep the memory). Damaged file: new `AppSettings::remember` keeps app-made changes in memory only until the user saves over the damaged file; 08 should use it for `last_run_version`. `set_auto_check` is a user save and uses `modify` (like start-with-Windows).
- Page: switch in the Updates section, applied at once via `set_auto_check` (build.rs, handler list, capability). It reads `autoCheck` with its own `getSettings()` call so `main.ts` stays untouched.
- No notification click handler (07). Open: manual checklist rows for automatic checks not added; not verified by hand on an installed build.
