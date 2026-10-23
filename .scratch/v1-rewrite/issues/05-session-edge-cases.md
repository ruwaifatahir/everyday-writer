# 05: Session edge cases

**What to build:** The rewrite session shows a short, friendly popup message whenever a rewrite can't go ahead normally:

- **Too long:** a selection over 5,000 characters shows "too long", and nothing is sent.
- **No key:** if the provider isn't ready (it needs a key and has none), the popup shows "Add your API key to get started" with an **Open Settings** button, and nothing is sent.
- **Timeout:** a rewrite with no answer after 30 seconds is cancelled and shows "The AI took too long. Try again."
- **Re-run:** pressing the shortcut while the popup is open cancels any rewrite in progress and starts again on the same selection, without capturing again.
- **Rewrite errors:** each one (no internet, key rejected, rate limited, provider error, empty response, other) maps to one message that carries no request or response content.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 03 (Tracer bullet: shortcut, popup, Copy)

**Status:** resolved

- [x] Session tests cover too long, no key, timeout (with a controllable clock or paused tokio time), re-run while open, and every rewrite error
- [x] The too-long and no-key paths never call the Rewriter
- [ ] Open Settings opens the settings window and closes the popup
- [x] Re-run uses the current active instruction set and never captures again
- [x] The 30-second limit and the 5,000-character limit are constants, not settings

## Comments

Merged into feat/v1-rewrite at d59f366; pnpm check passes (86 Rust tests).

Manual checks still to do in the running app:
- Open Settings button opens the settings window and closes the popup
- Re-run via the real shortcut while the popup is open
- Long error messages wrap properly in the popup
- Real slow-provider timeout (30 s)
