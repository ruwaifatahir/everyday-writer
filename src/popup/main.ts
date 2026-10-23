// Popup window. It has no logic of its own: it shows the state the rewrite
// session sends and turns buttons and keys into session events.
//
// The result is user data: it is only ever put into the page, never logged.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type PopupState =
  | { kind: "hidden" }
  | { kind: "working" }
  | { kind: "result"; text: string }
  | { kind: "message"; text: string; action: "openSettings" | null };

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const popup = element<HTMLElement>("popup");
const status = element<HTMLParagraphElement>("status");
const working = element<HTMLDivElement>("working");
const result = element<HTMLParagraphElement>("result");
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

  switch (state.kind) {
    case "hidden":
      // Drop the last result so it doesn't linger in the hidden page.
      result.textContent = "";
      messageText.textContent = "";
      break;
    case "working":
      status.textContent = "Rewriting…";
      result.textContent = "";
      break;
    case "result":
      status.textContent = "Rewritten";
      result.textContent = state.text;
      result.scrollTop = 0;
      replace.focus();
      break;
    case "message":
      status.textContent = "Not rewritten";
      messageText.textContent = state.text;
      openSettings.hidden = state.action !== "openSettings";
      if (!openSettings.hidden) openSettings.focus();
      break;
  }
}

function send(command: string): void {
  invoke(command).catch(() => {
    // The session ignores events that don't fit its state; nothing to show.
  });
}

copy.addEventListener("click", () => send("popup_copy"));
replace.addEventListener("click", () => send("popup_replace"));
close.addEventListener("click", () => send("popup_close"));
openSettings.addEventListener("click", () => send("popup_open_settings"));

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    event.preventDefault();
    send("popup_close");
  } else if (event.key === "Enter" && current === "result") {
    // Enter is Replace, whichever button has focus.
    event.preventDefault();
    send("popup_replace");
  }
});

await listen<PopupState>("popup-state", (event) => show(event.payload));
show(await invoke<PopupState>("popup_state"));
document.body.dataset.ready = "";
