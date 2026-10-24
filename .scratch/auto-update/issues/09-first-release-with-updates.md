# 09: First release with updates

**What to build:** Everything needed to ship the first version that can update itself, with the real update path proven before any user gets it. See the spec, user stories 41–45, and "Further Notes".

**Blocked by:** 01, 02, 03, 04, 05, 06, 07, 08

**Status:** resolved

- [x] `docs/manual-test-checklist.md` gains the update steps from the spec's Testing Decisions (older build → test release, notification, click, Update now, restart, "Updated to", data kept, unsaved-changes prompt, mid-rewrite wait, offline, tampered signature)
- [x] `CONTRIBUTING.md` release steps include "update an installed older build to the draft before publishing"
- [x] `docs/PRD.md` and the v1 spec no longer list in-app update notices as out of scope; the PRD describes the update behaviour in user terms
- [x] README explains that the app checks GitHub for updates, what the check sends, how to turn it off, and that v1.0.0 users must install this version by hand once
- [x] The release notes for this version include the same one-time manual update note
- [ ] The whole checklist has been run on the installed NSIS build against a test release, and the result is noted in this ticket's comments

## Comments

**Implementer (ticket/09-first-release-with-updates):**

- Checklist: new "Updates" section, U1–U22, with setup. A draft can't be fetched by an installed copy and `releases/latest` skips pre-releases, so the test publishes a `v1.1.0-rc.N` tag as a **pre-release** and installs an older local build (lower version, `--config` endpoint pointed at `releases/download/<tag>/latest.json`, no updater artifacts so no key needed). Covers the spec's list plus open items from 01 (open at Updates, window closed/open), 03/06 (manual and automatic checks, offline, switch off), 04 (data kept, tampered signature, failed download), 07 (toast click, Action Center), 08 ("Updated to" once, not on first install), 02 (`latest.json` url downloads). Unsaved-changes stays in S12–S16. T1, S10 and the results table updated; V2 checks update log lines.
- CONTRIBUTING: release step 6 runs the update test before publishing; step 1 renames `## Unreleased` in CHANGELOG; the dry-run note allows publishing as pre-release.
- Privacy wording checked against `tauri-plugin-updater` 2.14.0: a GET to the fixed URL, user agent `tauri-plugin-updater/2.14.0` (the plugin's version, not the app's), no query or template variables, no custom headers. Docs say: no ID, no app version, nothing from settings or text; GitHub sees the IP.
- PRD: "Staying up to date" section, tray/Settings lines, privacy line; removed from Not Included. v1 spec: removed from Out of Scope with a pointer note.
- README "Updates" section and CHANGELOG `## Unreleased` carry the one-time manual update note for 1.0.0.
- Open (maintainer): run the whole checklist on the installed NSIS build against a test pre-release and record it here; the last box stays unticked. Also unverified: that installing the new installer by hand over 1.0.0 keeps settings and key (expected, same per-user install and identifier).
