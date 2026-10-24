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
