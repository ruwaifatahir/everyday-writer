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
  | { kind: "message"; text: string; action: "openSettings" | null };

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
const copy = element<HTMLButtonElement>("copy");
const replace = element<HTMLButtonElement>("replace");
const close = element<HTMLButtonElement>("close");

let current: PopupState["kind"] = "hidden";

function show(state: PopupState): void {
  current = state.kind;
  popup.dataset.state = state.kind;
  const hasResult = state.kind === "result";
  working.hidden = state.kind !== "working";
  result.hidden = !hasResult;
  message.hidden = state.kind !== "message";
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
      statusText.textContent = "Not rewritten";
      messageText.textContent = state.text;
      openSettings.hidden = state.action !== "openSettings";
      if (!openSettings.hidden) openSettings.focus();
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
  if (current !== "result") return;
  blank();
  send("popup_replace");
}

function closePopup(): void {
  blank();
  send("popup_close");
}

copy.addEventListener("click", () => send("popup_copy"));
replace.addEventListener("click", replaceText);
close.addEventListener("click", closePopup);
openSettings.addEventListener("click", () => send("popup_open_settings"));

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    event.preventDefault();
    closePopup();
    return;
  }
  if (event.key !== "Enter" || current !== "result") return;
  // Enter is Replace from the text or Replace itself; on Copy or Close it
  // presses that button instead.
  const focused = document.activeElement;
  if (
    focused === null ||
    focused === document.body ||
    focused === result ||
    focused === replace
  ) {
    event.preventDefault();
    replaceText();
  }
});

await listen<PopupState>("popup-state", (event) => show(event.payload));
show(await invoke<PopupState>("popup_state"));
document.body.dataset.ready = "";
