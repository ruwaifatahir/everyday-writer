# 02: Minimal settings: provider, model, key

**What to build:** A settings window where the user picks a provider (OpenAI, Anthropic, Gemini, or Custom with a base URL), types a model name and enters a key. The settings are saved to a plain settings file in the app's data folder. The key goes to the Windows Credential Manager, one entry per provider, never into the file. The tray menu gets a **Settings** item. On first launch, when no settings file exists yet, Settings opens on its own. Closing Settings leaves the app running in the tray.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** None (can start immediately)

**Status:** resolved

- [ ] Settings opens from the tray, and on its own only on first launch
- [ ] Provider, model and Custom base URL survive an app restart
- [ ] The key is saved per provider in the Credential Manager; switching provider and back keeps each key
- [ ] The key field never shows a saved key in plain text; it shows that a key is saved and lets the user replace or clear it
- [x] The settings file never contains a key (a round-trip test checks this)
- [x] The rest of the app reads settings through a settings view: current provider, model, and whether the provider is ready
- [ ] The settings window looks modern, minimal and clean (`impeccable` skill)
- [x] Nothing private is logged

## Comments

Merged as 97fbf2c (`merge: ticket 02 minimal settings`). `pnpm check` passes (44 Rust tests, biome, tsc, cargo-deny).

Still needs manual checking in the running app:
- Settings opens from the tray, and on its own only on first launch
- Provider, model and Custom base URL survive an app restart
- Key saved per provider in the real Credential Manager; switching provider and back keeps each key
- Key field never shows a saved key in plain text; replace and clear work
- Visual look (modern, minimal, clean)
