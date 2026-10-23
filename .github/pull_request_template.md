## What and why

<!-- What does this change do, and why? Link the issue or ticket. -->

## Checklist

- [ ] `pnpm check` passes locally
- [ ] PR title and commits follow Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:` ...)
- [ ] Tests added or updated for changed behavior (no real LLM calls, no API keys)
- [ ] No selected text or clipboard is logged
- [ ] Selected text is passed to the LLM as data, never as instructions
- [ ] No secrets committed; API keys stay in the OS keyring
- [ ] New dependencies use minimal features and pass `cargo deny check`
