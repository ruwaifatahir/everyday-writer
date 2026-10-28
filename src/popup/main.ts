// Popup window. It has no logic of its own: it shows the state the rewrite
// session sends and turns buttons and keys into session events.
//
// The selection and the result are user data: they are only ever put into
// the page, never logged.

import "../shared/native-feel";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/** The instruction sets the dropdown lists: names only, never instructions. */
type Choices = {
  sets: { id: number; name: string }[];
  active: number;
};

type PopupState =
  | { kind: "hidden" }
  | { kind: "ready"; text: string; choices: Choices }
  | { kind: "working"; text: string; choices: Choices }
  | { kind: "result"; text: string; choices: Choices }
  | {
      kind: "message";
      text: string;
      action: "openSettings" | null;
      copied: boolean;
      /** The rewrite failed in a way trying again can fix. */
      retryable: boolean;
      choices: Choices;
    };

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const popup = element<HTMLElement>("popup");
const statusText = element<HTMLSpanElement>("status-text");
const statusDetail = element<HTMLSpanElement>("status-detail");
const instructionSet = element<HTMLSelectElement>("instruction-set");
const working = element<HTMLDivElement>("working");
const preview = element<HTMLElement>("preview");
const result = element<HTMLElement>("result");
const message = element<HTMLDivElement>("message");
const messageText = element<HTMLParagraphElement>("message-text");
const openSettings = element<HTMLButtonElement>("open-settings");
const rewrite = element<HTMLButtonElement>("rewrite");
const rewriteKey = element<HTMLElement>("rewrite-key");
const copy = element<HTMLButtonElement>("copy");
const replace = element<HTMLButtonElement>("replace");
const close = element<HTMLButtonElement>("close");

let current: PopupState = { kind: "hidden" };

/** Whether Rewrite leads, primary with Enter: when a selection waits
 * (Ready) and on a failed rewrite. */
function rewriteLeads(state: PopupState): boolean {
  return (
    state.kind === "ready" || (state.kind === "message" && state.retryable)
  );
}

/** The sets the dropdown lists now, as JSON, so it is rebuilt only when
 * they change. */
let listedKey = "";

/** Lists `choices` in the dropdown with the active set picked. Disabled
 * while there is only one set, so the layout never changes. */
function showChoices(choices: Choices | null): void {
  const sets = choices?.sets ?? [];
  const names = JSON.stringify(sets);
  if (names !== listedKey) {
    listedKey = names;
    instructionSet.replaceChildren(
      ...sets.map(({ id, name }) => new Option(name, String(id))),
    );
  }
  if (choices) instructionSet.value = String(choices.active);
  instructionSet.disabled = sets.length < 2;
}

/** Moves focus to `target`, unless the user is in the dropdown: after a
 * pick, focus stays there so another set is one Alt+↓ away. */
function focusUnlessPicking(target: HTMLElement): void {
  if (document.activeElement !== instructionSet) target.focus();
}

function show(state: PopupState): void {
  current = state;
  popup.dataset.state = state.kind;
  const hasResult = state.kind === "result";
  const leads = rewriteLeads(state);
  showChoices(state.kind === "hidden" ? null : state.choices);
  working.hidden = state.kind !== "working";
  preview.hidden = state.kind !== "ready";
  result.hidden = !hasResult;
  message.hidden = state.kind !== "message";
  // Copy and Replace show only on a result, with Rewrite beside them.
  // While working Rewrite shows alone, disabled; when a selection waits or
  // on a failed rewrite it is the only action, with Enter.
  rewrite.hidden = !(state.kind === "working" || hasResult || leads);
  rewrite.disabled = state.kind === "working";
  rewrite.classList.toggle("primary", leads);
  rewrite.classList.toggle("secondary", !leads);
  rewriteKey.hidden = !leads;
  copy.hidden = !hasResult;
  replace.hidden = !hasResult;
  // The message itself is read out with the status, not only "Not rewritten".
  statusDetail.textContent = state.kind === "message" ? `. ${state.text}` : "";
  // The selection only stays in the page while it is shown.
  working.textContent = state.kind === "working" ? state.text : "";
  preview.textContent = state.kind === "ready" ? state.text : "";

  switch (state.kind) {
    case "hidden":
      // Drop the last result so it doesn't linger in the hidden page, and
      // leave a blank frame behind, not the old text (see `blank`).
      statusText.textContent = "";
      result.textContent = "";
      messageText.textContent = "";
      openSettings.hidden = true;
      break;
    case "ready":
      statusText.textContent = "Ready to rewrite";
      result.textContent = "";
      preview.scrollTop = 0;
      focusUnlessPicking(rewrite);
      break;
    case "working":
      statusText.textContent = "Rewriting…";
      result.textContent = "";
      working.scrollTop = 0;
      break;
    case "result":
      statusText.textContent = "Rewritten";
      result.textContent = state.text;
      result.scrollTop = 0;
      focusUnlessPicking(replace);
      break;
    case "message":
      statusText.textContent = state.copied ? "Copied" : "Not rewritten";
      messageText.textContent = state.text;
      openSettings.hidden = state.action !== "openSettings";
      if (leads) focusUnlessPicking(rewrite);
      else if (!openSettings.hidden) focusUnlessPicking(openSettings);
      break;
  }
}

