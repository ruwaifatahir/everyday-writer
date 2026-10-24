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
