// "Updates" section: shows the running version and the update controller's
// state, and starts a check. The rules live in Rust (`updates.rs`); this
// page only renders the `UpdatesState` it is sent.
//
// Rust can open the window scrolled to a section (see `settings_window::
// open_at`): a new window gets the section as its URL fragment, an open one
// gets a scroll event.

import {
  appVersion,
  checkForUpdates,
  getUpdatesState,
  onScrollTo,
  onUpdatesState,
  type UpdatesState,
  updateNow,
} from "./api";

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const versionLine = element<HTMLParagraphElement>("app-version");
const checkButton = element<HTMLButtonElement>("updates-check");
const statusLine = element<HTMLParagraphElement>("updates-status");
const updateNowButton = element<HTMLButtonElement>("updates-install");

/** Runs before Update now; resolving false stops it. */
let confirmUpdateNow: () => Promise<boolean> = () => Promise.resolve(true);

/**
 * Sets the check Update now runs first, such as asking about unsaved
 * Settings changes. The update only starts if it resolves true.
 */
export function beforeUpdateNow(confirm: () => Promise<boolean>): void {
  confirmUpdateNow = confirm;
}

/** The status line for each state; `kind` colours it. */
function statusFor(state: UpdatesState): {
  text: string;
  kind?: "ok" | "error";
} {
  switch (state.kind) {
    case "idle":
      return { text: "" };
    case "checking":
      return { text: "Checking…" };
    case "upToDate":
      return { text: "You're up to date", kind: "ok" };
    case "available":
      return { text: `Version ${state.version} is available` };
    case "failed":
      return { text: "Couldn't check for updates", kind: "error" };
    case "downloading":
      return { text: `Downloading… ${state.percent}%` };
    case "installFailed":
      return { text: "Update failed. Try again.", kind: "error" };
  }
}

function render(state: UpdatesState): void {
  const { text, kind } = statusFor(state);
  statusLine.textContent = text;
  if (kind) statusLine.dataset.kind = kind;
  else delete statusLine.dataset.kind;
  const downloading = state.kind === "downloading";
  checkButton.disabled = state.kind === "checking" || downloading;
  // Shown while an update is offered; disabled so it can't start twice.
  updateNowButton.hidden = !(
    state.kind === "available" ||
    state.kind === "installFailed" ||
    downloading
  );
  updateNowButton.disabled = downloading;
}

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
  checkButton.addEventListener("click", () => {
    checkForUpdates().catch(() => render({ kind: "failed", step: "check" }));
  });
  updateNowButton.addEventListener("click", () => {
    updateNowButton.disabled = true;
    confirmUpdateNow()
      .then((go) => {
        if (go) return updateNow();
        updateNowButton.disabled = false;
        updateNowButton.focus();
      })
      .catch(() => {
        updateNowButton.disabled = false;
      });
  });
  try {
    // Listen first, so a state sent while the getter runs isn't missed, and
    // let such an event win over the getter's possibly older answer.
    let heard = false;
    await onUpdatesState((state) => {
      heard = true;
      render(state);
    });
    const current = await getUpdatesState();
    if (!heard) render(current);
  } catch {
    // The section still shows the version and the button.
  }
  try {
    versionLine.textContent = `everyday-writer ${await appVersion()}`;
  } catch {
    versionLine.textContent = "everyday-writer";
  }
}
