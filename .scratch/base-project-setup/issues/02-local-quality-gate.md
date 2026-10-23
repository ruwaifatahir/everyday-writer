# 02: Local quality gate

**What to build:** A developer runs one command and every quality check runs on the whole project. Badly formatted or lint-failing code is blocked before it is committed. Checks: rustfmt, clippy with lints set in the Cargo manifest `[lints]` table (warnings as errors), cargo-deny (licenses, advisories, bans, sources), Biome lint and format for TypeScript, and a strict TypeScript type check. Lefthook pre-commit hooks run the fast checks (format and lint).

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 01: Runnable empty app

**Status:** resolved

- [x] rustfmt config present; `cargo fmt --check` passes
- [x] Clippy lints declared in the Cargo manifest; `cargo clippy -- -D warnings` passes
- [x] cargo-deny config allows only licenses compatible with `MIT OR Apache-2.0`; `cargo deny check` passes
- [x] Biome config at repo root; Biome check passes
- [x] TypeScript `strict` on; `tsc --noEmit` passes
- [x] One documented pnpm script runs all checks above
- [x] Lefthook installed through pnpm; the pre-commit hook blocks a deliberately misformatted Rust or TS file
- [x] Hooks only check staged files, so commits stay fast

## Comments

Done on branch `feat/02-local-quality-gate`. `pnpm check` (documented in README) runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo deny check`, `biome ci .` and `tsc --noEmit`. `pnpm fix` applies rustfmt and Biome fixes. Tools: @biomejs/biome 2.6.2, lefthook 2.2.2, cargo-deny 0.20.2.

Verified:
- `pnpm check` passes. cargo-deny prints only `duplicate` warnings from Tauri's tree (`multiple-versions = "warn"`).
- Lints are live: the scaffold's `tauri_build::build()` (no `;`) fails clippy pedantic.
- Pre-commit blocked a staged, misformatted `src/bad.ts` (Biome) and `src-tauri/src/bad.rs` (rustfmt). A well-formatted staged `.rs` passed. Each job only received the staged file. Test files removed.

Lint levels for new Rust code (all targets; `-D warnings` makes every `warn` an error):
- deny: `unsafe_code` (override locally with `#[allow(unsafe_code)]` plus a `// SAFETY:` comment)
- warn: `missing_docs`, `clippy::all`, `clippy::pedantic`, `dbg_macro`, `print_stdout`, `print_stderr`, `unwrap_used`, `todo`, `undocumented_unsafe_blocks`
- allow: `module_name_repetitions`, `must_use_candidate`
- `clippy.toml`: `allow-unwrap-in-tests` and `allow-expect-in-tests`

Deviations:
- `pnpm check` also runs `cargo test`, to match CI.
- `[lints]` sits at the end of `Cargo.toml`, after `[profile.release]`, so it doesn't conflict with dependency additions.
- Fixed the scaffold's `build.rs` and `main.rs` for pedantic and `missing_docs` (crate doc comments, trailing `;`).
- Biome excludes `.agents/`, `.claude/` (vendored skills) and `*.md`/`*.yaml`/`*.yml`. Biome 2.6 formats Markdown and YAML, and would rewrite docs and `pnpm-lock.yaml`. `biome check --write` reordered the imports in `vite.config.ts`.
- `pnpm-workspace.yaml` has `allowBuilds: { lefthook: true }` (pnpm 11), so `pnpm install` runs lefthook's postinstall, which installs the hooks.
- The rustfmt hook job prepends `$HOME/.cargo/bin` to PATH. Git's `sh` often lacks it, and the job then failed with "command not found". Lefthook mangles `${VAR:-default}`, so the job uses plain `$HOME`.
- Clippy is not in a hook: it is crate-wide and slow. `pnpm check` and CI run it.
- Git worktrees share `.git/hooks`. The installed hook hardcodes the lefthook path of the checkout that last ran `pnpm install`, and falls back to `node_modules` in the current checkout.
