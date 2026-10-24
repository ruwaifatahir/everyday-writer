# Changelog

## 1.1.0 - 2026-10-24

**Updating from 1.0.0:** version 1.0.0 can't update itself. Download this version's installer below and run it over your current copy, once. Your settings, instruction sets, key and start-with-Windows choice are kept, and from now on the app updates itself.

### Features
- The app checks GitHub for a new version about 10 seconds after it starts and then once a day. When it finds one, it shows one Windows notification per version, and the tray item **Check for updates** becomes **Update to** the new version.
- Settings has a new **Updates** section: the installed version, a **Check for updates automatically** switch, a **Check for updates** button and **Update now**. Clicking the notification while it's on screen, or the tray item, opens it.
- **Update now** shows the download progress, waits for a running rewrite or replace to finish, asks to save unsaved Settings changes first, then installs with a small progress window and no admin prompt. The app restarts and says "Updated to" the new version. Your settings, instruction sets, key and start-with-Windows choice are kept.
- Updates are only accepted when signed with the project's key, and never move to an older version. If an update fails, the old version keeps working and you can try again.

### Privacy
- The update check downloads a public file from GitHub. It sends no ID, no app version and nothing about your settings or text; GitHub sees your IP address, as with any website visit. Turn off **Check for updates automatically** and the app contacts GitHub only when you click **Check for updates** or **Update now**.

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
