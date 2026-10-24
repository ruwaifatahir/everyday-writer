# 01: Show the app version in Settings

**What to build:** Settings gains an **Updates** section that shows "everyday-writer 1.0.0" (the running version). It's the home for everything later tickets add (switch, Check button, status, Update now). See the spec, user story 21.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Settings shows an Updates section with "everyday-writer {version}", where the version comes from the running app, not a hard-coded string
- [ ] The section matches the existing Settings styles and layout (plain HTML/TS, no new frontend deps)
- [ ] The settings window can be opened scrolled to the Updates section; if it's already open it focuses and scrolls there (later tickets reuse this)
- [ ] Any new command has an allow permission in the settings capability and is registered in the handler list
- [ ] `pnpm check` passes
