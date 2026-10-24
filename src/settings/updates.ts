// "Updates" section: shows the running version.
//
// Rust can open the window scrolled to a section (see `settings_window::
// open_at`): a new window gets the section as its URL fragment, an open one
// gets a scroll event.

import { appVersion, onScrollTo } from "./api";

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const versionLine = element<HTMLParagraphElement>("app-version");

/** Scrolls to the section with this id, if the page has one. */
function scrollToSection(id: string): void {
  document.getElementById(id)?.scrollIntoView({ block: "start" });
}

/**
 * Scrolls to the section named in the URL fragment. Call it once the page's
 * content has loaded, so the sections above have their final height.
 */
export function scrollToRequestedSection(): void {
  const id = window.location.hash.slice(1);
  if (id !== "") scrollToSection(id);
}

export async function initUpdates(): Promise<void> {
  try {
    await onScrollTo(scrollToSection);
  } catch {
    // Without the event an open window just stays where it is.
  }
  try {
    versionLine.textContent = `everyday-writer ${await appVersion()}`;
  } catch {
    versionLine.textContent = "everyday-writer";
  }
}
