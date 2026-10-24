# Everyday Writer: Product Requirements

## 1. Problem

- Typos and grammar mistakes slip into everyday writing: emails, chats, docs and posts.
- Quick messages often come out messy or hard to read.
- Fixing them means copying text into ChatGPT, typing instructions and pasting the result back.
- People retype the same instructions every time, like "fix grammar, make it readable, no hyphens".

## 2. Solution

- A quiet helper that runs in the background on Windows.
- Select text anywhere and press one shortcut.
- It rewrites the text using your own instructions, for example fixing mistakes and making it readable.
- A small popup shows the result.
- One click swaps your original text for the new one.

**Example**

Before:

> just saw the videos on X, riots happening all over europe
>
> ae you alright?

After:

> Just saw the videos on X. There are riots happening all over Europe.
>
> Are you alright?

## 3. Key Features

**Using it**

- Works in any app where you can select text.
- One shortcut, which you can change. The default is `Ctrl+Shift+Space`.
- To change it, click the shortcut field in Settings and press the keys. The field only records key presses, and a new shortcut is saved the moment you press it, with no extra button. While the field records, the current shortcut pauses so you can record it again. Esc cancels and keeps the old one.
- The popup opens right away near your mouse, fully on that screen, and shows a small animation while the AI works.
- The popup shows the result exactly as it came back, with three buttons: **Replace** (Enter), **Copy** and **✕** (Esc). The buttons are always the same, even if nothing changed.
- **Replace** puts the result into the original app in place of your selection. If that app was closed in the meantime, the result is copied instead and the popup says so.
- Clicking outside the popup closes it. Closing while the AI works cancels it.
- Where text can't be edited, like a web page, Replace does nothing. Use **Copy** there.
- Pressing the shortcut while the popup is open cancels the current rewrite and tries again on the same selection.
- If nothing is selected, nothing happens and no popup appears.
- If no key is set up yet, the popup says so and offers **Open Settings**.
- Text up to 5,000 characters (about one page). Longer text gets a short "too long" message.
- Problems like no internet, a key that doesn't work or an AI that takes longer than 30 seconds show as a short, friendly message in the popup.
- Your clipboard is put back the way it was.

**Setting it up**

- Bring your own AI: choose OpenAI, Anthropic (Claude), Google Gemini, OpenRouter, Ollama or a Custom provider.
- Custom works with any OpenAI-compatible service, like Vercel AI Gateway or LM Studio. There is one Custom provider at a time.
- Choose your model from the provider's list, or type any model name.
- Add your own key for that provider. Ollama and Custom work without a key, for local AI.
- The first time you open the app, settings open on their own so you can set this up.
- Starts with Windows. You can turn this off.
- If another app already uses the shortcut, Settings says so and keeps the old one.

**App shell**

- Lives in the system tray, out of sight.
- The tray menu has an **Instruction set** submenu with a check by the active one, then **Check for updates**, **Settings** and **Quit**.
- Settings hold the provider, model, key, shortcut, instruction sets, the start-with-Windows switch and an **Updates** section. Nothing more.

**Getting it**

- Download the installer from the project's GitHub page.
- Version 1 isn't signed, so Windows shows a "Windows protected your PC" warning. Click "More info", then "Run anyway". The download page explains this.

**Staying up to date**

- The app checks the project's GitHub page for a new version by itself, about 10 seconds after it starts and then once a day.
- When it finds one, it shows one quiet Windows notification, such as "Everyday Writer 1.1.0 is available. Click to update.", once per version. The tray item **Check for updates** becomes **Update to 1.1.0**.
- Clicking the notification while it's on screen, or the tray item, opens Settings at **Updates**. It shows the version you have, a **Check for updates automatically** switch, a **Check for updates** button and, when there is one, **Update now**.
- A check you start yourself shows its result only in Settings: "You're up to date.", the new version, or "Couldn't check for updates. Check your connection and try again.". A failed automatic check stays silent.
- Nothing installs until you click **Update now**. It shows the download progress, waits for any rewrite or replace to finish, asks first if Settings has unsaved changes, then installs with a small progress window, no questions and no admin prompt. The app restarts by itself and says "Updated to 1.1.0".
- Your settings, instruction sets, key and start-with-Windows choice are kept.
- The app only installs updates signed with the project's own key, and never moves to an older version. If an update fails, the old version keeps working and you can try again.
- Version 1.0.0 can't update itself. People on 1.0.0 install the first version with updates by hand, once.

## 4. Instructions

Instructions are the heart of the app. They decide what the rewrite does.

- Save several instruction sets, for example "Chat", "Email" or "Fix only".
- One set is active at a time, and the shortcut uses it. Switch the active set in settings or from the tray menu.
- Starter templates let you begin instantly:
  - **Fix & make readable** (active by default)
  - **Fix mistakes only** (no rewording)
  - **Professional** (for emails and work messages)
  - **Shorter** (same message, fewer words)
- Edit any template, delete it or write your own from scratch.
- A **Restore starter templates** button brings the starters back, after a confirm. Deleted ones return and edited ones reset to the original. Your own sets are never touched.
- The last set can't be deleted. Deleting the active set makes the first remaining set active.
- There are no limits on what instructions can do. Your instructions decide the style, tone, language and purpose. For example, write in Roman Urdu and get clean English back.
- By default the result stays in the language you wrote in, unless your instructions say otherwise.
- All instruction sets use the same provider and model.
- Two rules always apply, whatever the instructions say:
  - The result is only the rewritten text, with no chatty replies like "Sure! Here's…".
  - The selected text is always treated as text to rewrite, never as commands to follow.

## 5. Privacy & Trust

- The app runs fully on your computer. No server of ours is involved: updates come straight from the project's GitHub page.
- Your text goes only to the AI provider you chose, using your own key.
- The app never stores your text, logs it or sends it anywhere else.
- If you use Windows clipboard history (Win+V), the text you rewrite may appear there.
- Your key is kept safely by Windows, never in a plain file.
- No tracking and no analytics.
- The update check is a plain download of a public file from GitHub. It sends no ID, no app version and nothing about your settings or text. GitHub sees your IP address, as with any website visit. Turn off **Check for updates automatically** and the app only contacts GitHub when you ask it to check or update.

## 6. Not Included (Yet)

- Correcting automatically as you type.
- Editing the result inside the popup.
- Highlighting what changed.
- A saved history of past rewrites.
- A different model for each instruction set.
- A signed installer with no Windows warning (a later goal).
- Mac, Linux or mobile versions.

## 7. Success

- The app feels instant. The popup opens, text gets replaced and menus respond with no waiting on the app's side.
- The only wait is the AI writing its answer, and the popup shows clearly that it's working.
- The app starts with Windows and stays out of sight. You won't notice it in memory, battery or speed.
- Every screen looks modern, minimal and clean, with nothing extra.
- The result follows your instructions and reads like a cleaner version of you, not like a robot.
- Anyone can set it up in a minute and start with a template, with no writing needed.
- Users trust it because their text never goes anywhere except the AI provider they chose.
