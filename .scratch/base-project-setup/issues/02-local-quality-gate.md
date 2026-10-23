# 02: Local quality gate

**What to build:** A developer runs one command and every quality check runs on the whole project. Badly formatted or lint-failing code is blocked before it is committed. Checks: rustfmt, clippy with lints set in the Cargo manifest `[lints]` table (warnings as errors), cargo-deny (licenses, advisories, bans, sources), Biome lint and format for TypeScript, and a strict TypeScript type check. Lefthook pre-commit hooks run the fast checks (format and lint).

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 01: Runnable empty app

**Status:** ready-for-agent

- [ ] rustfmt config present; `cargo fmt --check` passes
- [ ] Clippy lints declared in the Cargo manifest; `cargo clippy -- -D warnings` passes
- [ ] cargo-deny config allows only licenses compatible with `MIT OR Apache-2.0`; `cargo deny check` passes
- [ ] Biome config at repo root; Biome check passes
- [ ] TypeScript `strict` on; `tsc --noEmit` passes
- [ ] One documented pnpm script runs all checks above
- [ ] Lefthook installed through pnpm; the pre-commit hook blocks a deliberately misformatted Rust or TS file
- [ ] Hooks only check staged files, so commits stay fast
