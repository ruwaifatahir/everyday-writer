Labels: ready-for-agent

# An ARM64 Build alongside the x64 Build

## Problem Statement

Everyday Writer ships one installer, for x64 PCs. People with Windows on ARM laptops (Snapdragon, Surface Pro X and similar) can only run it under x64 emulation, which is slower and uses more battery. Most popular open-source Windows apps now publish an x64 and an ARM64 download side by side. The 9 Tauri apps and 14 other Windows apps researched for this spec include pot, clash-verge-rev, yaak, readest, EcoPaste, PowerToys, Windows Terminal, Zed, OBS, KeePassXC and LocalSend. Everyday Writer doesn't, so it looks less finished and serves ARM users worse.

No ARM PC runs the app today, so there are no existing installs to move over.

## Solution

Every release publishes two Builds, an x64 installer and an ARM64 installer. The README's install line names both and says which PC each is for. Installed copies keep updating themselves the same way as today, and each copy only ever receives Updates for its own Build. CI also checks the code on a Windows ARM machine, and a release only goes out when both Builds were built and published correctly.

The first release with both Builds is v1.4.0.

## User Stories

1. As a user on an ARM laptop, I want a native ARM64 installer, so that the app runs at full speed without emulation.
2. As a user on an ARM laptop, I want the app to use less battery, so that a background helper doesn't drain my laptop.
3. As a user on a normal Windows PC, I want the x64 installer to keep working exactly as before, so that nothing changes for me.
4. As a user, I want each installer's file name to say which PC it's for (`x64` or `arm64`), so that I can pick the right one on the Releases page.
5. As a user reading the README, I want the install line to name both installers and which PC each one is for, so that I don't have to guess.
6. As a user with the x64 Build installed, I want Updates to keep bringing me the x64 Build, so that my install never breaks.
7. As a user with the ARM64 Build installed, I want Updates to bring me the ARM64 Build, so that I stay native after updating.
8. As a user with the ARM64 Build, I want Updates signed with the same project key as the x64 Build, so that my copy refuses anything we didn't publish.
9. As a user with the ARM64 Build, I want the same "Update now" flow in Settings, so that updating works the same on every PC.
10. As a user with the ARM64 Build, I want my settings, API key and Instruction Sets kept across Updates, so that updating costs me nothing.
11. As a user with the ARM64 Build, I want rewrites, the popup and Replace to behave exactly as on x64, so that the app is the same app.
12. As a user on an ARM laptop who installed the x64 Build, I want it to keep working under emulation, so that I'm never forced to reinstall.
13. As the maintainer, I want both Builds made by one tag push, so that releasing stays one step.
14. As the maintainer, I want the release to fail if either Build fails, so that I never publish a release that has only one installer.
15. As the maintainer, I want the release to fail if `latest.json` doesn't list both Builds, so that ARM64 users are never left without Updates.
16. As the maintainer, I want the release to fail if any installer is missing its update signature, so that no unverifiable Update reaches users.
17. As the maintainer, I want the release to stay a draft until I publish it, so that I still decide when an Update reaches users (ADR 0002).
18. As the maintainer, I want CI to build and test the code on a real Windows ARM machine, so that ARM-only breakage is caught before a release, even though I don't own an ARM PC.
19. As the maintainer, I want the ARM CI check to be required on `main`, so that nothing merges that breaks the ARM64 Build.
20. As the maintainer, I want the existing required `check` job to keep its name, so that branch protection on `main` keeps working.
21. As the maintainer, I want checks that give the same result on any machine (formatting, cargo-deny, Biome, tsc) to run only once, so that CI stays fast.
22. As the maintainer, I want both Builds made on the same Windows x64 build machine, so that the release workflow stays simple and quick.
23. As a contributor, I want local development to keep working on an x64 PC with no extra setup beyond what rustup installs from the pinned toolchain, so that contributing stays easy.
24. As a contributor, I want the GLOSSARY to define **Build**, so that "x64 version" is never confused with a version number like 1.4.0.

## Implementation Decisions

