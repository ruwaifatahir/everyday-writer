# everyday-writer

Background Windows app: select text in any app, press a shortcut, and an LLM rewrites it: typos, grammar, tone. One click replaces the original.

[![CI](https://github.com/ruwaifatahir/everyday-writer/actions/workflows/ci.yml/badge.svg)](https://github.com/ruwaifatahir/everyday-writer/actions/workflows/ci.yml)

## What it does

Typos and messy grammar slip into everyday writing, and fixing them usually means copying text into a chatbot, retyping the same instructions and pasting the result back. everyday-writer runs quietly in the background on Windows. Select text in any app and press one shortcut (default `Ctrl+Shift+Space`). It rewrites the text with your own instructions, using the LLM provider you choose (OpenAI, Anthropic, Gemini, OpenRouter, a local Ollama, or any OpenAI-compatible endpoint) and your own API key. A small popup near the mouse shows the result, and one click swaps your original text for the new one. The API key lives in the Windows Credential Manager, your clipboard is restored afterwards, and nothing leaves your machine except the rewrite request and the update check (see [Updates](#updates)). Windows clipboard history (Win+V) may still show text you rewrite, because the app you copy from adds it there. Full requirements: [docs/PRD.md](docs/PRD.md).

Download the installer from [Releases](https://github.com/ruwaifatahir/everyday-writer/releases). It isn't code-signed yet, so Windows SmartScreen may warn on first run.

## Updates

The app checks GitHub for a new version about 10 seconds after it starts and then once a day. If there is one, you get a single Windows notification and the tray item reads "Update to 1.1.0". Nothing installs until you click **Update now** in Settings > Updates; the app then installs the update, restarts and keeps your settings, instruction sets and key. It only accepts updates signed with the project's key.

The check downloads a public file (`latest.json` from the latest GitHub release). It sends no ID, no app version and nothing about your settings or text; GitHub sees your IP address, as with any website visit. To stop automatic checks, turn off **Check for updates automatically** in Settings > Updates. **Check for updates** in the tray still works when you want it.

**On version 1.0.0?** It has no updater, so install the newest version from [Releases](https://github.com/ruwaifatahir/everyday-writer/releases) by hand, once, over your current copy. Your settings and key are kept, and later versions update themselves.

## Build

Prerequisites: Rust via rustup (stable MSVC), Microsoft C++ Build Tools ("Desktop development with C++"), WebView2, Node 24 and pnpm.

```sh
pnpm install
pnpm tauri dev     # run the app
pnpm tauri build   # build the NSIS installer
```

The installer build signs the installer for the updater, so it needs a signing key. See "Building the installer locally" in [CONTRIBUTING.md](CONTRIBUTING.md).

## Checks

```sh
pnpm check   # every check on the whole project
pnpm fix     # apply rustfmt and Biome fixes
```

`pnpm check` runs rustfmt, clippy (warnings as errors), `cargo test`, cargo-deny (licenses, advisories, bans, sources), Biome lint and format, and `tsc --noEmit`. Install cargo-deny 0.20.2 first (`cargo binstall cargo-deny@0.20.2`, or `cargo install cargo-deny --locked`).

Clippy lint levels live in the `[lints]` table of `src-tauri/Cargo.toml`. cargo-deny config is `src-tauri/deny.toml`; Biome config is `biome.json`.

`pnpm install` also installs lefthook git hooks. The pre-commit hook runs rustfmt and Biome in check-only mode on staged files, so a badly formatted or lint-failing file blocks the commit. Run `pnpm fix`, re-stage and commit again.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Report security problems privately as described in [SECURITY.md](SECURITY.md).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
