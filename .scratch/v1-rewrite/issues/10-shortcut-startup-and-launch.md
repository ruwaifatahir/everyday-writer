# 10: Shortcut, startup and launch behaviour

**What to build:**

- **Changing the shortcut:** the user can change it in Settings. If Windows refuses the new one because another app already holds it, Settings says "This shortcut is used by another app" and keeps the previous one.
- **Startup failure:** if the saved shortcut can't be registered at startup, the app shows a tray notification and opens Settings.
- **Start with Windows:** a switch in Settings, on by default.
- **Second launch:** launching the app again opens Settings in the copy that's already running, instead of doing nothing.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 02 (Minimal settings), 03 (Tracer bullet: shortcut, popup, Copy)

**Status:** ready-for-agent

- [ ] A new shortcut works immediately and survives a restart
- [ ] A shortcut held by another app is refused with the message, and the old shortcut keeps working
- [ ] A registration failure at startup shows a tray notification and opens Settings
- [ ] Start with Windows is on after install, and turning it off removes the startup entry
- [ ] A second launch opens Settings in the running app, and no second copy runs
- [ ] Settings holds only provider, model, key, shortcut, instruction sets and the start-with-Windows switch
