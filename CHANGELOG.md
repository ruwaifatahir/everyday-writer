# Changelog

## Unreleased

### Features
- The popup has a **Rewrite** button. It runs a new rewrite of the same selection with your active instruction set, so you don't have to select the text again. It sits next to Copy and Replace when a result is shown, and it's the main button (press Enter) when a rewrite fails because the provider was busy, unreachable, too slow or sent back an error.
- The popup shows your instruction sets in a dropdown at the top, with the active one picked. Pick another set there and it becomes the active set everywhere, in the tray and Settings too. Picking a set doesn't rewrite on its own: it stops a rewrite in progress, clears the result and waits with your selection until you press **Rewrite** (or Enter).
- New **Rewrite on open** switch in Settings, under Shortcut and startup. It starts on, so the shortcut rewrites at once as before. Turn it off and the shortcut opens the popup with your selection and sends nothing until you press **Rewrite** (or Enter), so you can pick a set first.

### Changes
- Pressing the shortcut while the popup is open now works like **Rewrite**. While a rewrite is running it does nothing, instead of starting over.

## 1.5.0 - 2026-10-26

### Changes
- The provider card in Settings shows no buttons until you change something. Change the provider, model or base URL, or type a key, and **Discard** and **Save** appear at the bottom of the card. Save shows "Saved." and the buttons go away; Discard puts back what you last saved.
- The API key is one field. When a key is saved, the field says "Key saved. Type a new one to replace it." Type a new key and press **Save** to replace it. **Change key** and **Cancel** are gone.
- **Remove key** shows under the key field only when a key is saved, and removes it straight away. It no longer clears a key you've typed but not saved.
- **Test key** is gone from Settings. A wrong key or model now shows up on your first rewrite, in the popup's error message.
- The Settings window is wider, so the provider card has more room.

### Fixes
- After switching provider or pressing **Discard**, **Remove key** no longer shows for the wrong provider.
- Pressing Enter in Settings with nothing changed no longer does anything.

## 1.4.0 - 2026-10-25

### Features
- Every release now has an ARM64 installer next to the x64 one, so the app runs natively on ARM laptops like Snapdragon. Each installed copy keeps updating to the installer for its own PC.

## 1.3.1 - 2026-10-24

### Changes
- Settings no longer opens as an empty window. It appears once it's fully drawn, already at the section it was opened for, in front and ready to type in.

## 1.3.0 - 2026-10-24

### Features
- The shortcut field in Settings now records key presses. Click it and press the keys you want, and the shortcut is saved straight away. There's no **Use shortcut** button any more.
- While you record, the app's shortcut is paused, so you can record the current one again, such as Ctrl + Shift + Space. It comes back when you leave the field, press Esc or close Settings.
- **Reset to default** brings back Ctrl + Shift + Space in one click. It shows only when you use a different shortcut.
- When a shortcut can't be used, Settings says why and keeps recording: it needs Ctrl or Alt, Win can't be used, or the app needs Ctrl + C and Ctrl + V to copy and paste. Ctrl + Alt shortcuts save, with a note that they can clash with AltGr on some keyboards.

### Changes
- A shortcut with Win saved by an earlier version keeps working until you change it.
- If another app takes your shortcut while it's paused, a notification tells you to pick a new one.
- Keys are written the way the shortcut field shows them, such as "Press Ctrl + V".

## 1.2.0 - 2026-10-24

**The app is now called Everyday Writer.** Whether you update from the app or run the installer, it installs as a new app next to your current copy. Afterwards, uninstall **everyday-writer** from Settings → Apps → Installed apps. Your settings, instruction sets, key and start-with-Windows choice are kept.

### Features
- Settings has a **Provider** picker with logos and a **Model** picker that suggests current models, fastest first. You can still type any model name.
- **Test key** checks your key without spending tokens, and also tells you if the provider doesn't offer the model you typed.
- New installs start on Gemini with `gemini-3.5-flash-lite`. Switching provider picks its first suggested model.
- Settings shows **Unsaved changes** next to Save when the form differs from what you saved.
- While rewriting, the popup shows your selection with a moving shimmer, and the result fades in where it was.

### Changes
- New look: a cream and ink design with bundled fonts, for the popup and Settings. On Windows 11 the Settings title bar matches the page.
- New app icon: a cream "e" on a dark tile, in the tray, title bars, taskbar, installer and Start menu.
- The app shows its name as **Everyday Writer** in the installer, Start menu, Installed apps, tray, windows and notifications. Installed apps also shows the publisher and a short description. Settings shows the version as "Version 1.2.0".
- Errors name the provider and the real cause, such as "Couldn't reach OpenAI". Buttons say what they act on, such as **Test key** and **Delete set**.
- When Replace can't put the result back, the popup says "Copied" and tells you to press Ctrl+V.
- Enter only replaces when the result or **Replace** has focus. Esc and Ctrl+W close Settings and ask about unsaved changes first.
- Better keyboard and screen reader support, and calmer motion when Windows is set to reduce animations.

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
