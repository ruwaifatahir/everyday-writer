// Typed wrappers around the instruction set commands in
// `instruction_set_commands.rs`. Every change returns the whole library as
// saved, so the window never guesses.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type InstructionSetId = number;

export type StarterTemplate =
  | "fix-and-make-readable"
  | "fix-mistakes-only"
  | "professional"
  | "shorter";

export interface InstructionSet {
  id: InstructionSetId;
  name: string;
  instructions: string;
  /** Present on sets installed from a starter template. */
  starter?: StarterTemplate;
}

export interface InstructionSets {
  sets: InstructionSet[];
  active: InstructionSetId;
}

export function getInstructionSets(): Promise<InstructionSets> {
  return invoke<InstructionSets>("get_instruction_sets");
}

export function addInstructionSet(
  name: string,
  instructions: string,
): Promise<InstructionSets> {
  return invoke<InstructionSets>("add_instruction_set", {
    name,
    instructions,
  });
}

export function editInstructionSet(
  id: InstructionSetId,
  name: string,
  instructions: string,
): Promise<InstructionSets> {
  return invoke<InstructionSets>("edit_instruction_set", {
    id,
    name,
    instructions,
  });
}

export function deleteInstructionSet(
  id: InstructionSetId,
): Promise<InstructionSets> {
  return invoke<InstructionSets>("delete_instruction_set", { id });
}

export function setActiveInstructionSet(
  id: InstructionSetId,
): Promise<InstructionSets> {
  return invoke<InstructionSets>("set_active_instruction_set", { id });
}

export function restoreStarterTemplates(): Promise<InstructionSets> {
  return invoke<InstructionSets>("restore_starter_templates");
}

/** Fires after every change, including ones made from the tray. */
export function onInstructionSetsChanged(
  handler: (sets: InstructionSets) => void,
): Promise<UnlistenFn> {
  return listen<InstructionSets>("instruction-sets-changed", (event) =>
    handler(event.payload),
  );
}
