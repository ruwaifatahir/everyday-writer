# 08: "Updated to 1.1.0" after restart

**What to build:** After an update restarts the app, one Windows notification says "Updated to 1.1.0". A fresh install, or a normal restart without an update, shows nothing. See the spec, user stories 29–30.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] On start, if `last_run_version` exists and is older than the running version, send `updated_to(running)`; then always record the running version
- [ ] A missing `last_run_version` (fresh install or a file from before this feature) shows nothing and just records the version
- [ ] The new settings-file field loads from older files via serde defaults and isn't overwritten by saving the Settings form
- [ ] Controller tests cover: real increase notifies once, same version is silent, first run is silent
- [ ] `pnpm check` passes
