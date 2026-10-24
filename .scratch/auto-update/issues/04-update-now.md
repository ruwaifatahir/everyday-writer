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

## Comments

**Implementer (ticket/04-update-now):**

- Busy gate: `RewriteSession::hold_when_idle()` waits until no capture, rewrite or replace runs (a popup showing a result or message counts as idle and is closed), then holds the session so `ShortcutPressed` does nothing; `release()` undoes it. Woken by a `Notify` fired after every `handle`. Tests in `session/tests.rs` (`holding_*`, `a_released_session_rewrites_again`).
- Controller: new port `BusyGate { type Hold; hold_when_idle() }` (hold released on drop) as the 4th generic/`new` arg; `UpdateSource::install(&handle, &dyn Fn(u8)) -> Result<Infallible, InstallError>`; event `UpdateNowRequested`. Update now from `Available` or install-failed: shows `Downloading { percent: 0 }` at once (also while waiting for the gate, so the button is disabled), holds the gate, installs, renders each changed percent. Checks and a second Update now are ignored while downloading.
- Deviation: the install failure is `UpdatesState::InstallFailed { version }` (`{ "kind": "installFailed", "version": "1.1.0" }`), not `Failed { step: Install }`, because the tray must keep "Update to {version}" and `Failed` carries no version. `FailedStep` still has only `Check`.
- Wiring: `TauriUpdateSource::install` uses `download_and_install` (no timeout on the download), maps errors to `InstallError` kinds (signature errors -> `Signature`) before logging. `SessionGate`/`SessionHold` wrap `Arc<AppSession>`; `live_updates::start` must run after the session is managed (it already does). Command `update_now` (build.rs, handler list, `allow-update-now` in the settings capability).
- Tray: new `TrayItem::Disabled { label }` -> disabled "Downloading update…"; `InstallFailed` shows "Update to {version}" (`ShowUpdate`, opens Settings only).
- Page: `#updates-install` "Update now" (primary) shown for available / install failed / downloading, disabled while downloading; status "Downloading… 45%" and "Update failed. Try again.". It calls `update_now` directly: the unsaved-changes prompt is ticket 05.
- Open: both "Verified by hand" items (NSIS update keeping settings/sets/key/autostart; tampered signature refused) are not done.
