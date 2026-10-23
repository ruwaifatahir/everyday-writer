# 05: GitHub repo and CI

**What to build:** The project is public on GitHub at `ruwaifatahir/everyday-writer`. Every pull request runs the full quality gate in CI on a Windows runner, and `main` only accepts PRs with green CI. Dependabot opens weekly update PRs. Contributors find a README, CONTRIBUTING guide, SECURITY policy and PR template. The agent creates the repo with `gh`; the user has approved creating it publicly.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 02: Local quality gate

**Status:** resolved

- [x] Public repo `ruwaifatahir/everyday-writer` created with `gh` and the local history pushed
- [x] CI workflow on pull requests and pushes to `main`: fmt check, clippy `-D warnings`, `cargo test`, `cargo deny check`, Biome check, `tsc --noEmit`
- [x] CI caches Rust and pnpm dependencies
- [x] All third-party actions pinned to commit SHAs; workflow token permissions set to minimum
- [x] Dependabot configured weekly for `cargo`, `npm` and `github-actions`
- [x] `main` protected: PR required, CI checks required, no force pushes
- [x] README: what the app does, the PRD in one paragraph, prerequisites, dev and build commands
- [x] CONTRIBUTING: branch workflow, Conventional Commits, how to run checks
- [x] SECURITY: private reporting through GitHub security advisories
- [x] PR template with a checklist that includes "no selected text or clipboard is logged"
- [x] A test PR shows green CI; a direct push to `main` is rejected

## Comments

- Repo: https://github.com/ruwaifatahir/everyday-writer (public). The owner moved from `ruwaifacodes` to `ruwaifatahir` because `ruwaifacodes` is a separate user account, not an org. The app ID is now `com.ruwaifatahir.everydaywriter`.
- Test PR (closed without merging): https://github.com/ruwaifatahir/everyday-writer/pull/1
- Green CI run: https://github.com/ruwaifatahir/everyday-writer/actions/runs/39964713175 (job `check`, windows-latest, 3m39s: rustfmt, clippy, cargo test, cargo-deny, Biome, tsc all passed)
- Branch protection on `main`: required status check `check` (bound to the GitHub Actions app, strict), PR required (0 approvals so a solo maintainer can merge), `enforce_admins: true`, force pushes and deletions off.
- Push rejection: a throwaway commit `b35b177` (parent `53a79e7`, the `main` tip) pushed with `git push origin b35b177:refs/heads/main`, then again with `--force`. Both were rejected:
  ```
  remote: error: GH006: Protected branch update failed for refs/heads/main.
  remote: - Changes must be made through a pull request.
  remote: - Required status check "check" is expected.
   ! [remote rejected] b35b177... -> main (protected branch hook declined)
  ```
  `git ls-remote origin refs/heads/main` showed `53a79e7` before and after, so nothing landed.
