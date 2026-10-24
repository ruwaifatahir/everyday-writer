<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="design/logo/everyday-writer-cream.svg">
    <img src="design/logo/everyday-writer-ink.svg" alt="Everyday Writer logo" width="96">
  </picture>
</p>

<h1 align="center">Everyday Writer</h1>

<p align="center"><strong>Fix your writing in any app, without leaving it.</strong></p>

<p align="center">
  <a href="https://github.com/ruwaifatahir/everyday-writer/releases/latest"><strong>Download for Windows</strong></a>
  ·
  <a href="docs/PRD.md">All features</a>
  <br><br>
  <a href="https://github.com/ruwaifatahir/everyday-writer/actions/workflows/ci.yml"><img src="https://github.com/ruwaifatahir/everyday-writer/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
</p>

Typos and messy sentences slip into emails, chats and docs. Fixing them usually means copying the text into a chatbot, typing the same instructions again and pasting the result back. Everyday Writer does it where you're already typing.

**Before**

> hey cant make it to the meeting tmrw, can we moved it to friday. ill send the report befor then
>
> thx

**After**

> Hey, I can't make it to the meeting tomorrow. Can we move it to Friday? I'll send the report before then.
>
> Thanks

## How it works

1. Select text in any app.
2. Press `Ctrl + Shift + Space` (you can change it).
3. Check the result in the popup and press **Replace** (or Enter).

## Why Everyday Writer

- **Works everywhere.** Any app where you can select text: email, chat, browser, docs.
- **Your instructions.** Tell it once how you want text fixed, like "fix grammar, keep my tone". No retyping.
- **Your choice of AI.** OpenAI, Anthropic, Gemini, OpenRouter, a local Ollama, or any OpenAI-compatible endpoint.
- **Your own key.** You pay your provider directly. There's no account and no subscription.

## Privacy

- Your API key is stored in Windows Credential Manager.
- Your clipboard is put back the way it was.
- Only the rewrite request (to your provider) and the update check leave your machine.
- Windows clipboard history (Win + V) may still show the text you rewrite, because the app you copy from adds it there.

## Install

Download the installer from [Releases](https://github.com/ruwaifatahir/everyday-writer/releases/latest) and run it. It isn't code-signed yet, so Windows SmartScreen may warn on first run.

## Updates

The app checks GitHub for a new version once a day and tells you when there is one. Nothing installs until you click **Update now** in Settings > Updates, and your settings and key are kept. It only accepts updates signed with the project's key. The check sends nothing about you or your text, and you can turn it off in Settings > Updates.

On version 1.0.0? It has no updater, so install the newest version from [Releases](https://github.com/ruwaifatahir/everyday-writer/releases/latest) once, by hand. Later versions update themselves.

## Contributing

Setup, build and checks are in [CONTRIBUTING.md](CONTRIBUTING.md). Report security problems privately as described in [SECURITY.md](SECURITY.md).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
