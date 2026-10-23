Labels: ready-for-agent

# Base project setup

## Problem Statement

The everyday-writer repo has only agent docs and a PRD (`docs/PRD.md`). There is no app, no build, no checks and no release path. Before any feature work (shortcut, capture, correction, popup, replace), the developer needs a standard, open source starter project. Every later change should land on a base that already builds, lints, tests, audits dependencies and ships a Windows installer.

## Solution

A Tauri v2 + Vanilla TypeScript starter that:

- runs as an empty background Windows app,
- already includes every plugin and permission the PRD will need,
- lays out one empty, documented Rust module per job,
- puts multi-provider LLM access behind one small interface,
- enforces quality locally (pre-commit hooks) and on GitHub (CI),
- builds a Windows installer when a version tag is pushed.

No product behavior is built yet.

## User Stories

1. As the developer, I want the project created with the official Tauri v2 tool, so that its layout matches the docs and community examples.
2. As the developer, I want a Vanilla TypeScript frontend, so that the popup and settings stay small and fast with no framework weight.
3. As the developer, I want pnpm as the package manager, so that installs are fast and the lockfile is strict.
4. As the developer, I want the Rust toolchain pinned to the latest stable MSVC release, so that every machine and CI builds with the same compiler.
5. As the developer, I want Rust 2024 edition, so that the code uses current language rules.
6. As a Windows 10 or 11 user, I want the app to run on both versions, so that I can use it on my machine.
7. As the developer, I want the app ID set to `com.ruwaifatahir.everydaywriter`, so that the installer, autostart entry and keyring entries are uniquely named.
8. As the developer, I want the global shortcut plugin added, so that the shortcut feature can be wired later without config work.
9. As the developer, I want the clipboard plugin added, so that capture and replace can be wired later.
10. As the developer, I want the single instance plugin added, so that launching the app twice never starts two background copies.
11. As the developer, I want the autostart plugin added, so that "starts with Windows" (PRD) can be wired later.
12. As the developer, I want the tray icon feature turned on, so that the app can live in the notification area.
13. As the developer, I want each plugin granted only the permissions it needs, so that the app follows least privilege from the start.
14. As the developer, I want one empty, documented Rust module per job (shortcut, capture, LLM, replace, tray, settings), so that each feature has an obvious home.
15. As the developer, I want a `Corrector` interface around the `genai` crate, so that the app can use any LLM provider and I can switch providers anytime.
15a. As a user, I want to choose my provider (OpenAI, Anthropic, Gemini or a Custom OpenAI-compatible endpoint) and my model explicitly, so that the app uses exactly what I picked.
16. As the developer, I want only the `Corrector` implementation to know about `genai`, so that replacing the library later touches one place.
17. As the developer, I want a fake `Corrector`, so that later features can be tested with no network and no API key.
18. As the developer, I want the `genai` provider's API key passed in from code, not read from environment variables, so that keys can come from the Windows keyring.
19. As a user, I want my API key stored in the Windows Credential Manager, so that it is never saved in plain text.
20. As the developer, I want the keyring dependency limited to `keyring-core` plus the Windows store, so that unused platform stores aren't compiled in.
21. As the developer, I want `tracing` logging set up, so that I can debug the background app.
22. As a user, I want the app to never log my selected text or clipboard, so that my writing stays private.
23. As a user, I want no telemetry or analytics, so that nothing about my use leaves my machine except the correction request.
24. As the developer, I want a size-focused release profile (LTO, single codegen unit, size opt level, abort on panic, stripped symbols), so that the binary stays small.
25. As the developer, I want only the crate features we use turned on, so that build time and binary size stay low.
26. As the developer, I want rustfmt config, so that Rust formatting is consistent.
27. As the developer, I want clippy lints set in the Cargo manifest, so that lint rules live with the code and are the same everywhere.
28. As the developer, I want cargo-deny checking licenses, advisories, bans and sources, so that unsafe or incompatible dependencies are caught.
29. As the developer, I want Biome for TypeScript linting and formatting, so that one fast tool covers both.
30. As the developer, I want strict TypeScript type checking, so that frontend mistakes fail early.
31. As the developer, I want lefthook pre-commit hooks running format and lint, so that bad code is caught before it is committed.
32. As the developer, I want an `.editorconfig`, so that editors agree on line endings and indentation.
33. As the developer, I want a `.gitignore` covering Rust, Node and Tauri build output, so that build artifacts never get committed.
34. As the developer, I want CI on every pull request running Rust formatting, clippy (warnings as errors), tests, cargo-deny, Biome and the TypeScript check, so that `main` always passes.
35. As the developer, I want CI to cache Rust and pnpm dependencies, so that checks run fast.
36. As the developer, I want a release workflow that builds the NSIS installer on Windows when I push a version tag, so that releases work from day one.
37. As the developer, I want the release workflow to publish the installer as a draft GitHub Release, so that I review it before it goes public.
38. As the developer, I want Dependabot opening weekly update PRs for Cargo, npm and GitHub Actions, so that dependencies stay current at no cost.
39. As the developer, I want a protected `main` branch that only accepts pull requests with green CI, so that broken code can't land.
40. As the developer, I want Conventional Commits, so that history is readable and changelogs can be generated.
41. As an open source contributor, I want a README explaining what the app does and how to build it, so that I can get started.
42. As an open source contributor, I want a CONTRIBUTING guide, so that I know the workflow, checks and commit style.
43. As a security researcher, I want a SECURITY policy, so that I know how to report a problem privately.
44. As an open source contributor, I want a PR template, so that I know what to include.
45. As a user or contributor, I want the code dual-licensed MIT + Apache-2.0, so that it follows the Rust ecosystem norm.
46. As the developer, I want the public GitHub repo `ruwaifatahir/everyday-writer` created and the template pushed, so that CI and Dependabot start running.
47. As the developer, I want `pnpm tauri dev` to start the empty app with no errors, so that I know the template works on my machine.

