// Settings window: provider, model, Custom base URL and the provider's key.
//
// The saved key is never shown: the window only learns whether one exists,
// and says so in the key field's placeholder. Typing a new key replaces it.
// A typed key is sent only on Save, and the field is emptied right after
// saving.

import "../shared/native-feel";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  clearApiKey,
  errorMessage,
  getSettings,
  isKeySaved,
  type ProviderKind,
  type ProviderSettings,
  type Settings,
  saveApiKey,
  saveSettings,
  settingsReady,
  suggestedModels,
} from "./api";
import { type Option, Select, Suggest } from "./combobox";
import { clearFieldError, showFieldError } from "./field-error";
import {
  initInstructionSets,
  isEditingSet,
  leaveEditor,
} from "./instruction-sets";
import { initLaunch } from "./launch";
import {
  askToSave,
  askToSaveBeforeUpdating,
  BEFORE_CLOSING,
  hasUnsavedChanges,
  isDialogOpen,
} from "./unsaved-changes";
import {
  beforeUpdateNow,
  initUpdates,
  scrollToRequestedSection,
} from "./updates";

/**
 * Each provider's name inside a sentence. Matches `display_name` in
 * `src-tauri/src/settings.rs`, so Settings and the popup name it alike.
 */
const PROVIDER_NAMES: Record<ProviderKind, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Gemini",
  openrouter: "OpenRouter",
  ollama: "Ollama",
  custom: "your server",
};

/** The provider list: the hosted services, then the two you run yourself. */
const PROVIDER_OPTIONS: Option[] = [
  { value: "openai", label: "OpenAI", icon: "logo-openai" },
  { value: "anthropic", label: "Anthropic", icon: "logo-anthropic" },
  { value: "gemini", label: "Gemini", icon: "logo-gemini" },
  { value: "openrouter", label: "OpenRouter", icon: "logo-openrouter" },
  {
    value: "ollama",
    label: "Ollama",
    icon: "logo-ollama",
    detail: "On this PC",
    divided: true,
  },
  {
    value: "custom",
    label: "Custom",
    icon: "logo-custom",
    detail: "OpenAI-compatible",
  },
];

// Example names only, shown as placeholders.
const MODEL_EXAMPLES: Record<ProviderKind, string> = {
  openai: "e.g. gpt-6-luna",
  anthropic: "e.g. claude-haiku-5-5",
  gemini: "e.g. gemini-3.5-flash-lite",
  openrouter: "e.g. openai/gpt-6-luna",
  ollama: "e.g. gemma4",
  custom: "e.g. llama-3.1-8b-instruct",
};

/** Providers that work without a key. Ollama never sends one at all. */
function keyIsOptional(provider: ProviderKind): boolean {
  return provider === "ollama" || provider === "custom";
}

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const form = element<HTMLFormElement>("settings");
const baseUrlField = element<HTMLDivElement>("base-url-field");
const baseUrlInput = element<HTMLInputElement>("base-url");
const providerHint = element<HTMLParagraphElement>("provider-hint");
const modelInput = element<HTMLInputElement>("model");
const modelHint = element<HTMLParagraphElement>("model-hint");
const keyField = element<HTMLDivElement>("key-field");
const keyLabel = element<HTMLLabelElement>("key-label");
const keyInput = element<HTMLInputElement>("key");
const keyRemove = element<HTMLButtonElement>("key-remove");
const saveButton = element<HTMLButtonElement>("save");
const status = element<HTMLParagraphElement>("status");

const providerPicker = new Select(
  element<HTMLButtonElement>("provider"),
  "Providers",
  () => changeProvider(),
);
providerPicker.setOptions(PROVIDER_OPTIONS);
providerPicker.set("gemini");

const modelPicker = new Suggest(
  modelInput,
  element<HTMLButtonElement>("model-toggle"),
  "Models",
);

/** What the window shows, apart from the plain text fields. */
const view = {
  provider: "gemini" as ProviderKind,
  keySaved: false,
  confirmingRemove: false,
};

/** The form as last loaded or saved; undefined if loading failed. */
let savedForm: ProviderSettings | undefined;

/** The form as it stands, unchecked. */
function currentForm(): ProviderSettings {
  return {
    provider: view.provider,
    model: modelInput.value,
    customBaseUrl: baseUrlInput.value,
  };
}

let statusTimer: number | undefined;

function showStatus(message: string, kind: "ok" | "error" | "unsaved"): void {
  window.clearTimeout(statusTimer);
  // The line is read aloud on change; the same text again stays quiet.
  if (status.textContent !== message) status.textContent = message;
  status.dataset.kind = kind;
  if (kind === "ok") {
    statusTimer = window.setTimeout(renderUnsaved, 2500);
  }
}

/**
 * Says "Unsaved changes" while the form differs from what was saved, and
 * clears the line once it doesn't. A load error stays: with nothing loaded
 * there is nothing to compare against.
 */
