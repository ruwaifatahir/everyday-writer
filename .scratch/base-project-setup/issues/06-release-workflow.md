# 06: Release workflow

**What to build:** The developer pushes a version tag and GitHub builds the Windows NSIS installer, then attaches it to a draft GitHub Release for review before it goes public. Use the official `tauri-action`. Auto-updater and code signing are not set up yet.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 05: GitHub repo and CI

**Status:** ready-for-agent

- [ ] Workflow triggers on `v*` tags only
- [ ] Builds on a Windows runner with the official `tauri-action`, pinned to a commit SHA
- [ ] Creates a draft release named after the tag, with the NSIS `.exe` attached
- [ ] App version in the manifests matches the tag; the workflow fails if they differ
- [ ] Workflow token has only the permissions needed to create the release
- [ ] A test tag produces a draft release with the installer; the test release and tag are deleted afterward
- [ ] CONTRIBUTING documents how to cut a release
