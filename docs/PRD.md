# everyday-writer: Product Requirements

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
- The popup opens right away near your mouse and shows a small animation while the AI works.
- The popup shows the clean result with three buttons: **Replace** (Enter), **Copy** and **✕** (Esc).
- Clicking outside the popup closes it. Closing while the AI works cancels it.
- Where text can't be edited, like a web page, the popup offers **Copy** only.
- Pressing the shortcut again cancels the current rewrite and starts over with the new selection.
- If nothing is selected, nothing happens.
- Text up to about one page. Longer text gets a short "too long" message.
- Problems like no internet or a key that doesn't work show as a short, friendly message in the popup.
- Your clipboard is left exactly as it was.

**Setting it up**

- Bring your own AI: choose OpenAI, Anthropic (Claude), Google Gemini or a Custom provider.
- Choose your model from a suggested list, or type any model name.
- Add your own key for that provider.
- The first time you open the app, settings open on their own so you can set this up.
- Starts with Windows. You can turn this off.

**App shell**

- Lives in the system tray, out of sight.
- The tray menu shows the active instruction set, **Settings** and **Quit**.
- Settings hold the provider, model, key, shortcut, instruction sets and the start-with-Windows switch. Nothing more.

**Getting it**

- Download the installer from the project's GitHub page.
- Version 1 isn't signed, so Windows shows a "Windows protected your PC" warning. Click "More info", then "Run anyway". The download page explains this.

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
- A **Restore default templates** button brings the starters back.
- There are no limits on what instructions can do. Your instructions decide the style, tone, language and purpose. For example, write in Roman Urdu and get clean English back.
- By default the result stays in the language you wrote in, unless your instructions say otherwise.
- All instruction sets use the same provider and model.
- Two rules always apply, whatever the instructions say:
  - The result is only the rewritten text, with no chatty replies like "Sure! Here's…".
  - The selected text is always treated as text to rewrite, never as commands to follow.

## 5. Privacy & Trust

- The app runs fully on your computer. No server of ours is involved.
- Your text goes only to the AI provider you chose, using your own key.
- The app never stores your text, logs it or sends it anywhere else.
- Your key is kept safely by Windows, never in a plain file.
- No tracking and no analytics.

## 6. Not Included (Yet)

- Correcting automatically as you type.
- Editing the result inside the popup.
- Highlighting what changed.
- A saved history of past rewrites.
- A different model for each instruction set.
- Update notices inside the app.
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
