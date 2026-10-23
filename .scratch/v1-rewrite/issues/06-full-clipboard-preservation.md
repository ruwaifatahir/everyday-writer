# 06: Full clipboard preservation and manual test checklist

**What to build:** Whatever the user had copied before is still there after capture and after Replace.

- **Formats saved and restored:** every memory-based format, which covers text, rich text, HTML, file lists, DIB/PNG images and registered formats.
- **Formats skipped:** GDI-handle formats (bitmap handles, metafiles, palettes) and formats Windows synthesizes. This is an accepted gap.
- **History and cloud:** every clipboard write the app makes is tagged so it stays out of Win+V history and cloud clipboard.
- **Retries:** opening and flushing the clipboard are retried a few times.

This ticket also writes the spec's manual test checklist as a release document and runs it once.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 04 (Replace)

**Status:** resolved

- [ ] An image copied before a rewrite can still be pasted after capture and after Replace
- [ ] Files copied in Explorer can still be pasted after a rewrite
- [ ] Rich text copied from Word keeps its formatting after a rewrite
- [ ] After a rewrite, Win+V history shows no entry for the result or for the restored clipboard
- [ ] A clipboard briefly held by another app doesn't make capture or Replace fail
- [ ] The manual test checklist is in the repo docs, covers every case in the spec, and has been run once with results recorded
- [x] Nothing private is logged

## Comments

Merged in e74bb7f. `pnpm check` passes. No clipboard or selected text is logged (clipboard code has no tracing calls). Still to verify on a real desktop, left unticked:

- Image, Explorer files and Word rich text survive a rewrite (capture and Replace)
- Win+V history shows no entry for the result or the restored clipboard
- A clipboard briefly held by another app does not make capture or Replace fail
- The manual test checklist (docs/manual-test-checklist.md) has been run once with results recorded
