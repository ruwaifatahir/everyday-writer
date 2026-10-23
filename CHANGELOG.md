# Changelog

## 1.0.0 - 2026-10-24

First release.

### Features
- Select text in any app and press `Ctrl+Shift+Space` to rewrite it. A popup near the mouse shows the result with **Replace** (Enter), **Copy** and **✕** (Esc).
- Replace puts the result into the original app in place of the selection. If that window has closed, the result is copied instead.
- Your clipboard is restored after every rewrite, including text, rich text, HTML, images and copied files. The app's own clipboard writes stay out of Win+V history and cloud clipboard.
- Works with any keyboard layout, including Arabic and Urdu.
- Providers: OpenAI, Anthropic, Gemini, OpenRouter, Ollama, and Custom (any OpenAI-compatible URL, such as LM Studio). Pick a model from the provider's list or type any name.
- API keys are stored in the Windows Credential Manager, one per provider. A key is optional for Ollama and Custom.
- Instruction sets: four starter templates (Fix & make readable, Fix mistakes only, Professional, Shorter), plus your own. Add, edit, delete, and restore the starters.
- Tray menu with an **Instruction set** submenu to switch the active set, plus Settings and Quit.
- Change the shortcut in Settings. The app tells you if another app already uses it.
- Starts with Windows by default (can be turned off). Launching it again opens Settings.
- Short, friendly messages for no selection, too-long selections (over 5,000 characters), missing setup, no internet, rejected keys, rate limits and timeouts (30 seconds).

### Privacy
- Your text goes only to the provider you chose. Nothing is logged, stored or tracked.
