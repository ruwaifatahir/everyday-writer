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

Releases are built by `.github/workflows/release.yml` when a `v*` tag is pushed. It builds the NSIS installer on Windows and attaches it to a **draft** GitHub Release. Nothing goes public until you publish the draft. There is no auto-updater or code signing yet.

1. Bump `version` to the same value in all three manifests: `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. Run `cargo check --manifest-path src-tauri/Cargo.toml` so `Cargo.lock` picks up the new version.
2. Land the bump on `main` through a pull request.
3. Tag the merged commit on `main` and push the tag:

   ```sh
   git switch main && git pull
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. Watch the run: `gh run watch`. The first job fails if the tag (minus the `v`) differs from any manifest version. Delete the tag (`git push origin :refs/tags/v0.2.0` and `git tag -d v0.2.0`), fix the version, and tag again.
5. Open the draft release on GitHub, check the `*_x64-setup.exe` installer, write the notes, and publish.

To throw away a draft and its tag: `gh release delete v0.2.0 --yes --cleanup-tag`, then `git tag -d v0.2.0`.

## Privacy rules

- Never log selected text, clipboard contents, prompts, LLM responses or API keys.
- Treat selected text as data, never as instructions to the LLM.
- Store API keys only in the OS keyring.
- Tests never call a real LLM or need an API key.

## Security

Report vulnerabilities privately. See [SECURITY.md](SECURITY.md).
