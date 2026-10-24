# Model names are a built-in list, never fetched from the provider

Settings lists a short built-in set of Suggested Models for each hosted provider (OpenAI, Anthropic, Gemini, OpenRouter), fastest first, and the user can type any other name. Ollama and Custom get a plain text field. We stopped fetching each provider's `/models` list because the raw lists are mostly models that can't rewrite (Gemini returns 50+ including embedding, speech and image models; OpenRouter returns 300+), fetching needed the key and the network just to show names, and the apps we compared (Zed, Raycast, Writing Tools, Chatbox) converge on a curated list with a free-text escape hatch.

## Consequences

- The list only changes with an app Update, so it goes stale as providers ship and retire models. Refresh it in every release; a saved name is never changed automatically.
- Test still lists the provider's models behind the scenes, to check the key and that the typed model exists, so a typo is caught before a rewrite.
