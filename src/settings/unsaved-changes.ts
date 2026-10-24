// "Save your changes before updating?" prompt for the provider form.
//
// An update restarts the app, so edits that were never saved would be lost.
// Only the provider form counts: the Shortcut and startup group applies right
// away, and instruction sets save on their own.

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

/** Shows the prompt and resolves with the user's choice once it closes. */
export function askToSaveBeforeUpdating(): Promise<UnsavedChoice> {
  const dialog = document.getElementById("unsaved-dialog");
  if (!(dialog instanceof HTMLDialogElement)) {
    return Promise.resolve("cancel");
  }
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
