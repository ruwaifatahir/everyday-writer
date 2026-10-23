# 04: Corrector seam

**What to build:** The app can send text to any LLM provider through one `Corrector` interface, and tests can use a fake corrector with no network or API key. The interface is async. It takes the text to correct plus the user's writing rules, and returns the corrected text or a typed error. The selected text is always passed as data, never merged into the instructions. The real implementation uses `genai` (0.7.x): the provider comes from the model name or a provider namespace, and API keys come from an auth resolver backed by the Windows Credential Manager (`keyring-core` + `windows-native-keyring-store`, not the full `keyring` crate). Only this implementation knows about `genai`.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 03: Background app skeleton

**Status:** ready-for-agent

- [ ] `Corrector` interface defined in the `llm` module, with a typed error (`thiserror`)
- [ ] The `genai`-backed implementation is the only code that imports `genai`
- [ ] API keys are read from and written to the Windows Credential Manager through the `settings` module; never from env vars or files
- [ ] Selected text goes in its own message as data, separate from the instructions
- [ ] Fake `Corrector` returns preset results or errors
- [ ] At least one test calls the fake through the `Corrector` interface, with no network or key
- [ ] No test calls a real LLM
- [ ] `genai`, `keyring-core` and tokio use minimal features
- [ ] Nothing in this ticket logs text, prompts, responses or keys
