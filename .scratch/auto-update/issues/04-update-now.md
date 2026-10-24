# 04: Update now

**What to build:** When an update is available, Settings shows **Update now**. Clicking it waits until any running rewrite or replace has finished, closes the popup, shows "Downloading… 45%", then a small installer progress window with no questions and no admin prompt. The app then restarts on the new version. If the download or signature check fails, Settings shows "Update failed. Try again.", the old version keeps working and "Update to {version}" stays in the tray. See the spec, user stories 22–28, 32, 35–40.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] The rewrite session can report that no rewrite or replace is running and can be waited on (the busy gate). While the update holds it, a shortcut press doesn't start a new rewrite. Existing session tests still pass, and new ones cover the gate
- [ ] Update now: wait for idle, close the popup, install with progress (`Downloading { percent }`); the Update now button and the tray item are disabled while downloading ("Downloading update…" in the tray)
- [ ] An install failure returns to `Available` with `Failed { Install }` in Settings; nothing else changes
- [ ] The tray item and notifications never install; only Update now does
- [ ] Controller tests with fakes cover: install waits for a held busy gate, progress states, the install failure path, and no second install while downloading
- [ ] Verified by hand on the installed NSIS build: an older build updates to a test release, restarts, and keeps settings, instruction sets, API key and start-with-Windows
- [ ] Verified by hand: a tampered signature is refused and the old version keeps running
- [ ] `pnpm check` passes
