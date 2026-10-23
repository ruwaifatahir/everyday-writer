// Settings window: provider, model, Custom base URL and the provider's key.
//
// The saved key is never shown: the window only learns whether one exists,
// and offers Replace or Clear. A typed key is sent once, on Save, and the
// field is emptied right after.

import {
  clearApiKey,
  errorMessage,
  getSettings,
  isKeySaved,
  type ProviderKind,
  type Settings,
  saveApiKey,
  saveSettings,
} from "./api";

const PROVIDER_NAMES: Record<ProviderKind, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Gemini",
  custom: "Custom",
};

// Example names only, shown as placeholders. The user always types the model.
const MODEL_EXAMPLES: Record<ProviderKind, string> = {
  openai: "e.g. gpt-4.1-mini",
  anthropic: "e.g. claude-haiku-4-5",
  gemini: "e.g. gemini-2.5-flash",
  custom: "e.g. llama-3.1-8b-instruct",
};

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const form = element<HTMLFormElement>("settings");
const baseUrlField = element<HTMLDivElement>("base-url-field");
const baseUrlInput = element<HTMLInputElement>("base-url");
const modelInput = element<HTMLInputElement>("model");
const keyLabel = element<HTMLLabelElement>("key-label");
const keySaved = element<HTMLDivElement>("key-saved");
const keySavedText = element<HTMLSpanElement>("key-saved-text");
const keyEntry = element<HTMLDivElement>("key-entry");
const keyInput = element<HTMLInputElement>("key");
const keyReplace = element<HTMLButtonElement>("key-replace");
const keyClear = element<HTMLButtonElement>("key-clear");
const keyCancel = element<HTMLButtonElement>("key-cancel");
const saveButton = element<HTMLButtonElement>("save");
const status = element<HTMLParagraphElement>("status");

/** What the window shows, apart from the plain text fields. */
const view = {
  provider: "openai" as ProviderKind,
  keySaved: false,
  replacingKey: false,
  confirmingClear: false,
};

let statusTimer: number | undefined;

function showStatus(message: string, kind: "ok" | "error"): void {
  window.clearTimeout(statusTimer);
  status.textContent = message;
  status.dataset.kind = kind;
  if (kind === "ok") {
    statusTimer = window.setTimeout(() => {
      status.textContent = "";
    }, 2500);
  }
}

function clearStatus(): void {
  window.clearTimeout(statusTimer);
  status.textContent = "";
  for (const input of [modelInput, baseUrlInput]) {
    input.removeAttribute("aria-invalid");
  }
}

function selectedProvider(): ProviderKind {
  const checked = form.querySelector<HTMLInputElement>(
    'input[name="provider"]:checked',
  );
  return (checked?.value ?? "openai") as ProviderKind;
}

function renderProvider(): void {
  const custom = view.provider === "custom";
  baseUrlField.hidden = !custom;
  modelInput.placeholder = MODEL_EXAMPLES[view.provider];
  keyLabel.textContent = custom ? "API key (optional)" : "API key";
}

function renderKey(): void {
  const name = PROVIDER_NAMES[view.provider];
  const showSaved = view.keySaved && !view.replacingKey;
  keySaved.hidden = !showSaved;
  keyEntry.hidden = showSaved;
  keyCancel.hidden = !view.replacingKey;
  keySavedText.textContent = "Key saved";
  keyClear.textContent = view.confirmingClear ? "Confirm clear" : "Clear";
  keyClear.classList.toggle("confirming", view.confirmingClear);
  keyInput.placeholder =
    view.provider === "custom"
      ? "Leave empty for local servers"
      : `Paste your ${name} key`;
}

async function loadKeyState(): Promise<void> {
  const provider = view.provider;
  try {
    const saved = await isKeySaved(provider);
    if (provider === view.provider) view.keySaved = saved;
  } catch {
    view.keySaved = false;
  }
  renderKey();
}

function resetKeyEntry(): void {
  view.replacingKey = false;
  view.confirmingClear = false;
  keyInput.value = "";
}

function isHttpUrl(url: string): boolean {
  return /^https?:\/\/\S+$/i.test(url);
}

/** Checks the form and marks the first problem. Returns the settings if valid. */
function readForm(): Settings | undefined {
  const settings: Settings = {
    provider: view.provider,
    model: modelInput.value.trim(),
    customBaseUrl: baseUrlInput.value.trim(),
  };
  if (settings.provider === "custom" && !isHttpUrl(settings.customBaseUrl)) {
    baseUrlInput.setAttribute("aria-invalid", "true");
    baseUrlInput.focus();
    showStatus("Enter a base URL starting with http:// or https://.", "error");
    return undefined;
  }
  if (settings.model === "") {
    modelInput.setAttribute("aria-invalid", "true");
    modelInput.focus();
    showStatus("Enter a model name.", "error");
    return undefined;
  }
  return settings;
}

async function save(): Promise<void> {
  clearStatus();
  const settings = readForm();
  if (!settings) return;
  const key = keyInput.value.trim();
  saveButton.disabled = true;
  try {
    await saveSettings(settings);
    if (key !== "") {
      await saveApiKey(settings.provider, key);
      view.keySaved = true;
    }
    resetKeyEntry();
    renderKey();
    showStatus("Saved", "ok");
  } catch (error) {
    showStatus(errorMessage(error), "error");
  } finally {
    saveButton.disabled = false;
  }
}

async function clearKey(): Promise<void> {
  if (!view.confirmingClear) {
    view.confirmingClear = true;
    renderKey();
    return;
  }
  keyClear.disabled = true;
  try {
    await clearApiKey(view.provider);
    view.keySaved = false;
    resetKeyEntry();
    renderKey();
    showStatus("Key cleared", "ok");
    keyInput.focus();
  } catch (error) {
    view.confirmingClear = false;
    renderKey();
    showStatus(errorMessage(error), "error");
  } finally {
    keyClear.disabled = false;
  }
}

form.addEventListener("change", (event) => {
  const target = event.target as HTMLInputElement;
  if (target.name !== "provider") return;
  view.provider = selectedProvider();
  resetKeyEntry();
  clearStatus();
  renderProvider();
  renderKey();
  void loadKeyState();
});

form.addEventListener("input", (event) => {
  (event.target as HTMLElement).removeAttribute("aria-invalid");
});

form.addEventListener("submit", (event) => {
  event.preventDefault();
  void save();
});

keyReplace.addEventListener("click", () => {
  view.replacingKey = true;
  view.confirmingClear = false;
  renderKey();
  keyInput.focus();
});

keyCancel.addEventListener("click", () => {
  resetKeyEntry();
  renderKey();
});

keyClear.addEventListener("click", () => {
  void clearKey();
});

keyClear.addEventListener("blur", () => {
  if (!view.confirmingClear) return;
  view.confirmingClear = false;
  renderKey();
});

async function init(): Promise<void> {
  try {
    const settings = await getSettings();
    view.provider = settings.provider;
    modelInput.value = settings.model;
    baseUrlInput.value = settings.customBaseUrl;
    const radio = form.querySelector<HTMLInputElement>(
      `input[name="provider"][value="${settings.provider}"]`,
    );
    if (radio) radio.checked = true;
  } catch (error) {
    showStatus(errorMessage(error), "error");
  }
  renderProvider();
  renderKey();
  await loadKeyState();
  document.body.dataset.ready = "";
}

void init();
