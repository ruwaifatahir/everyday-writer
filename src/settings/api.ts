// Typed wrappers around the settings commands in `settings_window.rs`.
// A key only ever travels from here to Rust; no command sends one back.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type ProviderKind =
  | "openai"
  | "anthropic"
  | "gemini"
  | "openrouter"
  | "ollama"
  | "custom";

/** What a connection test found; mirrors `Connection` in `genai_rewriter.rs`. */
export interface Connection {
  /** Whether the provider lists the model; null when none was given. */
  modelFound: boolean | null;
}

export interface Settings {
  provider: ProviderKind;
  model: string;
  customBaseUrl: string;
  /** For example `Ctrl+Shift+Space`. Change it with `setShortcut`. */
  shortcut: string;
  /** Change it with `setStartWithWindows`. */
  startWithWindows: boolean;
  /** Change it with `setAutoCheck`. */
  autoCheck: boolean;
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

/** Rejects with "This shortcut is used by another app. Pick a new one." if Windows refuses it; the old one stays. */
export function setShortcut(shortcut: string): Promise<void> {
  return invoke("set_shortcut", { shortcut });
}

/** Stops the saved shortcut while the field records, so pressing it reaches the field. */
export function pauseShortcut(): Promise<void> {
  return invoke("pause_shortcut");
}

/** Brings the saved shortcut back after `pauseShortcut`; harmless if it already works. */
export function resumeShortcut(): Promise<void> {
  return invoke("resume_shortcut");
}

/** False when the saved shortcut couldn't be registered at startup. */
export function isShortcutWorking(): Promise<boolean> {
  return invoke<boolean>("is_shortcut_working");
}

/** The shortcut "Reset to default" sets. Matches `DEFAULT_SHORTCUT` in Rust. */
export const DEFAULT_SHORTCUT = "Ctrl+Shift+Space";

/** Whether the saved shortcut is the default, compared as keys, not text. */
export function isDefaultShortcut(): Promise<boolean> {
  return invoke<boolean>("is_default_shortcut");
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

/** The built-in Suggested Model names per provider, fastest first. */
export function suggestedModels(): Promise<Record<ProviderKind, string[]>> {
  return invoke<Record<ProviderKind, string[]>>("suggested_models");
}

/**
 * Checks that `provider` accepts a key without spending tokens: `key` if
 * one was typed, otherwise the saved one. Also says whether it offers
 * `model`. Rejects with a message to show as is.
 */
export function testConnection(
  provider: ProviderKind,
  customBaseUrl: string,
  key: string | null,
  model: string,
): Promise<Connection> {
  return invoke<Connection>("test_connection", {
    provider,
    customBaseUrl,
    key,
    model,
  });
}

export function clearApiKey(provider: ProviderKind): Promise<void> {
  return invoke("clear_api_key", { provider });
}

/** The running app's version, such as `1.0.0`. */
export function appVersion(): Promise<string> {
  return invoke<string>("app_version");
}

/** What Settings shows about updates; mirrors `UpdatesState` in `updates.rs`. */
export type UpdatesState =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "upToDate" }
  | { kind: "available"; version: string }
  | { kind: "checkFailed"; version?: string }
  | { kind: "downloading"; percent: number }
  | { kind: "installFailed"; version: string };

export function getUpdatesState(): Promise<UpdatesState> {
  return invoke<UpdatesState>("get_updates_state");
}

/** Starts a check; the answer arrives through `onUpdatesState`. */
export function checkForUpdates(): Promise<void> {
  return invoke("check_for_updates");
}

/** Saves the choice and starts or stops automatic checks at once. */
export function setAutoCheck(enabled: boolean): Promise<void> {
  return invoke("set_auto_check", { enabled });
}

/** Installs the offered update and restarts; progress arrives through onUpdatesState. */
export function updateNow(): Promise<void> {
  return invoke("update_now");
}

export function onUpdatesState(
  handler: (state: UpdatesState) => void,
): Promise<UnlistenFn> {
  return listen<UpdatesState>("updates-state", (event) =>
    handler(event.payload),
  );
}

/** Rust asks the open window to scroll to a section, by element id. */
export function onScrollTo(handler: (id: string) => void): Promise<UnlistenFn> {
  return listen<string>("settings-scroll-to", (event) =>
    handler(event.payload),
  );
}

/**
 * Command errors arrive as a short, user-facing message. Anything else names
 * what failed, `action` being the attempt, as in "save your settings".
 */
export function errorMessage(error: unknown, action: string): string {
  return typeof error === "string" ? error : `Couldn't ${action}. Try again.`;
}
