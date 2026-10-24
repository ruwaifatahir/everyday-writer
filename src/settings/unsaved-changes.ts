// The "Save your changes?" prompt: Save, Discard or Cancel.
//
// Updating (which restarts the app) and closing the window ask about unsaved
// edits to the provider form; leaving the instruction set editor asks about
// its draft. The Shortcut and startup group applies right away, so it never
// counts.

import type { ProviderSettings } from "./api";

/** What the user picked in the prompt. Esc and closing count as Cancel. */
export type UnsavedChoice = "save" | "discard" | "cancel";

/**
 * True when Save would change something: the form differs from what was last
 * loaded or saved, or a key has been typed. `saved` is undefined when the
 * settings couldn't be loaded; then only a typed key counts. Ollama never
 * saves a key, so a key typed for it doesn't count.
 */
export function hasUnsavedChanges(
  saved: ProviderSettings | undefined,
  form: ProviderSettings,
  typedKey: string,
): boolean {
  if (form.provider !== "ollama" && typedKey.trim() !== "") return true;
  if (!saved) return false;
  return (
    saved.provider !== form.provider ||
    saved.model.trim() !== form.model.trim() ||
    saved.customBaseUrl.trim() !== form.customBaseUrl.trim()
  );
}

/** Reads the dialog's `returnValue`; anything unknown is Cancel. */
export function choiceFrom(returnValue: string): UnsavedChoice {
  return returnValue === "save" || returnValue === "discard"
    ? returnValue
    : "cancel";
}

/** The prompt's words for one situation. */
export interface UnsavedPrompt {
  title: string;
  body: string;
}

export const BEFORE_UPDATING: UnsavedPrompt = {
  title: "Save your changes before updating?",
  body: "The app restarts to finish the update. Changes you haven't saved will be lost.",
};

export const BEFORE_CLOSING: UnsavedPrompt = {
  title: "Save your changes before closing?",
  body: "Provider, model and key changes you haven't saved will be lost.",
};

export const LEAVING_SET_EDITOR: UnsavedPrompt = {
  title: "Save this instruction set?",
  body: "Discard closes the editor and drops your edits.",
};

/** Shows the prompt for an update. */
export function askToSaveBeforeUpdating(): Promise<UnsavedChoice> {
  return askToSave(BEFORE_UPDATING);
}

/** Whether any dialog is showing now. Dialogs handle Esc themselves. */
export function isDialogOpen(): boolean {
  return document.querySelector("dialog[open]") !== null;
}

/**
 * Shows the Save / Discard / Cancel prompt with `prompt`'s words and
 * resolves with the user's choice once it closes.
 */
export function askToSave(prompt: UnsavedPrompt): Promise<UnsavedChoice> {
  const dialog = document.getElementById("unsaved-dialog");
  const title = document.getElementById("unsaved-title");
  const body = document.getElementById("unsaved-body");
  if (!(dialog instanceof HTMLDialogElement) || !title || !body) {
    return Promise.resolve("cancel");
  }
  if (dialog.open) return Promise.resolve("cancel");
  title.textContent = prompt.title;
  body.textContent = prompt.body;
  return new Promise((resolve) => {
    dialog.addEventListener(
      "close",
      () => resolve(choiceFrom(dialog.returnValue)),
      { once: true },
    );
    // Esc closes a dialog without touching `returnValue`.
    dialog.returnValue = "";
    dialog.showModal();
  });
}
