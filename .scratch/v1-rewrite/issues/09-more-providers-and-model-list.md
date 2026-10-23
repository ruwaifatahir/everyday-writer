# 09: More providers and model list

**What to build:** The provider picker offers OpenAI, Anthropic, Gemini, OpenRouter, Ollama and Custom.

- **Endpoints:** OpenRouter and Ollama have their endpoints filled in. Custom accepts any OpenAI-compatible base URL, such as Vercel AI Gateway or LM Studio. There is one Custom provider at a time.
- **Keys:** a key is required for OpenAI, Anthropic, Gemini and OpenRouter. It's optional for Ollama and Custom, so local AI works. A blank key never makes the app look for a key anywhere else (see ADR 0001 and its amendment).
- **Model list:** the model field becomes a dropdown filled from the provider's own model list. If the list can't be fetched, it falls back to a short built-in list per provider. Typing a model name always works.

Spec: `.scratch/v1-rewrite/spec.md`

**Blocked by:** 01 (Rename Corrector to Rewriter), 02 (Minimal settings)

**Status:** ready-for-agent

- [ ] Rewriter tests cover the adapter and endpoint for every provider, including OpenRouter, Ollama and a Custom URL
- [ ] Tests cover the key rules: a blank key is refused for OpenAI, Anthropic, Gemini and OpenRouter, and accepted for Ollama and Custom
- [ ] The settings view reports Ollama and Custom as ready without a key
- [ ] The model list is fetched with the same explicit service target, never environment variables
- [ ] Checked once against a local Ollama and LM Studio with no key; LM Studio's default port confirmed and noted
- [ ] Offline, the dropdown falls back to the built-in list and still accepts a typed name

## Comments

**Implementer (ticket/09-providers):**
- No local Ollama or LM Studio was running during implementation, so the check against real servers is still to do by hand: Ollama at `http://localhost:11434` (no key) and LM Studio as Custom at `http://localhost:1234/v1` with a blank key. Confirm that LM Studio accepts the empty `Authorization: Bearer ` header and that 1234 is its default port, then record the result here.
- Port 1234 and auth being off by default come from LM Studio's docs (https://lmstudio.ai/docs/developer/core/authentication). They are not yet confirmed against a running server.
- The settings window hides the key field for Ollama, because genai's Ollama adapter never sends a key. Custom shows the field as "API key (optional)".
- The model list uses the provider's saved key. A key typed into the field but not yet saved isn't used, so the list refreshes after Save.
- The built-in suggestions in `llm::suggested_models` are a best guess at current model names. Check them before release.
