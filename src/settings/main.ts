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
  listModels,
  type ProviderKind,
  type ProviderSettings,
  type Settings,
  saveApiKey,
  saveSettings,
} from "./api";
import { initLaunch } from "./launch";

const PROVIDER_NAMES: Record<ProviderKind, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Gemini",
  openrouter: "OpenRouter",
  ollama: "Ollama",
  custom: "Custom",
};

// Example names only, shown as placeholders.
const MODEL_EXAMPLES: Record<ProviderKind, string> = {
  openai: "e.g. gpt-4.1-mini",
  anthropic: "e.g. claude-haiku-4-5",
  gemini: "e.g. gemini-2.5-flash",
  openrouter: "e.g. openai/gpt-4.1-mini",
  ollama: "e.g. llama3.2",
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
const modelOptions = element<HTMLDataListElement>("model-options");
const modelHint = element<HTMLParagraphElement>("model-hint");
const keyField = element<HTMLDivElement>("key-field");
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
  const ollama = view.provider === "ollama";
  baseUrlField.hidden = !custom;
  providerHint.hidden = !ollama;
  keyField.hidden = ollama;
  modelInput.placeholder = MODEL_EXAMPLES[view.provider];
  keyLabel.textContent = keyIsOptional(view.provider)
    ? "API key (optional)"
    : "API key";
}

/** Counts list requests, so a slow answer for an old provider is dropped. */
let modelRequest = 0;

/**
 * Fills the model dropdown from the provider's own list, or the built-in
 * suggestions if it can't be fetched. The field always accepts a typed name.
 */
async function loadModels(): Promise<void> {
  const request = ++modelRequest;
  const provider = view.provider;
  const name = PROVIDER_NAMES[provider];
  const baseUrl = baseUrlInput.value.trim();
  if (provider === "custom" && !isHttpUrl(baseUrl)) {
    modelOptions.replaceChildren();
    modelHint.textContent =
      "Add the base URL to see its models, or type a model name.";
    return;
  }
  modelHint.textContent = `Loading ${name} models…`;
  let names: string[] = [];
  let fromProvider = false;
  try {
    ({ names, fromProvider } = await listModels(provider, baseUrl));
  } catch {
    // Treated like any other failure: no list, typing still works.
  }
  if (request !== modelRequest) return;
  modelOptions.replaceChildren(
    ...names.map((model) => {
      const option = document.createElement("option");
      option.value = model;
      return option;
    }),
  );
  if (fromProvider) {
    modelHint.textContent = `${names.length} ${name} models. Pick one or type any name.`;
  } else if (names.length > 0) {
    modelHint.textContent = `Couldn't load ${name}'s model list, so these are suggestions. You can type any name.`;
  } else {
    modelHint.textContent =
      "Couldn't load the model list. Type the model name exactly as the server lists it.";
  }
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
  keyInput.placeholder = keyIsOptional(view.provider)
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
function readForm(): ProviderSettings | undefined {
  const settings: ProviderSettings = {
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
  // Ollama never sends a key, so none is saved for it.
  const key = settings.provider === "ollama" ? "" : keyInput.value.trim();
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
    // A new key or base URL can unlock the provider's own list.
    void loadModels();
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
  if (target === baseUrlInput) {
    void loadModels();
    return;
  }
  if (target.name !== "provider") return;
  view.provider = selectedProvider();
  resetKeyEntry();
  clearStatus();
  renderProvider();
  renderKey();
  void loadKeyState();
  void loadModels();
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
  let settings: Settings | undefined;
  try {
    settings = await getSettings();
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
  void loadModels();
  await Promise.all([loadKeyState(), initLaunch(settings)]);
  document.body.dataset.ready = "";
}

void init();
