# 01: Show the app version in Settings

**What to build:** Settings gains an **Updates** section that shows "everyday-writer 1.0.0" (the running version). It's the home for everything later tickets add (switch, Check button, status, Update now). See the spec, user story 21.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Settings shows an Updates section with "everyday-writer {version}", where the version comes from the running app, not a hard-coded string
- [ ] The section matches the existing Settings styles and layout (plain HTML/TS, no new frontend deps)
- [ ] The settings window can be opened scrolled to the Updates section; if it's already open it focuses and scrolls there (later tickets reuse this)
- [ ] Any new command has an allow permission in the settings capability and is registered in the handler list
- [ ] `pnpm check` passes

## Comments

**Implementer (ticket/01-show-app-version):**

- Settings has a new **Updates** section (`#updates`, after Instruction sets, outside the provider form so it never affects Save). It shows "everyday-writer {version}", with the version from the new `app_version` command (`app.package_info().version`). The command is listed in `build.rs`, registered in `lib.rs`, and allowed in `capabilities/settings.json`.
- Entry point for later tickets: `settings_window::open_at(app, Some(Section::Updates))`, or `open_at_later(...)` from tray/menu/notification callbacks (building windows inside the event loop deadlocks). A new window loads `settings.html#updates` and scrolls there once all sections have loaded. An open window is unminimized, focused and sent `settings-scroll-to` with the section id. `open` and `open_later` are unchanged for callers.
- Tests: Rust unit tests cover the page URL per section. The scroll itself is not automated. Worth adding to the manual checklist: open Settings at Updates from the tray item (ticket 03), both when the window is closed and when it's already open.
- Ticket 03 can fold `app_version` into its "get the update state and version" command, or keep it.
