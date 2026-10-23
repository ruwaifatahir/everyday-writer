# 01: Runnable empty app

**What to build:** A developer clones the repo, runs one command and an empty everyday-writer window opens. A release build produces a small NSIS installer for Windows 10 and 11. Use the official Tauri v2 `create-tauri-app` scaffold (Vanilla + TypeScript, pnpm), with all demo code removed. Pin the Rust toolchain to latest stable MSVC with edition 2024. Set the app ID to `com.ruwaifacodes.everydaywriter`. Add the size-focused release profile (`codegen-units = 1`, `lto = true`, `opt-level = "s"`, `panic = "abort"`, `strip = true`), `.gitignore` covering Rust, Node and Tauri output, `.editorconfig` and dual MIT + Apache-2.0 licenses.

Spec: `.scratch/base-project-setup/spec.md`

**Precondition:** Rust (rustup, stable-msvc) and Microsoft C++ Build Tools ("Desktop development with C++") are installed by the developer.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Scaffolded with the official `create-tauri-app`, Vanilla + TypeScript template, pnpm
- [ ] Demo greet command and demo UI removed
- [ ] Rust toolchain pinned to stable MSVC in a toolchain file; Rust edition 2024
- [ ] App ID is `com.ruwaifacodes.everydaywriter`; product name is everyday-writer
- [ ] Release profile matches the spec's size settings
- [ ] Bundle target is NSIS only
- [ ] `.gitignore` and `.editorconfig` present; build output is not tracked
- [ ] `LICENSE-MIT` and `LICENSE-APACHE` present; Cargo and package manifests declare `MIT OR Apache-2.0`
- [ ] `pnpm tauri dev` opens the empty app with no errors or warnings
- [ ] `pnpm tauri build` produces an NSIS installer
