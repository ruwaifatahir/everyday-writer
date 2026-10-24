// "Shortcut and startup" section. Both settings apply right away, through
// their own commands, and never through the Save button.
//
// The shortcut field only records key presses. While it has focus it is
// recording: the saved shortcut is paused, held modifiers show live, and a
// press is saved at once. Rust decides whether the press is allowed.

import {
  DEFAULT_SHORTCUT,
  errorMessage,
  isDefaultShortcut,
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
const shortcutReset = element<HTMLButtonElement>("shortcut-reset");
const shortcutStatus = element<HTMLParagraphElement>("shortcut-status");
const startSwitch = element<HTMLInputElement>("start-with-windows");
const startStatus = element<HTMLParagraphElement>("startup-status");

/** The shortcut that works now, as saved (`Super` for the Windows key). */
let savedShortcut = "";

/** A save is on its way to Rust; presses wait for it. */
let saving = false;

/** The field shows the shortcut saved during this recording, not the prompt. */
let showingSaved = false;

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

const PROMPT = "Press a shortcut…";

/** `Super` is what Rust parses; people know the key as Win. */
function toDisplay(shortcut: string): string {
  return shortcut
    .split("+")
    .map((part) => part.trim())
    .map((part) => (/^super$/i.test(part) ? "Win" : part))
    .join(" + ");
}

/** The key's name as the shortcut parser knows it, from its layout-free code. */
function keyName(code: string): string {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit\d$/.test(code)) return code.slice(5);
  return code;
}

/** The modifiers held during `event`, in the order Rust saves them. */
function heldModifiers(event: KeyboardEvent): string[] {
  const held: string[] = [];
  if (event.ctrlKey) held.push("Ctrl");
  if (event.altKey) held.push("Alt");
  if (event.shiftKey) held.push("Shift");
  if (event.metaKey) held.push("Super");
  return held;
}

function isRecording(): boolean {
  return document.activeElement === shortcutInput;
}

function show(
  target: HTMLParagraphElement,
  message: string,
  kind: "ok" | "error",
): void {
  target.textContent = message;
  target.dataset.kind = kind;
}

/** What the field shows: the saved shortcut, or while recording the prompt. */
function renderField(): void {
  const prompt = isRecording() && !showingSaved;
  shortcutInput.value = prompt ? PROMPT : toDisplay(savedShortcut);
  shortcutInput.toggleAttribute("data-prompt", prompt);
}

/** While recording, shows the modifiers held, as in "Ctrl + Shift + …". */
function renderHeld(event: KeyboardEvent): void {
  const held = heldModifiers(event);
  if (held.length === 0) {
    renderField();
    return;
  }
  shortcutInput.value = `${toDisplay(held.join("+"))} + …`;
  shortcutInput.removeAttribute("data-prompt");
}

/** Offers "Reset to default" only while the saved shortcut isn't the default. */
async function renderReset(): Promise<void> {
  try {
    shortcutReset.hidden = await isDefaultShortcut();
  } catch {
    // Unknown: don't offer a reset that may change nothing.
    shortcutReset.hidden = true;
  }
}

// Pause, resume and save run one at a time: async commands can otherwise
// reach Rust out of order, and a quick focus then blur would leave the
// shortcut paused.
let turns: Promise<void> = Promise.resolve();

function inTurn(call: () => Promise<void>): Promise<void> {
  const turn = turns.then(call).catch(() => {
    // A failed pause keeps the shortcut working, so pressing it won't
    // record. A failed resume is retried by Rust when the window loses
    // focus or closes.
  });
  turns = turn;
  return turn;
}

/** Saves `wanted` (as Rust parses it) and says how it went. */
async function applyShortcut(wanted: string): Promise<void> {
  if (saving) return;
  saving = true;
  shortcutReset.disabled = true;
  // Emptied first, so the same message twice is still announced.
  shortcutStatus.textContent = "";
  await inTurn(async () => {
    try {
      await setShortcut(wanted);
      savedShortcut = wanted;
      showingSaved = true;
      shortcutInput.removeAttribute("aria-invalid");
      show(shortcutStatus, "Shortcut saved.", "ok");
      await renderReset();
    } catch (error) {
      // The previous shortcut still works and recording goes on.
      show(shortcutStatus, errorMessage(error, "save the shortcut"), "error");
    }
    // A save registers the new shortcut. Pause it too while the field still
    // records, so pressing it again records instead of starting a rewrite.
    if (isRecording()) await pauseShortcut();
  });
  saving = false;
  shortcutReset.disabled = false;
  renderField();
}

shortcutInput.addEventListener("keydown", (event) => {
  // Tab moves on; a blur ends recording.
  if (event.code === "Tab" || event.isComposing) return;
  event.preventDefault();
  if (event.code === "Escape") {
    // Ends recording; blur brings the saved shortcut back.
    shortcutInput.blur();
    return;
  }
  if (MODIFIER_CODES.has(event.code)) {
    showingSaved = false;
    renderHeld(event);
    return;
  }
  // Injected keys have no code; held keys repeat.
  if (event.repeat || event.code === "" || saving) return;
  showingSaved = false;
  const pressed = [...heldModifiers(event), keyName(event.code)].join("+");
  shortcutInput.value = toDisplay(pressed);
  shortcutInput.removeAttribute("data-prompt");
  void applyShortcut(pressed);
});

shortcutInput.addEventListener("keyup", (event) => {
  if (MODIFIER_CODES.has(event.code) && !saving) renderHeld(event);
});

// While the field has focus the saved shortcut is paused, so pressing it
// records it instead of starting a rewrite. Rust also resumes it when the
// window loses focus or closes, in case a resume here never arrives.
shortcutInput.addEventListener("focus", () => {
  showingSaved = false;
  renderField();
  void inTurn(pauseShortcut);
});

function endRecording(): void {
  void inTurn(resumeShortcut);
}

shortcutInput.addEventListener("blur", () => {
  renderField();
  endRecording();
});

document.addEventListener("visibilitychange", () => {
  if (document.hidden) endRecording();
});

shortcutReset.addEventListener("click", async () => {
  await applyShortcut(DEFAULT_SHORTCUT);
  // Disabling the button while it saved dropped its focus. Give it back, or
  // once the default is set and the button hides, move on to the next
  // control: the field would start recording.
  if (document.activeElement === document.body) {
    (shortcutReset.hidden ? startSwitch : shortcutReset).focus();
  }
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
  renderField();
  void renderReset();
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
