// Typed wrappers around the settings commands in `settings_window.rs`.
// A key only ever travels from here to Rust; no command sends one back.

import { invoke } from "@tauri-apps/api/core";

export type ProviderKind = "openai" | "anthropic" | "gemini" | "custom";

export interface Settings {
  provider: ProviderKind;
  model: string;
  customBaseUrl: string;
}

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function saveSettings(newSettings: Settings): Promise<void> {
  return invoke("save_settings", { newSettings });
}

export function isKeySaved(provider: ProviderKind): Promise<boolean> {
  return invoke<boolean>("is_key_saved", { provider });
}

export function saveApiKey(provider: ProviderKind, key: string): Promise<void> {
  return invoke("save_api_key", { provider, key });
}

export function clearApiKey(provider: ProviderKind): Promise<void> {
  return invoke("clear_api_key", { provider });
}

/** Command errors arrive as a short, user-facing message. */
export function errorMessage(error: unknown): string {
  return typeof error === "string" ? error : "Something went wrong. Try again.";
}