function renderUnsaved(): void {
  if (hasUnsavedChanges(savedForm, currentForm(), keyInput.value)) {
    showStatus("Unsaved changes", "unsaved");
  } else if (savedForm) {
    window.clearTimeout(statusTimer);
    status.textContent = "";
  }
}

function clearStatus(): void {
  window.clearTimeout(statusTimer);
  status.textContent = "";
  clearFieldError(modelInput);
  clearFieldError(baseUrlInput);
}

function renderProvider(): void {
  const custom = view.provider === "custom";
  const ollama = view.provider === "ollama";
  baseUrlField.hidden = !custom;
  providerHint.hidden = !ollama;
  keyField.hidden = ollama;
  modelInput.placeholder = MODEL_EXAMPLES[view.provider];
  keyLabel.textContent = keyIsOptional(view.provider)
    ? "API key (optional)"
    : "API key";
}

/** The built-in Suggested Models per provider, loaded once from Rust. */
let suggested: Partial<Record<ProviderKind, string[]>> = {};

function suggestedFor(provider: ProviderKind): string[] {
  return suggested[provider] ?? [];
}

/**
 * Lists the provider's Suggested Models under the model field. Ollama and
 * Custom have none, so theirs is a plain field with a hint on where to find
 * the name.
 */
function renderModel(): void {
  const provider = view.provider;
  modelPicker.setNames(suggestedFor(provider));
  if (provider === "ollama") {
    modelHint.textContent =
      "Type an installed model, like gemma4. Run ollama list to see yours.";
  } else if (provider === "custom") {
    modelHint.textContent =
      "Type the model name exactly as your server lists it.";
  } else {
    modelHint.textContent = `Pick one or type any ${PROVIDER_NAMES[provider]} model.`;
  }
}

/**
 * The key field's placeholder says whether the provider has a key saved;
 * Remove key shows only when it does.
 */
function renderKey(): void {
  if (view.keySaved) {
    keyInput.placeholder = "Key saved. Type a new one to replace it.";
  } else if (keyIsOptional(view.provider)) {
    keyInput.placeholder = "Paste your API key (optional)";
  } else {
    keyInput.placeholder = "Paste your API key";
  }
  keyRemove.hidden = !view.keySaved;
  keyRemove.textContent = view.confirmingRemove
    ? "Remove permanently"
    : "Remove key";
  keyRemove.classList.toggle("confirming", view.confirmingRemove);
}

async function loadKeyState(): Promise<void> {
  const provider = view.provider;
  let saved = false;
  try {
    saved = await isKeySaved(provider);
  } catch {
    // Shown as no key saved; typing one still saves it.
  }
  // A later provider switch has loaded, or will load, its own state.
  if (provider !== view.provider) return;
  view.keySaved = saved;
  renderKey();
}

function resetKeyEntry(): void {
  view.confirmingRemove = false;
  keyInput.value = "";
}

function isHttpUrl(url: string): boolean {
  return /^https?:\/\/\S+$/i.test(url);
}

/**
 * Checks the form and shows the first problem under its field. Returns the
 * settings if valid.
 */
function readForm(): ProviderSettings | undefined {
  const settings: ProviderSettings = {
    provider: view.provider,
    model: modelInput.value.trim(),
    customBaseUrl: baseUrlInput.value.trim(),
  };
  if (settings.provider === "custom" && !isHttpUrl(settings.customBaseUrl)) {
    showFieldError(
      baseUrlInput,
      "Enter a base URL starting with http:// or https://.",
    );
    baseUrlInput.focus();
    return undefined;
  }
  if (settings.model === "") {
    showFieldError(modelInput, "Enter a model name.");
    modelInput.focus();
    return undefined;
  }
  return settings;
}

/** Saves the form. Resolves false if it was invalid or couldn't be saved. */
async function save(): Promise<boolean> {
  // Without a loaded form there's nothing to compare, so it always saves.
  if (
    savedForm &&
    !hasUnsavedChanges(savedForm, currentForm(), keyInput.value)
  ) {
    clearStatus();
    showStatus("Saved.", "ok");
    return true;
  }
  clearStatus();
  const settings = readForm();
  if (!settings) {
    renderUnsaved();
    return false;
  }
  // Ollama never sends a key, so none is saved for it.
  const key = settings.provider === "ollama" ? "" : keyInput.value.trim();
  saveButton.disabled = true;
  try {
    await saveSettings(settings);
    savedForm = settings;
    if (key !== "") {
      await saveApiKey(settings.provider, key);
      view.keySaved = true;
    }
    resetKeyEntry();
    renderKey();
    showStatus("Saved.", "ok");
    return true;
  } catch (error) {
    showStatus(errorMessage(error, "save your settings"), "error");
    return false;
  } finally {
    saveButton.disabled = false;
  }
}

