# 07: Clicking the update notification opens Settings

**What to build:** Clicking the "1.1.0 is available" notification while it's on screen opens Settings at the Updates section. Clicks from the Windows notification panel (Action Center) are not caught; that limit is accepted because the tray item is the reliable path. See the spec, user story 8.

**Blocked by:** 06

**Status:** ready-for-agent

- [ ] The notification plugin's click handler is registered at startup, before any toast can be shown
- [ ] A click on the update-available toast opens Settings at Updates and never installs anything
- [ ] Clicks on other toasts (e.g. shortcut failed) keep their current behaviour
- [ ] The handler only dispatches to the app's main thread; no blocking work on the plugin's thread
- [ ] Verified by hand on the installed NSIS build (clicks aren't reported from `target/` builds); the result, including what happens on an Action Center click, is noted in this ticket's comments
- [ ] `pnpm check` passes

## Comments

**Implementer (ticket/07-notification-click):**

- Seam tested (chosen by the orchestrator, no user round-trip): `live_updates::click_opens_updates(&ActionPerformed) -> bool`, fed the plugin's real `ActionPerformed` built from JSON. Covers: tap on the update toast opens Updates; tap on the shortcut-failed toast, a tap whose toast can't be read, and a non-`tap` action on the update toast do nothing.
- The update-available toast now carries `extra("kind", "update-available")`; the decision matches on that, not on title/body text.
- `live_updates::start` registers the handler first (`open_updates_on_click`), before the tray, the shortcut-failed toast and the schedule. The handler runs on the plugin's per-toast thread and only calls `settings_window::open_at_later(.., Some(Section::Updates))`, which spawns on the async runtime (Tauri builds the window via the main thread). Nothing installs.
- Registering a handler makes every toast tracked (one short-lived "notification" thread per toast while it's on screen); the shortcut-failed toast's clicks reach the handler and are ignored, so its behaviour is unchanged.
- Open: not verified by hand on the installed NSIS build (clicks aren't reported from `target/` builds), including what an Action Center click does. Expected per exploration notes: no report; Windows may relaunch the exe, which single-instance turns into Settings opened without scrolling to Updates.
