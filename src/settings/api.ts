// Typed wrappers around the settings commands in `settings_window.rs`.
// A key only ever travels from here to Rust; no command sends one back.

import { invoke } from "@tauri-apps/api/core";

export type ProviderKind =
  | "openai"
  | "anthropic"
  | "gemini"
  | "openrouter"
  | "ollama"
  | "custom";

export interface ModelList {
  names: string[];
  /** False when these are built-in suggestions because the list couldn't be fetched. */
  fromProvider: boolean;
}

export interface Settings {
  provider: ProviderKind;
  model: string;
  customBaseUrl: string;
  /** For example `Ctrl+Shift+Space`. Change it with `setShortcut`. */
  shortcut: string;
  /** Change it with `setStartWithWindows`. */
  startWithWindows: boolean;
}

/** What the Save button saves. The rest has its own commands. */
export type ProviderSettings = Pick<
  Settings,
  "provider" | "model" | "customBaseUrl"
>;

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function saveSettings(newSettings: ProviderSettings): Promise<void> {
  return invoke("save_settings", { newSettings });
}

/** Rejects with "This shortcut is used by another app" if Windows refuses it; the old one stays. */
export function setShortcut(shortcut: string): Promise<void> {
  return invoke("set_shortcut", { shortcut });
}

/** False when the saved shortcut couldn't be registered at startup. */
export function isShortcutWorking(): Promise<boolean> {
  return invoke<boolean>("is_shortcut_working");
}

export function setStartWithWindows(enabled: boolean): Promise<void> {
  return invoke("set_start_with_windows", { enabled });
}

export function isKeySaved(provider: ProviderKind): Promise<boolean> {
  return invoke<boolean>("is_key_saved", { provider });
}

export function saveApiKey(provider: ProviderKind, key: string): Promise<void> {
  return invoke("save_api_key", { provider, key });
}

/** Never rejects for a provider problem: falls back to built-in suggestions. */
export function listModels(
  provider: ProviderKind,
  customBaseUrl: string,
): Promise<ModelList> {
  return invoke<ModelList>("list_models", { provider, customBaseUrl });
}

export function clearApiKey(provider: ProviderKind): Promise<void> {
  return invoke("clear_api_key", { provider });
}

/** Command errors arrive as a short, user-facing message. */
export function errorMessage(error: unknown): string {
  return typeof error === "string" ? error : "Something went wrong. Try again.";
}