async function removeKey(): Promise<void> {
  if (!view.confirmingRemove) {
    view.confirmingRemove = true;
    renderKey();
    return;
  }
  keyRemove.disabled = true;
  try {
    await clearApiKey(view.provider);
    view.keySaved = false;
    view.confirmingRemove = false;
    renderKey();
    showStatus("Key removed.", "ok");
    keyInput.focus();
  } catch (error) {
    view.confirmingRemove = false;
    renderKey();
    showStatus(errorMessage(error, "remove the key"), "error");
  } finally {
    keyRemove.disabled = false;
  }
}

function changeProvider(): void {
  view.provider = providerPicker.value as ProviderKind;
  resetKeyEntry();
  clearStatus();
  renderProvider();
  renderKey();
  // A name from the old provider won't work on the new one.
  modelInput.value = suggestedFor(view.provider)[0] ?? "";
  renderModel();
  renderUnsaved();
  void loadKeyState();
}

form.addEventListener("input", (event) => {
  if (event.target instanceof HTMLInputElement) clearFieldError(event.target);
  renderUnsaved();
});

form.addEventListener("submit", (event) => {
  event.preventDefault();
  void save();
});

keyRemove.addEventListener("click", () => {
  void removeKey();
});

keyRemove.addEventListener("blur", () => {
  if (!view.confirmingRemove) return;
  view.confirmingRemove = false;
  renderKey();
});

// Update now restarts the app, so unsaved form edits are asked about first.
// Save only lets the update go ahead if the save worked; a failed save shows
// its error as usual.
beforeUpdateNow(async () => {
  if (!hasUnsavedChanges(savedForm, currentForm(), keyInput.value)) {
    return true;
  }
  switch (await askToSaveBeforeUpdating()) {
    case "save": {
      const saved = await save();
      if (!saved) status.scrollIntoView({ block: "nearest" });
      return saved;
    }
    case "discard":
      return true;
    case "cancel":
      return false;
  }
});

/** True while a close is asking about unsaved changes. */
let closing = false;

/**
 * Closes the window, like Esc or Ctrl+W in a native dialog. An open set
 * editor with edits, then unsaved provider edits, are asked about first.
 */
async function closeWindow(): Promise<void> {
  if (closing) return;
  closing = true;
  try {
    if (!(await leaveEditor())) return;
    if (hasUnsavedChanges(savedForm, currentForm(), keyInput.value)) {
      switch (await askToSave(BEFORE_CLOSING)) {
        case "save":
          if (!(await save())) {
            status.scrollIntoView({ block: "nearest" });
            return;
          }
          break;
        case "discard":
          break;
        case "cancel":
          return;
      }
    }
    await getCurrentWindow().close();
  } catch {
    // The window stays open; the title bar still closes it.
  } finally {
    closing = false;
  }
}

// Bubble phase, so fields that use these keys themselves (the shortcut
// field records every key but Tab, and Esc ends recording) get them first and cancel them.
// Dialogs and the set editor handle Esc on their own.
document.addEventListener("keydown", (event) => {
  if (event.defaultPrevented || event.repeat || isDialogOpen()) return;
  const ctrlW =
    event.code === "KeyW" &&
    event.ctrlKey &&
    !(event.altKey || event.shiftKey || event.metaKey);
  const esc = event.key === "Escape" && !isEditingSet();
  if (!(ctrlW || esc)) return;
  event.preventDefault();
  void closeWindow();
});

async function init(): Promise<void> {
  let settings: Settings | undefined;
  // Both at once: the window stays hidden until the page is drawn.
  const [models, loaded] = await Promise.allSettled([
    suggestedModels(),
    getSettings(),
  ]);
  // Without the list the model field still takes any typed name.
  if (models.status === "fulfilled") suggested = models.value;
  if (loaded.status === "fulfilled") {
    settings = loaded.value;
    view.provider = settings.provider;
    modelInput.value = settings.model;
    baseUrlInput.value = settings.customBaseUrl;
    savedForm = currentForm();
    providerPicker.set(settings.provider);
  } else {
    showStatus(errorMessage(loaded.reason, "load your settings"), "error");
  }
  renderProvider();
  renderKey();
  renderModel();
  await Promise.all([loadKeyState(), initLaunch(settings)]);
}

function nextFrame(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()));
}

// The Instruction sets and Updates sections run on their own; see
// instruction-sets.ts and updates.ts. Once all have loaded and the fonts
// are in, the window scrolls to the section Rust opened it at, if any, and
// tells Rust to show it, so it never appears blank or half drawn. A failed
// step still shows the window, with what did load.
void Promise.allSettled([init(), initInstructionSets(), initUpdates()]).then(
  async () => {
    // Laying the page out starts its font loads. Scroll once they're in,
    // as they change the sections' heights.
    void document.body.offsetHeight;
    await document.fonts.ready;
    scrollToRequestedSection();
    // The second frame starts once the first, with everything above, is
    // painted; showing the window sooner can show an older frame.
    await nextFrame();
    await nextFrame();
    await settingsReady().catch(() => {
      // Rust shows the window anyway after its timeout.
    });
  },
);