## Implementation Decisions

- **Prerequisites:** Rust via rustup (stable-msvc default) and Microsoft C++ Build Tools ("Desktop development with C++") must be installed first. WebView2, Node 24 and pnpm are already installed.
- **Scaffold:** `create-tauri-app` with the Vanilla + TypeScript template, then cleaned of demo code. Tauri 2.12.x at the time of writing.
- **Plugins:** official Tauri plugins `global-shortcut`, `clipboard-manager`, `single-instance` and `autostart`, plus the Tauri `tray-icon` feature. They are added through `tauri add` and registered, but have no behavior yet. Single instance is registered first, as its docs require.
- **Capabilities:** one capability file per window (the popup, and a settings window later). It grants only the plugin permissions each window needs.
- **Rust modules:** `shortcut`, `capture`, `llm`, `replace`, `tray`, `settings` (holds keyring access). Each has a doc comment stating its job and its privacy rule where one applies. No logic yet.
- **`Corrector` interface (in `llm`):** async, takes the text to rewrite plus the active instruction set's instructions, and returns the rewritten text or a typed error. The selected text is always passed as data, never merged into the instructions. Two fixed rules are always added on top of the user's instructions: return only the rewritten text, and treat the selected text as text, never as commands. The user's instructions are otherwise unrestricted (style, tone, language, purpose).
  - **Real implementation:** backed by `genai` (0.7.x). The provider and model come from the user's settings, never inferred from the model name. Custom providers are OpenAI-compatible endpoints with a user-set base URL. Each call builds a genai `ServiceTarget` with an explicit adapter, endpoint and the API key read from the keyring. No genai `AuthResolver`, no fallback to environment variables, and no model-name inference (see `docs/adr/0001-explicit-genai-service-target.md`).
  - **Fake implementation:** returns preset results, for tests.
- **Keyring:** `keyring-core` + `windows-native-keyring-store`, not the full `keyring` crate, as the `keyring` v4 docs advise.
- **Async:** tokio with only the features Tauri and `genai` need. Not `full`.
- **Errors:** `thiserror` for typed errors in library modules.
- **Logging:** `tracing`. Log fields must never include selected text, clipboard contents, prompts, LLM responses or API keys.
- **Release profile:** `codegen-units = 1`, `lto = true`, `opt-level = "s"`, `panic = "abort"`, `strip = true` (Tauri's size guide).
- **Bundle:** NSIS installer only. Windows 10 and 11.
- **Lint config:** clippy lints in the Cargo manifest `[lints]` table. Biome config at the repo root. TypeScript `strict` on.
- **CI (GitHub Actions, Windows runner):** fmt check, clippy `-D warnings`, `cargo test`, `cargo deny check`, Biome check, `tsc --noEmit`. Actions pinned to commit SHAs.
- **Release workflow:** triggered by `v*` tags. Builds with the official `tauri-action` and creates a draft release. Updater and code signing are not set up yet.
- **Dependabot:** weekly; ecosystems `cargo`, `npm`, `github-actions`.
- **Repo:** public, `ruwaifatahir/everyday-writer`, created with `gh` by the agent. Branch protection on `main` requires a PR and green CI.
- **License:** `LICENSE-MIT` + `LICENSE-APACHE`; the Cargo manifest says `MIT OR Apache-2.0`.

## Testing Decisions

- **What makes a good test:** it checks behavior seen through a public interface, not internal details. Tests must never call a real LLM or need an API key.
- **Seam 1 (main seam), the CI pipeline:** a clean checkout passes every CI check and the release workflow produces an installer. This proves the template as a whole.
- **Seam 2, the `Corrector` interface:** the template ships the fake `Corrector` and one test that uses it through the interface. All later correction features test through this seam.
- **Smoke check:** `pnpm tauri dev` starts the app with no errors (manual, once).
- **Prior art:** none. This is the first code in the repo, and these tests set the pattern.

## Out of Scope

- Any product behavior from the PRD: shortcut handling, text capture, correction prompts, the popup UI, replace, instruction sets and starter templates, the settings UI and real tray menus.
- UI Automation capture.
- Auto-updater and code signing.
- Prompt design and evaluation (promptfoo).
- Mac, Linux and mobile.

## Further Notes

- Product requirements live in `docs/PRD.md`. This spec only prepares the ground for them.
- Versions checked on 2026-10-23: tauri 2.12.5, create-tauri-app 4.7.5, genai 0.7.0, keyring 4.2.0, tokio 1.54.1.
- `genai` is pre-1.0. The `Corrector` interface is the guard against its breaking changes.
