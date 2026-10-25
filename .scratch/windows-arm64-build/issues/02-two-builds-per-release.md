# 02: Two Builds per release, plus the release check

**What to build:** One tag push produces a draft release with an x64 Build and an ARM64 Build. The run fails unless both installers, both signatures and a `latest.json` listing both Builds are on the draft. See the spec, user stories 1–17 and 22, Implementation Decisions "Release workflow", "File names", "Updater" and "Release check", and Testing Decisions "Seam 2".

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] The release build job is a matrix of two Builds on the existing Windows x64 build machine; the ARM64 Build passes `--target aarch64-pc-windows-msvc` to `tauri-action`
- [ ] The tag-version check job still runs first, unchanged
- [ ] The draft has `Everyday.Writer_<version>_x64-setup.exe` and `Everyday.Writer_<version>_arm64-setup.exe`, each with its `.sig`
- [ ] `latest.json` has `windows-x86_64` and `windows-aarch64` entries, each pointing at its own Build's installer with a non-empty signature
- [ ] A final release check job runs after both Builds and fails the run if any of the above is missing or wrong, including one matrix job overwriting the other's `latest.json`
- [ ] The release stays a draft; the update URL, public key and ask-first flow are unchanged (ADR 0002)
- [ ] The draft release text tells users to pick x64 for most PCs and ARM64 for ARM laptops like Snapdragon
- [ ] Proven with a throwaway test tag (see the dry-run note in CONTRIBUTING): the release check passes, and it fails when an asset is removed. The test release and tag are deleted afterwards, and the result is noted in this ticket's comments

## Comments

**Dry-run, 2026-10-25 (tag `v1.4.0-rc.1` from throwaway branch `dryrun/02-two-builds-rc1`, manifests set to `1.4.0-rc.1`):** passed on the first attempt.

- Run: https://github.com/ruwaifatahir/everyday-writer/actions/runs/40214890804
- Attempt 1: `tag matches app version`, `build and draft release (x64)`, `build and draft release (arm64)` and `release check` all passed. The draft had `Everyday.Writer_1.4.0-rc.1_x64-setup.exe`, `Everyday.Writer_1.4.0-rc.1_arm64-setup.exe`, both `.sig` files and `latest.json` with `windows-x86_64`, `windows-x86_64-nsis`, `windows-aarch64` and `windows-aarch64-nsis`, each pointing at its own Build's installer. The two jobs' `latest.json` uploads merged cleanly. The release body names both installers.
- Attempt 2 (https://github.com/ruwaifatahir/everyday-writer/actions/runs/40214890804/attempts/2): deleted `Everyday.Writer_1.4.0-rc.1_arm64-setup.exe.sig` from the draft and re-ran only `release check` (`gh run rerun --job`). It failed with `Missing update signature Everyday.Writer_1.4.0-rc.1_arm64-setup.exe.sig`, as intended.
- Cleanup: the draft release, the `v1.4.0-rc.1` tag (remote and local) and the throwaway branch (remote and local) are deleted. The draft was never published.
- The other failure cases (missing installer, missing platform key, an entry pointing at the other Build's installer, empty signature, wrong version, not a draft) are covered by the fixture tests in `.github/scripts/release-check.tests.ps1`, which CI runs.
