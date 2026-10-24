---
name: everyday-writer
description: Background Windows tray app that rewrites selected text. One cream theme after Wispr Flow's design system, scaled down to a 400px popup and a 440px settings window.
colors:
  cream: "#ffffeb"
  ink: "#1a1a1a"
  stone: "#e4e4d0"
  fog: "#8a8a80"
  ink-soft: "#5f5f56"
  lavender: "#f0d7ff"
  lavender-press: "#e4c2fa"
  forest: "#034f46"
  ember: "#ffa946"
  ember-ink: "#8a3d00"
typography:
  ui: { fontFamily: "Figtree", weights: [400, 500, 600] }
  serif: { fontFamily: "EB Garamond", weight: 400 }
  micro: { fontSize: "12px" }
  caption: { fontSize: "13px", lineHeight: "18px" }
  body: { fontSize: "15px", lineHeight: 1.4 }
  heading: { fontFamily: "EB Garamond", fontSize: "24px", lineHeight: 1.1 }
  title: { fontFamily: "EB Garamond", fontSize: "40px", lineHeight: 1, letterSpacing: "-0.02em" }
rounded:
  chip: "6px"
  control: "12px"
  card: "18px"
  window: "8px"
  pill: "999px"
border: "2px"
spacing: "4px base: 2 4 6 8 10 12 16 20 24 32 40"
---

# everyday-writer design

The look is taken from **Wispr Flow** (wisprflow.ai): a warm cream page,
near-black ink, 2px ink borders on everything you can press, lavender as the
one action color, a Garamond serif for headings over a Figtree UI. Wispr is
a marketing site at web scale; this app is two small utility windows, so its
character is kept and its sizes are cut down.

One theme: cream. No dark mode, no per-window variants.

## Files

- `src/styles/fonts.css`: Figtree (variable, latin) and EB Garamond 400
  (latin), bundled from `@fontsource-variable/figtree` and
  `@fontsource/eb-garamond`. About 44 KB of woff2. CSP is `font-src 'self'`;
  never load fonts from a CDN.
- `src/styles/tokens.css`: every color, size, radius and duration.
- `src/styles/base.css`: reset, document defaults, serif headings,
  selection, no-select chrome, scrollbars, fade-in on `body[data-ready]`,
  reduced motion.
- `src/styles/components.css`: buttons, `kbd`, `.badge`, icon buttons,
  inputs, switch, radio, `.panel` / `.panel-foot`, text roles.
- `src/popup/popup.css` and `src/settings/settings.css`: layout only.

Both windows paint `--cream` natively (`BACKGROUND` in `popup.rs` and
`settings_window.rs`) and use the light Windows theme, so there is no flash
and the Settings title bar matches. Change both sides together.

## Color

| Token | Use |
|---|---|
| `cream` | Every surface: page, cards, inputs, secondary buttons |
| `ink` | Text, every 2px border, `kbd` fill, focus ring |
| `stone` | Card outlines, row rules, segmented track |
| `ink-soft` | Hints and secondary text (6.4:1 on cream) |
| `fog` | Non-text: the off switch knob; the reading band's resting text |
| `lavender` | Primary buttons and the on switch. Nothing else |
| `forest` | Success: ok status, saved key |
| `ember` | Active: the active instruction set, a confirming delete |
| `ember-ink` | Error text, invalid borders, danger buttons |

Wispr's Fog (#8a8a80) is 3.4:1 on cream, too light for small text, so text
uses `ink-soft` instead. There is no blue, no red, and no gradient but the
popup's reading band.

## Type

Figtree for all UI, body 15px, labels and buttons 500, hints 13px.
EB Garamond at weight 400 only for headings: page title 40px, section
headings and dialog titles 24px, and the popup's message line (20px). It
leads by size, never by bold.

## Geometry

