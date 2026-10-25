# Contributing

Thanks for helping.

## Setup

Prerequisites: Rust via rustup (`rustup toolchain install` installs the pinned toolchain and both the x64 and ARM64 targets), Microsoft C++ Build Tools ("Desktop development with C++"), WebView2, Node 24 and pnpm.

```sh
pnpm install
pnpm tauri dev     # run the app
pnpm tauri build   # build the NSIS installer
```

The installer build signs the installer for the updater, so it needs a signing key. See [Building the installer locally](#building-the-installer-locally).

## Branch workflow

`main` is protected: every change lands through a pull request with green CI. Nobody can push to it directly.

1. Branch off `main`: `feat/<short-name>`, `fix/<short-name>`, `docs/<short-name>` or `chore/<short-name>`.
2. Make small, focused commits.
3. Run `pnpm check` before you push.
4. Open a pull request into `main` and fill in the template.
5. CI must pass before merging: the `check` job (x64) and the `check-arm64` job (Windows on ARM).

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add replace button to popup
fix: restore clipboard after failed capture
docs: explain keyring setup
chore: bump tauri to 2.12.5
```

Common types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`, `build`. Mark breaking changes with `!` (`feat!: ...`).

## Checks

```sh
pnpm check   # everything CI runs
pnpm fix     # apply rustfmt and Biome fixes
```

`pnpm check` runs rustfmt, clippy with warnings as errors, `cargo test`, cargo-deny (licenses, advisories, bans, sources), Biome lint and format, `tsc --noEmit`, and the release check fixture tests (`.github/scripts/release-check.tests.ps1`). CI (`.github/workflows/ci.yml`) runs the same commands on Windows. Install cargo-deny 0.20.2 first (`cargo binstall cargo-deny@0.20.2`, or `cargo install cargo-deny --locked`).

Clippy lint levels live in the `[lints]` table of `src-tauri/Cargo.toml`. cargo-deny config is `src-tauri/deny.toml`; Biome config is `biome.json`.

`pnpm install` also installs lefthook git hooks. The pre-commit hook runs rustfmt and Biome in check-only mode on staged files, so a badly formatted or lint-failing file blocks the commit. Run `pnpm fix`, re-stage and commit again.

## Cutting a release

Releases are built by `.github/workflows/release.yml` when a `v*` tag is pushed. It builds both Builds on the Windows x64 machine (the ARM64 one cross-compiled with `--target aarch64-pc-windows-msvc`), signs each NSIS installer with the update signing key, and attaches both installers (`*_x64-setup.exe`, `*_arm64-setup.exe`), their update signatures (`.sig`) and one `latest.json` listing both to a **draft** GitHub Release. The last job, `release check`, fails the run unless all of those are on the draft and each `latest.json` entry points at its own Build's installer. Its logic is `.github/scripts/release-check.ps1`; its fixture tests (`.github/scripts/release-check.tests.ps1`) run in `pnpm check` and CI. Nothing goes public until you publish the draft. Installed copies check `releases/latest/download/latest.json`, which skips drafts, so publishing is the moment users get the update. There is no Windows code signing yet (see [ADR 0002](docs/adr/0002-updates-from-github-releases-signed.md)).

### Update signing key (one-time setup)

Installed copies only accept an update signed with the project's private key; the matching public key is built into every copy (`plugins.updater.pubkey` in `src-tauri/tauri.conf.json`). **If the private key or its password is lost, existing installs can never update again**: every user has to download and reinstall by hand. A leaked key lets whoever also controls a release push code to every user. So the key gets a password, two offline backups, and lives nowhere else but the GitHub secret.

Do this once, on your own machine, in PowerShell. Never commit the private key, paste it in an issue or chat, or put it in the repo folder.

1. Generate the key pair outside the repo. The command asks for a password twice; use a long random one from your password manager.

   ```powershell
   pnpm tauri signer generate -w "$HOME\.tauri\everyday-writer.key"
   ```

   This writes `everyday-writer.key` (private, password-protected) and `everyday-writer.key.pub` (public).
2. Back up both files and the password:
   - in your password manager: a new entry holding the password, with `everyday-writer.key` and `everyday-writer.key.pub` attached (or their contents pasted as notes);
   - on a USB drive kept offline: copy both files there. Keep the password off the drive (it stays in the password manager).
3. Add the two repo secrets (GitHub CLI, logged in as a repo admin). The first reads the file directly, so the key never shows on screen; the second prompts for the password without echoing it.

   ```powershell
   Get-Content -Raw "$HOME\.tauri\everyday-writer.key" | gh secret set TAURI_SIGNING_PRIVATE_KEY
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD
   gh secret list   # both names should be listed
   ```

   Or in the browser: Settings > Secrets and variables > Actions > New repository secret, once per name.
4. Put the public key in the app: copy the single line in `everyday-writer.key.pub` into `plugins.updater.pubkey` in `src-tauri/tauri.conf.json` and land it through a pull request. The public key is not secret.

The release job fails at "Require update signing secrets" if either secret is missing or empty. Don't replace the key unless it is lost or leaked: a new key means shipping one last update signed with the old key that carries the new public key, and copies that miss it must reinstall by hand.

### Building the installer locally

`createUpdaterArtifacts` makes `pnpm tauri build` sign the installer, so it fails without a key. Either point it at a key for the session (your real one, or a throwaway pair generated outside the repo; installers signed with a throwaway key just can't be served as updates):

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw "$HOME\.tauri\everyday-writer.key"
$password = Read-Host -AsSecureString "Key password"   # keeps it out of the shell history
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = [Net.NetworkCredential]::new("", $password).Password
pnpm tauri build
```

or skip the update artifacts:

```powershell
pnpm tauri build --config '{\"bundle\":{\"createUpdaterArtifacts\":false}}'
```

### Steps

1. Bump `version` to the same value in all three manifests: `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. Run `cargo check --manifest-path src-tauri/Cargo.toml` so `Cargo.lock` picks up the new version. In `CHANGELOG.md`, rename `## Unreleased` to the new version and date.
2. Land the bump on `main` through a pull request.
3. Tag the merged commit on `main` and push the tag:

   ```sh
   git switch main && git pull
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. Watch the run: `gh run watch`. The first job fails if the tag (minus the `v`) differs from any manifest version. Delete the tag (`git push origin :refs/tags/v0.2.0` and `git tag -d v0.2.0`), fix the version, and tag again.
5. Confirm the `release check` job passed, then open the draft release on GitHub and check it has both installers (`*_x64-setup.exe`, `*_arm64-setup.exe`), their `.sig` files and `latest.json`. If `release check` failed because the two Build jobs overwrote each other's `latest.json`, delete the draft and its tag and push the tag again.
6. Before publishing, prove the real update path: update an installed older build to this version and run the rest of [docs/manual-test-checklist.md](docs/manual-test-checklist.md). A draft can't be fetched by an installed copy, so the "Updates" section there uses a pre-release test tag of the same commit and an older build pointed at it. Record the run in the checklist's results table. Don't publish if any update case fails: once a broken updater ships, users must reinstall by hand.
7. Paste that version's `CHANGELOG.md` notes into the release and publish.

To dry-run the pipeline without a real version, push a test tag such as `v1.1.0-rc.1` from a throwaway branch whose three manifests say `1.1.0-rc.1` (the version job checks they match). Check `release check` passed and the draft has the five files from step 5, then delete it as below. Only publish it with **Set as a pre-release** ticked, as the update test in step 6 does: a published release that isn't marked pre-release becomes `latest` and goes to every user.

To throw away a draft and its tag: `gh release delete v0.2.0 --yes --cleanup-tag`, then `git tag -d v0.2.0`.

## Privacy rules

- Never log selected text, clipboard contents, prompts, LLM responses or API keys.
- Treat selected text as data, never as instructions to the LLM.
- Store API keys only in the OS keyring.
- Tests never call a real LLM or need an API key.

## Security

Report vulnerabilities privately. See [SECURITY.md](SECURITY.md).
