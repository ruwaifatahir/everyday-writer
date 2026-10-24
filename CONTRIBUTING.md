# Contributing

Thanks for helping. Setup and build commands are in the [README](README.md).

## Branch workflow

`main` is protected: every change lands through a pull request with green CI. Nobody can push to it directly.

1. Branch off `main`: `feat/<short-name>`, `fix/<short-name>`, `docs/<short-name>` or `chore/<short-name>`.
2. Make small, focused commits.
3. Run `pnpm check` before you push.
4. Open a pull request into `main` and fill in the template.
5. CI (the `check` job) must pass before merging.

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

`pnpm check` runs rustfmt, clippy with warnings as errors, `cargo test`, cargo-deny, Biome and `tsc --noEmit`. CI (`.github/workflows/ci.yml`) runs the same commands on Windows. The lefthook pre-commit hook checks staged files with rustfmt and Biome.

## Cutting a release

Releases are built by `.github/workflows/release.yml` when a `v*` tag is pushed. It builds the NSIS installer on Windows, signs it with the update signing key, and attaches the installer, its update signature (`*_x64-setup.exe.sig`) and `latest.json` to a **draft** GitHub Release. Nothing goes public until you publish the draft. Installed copies check `releases/latest/download/latest.json`, which skips drafts, so publishing is the moment users get the update. There is no Windows code signing yet (see [ADR 0002](docs/adr/0002-updates-from-github-releases-signed.md)).

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

1. Bump `version` to the same value in all three manifests: `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. Run `cargo check --manifest-path src-tauri/Cargo.toml` so `Cargo.lock` picks up the new version.
2. Land the bump on `main` through a pull request.
3. Tag the merged commit on `main` and push the tag:

   ```sh
   git switch main && git pull
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. Watch the run: `gh run watch`. The first job fails if the tag (minus the `v`) differs from any manifest version. Delete the tag (`git push origin :refs/tags/v0.2.0` and `git tag -d v0.2.0`), fix the version, and tag again.
5. Open the draft release on GitHub and check it has the `*_x64-setup.exe` installer, its `*_x64-setup.exe.sig` and `latest.json` (its `version` is the new one, and `platforms.windows-x86_64` has a `signature` and a `url`).
6. Write the notes and publish.

To dry-run the pipeline without a real version, push a test tag such as `v1.1.0-rc.1` from a throwaway branch whose three manifests say `1.1.0-rc.1` (the version job checks they match). Check the draft has the three files from step 5, then delete it as below. Never publish it: a published release that isn't marked pre-release becomes `latest` and goes to every user.

To throw away a draft and its tag: `gh release delete v0.2.0 --yes --cleanup-tag`, then `git tag -d v0.2.0`.

## Privacy rules

- Never log selected text, clipboard contents, prompts, LLM responses or API keys.
- Treat selected text as data, never as instructions to the LLM.
- Store API keys only in the OS keyring.
- Tests never call a real LLM or need an API key.

## Security

Report vulnerabilities privately. See [SECURITY.md](SECURITY.md).
