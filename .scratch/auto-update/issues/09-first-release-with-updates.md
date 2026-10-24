# 09: First release with updates

**What to build:** Everything needed to ship the first version that can update itself, with the real update path proven before any user gets it. See the spec, user stories 41–45, and "Further Notes".

**Blocked by:** 01, 02, 03, 04, 05, 06, 07, 08

**Status:** ready-for-agent

- [ ] `docs/manual-test-checklist.md` gains the update steps from the spec's Testing Decisions (older build → test release, notification, click, Update now, restart, "Updated to", data kept, unsaved-changes prompt, mid-rewrite wait, offline, tampered signature)
- [ ] `CONTRIBUTING.md` release steps include "update an installed older build to the draft before publishing"
- [ ] `docs/PRD.md` and the v1 spec no longer list in-app update notices as out of scope; the PRD describes the update behaviour in user terms
- [ ] README explains that the app checks GitHub for updates, what the check sends, how to turn it off, and that v1.0.0 users must install this version by hand once
- [ ] The release notes for this version include the same one-time manual update note
- [ ] The whole checklist has been run on the installed NSIS build against a test release, and the result is noted in this ticket's comments
