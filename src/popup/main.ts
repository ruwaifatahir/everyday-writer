// Popup window. It has no logic of its own: it shows the state the rewrite
// session sends and turns buttons and keys into session events.
//
// The result is user data: it is only ever put into the page, never logged.

import "../shared/native-feel";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type PopupState =
  | { kind: "hidden" }
  | { kind: "working"; text: string }
  | { kind: "result"; text: string }
  | {
      kind: "message";
      text: string;
      action: "openSettings" | null;
      copied: boolean;
      /** The rewrite failed in a way trying again can fix. */
      rewrite: boolean;
    };

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const popup = element<HTMLElement>("popup");
const statusText = element<HTMLSpanElement>("status-text");
const statusDetail = element<HTMLSpanElement>("status-detail");
const working = element<HTMLDivElement>("working");
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

/** Whether Rewrite leads, as on a failed rewrite: primary, with Enter. */
function rewriteLeads(state: PopupState): boolean {
  return state.kind === "message" && state.rewrite;
}

function show(state: PopupState): void {
  current = state;
  popup.dataset.state = state.kind;
  const hasResult = state.kind === "result";
  const leads = rewriteLeads(state);
  working.hidden = state.kind !== "working";
  result.hidden = !hasResult;
  message.hidden = state.kind !== "message";
  // Rewrite sits beside Copy and Replace while working and on a result;
  // on a failed rewrite it is the only action, with Enter.
  rewrite.hidden = !(state.kind === "working" || hasResult || leads);
  rewrite.disabled = state.kind === "working";
  rewrite.classList.toggle("primary", leads);
  rewrite.classList.toggle("secondary", !leads);
  rewriteKey.hidden = !leads;
  copy.hidden = leads;
  replace.hidden = leads;
  copy.disabled = !hasResult;
  replace.disabled = !hasResult;
  // The message itself is read out with the status, not only "Not rewritten".
  statusDetail.textContent = state.kind === "message" ? `. ${state.text}` : "";
  // The selection is user data too: it only stays in the page while working.
  working.textContent = state.kind === "working" ? state.text : "";

  switch (state.kind) {
    case "hidden":
      // Drop the last result so it doesn't linger in the hidden page, and
      // leave a blank frame behind, not the old text (see `blank`).
      statusText.textContent = "";
      result.textContent = "";
      messageText.textContent = "";
      openSettings.hidden = true;
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
      replace.focus();
      break;
    case "message":
      statusText.textContent = state.copied ? "Copied" : "Not rewritten";
      messageText.textContent = state.text;
      openSettings.hidden = state.action !== "openSettings";
      if (leads) rewrite.focus();
      else if (!openSettings.hidden) openSettings.focus();
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

function send(command: string): void {
  invoke(command).catch(() => {
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
  // Enter is the main action from the text or that button itself: Replace
  // on a result, Rewrite on a failed rewrite. On another button it presses
  // that button instead.
  if (current.kind === "result" && enterIsMainAction(result, replace)) {
    event.preventDefault();
    replaceText();
  } else if (rewriteLeads(current) && enterIsMainAction(message, rewrite)) {
    event.preventDefault();
    rewriteText();
  }
});

await listen<PopupState>("popup-state", (event) => show(event.payload));
show(await invoke<PopupState>("popup_state"));
document.body.dataset.ready = "";
