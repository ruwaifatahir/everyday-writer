# 02: Release builds make signed updates

**What to build:** Tagging a release produces, in the draft GitHub Release, the NSIS installer, its update signature and a `latest.json` that points at it. Installed copies will later trust only updates signed with the project's key. See the spec ("Release and delivery") and ADR 0002.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Needs the maintainer:** generating the key pair and adding the two repo secrets are manual steps. Write them as exact steps in `CONTRIBUTING.md` and hand them over; never generate, print or commit the private key.

- [ ] `tauri-plugin-updater` 2.x is added and registered Rust-side, with default features off and only the TLS feature that matches the `reqwest` already in the graph (no `zip`)
- [ ] Config has `createUpdaterArtifacts: true`, the endpoint `https://github.com/ruwaifatahir/everyday-writer/releases/latest/download/latest.json` (no `{{current_version}}`), the public key, and `windows.installMode: "passive"`; install stays per-user
- [ ] The release workflow passes `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from secrets, sets `uploadUpdaterJson: true` and `updaterJsonPreferNsis: true`, keeps signatures upload and `releaseDraft: true`
- [ ] The release job fails early with a clear message if either secret is empty
- [ ] `CONTRIBUTING.md` documents key setup: generate with a password, back up key and password to a password manager and USB, add both secrets, and why losing the key strands existing installs
- [ ] Before/after numbers for installer size and `cargo tree -d` are recorded in this ticket's comments
- [ ] `cargo deny check` passes; any new license is added to the allowlist only after review, noted in comments
- [ ] Confirm whether Rust-only use needs any `updater:*` capability; if one is required, grant it to no window
- [ ] A test tag (e.g. a `-rc` draft that is deleted afterwards) shows `latest.json`, the installer and its `.sig` in the draft release
