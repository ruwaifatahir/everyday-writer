# 01: Runnable empty app

**What to build:** A developer clones the repo, runs one command and an empty everyday-writer window opens. A release build produces a small NSIS installer for Windows 10 and 11. Use the official Tauri v2 `create-tauri-app` scaffold (Vanilla + TypeScript, pnpm), with all demo code removed. Pin the Rust toolchain to latest stable MSVC with edition 2024. Set the app ID to `com.ruwaifatahir.everydaywriter`. Add the size-focused release profile (`codegen-units = 1`, `lto = true`, `opt-level = "s"`, `panic = "abort"`, `strip = true`), `.gitignore` covering Rust, Node and Tauri output, `.editorconfig` and dual MIT + Apache-2.0 licenses.

Spec: `.scratch/base-project-setup/spec.md`

**Precondition:** Rust (rustup, stable-msvc) and Microsoft C++ Build Tools ("Desktop development with C++") are installed by the developer.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] Scaffolded with the official `create-tauri-app`, Vanilla + TypeScript template, pnpm
- [x] Demo greet command and demo UI removed
- [x] Rust toolchain pinned to stable MSVC in a toolchain file; Rust edition 2024
- [x] App ID is `com.ruwaifatahir.everydaywriter`; product name is everyday-writer
- [x] Release profile matches the spec's size settings
- [x] Bundle target is NSIS only
- [x] `.gitignore` and `.editorconfig` present; build output is not tracked
- [x] `LICENSE-MIT` and `LICENSE-APACHE` present; Cargo and package manifests declare `MIT OR Apache-2.0`
- [x] `pnpm tauri dev` opens the empty app with no errors or warnings
- [x] `pnpm tauri build` produces an NSIS installer

## Comments

Done on branch `feat/01-runnable-empty-app`. Scaffolded with `create-tauri-app` 4.7.5 (`vanilla-ts`, pnpm); Tauri CLI/API 2.12.4.

- `pnpm tauri dev`: builds and opens the empty `everyday-writer` window. No compiler warnings or errors.
- `pnpm tauri build`: `src-tauri/target/release/bundle/nsis/everyday-writer_0.1.0_x64-setup.exe` is **1.08 MiB** (1,132,830 bytes). `everyday-writer.exe` is **2.96 MiB** (3,104,768 bytes).

Deviations:
- `rust-toolchain.toml` pins the exact release `1.99.1` (latest stable at setup) instead of the moving `stable` channel, so every machine and CI use the same compiler. Target is `x86_64-pc-windows-msvc`.
- Removed the scaffold's `tauri-plugin-opener` and `@tauri-apps/plugin-opener` along with the demo. They were used only by the demo links.
- Removed `withGlobalTauri` and the `.icns` icon entry (macOS only). Bundle target is `["nsis"]`.
- Merged the scaffold's two `.gitignore` files into one at the root. Added `.gitattributes` (`eol=lf`) to match `.editorconfig`.
- Replaced the scaffold README with a short project README. Ticket 05 expands it.
- CSP is still `null`, as in the scaffold. Setting it is left for when the popup UI exists.
