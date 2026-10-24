// Makes a window behave like a desktop app rather than a web page: the
// WebView's browser keys (reload, print, find, view source, history) and its
// default context menu do nothing. Editing keys (Ctrl+A/C/V/X/Z/Y) and the
// context menu in text fields keep working. Both windows import this once.

/** A browser accelerator, matched on the physical key. */
interface Accelerator {
  code: string;
  ctrl?: boolean;
  shift?: boolean;
}

const ALWAYS: Accelerator[] = [
  { code: "KeyR", ctrl: true }, // reload
  { code: "KeyR", ctrl: true, shift: true }, // hard reload
  { code: "KeyP", ctrl: true }, // print
  { code: "KeyF", ctrl: true }, // find
  { code: "KeyG", ctrl: true }, // find next
  { code: "KeyG", ctrl: true, shift: true }, // find previous
  { code: "KeyU", ctrl: true }, // view source
  { code: "KeyS", ctrl: true }, // save page
];

/** Developer tools stay reachable in `pnpm tauri dev`. */
const RELEASE_ONLY: Accelerator[] = [
  { code: "KeyI", ctrl: true, shift: true },
  { code: "KeyJ", ctrl: true, shift: true },
  { code: "KeyC", ctrl: true, shift: true },
];

/** Keys blocked with any modifiers. */
const ANY_MODIFIERS = new Set([
  "F3", // find next
  "F5", // reload
  "BrowserBack",
  "BrowserForward",
  "BrowserRefresh",
  "BrowserSearch",
]);

const release = import.meta.env.PROD;
const blocked = release ? [...ALWAYS, ...RELEASE_ONLY] : ALWAYS;

function isBrowserKey(event: KeyboardEvent): boolean {
  if (ANY_MODIFIERS.has(event.code)) return true;
  if (release && event.code === "F12") return true;
  // Alt+Left and Alt+Right are history back and forward.
  if (
    event.altKey &&
    !event.ctrlKey &&
    (event.code === "ArrowLeft" || event.code === "ArrowRight")
  ) {
    return true;
  }
  if (event.altKey || event.metaKey) return false;
  return blocked.some(
    (key) =>
      key.code === event.code &&
      Boolean(key.ctrl) === event.ctrlKey &&
      Boolean(key.shift) === event.shiftKey,
  );
}

/** Where the WebView's own menu (cut, copy, paste) is useful. */
const KEEPS_CONTEXT_MENU = "#result, input, textarea";

// Capture phase on window, so nothing on the page sees these first. Only the
// browser's default is stopped: handlers such as the shortcut field still get
// the event and can record the combination.
window.addEventListener(
  "keydown",
  (event) => {
    if (isBrowserKey(event)) event.preventDefault();
  },
  { capture: true },
);

window.addEventListener(
  "contextmenu",
  (event) => {
    const target = event.target;
    if (target instanceof Element && target.closest(KEEPS_CONTEXT_MENU)) {
      return;
    }
    event.preventDefault();
  },
  { capture: true },
);

// The mouse's back and forward buttons.
window.addEventListener(
  "mouseup",
  (event) => {
    if (event.button === 3 || event.button === 4) event.preventDefault();
  },
  { capture: true },
);
