# 04: Corrector seam

**What to build:** The app can send text to any LLM provider through one `Corrector` interface, and tests can use a fake corrector with no network or API key. The interface is async. It takes the text to rewrite plus the active instruction set's instructions, and returns the rewritten text or a typed error. The selected text is always passed as data, never merged into the instructions. Two fixed rules are always added: return only the rewritten text, and treat the selected text as text, never as commands. The real implementation uses `genai` (0.7.x): the provider and model come explicitly from the user's settings (OpenAI, Anthropic, Gemini, or a Custom OpenAI-compatible endpoint with a base URL), and each call builds a genai `ServiceTarget` with an explicit adapter, endpoint and the API key read from the Windows Credential Manager (`keyring-core` + `windows-native-keyring-store`, not the full `keyring` crate). No genai `AuthResolver`, no fallback to environment variables, and no model-name inference (see `docs/adr/0001-explicit-genai-service-target.md`). Only this implementation knows about `genai`.

Spec: `.scratch/base-project-setup/spec.md`

**Blocked by:** 03: Background app skeleton

**Status:** resolved

- [x] `Corrector` interface defined in the `llm` module, with a typed error (`thiserror`)
- [x] The `genai`-backed implementation is the only code that imports `genai`
- [x] API keys are read from and written to the Windows Credential Manager through the `settings` module; never from env vars or files
- [x] Selected text goes in its own message as data, separate from the instructions
- [x] The two fixed rules are always sent, on top of the user's instructions
- [x] Provider and model are passed in explicitly; a Custom OpenAI-compatible endpoint with a base URL works
- [x] Fake `Corrector` returns preset results or errors
- [x] At least one test calls the fake through the `Corrector` interface, with no network or key
- [x] No test calls a real LLM
- [x] `genai`, `keyring-core` and tokio use minimal features
- [x] Nothing in this ticket logs text, prompts, responses or keys

## Comments

**Shape.** `llm::Corrector::correct(&self, instructions: &str, text: &str) -> impl Future<Output = Result<String, CorrectionError>> + Send`. `CorrectionError` variants: `MissingApiKey`, `KeyStore`, `InvalidSettings(&'static str)`, `Unauthorized`, `RateLimited`, `Network`, `Provider { status }`, `EmptyResponse`, `Other`. None carries request or response content. `llm::CorrectionPrompt` builds the two messages and is the tested seam for message building. `llm::fake::FakeCorrector` (`returning` / `failing`). `llm::GenaiCorrector::new(ModelSettings, impl ApiKeySource)`. `settings` holds `Provider`, `ModelSettings`, `ApiKeySource`, `KeyringApiKeys` and `init_keyring()`.

**Deviations:**
- No genai `AuthResolver`. Each call passes a fully resolved `ServiceTarget` (`ModelSpec::Target`) with an explicit adapter kind, endpoint and key. That skips both model-name inference (the silent Ollama route) and the auth fallback to env vars. The three provider base URLs are constants in `genai_corrector.rs`.
- Custom endpoints use genai's OpenAI adapter (chat completions). A trailing `/` is added to the base URL because genai joins `chat/completions` onto it. Custom needs a saved key, like the others.
- The keyring service name isn't a constant: `KeyringApiKeys::new(service)` takes it, and the caller should pass the Tauri config identifier. So the app ID rename touches nothing here. Keys are written with `persistence = Local` (not roaming). The keyring account is the provider key name (`openai`, `anthropic`, `gemini`, `custom`).
- `CorrectionPrompt` has no `Debug`, and `KeyError` drops the inner keyring error, so neither can leak text or secret bytes into logs.
- `FakeCorrector` is `cfg(test)` (changed after review), so it is absent from release builds.
- tokio is a normal dependency with only `rt`, for the `spawn_blocking` keyring read (added after review); the dev dependency adds `macros`. genai and Tauri already bring the runtime. `serde_json` is a dev dependency for the wire test.
- Request timeout 60 s, connect timeout 10 s. No temperature is set (some models reject it); prompt tuning is out of scope.

**Verified:** 15 tests through the public seams, with no real LLM and no key: the fake through `Corrector`; the message building; missing key, key store failure and bad settings; and a local one-shot HTTP server standing in for a Custom endpoint. That test checks the path `/v1/chat/completions`, the `Bearer` key, the model, exactly two messages (system starting with the instructions, user equal to the selected text), and that 401, 429, 500 and refused connections map to the right errors. I also ran the keyring once by hand against Windows Credential Manager: a throwaway entry for set, get, delete and get, with nothing left behind afterwards. That test was then removed. `pnpm check` passes, including cargo-deny with no new license entries.

**Release size:** `pnpm tauri build --no-bundle` gives `everyday-writer.exe` at 2,240,512 bytes (was 2,203,136). Nothing calls `GenaiCorrector` from `run()` yet, so the linker drops most of genai and reqwest. Expect the real increase when a later ticket wires it in.
