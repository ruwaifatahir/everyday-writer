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

## Comments

**Implementer (ticket 02), 2026-10-24**

- `tauri-plugin-updater = { version = "2.14.0", default-features = false, features = ["native-tls"] }` in the desktop-only deps, registered Rust-side in `lib.rs` after the notification plugin. `native-tls` matches the reqwest 0.13.5 genai already brings. No `rustls`, no `zip`.
- Capability: none needed. The build log says "Removed unused commands from updater: check, download, install, download_and_install" (`removeUnusedCommands`), so the plugin's JS commands aren't in the binary. No window gets an `updater:*` permission.
- Config: `createUpdaterArtifacts: true`, and `plugins.updater` with the static endpoint, the pubkey and `windows.installMode: "passive"`. There's no `bundle.windows.nsis` block, so the install stays per-user. **Deviation:** I also set `requireSignedVersion: true`. `tauri build` binds the version into the signature's trusted comment (checked: the throwaway-signed `.sig` reads `version:1.0.0`), so a tampered `latest.json` can't pair a new version number with an older signed installer. This is safe because no older updater-signed releases exist.
- Guard test `lib.rs::tests::updates_come_signed_from_the_static_github_file_and_install_per_user` reads `tauri.conf.json`. It fails if any of these change: the endpoint (fixed, no template vars), the pubkey being present, `requireSignedVersion`, passive mode, updater artifacts, or the per-user install. It failed (red) against the pre-ticket config. I tested at the shipped config because this ticket has no runtime logic. The seam wasn't confirmed with the user, since I ran as a subagent.
- Release workflow: a "Require update signing secrets" step fails before the build if either secret is empty. tauri-action gets both secrets plus `uploadUpdaterJson: true`, `uploadUpdaterSignatures: true`, `updaterJsonPreferNsis: true` and `releaseDraft: true`.
- `CONTRIBUTING.md` now has:
  - "Update signing key (one-time setup)": generate with a password outside the repo, back up to a password manager and an offline USB, add both secrets with `gh secret set`, copy the `.pub` line into `tauri.conf.json`, and why a lost key strands installs.
  - "Building the installer locally": set the key env vars, or pass `--config` to turn off updater artifacts. I checked both in PowerShell 5.1.
  - Release step 5 now checks for the `.sig` and `latest.json`, plus a note on dry runs with a `-rc` tag.
  - The "update an installed older build before publishing" step is left to ticket 09.
  - README points to the local-build section, because a plain `pnpm tauri build` now needs a key.
- Sizes are from local release builds of the same commit, before vs after. The after build was signed with a throwaway key kept outside the repo.
  - NSIS installer: 1,746,602 -> 1,754,003 bytes (+7,401, +0.4%). For reference, the published v1.0.0 installer is 1,732,359 bytes.
  - `everyday-writer.exe`: 4,342,784 -> 4,368,384 bytes (+25,600).
- Crates (`cargo tree -e normal --target x86_64-pc-windows-msvc`): 290 -> 294 unique crates, including `tauri-plugin-updater`, `minisign-verify` and `base64 0.22.1`. Crate names with more than one version (`cargo tree -d`): 7 -> 8. The only new duplicate is `base64` (0.22.1 from the updater, next to 0.23.1). Already duplicated before and unchanged: getrandom, hashbrown, indexmap, keyboard-types, miniz_oxide, syn, windows-sys.
- `cargo deny check`: advisories, bans, licenses and sources all ok. No new license was needed (minisign-verify is MIT, base64 is MIT/Apache-2.0).
- **Open for the maintainer:**
  1. Generate the key and add the two secrets, following CONTRIBUTING.
  2. Push a `-rc` test tag from a throwaway branch whose manifests match it. Confirm the draft has the installer, its `.sig` and `latest.json`, then delete the draft. Agents don't push tags.
  3. Once a release is published, check that the `latest.json` `url` downloads. tauri-action writes an `api.github.com/.../releases/assets/<id>` URL there.
