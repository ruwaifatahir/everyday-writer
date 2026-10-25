# 04: Release v1.4.0

**What to build:** The first published release with both Builds. See the spec, Implementation Decisions "Version", and Testing Decisions.

**Blocked by:** 01, 02, 03

**Status:** ready-for-agent

- [ ] The version is bumped to 1.4.0 everywhere the tag-version check reads it, and CHANGELOG `## Unreleased` becomes `## 1.4.0`
- [ ] The `v1.4.0` tag run passes, including the release check
- [ ] Before publishing, the maintainer confirms that the draft lists both installers, both `.sig` files and `latest.json`
- [ ] An installed x64 copy of v1.3.1 is offered v1.4.0 and updates to the x64 Build
- [ ] The draft is published
- [ ] The maintainer adds `check-arm64` as a required status check on `main`
