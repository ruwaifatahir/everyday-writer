# Explicit genai ServiceTarget per call, no AuthResolver

The real `Corrector` builds a genai `ServiceTarget` on every call, with an explicit adapter kind, endpoint and the API key read from the Windows Credential Manager. We don't use genai's `AuthResolver`. genai's defaults infer the provider from the model name, and unknown names silently go to a local Ollama; its default auth reads keys from environment variables. Both would send the user's selected text or key somewhere they didn't choose, so the provider always comes from settings and a missing key is an error, never a fallback.

**Amendment:** a blank key is allowed for Ollama and Custom so local servers work. The user chose that provider and left the key blank on purpose, so it is still an explicit choice: a blank key never triggers a fallback to environment variables or an inferred provider. For every other provider a missing key stays an error.
