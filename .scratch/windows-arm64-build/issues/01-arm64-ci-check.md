# 01: ARM64 CI check

**What to build:** Every PR and push to `main` is also checked on a real Windows ARM machine, so ARM-only breakage shows up before a release. See the spec, user stories 18–23, and Testing Decisions "Seam 1".

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The pinned Rust toolchain also lists the ARM64 target (`aarch64-pc-windows-msvc`); the Rust version stays the same
- [ ] A new CI job named `check-arm64` runs on `windows-11-arm`, builds the app and runs clippy (`-D warnings`) and `cargo test`
- [ ] The existing `check` job keeps its name and still runs every check on x64
- [ ] Formatting, cargo-deny, Biome and tsc run only in `check`
- [ ] Both jobs are green on the PR that adds this
- [ ] Local development on an x64 PC still works with plain `rustup toolchain install`; CONTRIBUTING is updated if its setup steps change