- **Builds shipped:** x64 (`x86_64-pc-windows-msvc`) and ARM64 (`aarch64-pc-windows-msvc`). No 32-bit x86, no portable version, no MSI. Both are NSIS installers, as today.
- **No app code changes.** The research found no architecture-specific code, inline assembly or C dependencies. The `windows` crate, clipboard, keyring, `native-tls` and WebView2 all support ARM64.
- **Toolchain:** the pinned Rust toolchain also lists the ARM64 target, so `rustup toolchain install` on any machine installs both. The Rust version stays pinned as it is.
- **Release workflow:**
  - The build job becomes a matrix of two Builds, both on the existing Windows x64 build machine.
  - The ARM64 Build is cross-compiled by passing `--target aarch64-pc-windows-msvc` to `tauri-action`. Tauri's docs and every Tauri app researched do it this way; the MSVC ARM64 build tools are already on GitHub's Windows machines.
  - Both matrix jobs upload to the same draft release.
  - The tag-version check job runs first, unchanged.
  - Fail-fast is acceptable: a partial draft is never published because the release check (below) fails the run.
- **File names:** Tauri's defaults: `Everyday.Writer_<version>_x64-setup.exe` and `Everyday.Writer_<version>_arm64-setup.exe`, each with its `.sig`.
- **Updater:**
  - One `latest.json` per release with platform keys `windows-x86_64` and `windows-aarch64`, plus the `-nsis` duplicates `tauri-action` writes.
  - The update URL, the public key and the ask-first install flow stay exactly as in ADR 0002. Each installed copy reads only its own key, so an x64 copy, even one running on an ARM PC, keeps getting x64 Updates. No migration logic.
- **Release check:**
  - A new final job runs after both Builds. It reads the draft release and fails the run unless it has both installers and both `.sig` files, and `latest.json` has `windows-x86_64` and `windows-aarch64` entries.
  - Each entry must point at the installer for its own Build and carry a non-empty signature.
  - This also catches the two parallel matrix jobs overwriting each other's `latest.json`.
- **CI:**
  - A new job, `check-arm64`, runs on GitHub's `windows-11-arm` machine (free, because the repo is public). It builds the app and runs clippy and `cargo test`.
  - The existing `check` job keeps its name and still runs every check on x64. The maintainer adds `check-arm64` as a required status check on `main`.
  - If a CI tool used only on x64 has no ARM64 binary, that doesn't matter, because those tools don't run on the ARM machine.
- **ARM64 ships when its CI passes.** No manual test on ARM hardware, and no "untested" note in the README or release notes.
- **README:** the install line names both installers, x64 for most PCs and ARM64 for ARM laptops like Snapdragon. No other README changes.
- **Release notes:** the draft release body tells users which installer to pick.
- **GLOSSARY:** add **Build**: "The installer for one kind of PC, x64 or ARM64. Every release publishes one Build for each." _Avoid_: version, edition, flavour.
- **Version:** the first release with both Builds is v1.4.0, a minor bump because it adds a new download.
- **No ADR.** Adding or dropping a Build later is easy to reverse.

## Testing Decisions

- Good tests here check what users and installed copies actually see: the files on the release and the contents of `latest.json`. They don't check how the workflow produced them.
- **Seam 1, CI on ARM:** the `check-arm64` job builds and runs the existing Rust test suite natively on Windows ARM. This reuses the existing `cargo test` / clippy steps of the `check` job as prior art; no new tests are written.
- **Seam 2, release check:** the final release job described above is the test for the release workflow. Prior art is the existing tag-version check job, which already fails a release early on a mismatch.
- No new Rust or frontend tests: no app code changes.
- The first real run is the v1.4.0 tag. Before publishing that draft, the maintainer confirms that the release check passed and that both installers are listed.

## Out of Scope

- 32-bit x86 Builds.
- Portable (no-install) versions, MSI, MSIX, winget.
- macOS and Linux.
- Windows code signing (the SmartScreen warning stays as today).
- Moving x64 copies on ARM PCs over to the ARM64 Build automatically.
- Manual testing on ARM hardware.
- A fixed (bundled) WebView2 runtime.

## Further Notes

- **Research basis (October 2026):**
  - Of the 9 Tauri apps checked, 5 ship ARM64, all cross-compiled on `windows-latest`, all NSIS only, with `windows-aarch64` in `latest.json`. Only pot ships x86.
  - Of the 14 other Windows apps, 8 ship ARM64, and only 3 older projects still ship x86.
- **Installer on ARM:** the NSIS installer program itself is x86 and runs under emulation on ARM PCs; the app it installs is native ARM64. This is normal Tauri behaviour.
- **Old README wording:** "Download the installer" becomes wrong as soon as v1.4.0 is published, so the README change ships in the same release.