/**
 * Clears the page before asking the session to hide the popup. Rust sends
 * the next state and shows the window in one go, so the page draws that
 * state a frame after the window appears; whatever was drawn last before
 * hiding is what flashes in between. Clearing first makes that a blank
 * frame instead of the previous result.
 */
function blank(): void {
  show({ kind: "hidden" });
}

function send(command: string, args?: Record<string, unknown>): void {
  invoke(command, args).catch(() => {
    // The session ignores events that don't fit its state; nothing to show.
  });
}

/** Replace hides the popup, or shows a new state if it couldn't go ahead. */
function replaceText(): void {
  if (current.kind !== "result") return;
  blank();
  send("popup_replace");
}

/** Rewrite runs a new rewrite of the same selection; the session ignores
 * it while one is running. */
function rewriteText(): void {
  if (rewrite.hidden || rewrite.disabled) return;
  send("popup_rewrite");
}

function closePopup(): void {
  blank();
  send("popup_close");
}

// Picking a set makes it active everywhere and rewrites with it at once.
// The session sends the new state, which also puts the dropdown right if
// the pick failed.
instructionSet.addEventListener("change", () => {
  send("popup_pick_instruction_set", { id: Number(instructionSet.value) });
});

/** Keys the closed dropdown lets through: they move focus, close the
 * popup, take the main action or open the list, and never change the set. */
const keysThatKeepTheSet = new Set([
  "Tab",
  "Escape",
  "Enter",
  "F4",
  " ",
  "Shift",
  "Control",
  "Alt",
  "Meta",
]);

/** Whether `event` on the closed dropdown leaves the picked set as it is. */
function keepsTheSet(event: KeyboardEvent): boolean {
  if (keysThatKeepTheSet.has(event.key) || event.ctrlKey || event.metaKey) {
    return true;
  }
  return event.altKey && (event.key === "ArrowDown" || event.key === "ArrowUp");
}

// Chromium picks another set at each arrow, Home, End, Page Up, Page Down
// or letter pressed on the closed dropdown, which would send a rewrite per
// press. Only keys that keep the set get through: Alt+↓, F4 or Space opens
// the list, where the arrows only move the highlight and Enter picks once.
// The open list's keys don't reach the page.
instructionSet.addEventListener("keydown", (event) => {
  if (!keepsTheSet(event)) event.preventDefault();
});
rewrite.addEventListener("click", rewriteText);
copy.addEventListener("click", () => send("popup_copy"));
replace.addEventListener("click", replaceText);
close.addEventListener("click", closePopup);
openSettings.addEventListener("click", () => send("popup_open_settings"));

/** Whether Enter here means the popup's main action, not a button's own. */
function enterIsMainAction(...owners: HTMLElement[]): boolean {
  const focused = document.activeElement;
  return (
    focused === null ||
    focused === document.body ||
    owners.some((owner) => focused === owner)
  );
}

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    event.preventDefault();
    closePopup();
    return;
  }
  if (event.key !== "Enter") return;
  // Enter is the main action from the text, the closed dropdown or that
  // button itself: Replace on a result, Rewrite when a selection waits or
  // on a failed rewrite. Enter in the open list picks a set and doesn't
  // reach the page. On another button it presses that button instead.
  if (
    current.kind === "result" &&
    enterIsMainAction(result, replace, instructionSet)
  ) {
    event.preventDefault();
    replaceText();
  } else if (
    rewriteLeads(current) &&
    enterIsMainAction(message, preview, rewrite, instructionSet)
  ) {
    event.preventDefault();
    rewriteText();
  }
});

await listen<PopupState>("popup-state", (event) => show(event.payload));
show(await invoke<PopupState>("popup_state"));
document.body.dataset.ready = "";