Wispr's 32–80px card radii and 20px body are web scale. Here: buttons and
inputs 12px radius (Wispr's own value), cards and dialogs 18px, badges and
switches full pills, `kbd` 6px, the segmented track 16px (12px plus its
4px inset). Every border is 2px. Controls are 36px tall,
quiet buttons 30px, inputs 40px.

## Surfaces and depth

Flat. No `box-shadow` anywhere, including dialogs. Separation is a 2px
border or a fill change:

- **Card** (`.panel`): cream, 2px `stone` outline, 18px radius. Controls
  inside carry 2px `ink`, so the card recedes and the controls lead.
  `.panel-foot` closes a card with a 2px stone rule and right-aligned
  actions. `.rows` splits a card with stone rules.
- **Dark chamber**: `ink` fill is for small contained things only (`kbd`
  on the primary button). It is never a theme or a section fill.
- **Popup**: the window draws no border of its own. Windows 11 gives the
  frameless window its rounded corners, hairline edge and shadow, which
  hold its edge against any desktop.

## Components

- **Primary** button: lavender, ink border, ink text. One per view.
- **Secondary** button: cream, ink border. Disabled primary and secondary
  drop to a stone outline with `ink-soft` text, and a disabled `kbd` is a
  stone chip with `ink-soft` text: legible, plainly not ready, never a
  faded color. A disabled quiet button is `ink-soft` too.
- **Quiet** button: no border, underline on hover; `.danger` is ember-ink;
  `.confirming` turns it into an ember pill with an ink border.
- **Badge** (`.badge`, `.set-badge`): full pill, 13px/500.
  Forest with cream text for done, ember with ink text for active, stone
  with ink text for neutral.
- **Reading band** (`.working`, popup only): the selection in `fog`, a
  wide band of `ink` sweeping across it every 1.6s after a 0.4s wait.
  Plain `ink-soft` under reduced motion. It replaces any spinner or
  skeleton.
- **Segmented** picker: stone track, the chosen option a cream tile with
  the ink border.
- **Switch**: cream track with ink border and fog knob; on is lavender with
  an ink knob. **Radio**: ink ring, ink dot.
- **kbd**: ink chip, cream text. The `Esc` hint in the popup footer uses a
  stone chip so only the Enter key on Replace is dark.
- **Focus**: one rule, a 2px ink outline offset 2px, on every control. A
  real outline, so Windows high contrast still draws it. The popup's
  result fades at its foot, which would clip an outline, so its ring is a
  2px ink border drawn around the text area instead.
- **Field errors**: under the field, 13px/500 `ember-ink`, named by the
  field's `aria-describedby`; the field border turns `ember-ink`. The
  footer status line is for save results, not validation.

## Popup states

- Working: a plain "Rewriting…" caption over the selection itself, set
  like the result, with the reading band. Copy and Replace disabled.
- Result: a plain "Rewritten" caption, the text in Figtree 15px fading in
  where the selection was, Replace focused. Long text fades into the
  footer rule and scrolls.
- Message: a plain "Not rewritten" caption, the message in Garamond 20px, and
  an outlined Open Settings button at the compact 30px size when there is
  something to fix, so a two-line message still clears the footer.

## Motion

120ms for state changes, 220ms for entrances, exponential ease-out. The
popup result fades in over 220ms where the selection was. Popup
messages rise 4px as they appear, and so do the Settings
parts that swap in (the saved-key row, the key entry, the base URL field,
status and error lines). The Active badge fades in from 95% over 120ms.
Dialogs scale in from 98% over 220ms while the scrim fades in; they leave
over 120ms, fading and dropping 4px (`transition-behavior: allow-discrete`
with `@starting-style`). The popup window itself opens and closes
instantly.

Under reduced motion nothing travels: every entrance becomes a 120ms
opacity fade, buttons don't press down, and the reading band stops.

## Native behavior

Both windows act like desktop windows, not web pages
(`src/shared/native-feel.ts`): browser keys (reload, print, find, view
source, history, and dev tools in release builds) do nothing, and the
WebView menu only opens on text fields and the popup result. Chrome text
can't be selected and the pointer stays an arrow everywhere, buttons
included; inputs, the result, and status and error lines stay selectable.
Scrollbars are the thin system bar in `GrayText`. Settings closes on Esc or
Ctrl+W, asking first about unsaved edits.

## Do

- Keep lavender for the single primary action in a view.
- Put 2px ink borders on interactive things and 2px stone on containers.
- Use Garamond 400 only for headings and the popup message.

## Don't

- Add shadows, gradients (beyond the reading band), blur, or a second
  accent color.
- Add a status dot, spinner, skeleton or status pill; say the state in
  plain caption text and let the reading band show the work.
- Use radii under 12px on buttons, inputs or cards (only `kbd` is smaller).
- Use `fog` for text (the reading band is the one exception), or put
  cream content on cream without a border.
