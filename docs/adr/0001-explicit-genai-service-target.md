# Explicit genai ServiceTarget per call, no AuthResolver

The real `Corrector` builds a genai `ServiceTarget` on every call, with an explicit adapter kind, endpoint and the API key read from the Windows Credential Manager. We don't use genai's `AuthResolver`. genai's defaults infer the provider from the model name, and unknown names silently go to a local Ollama; its default auth reads keys from environment variables. Both would send the user's selected text or key somewhere they didn't choose, so the provider always comes from settings and a missing key is an error, never a fallback.
