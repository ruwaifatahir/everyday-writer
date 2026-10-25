# 03: README and glossary

**What to build:** Users can tell from the README which installer to download, and the project language has a word for "the installer for one kind of PC". See the spec, user stories 5 and 24, and Implementation Decisions "README" and "GLOSSARY".

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] The README install line names both installers: x64 for most PCs, ARM64 for ARM laptops like Snapdragon. No other README changes
- [x] `GLOSSARY.md` gains **Build**: "The installer for one kind of PC, x64 or ARM64. Every release publishes one Build for each." _Avoid_: version, edition, flavour
- [x] CHANGELOG `## Unreleased` notes the new ARM64 Build
