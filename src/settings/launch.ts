// "Shortcut and startup" section. Both settings apply right away, through
// their own commands, and never through the Save button.
//
// The shortcut field takes a pressed combination or typed text. Typing is
// needed because Windows swallows a combination another app has registered,
// so the field never sees it pressed.

import {
  errorMessage,
  isShortcutWorking,
  pauseShortcut,
  resumeShortcut,
  type Settings,
  setShortcut,
  setStartWithWindows,
} from "./api";

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const shortcutInput = element<HTMLInputElement>("shortcut");
const shortcutApply = element<HTMLButtonElement>("shortcut-apply");
const shortcutStatus = element<HTMLParagraphElement>("shortcut-status");
const startSwitch = element<HTMLInputElement>("start-with-windows");
const startStatus = element<HTMLParagraphElement>("startup-status");

/** The shortcut that works now, as saved (`Super` for the Windows key). */
let savedShortcut = "";

const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "ShiftLeft",
  "ShiftRight",
  "MetaLeft",
  "MetaRight",
]);

const EDITING_CODES = new Set(["KeyA", "KeyC", "KeyV", "KeyX", "KeyY", "KeyZ"]);

/** `Super` is what Rust parses; people know the key as Win. */
function toDisplay(shortcut: string): string {
  return shortcut.replace(/\bsuper\b/gi, "Win");
}

function toSaved(shortcut: string): string {
  return shortcut
    .split("+")
    .map((part) => part.trim())
    .map((part) => (/^(win|windows|meta)$/i.test(part) ? "Super" : part))
    .join("+");
}

/** The key's name as the shortcut parser knows it, from its layout-free code. */
function keyName(code: string): string {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit\d$/.test(code)) return code.slice(5);
  return code;
}

/** Ctrl+A, C, V, X, Y, Z and Ctrl+Shift+Z: select all, clipboard, undo, redo. */
function isEditingKey(event: KeyboardEvent): boolean {
  if (!event.ctrlKey || event.altKey || event.metaKey) return false;
  if (event.shiftKey) return event.code === "KeyZ";
  return EDITING_CODES.has(event.code);
}

/** A pressed combination, or undefined while only modifiers are down. */
function combination(event: KeyboardEvent): string | undefined {
  if (MODIFIER_CODES.has(event.code)) return undefined;
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Win");
  parts.push(keyName(event.code));
  return parts.join("+");
}

function show(
  target: HTMLParagraphElement,
  message: string,
  kind: "ok" | "error",
): void {
  target.textContent = message;
  target.dataset.kind = kind;
}

function renderShortcut(): void {
  const changed = toSaved(shortcutInput.value) !== savedShortcut;
  shortcutApply.hidden = !changed;
}

function resetShortcut(): void {
  shortcutInput.value = toDisplay(savedShortcut);
  shortcutInput.removeAttribute("aria-invalid");
  renderShortcut();
}

async function applyShortcut(): Promise<void> {
  const wanted = toSaved(shortcutInput.value);
  if (wanted === savedShortcut) return;
  shortcutApply.disabled = true;
  try {
    await setShortcut(wanted);
    savedShortcut = wanted;
    resetShortcut();
    show(shortcutStatus, "Shortcut changed.", "ok");
  } catch (error) {
    // The previous shortcut still works, so show it again.
    resetShortcut();
    show(shortcutStatus, errorMessage(error, "change the shortcut"), "error");
  } finally {
    shortcutApply.disabled = false;
  }
}

shortcutInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    // Applies the shortcut instead of submitting the provider form.
    event.preventDefault();
    void applyShortcut();
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    resetShortcut();
    // Ends recording; blur brings the saved shortcut back.
    shortcutInput.blur();
    return;
  }
  // A combination with Ctrl, Alt or Win is recorded; plain keys type, and
  // so do the editing keys, so the typed text can be edited.
  if (!(event.ctrlKey || event.altKey || event.metaKey)) return;
  if (isEditingKey(event)) return;
  const pressed = combination(event);
  if (!pressed) return;
  event.preventDefault();
  shortcutInput.value = pressed;
  shortcutStatus.textContent = "";
  renderShortcut();
});

// While the field has focus the saved shortcut is paused, so pressing it
// records it instead of starting a rewrite. Rust also resumes it when the
// window loses focus or closes, in case a resume here never arrives.
// Calls run one at a time: async commands can otherwise reach Rust out of
// order, and a quick focus then blur would leave the shortcut paused.
let pausing: Promise<void> = Promise.resolve();

function inTurn(call: () => Promise<void>): void {
  pausing = pausing.then(call).catch(() => {
    // A failed pause keeps the shortcut working, so pressing it won't
    // record. A failed resume is retried by Rust when the window loses
    // focus or closes.
  });
}

shortcutInput.addEventListener("focus", () => inTurn(pauseShortcut));

function endRecording(): void {
  inTurn(resumeShortcut);
}

shortcutInput.addEventListener("blur", endRecording);

document.addEventListener("visibilitychange", () => {
  if (document.hidden) endRecording();
});

shortcutInput.addEventListener("input", () => {
  shortcutStatus.textContent = "";
  renderShortcut();
});

shortcutApply.addEventListener("click", () => {
  void applyShortcut();
});

startSwitch.addEventListener("change", async () => {
  const enabled = startSwitch.checked;
  startSwitch.disabled = true;
  startStatus.textContent = "";
  try {
    await setStartWithWindows(enabled);
  } catch (error) {
    startSwitch.checked = !enabled;
    show(
      startStatus,
      errorMessage(error, "change start with Windows"),
      "error",
    );
  } finally {
    startSwitch.disabled = false;
  }
});

/** Shows the saved values, and warns if the shortcut failed at startup. */
export async function initLaunch(
  settings: Settings | undefined,
): Promise<void> {
  if (settings) {
    savedShortcut = settings.shortcut;
    startSwitch.checked = settings.startWithWindows;
  }
  resetShortcut();
  let working = true;
  try {
    working = await isShortcutWorking();
  } catch {
    // Unknown: say nothing rather than raise a false alarm.
  }
  if (!working) {
    shortcutInput.setAttribute("aria-invalid", "true");
    show(
      shortcutStatus,
      "This shortcut isn't working: another app is using it. Pick a new one.",
      "error",
    );
  }
}
