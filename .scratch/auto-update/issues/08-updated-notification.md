# 08: "Updated to 1.1.0" after restart

**What to build:** After an update restarts the app, one Windows notification says "Updated to 1.1.0". A fresh install, or a normal restart without an update, shows nothing. See the spec, user stories 29–30.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] On start, if `last_run_version` exists and is older than the running version, send `updated_to(running)`; then always record the running version
- [ ] A missing `last_run_version` (fresh install or a file from before this feature) shows nothing and just records the version
- [ ] The new settings-file field loads from older files via serde defaults and isn't overwritten by saving the Settings form
- [ ] Controller tests cover: real increase notifies once, same version is silent, first run is silent
- [ ] `pnpm check` passes

## Comments

**Implementer (ticket/08-updated-notification):**

- Seams tested (chosen as an agent, without a user round-trip): the update controller through `UpdateEvent::Started` with fake ports; `SettingsFile` load/save and `AppSettings::save_form` (existing tests extended to `lastRunVersion`).
- `UpdateController::new` takes the running version as a new first argument (`app.package_info().version` in `live_updates::start`). Handling `Started` first runs `greet_update` (notify `updated_to(running)` only if `last_run_version` exists and is older, compared with the same `is_newer` as announcements), records the run, then runs the schedule.
- New ports: `Notifier::updated_to` (toast body "Updated to 1.1.0"), `UpdateMemory::remember_run`; `Remembered`/`Settings` gain `last_run_version` (`lastRunVersion`, serde default). Saved via `AppSettings::remember`, so a damaged file isn't overwritten.
- Tests: newer version notifies once, same version silent, first run silent, downgrade silent.
- Open: not verified by hand on an installed build.
