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
