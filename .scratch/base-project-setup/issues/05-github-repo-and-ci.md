# 05: GitHub repo and CI

**What to build:** The project is public on GitHub at `ruwaifacodes/everyday-writer`. Every pull request runs the full quality gate in CI on a Windows runner, and `main` only accepts PRs with green CI. Dependabot opens weekly update PRs. Contributors find a README, CONTRIBUTING guide, SECURITY policy and PR template. The agent creates the repo with `gh`; the user has approved creating it publicly.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 02: Local quality gate

**Status:** ready-for-agent

- [ ] Public repo `ruwaifacodes/everyday-writer` created with `gh` and the local history pushed
- [ ] CI workflow on pull requests and pushes to `main`: fmt check, clippy `-D warnings`, `cargo test`, `cargo deny check`, Biome check, `tsc --noEmit`
- [ ] CI caches Rust and pnpm dependencies
- [ ] All third-party actions pinned to commit SHAs; workflow token permissions set to minimum
- [ ] Dependabot configured weekly for `cargo`, `npm` and `github-actions`
- [ ] `main` protected: PR required, CI checks required, no force pushes
- [ ] README: what the app does, the PRD in one paragraph, prerequisites, dev and build commands
- [ ] CONTRIBUTING: branch workflow, Conventional Commits, how to run checks
- [ ] SECURITY: private reporting through GitHub security advisories
- [ ] PR template with a checklist that includes "no selected text or clipboard is logged"
- [ ] A test PR shows green CI; a direct push to `main` is rejected
