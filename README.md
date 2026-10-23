# everyday-writer

Background Windows app: select text in any app, press a shortcut, and an LLM fixes typos and grammar. One click replaces the original.

## Build

Prerequisites: Rust via rustup (stable MSVC), Microsoft C++ Build Tools ("Desktop development with C++"), WebView2, Node 24 and pnpm.

```sh
pnpm install
pnpm tauri dev     # run the app
pnpm tauri build   # build the NSIS installer
```

## Checks

```sh
pnpm check   # every check on the whole project
pnpm fix     # apply rustfmt and Biome fixes
```

`pnpm check` runs rustfmt, clippy (warnings as errors), `cargo test`, cargo-deny (licenses, advisories, bans, sources), Biome lint and format, and `tsc --noEmit`. Install cargo-deny 0.20.2 first (`cargo binstall cargo-deny@0.20.2`, or `cargo install cargo-deny --locked`).

Clippy lint levels live in the `[lints]` table of `src-tauri/Cargo.toml`. cargo-deny config is `src-tauri/deny.toml`; Biome config is `biome.json`.

`pnpm install` also installs lefthook git hooks. The pre-commit hook runs rustfmt and Biome in check-only mode on staged files, so a badly formatted or lint-failing file blocks the commit. Run `pnpm fix`, re-stage and commit again.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
