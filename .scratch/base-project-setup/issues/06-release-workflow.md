# 06: Release workflow

**What to build:** The developer pushes a version tag and GitHub builds the Windows NSIS installer, then attaches it to a draft GitHub Release for review before it goes public. Use the official `tauri-action`. Auto-updater and code signing are not set up yet.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 05: GitHub repo and CI

**Status:** resolved

- [x] Workflow triggers on `v*` tags only
- [x] Builds on a Windows runner with the official `tauri-action`, pinned to a commit SHA
- [x] Creates a draft release named after the tag, with the NSIS `.exe` attached
- [x] App version in the manifests matches the tag; the workflow fails if they differ
- [x] Workflow token has only the permissions needed to create the release
- [x] A test tag produces a draft release with the installer; the test release and tag are deleted afterward
- [x] CONTRIBUTING documents how to cut a release

## Comments

Workflow: `.github/workflows/release.yml`. Two jobs. `version` (`contents: read`) compares the tag without its `v` against `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. `release` (`contents: write`, the only job with write access) needs `version`, then builds with `tauri-apps/tauri-action@1deb371b0cd8bd54025b384f1cd735e725c4060f` (v1.0.0) as a draft release. Top-level permissions are `{}`. Other actions use the same SHAs and setup steps as `ci.yml`.

Test runs (both on commit 972cfbf):

- Mismatch guard, tag `v0.0.99-test`: https://github.com/ruwaifatahir/everyday-writer/actions/runs/39967269526. Failed in the `version` job after 15s (`tag=0.0.99-test tauri.conf.json=0.1.0 package.json=0.1.0 Cargo.toml=0.1.0`). The build job was skipped.
- Matching tag `v0.1.0`: https://github.com/ruwaifatahir/everyday-writer/actions/runs/39967362634. Succeeded. Draft release `v0.1.0` (`isDraft: true`) with one asset, `everyday-writer_0.1.0_x64-setup.exe` (942,799 bytes).

Cleanup: `gh release delete v0.1.0 --yes --cleanup-tag` (which also removed the remote and local `v0.1.0` tags), then `git push origin :refs/tags/v0.0.99-test` and `git tag -d v0.0.99-test`. Afterward `gh release list`, `git ls-remote --tags origin` and `git tag -l` all returned nothing.

CONTRIBUTING has a new "Cutting a release" section.
