// Settings window: the Instruction sets section.
//
// The list shows every set in order with a radio for the active one. Picking
// a radio, saving a set, deleting and restoring each save right away. One set
// at a time opens in an inline editor. The library itself enforces the rules
// (last set, active after delete); this file only shows what Rust returns.

import { errorMessage } from "./api";
import { clearFieldError, showFieldError } from "./field-error";
import {
  addInstructionSet,
  deleteInstructionSet,
  editInstructionSet,
  getInstructionSets,
  type InstructionSetId,
  type InstructionSets,
  onInstructionSetsChanged,
  restoreStarterTemplates,
  setActiveInstructionSet,
} from "./instruction-sets-api";
import { askToSave, LEAVING_SET_EDITOR } from "./unsaved-changes";

function element<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing #${id}`);
  return found as T;
}

const list = element<HTMLDivElement>("set-list");
const addButton = element<HTMLButtonElement>("set-add");
const restoreButton = element<HTMLButtonElement>("sets-restore");
const restoreDialog = element<HTMLDialogElement>("restore-dialog");
const status = element<HTMLParagraphElement>("sets-status");
const editorTemplate = element<HTMLTemplateElement>("set-editor-template");

/** Which set the editor is open for: an id, a new set, or none. */
type Editing = InstructionSetId | "new" | null;

const view = {
  library: { sets: [], active: -1 } as InstructionSets,
  editing: null as Editing,
  draft: { name: "", instructions: "" },
  /** The draft as the editor opened, to tell whether it changed. */
  original: { name: "", instructions: "" },
  confirmingDelete: false,
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
}

function editorFor(editing: Exclude<Editing, null>): HTMLFormElement {
  const fragment = editorTemplate.content.cloneNode(true) as DocumentFragment;
  const form = fragment.querySelector("form") as HTMLFormElement;
  form.setAttribute(
    "aria-label",
    editing === "new" ? "New instruction set" : "Edit instruction set",
  );
  const name = form.elements.namedItem("name") as HTMLInputElement;
  const instructions = form.elements.namedItem(
    "instructions",
  ) as HTMLTextAreaElement;
  name.value = view.draft.name;
  instructions.value = view.draft.instructions;
  name.placeholder = "e.g. Chat with friends";
  instructions.placeholder =
    "e.g. Fix my mistakes and keep it casual. Keep the language I wrote in.";

  const deleteButton = form.querySelector<HTMLButtonElement>(
    '[data-action="delete"]',
  ) as HTMLButtonElement;
  const onlySet = view.library.sets.length <= 1;
  deleteButton.hidden = editing === "new";
  deleteButton.disabled = onlySet;
  deleteButton.title = onlySet ? "Keep at least one instruction set." : "";
  deleteButton.textContent = view.confirmingDelete
    ? "Confirm delete"
    : "Delete";
  deleteButton.classList.toggle("confirming", view.confirmingDelete);
  return form;
}

function rowFor(
  set: InstructionSets["sets"][number],
  activeId: InstructionSetId,
): HTMLDivElement {
  const row = document.createElement("div");
  row.className = "set-row";
  row.dataset.id = String(set.id);
  const active = set.id === activeId;
  if (active) row.dataset.active = "";

  const line = document.createElement("div");
  line.className = "set-line";

  const pick = document.createElement("label");
  pick.className = "set-pick";
  const radio = document.createElement("input");
  radio.type = "radio";
  radio.name = "active-set";
  radio.value = String(set.id);
  radio.checked = active;
  const name = document.createElement("span");
  name.className = "set-name";
  name.textContent = set.name;
  const badge = document.createElement("span");
  badge.className = "set-badge";
  badge.textContent = "Active";
  pick.append(radio, name, badge);

  const edit = document.createElement("button");
  edit.type = "button";
  edit.className = "button quiet";
  edit.dataset.action = "edit";
  edit.textContent = "Edit";
  edit.setAttribute("aria-expanded", String(view.editing === set.id));
  edit.setAttribute("aria-label", `Edit ${set.name}`);
  edit.hidden = view.editing === set.id;

  line.append(pick, edit);
  row.append(line);
  if (view.editing === set.id) row.append(editorFor(set.id));
  return row;
}

function render(): void {
  const { sets, active } = view.library;
  const rows = sets.map((set) => rowFor(set, active));
  if (view.editing === "new") {
    const row = document.createElement("div");
    row.className = "set-row new";
    row.append(editorFor("new"));
    rows.push(row);
  }
  list.replaceChildren(...rows);
  addButton.hidden = view.editing === "new";
}

/** Same sets, names and order: only the active set may differ. */
function sameShape(a: InstructionSets, b: InstructionSets): boolean {
  return (
    a.sets.length === b.sets.length &&
    a.sets.every(
      (set, i) => set.id === b.sets[i].id && set.name === b.sets[i].name,
    )
  );
}

/** Shows `library`. An open editor keeps its draft and focus if it can. */
function apply(library: InstructionSets): void {
  const previous = view.library;
  view.library = library;
  if (view.editing !== null && sameShape(previous, library)) {
    for (const row of list.querySelectorAll<HTMLDivElement>(".set-row")) {
      const active = row.dataset.id === String(library.active);
      row.toggleAttribute("data-active", active);
      const radio = row.querySelector<HTMLInputElement>('input[type="radio"]');
      if (radio) radio.checked = active;
    }
    return;
  }
  const editing = view.editing;
  if (
    typeof editing === "number" &&
    !library.sets.some((set) => set.id === editing)
  ) {
    closeEditor();
  }
  render();
}

function openEditor(editing: Exclude<Editing, null>): void {
  const set =
    editing === "new"
      ? undefined
      : view.library.sets.find((candidate) => candidate.id === editing);
  view.editing = editing;
  view.confirmingDelete = false;
  view.draft = {
    name: set?.name ?? "",
    instructions: set?.instructions ?? "",
  };
  view.original = { ...view.draft };
  clearStatus();
  render();
  list
    .querySelector<HTMLInputElement>('.set-editor input[name="name"]')
    ?.focus();
}

function closeEditor(): void {
  view.editing = null;
  view.confirmingDelete = false;
  view.draft = { name: "", instructions: "" };
  view.original = { name: "", instructions: "" };
}

function draftChanged(): boolean {
  return (
    view.draft.name !== view.original.name ||
    view.draft.instructions !== view.original.instructions
  );
}

/** Closes the editor and puts focus back where the user started. */
function finishEditing(focusId?: InstructionSetId): void {
  closeEditor();
  render();
  if (focusId === undefined) {
    addButton.focus();
    return;
  }
  list
    .querySelector<HTMLButtonElement>(
      `.set-row[data-id="${focusId}"] [data-action="edit"]`,
    )
    ?.focus();
}

async function run(
  action: () => Promise<InstructionSets>,
  done: string,
  attempt: string,
): Promise<boolean> {
  try {
    apply(await action());
    showStatus(done, "ok");
    return true;
  } catch (error) {
    showStatus(errorMessage(error, attempt), "error");
    return false;
  }
}

/** Saves the open editor. Resolves false if it was invalid or failed. */
async function saveEditor(form: HTMLFormElement): Promise<boolean> {
  const editing = view.editing;
  if (editing === null) return false;
  const nameInput = form.elements.namedItem("name") as HTMLInputElement;
  const name = view.draft.name.trim();
  if (name === "") {
    clearStatus();
    showFieldError(nameInput, "Give the instruction set a name.");
    nameInput.focus();
    return false;
  }
  const submit = form.querySelector<HTMLButtonElement>('[type="submit"]');
  if (submit) submit.disabled = true;
  const { instructions } = view.draft;
  const before = new Set(view.library.sets.map((set) => set.id));
  try {
    const library =
      editing === "new"
        ? await addInstructionSet(name, instructions)
        : await editInstructionSet(editing, name, instructions);
    view.library = library;
    const focusId =
      editing === "new"
        ? library.sets.find((set) => !before.has(set.id))?.id
        : editing;
    showStatus(editing === "new" ? "Instruction set added" : "Saved", "ok");
    finishEditing(focusId);
    return true;
  } catch (error) {
    showStatus(errorMessage(error, "save the instruction set"), "error");
    if (submit) submit.disabled = false;
    return false;
  }
}

async function deleteEditing(button: HTMLButtonElement): Promise<void> {
  const editing = view.editing;
  if (typeof editing !== "number") return;
  if (!view.confirmingDelete) {
    view.confirmingDelete = true;
    button.textContent = "Confirm delete";
    button.classList.add("confirming");
    return;
  }
  button.disabled = true;
  const name =
    view.library.sets.find((set) => set.id === editing)?.name ?? "Set";
  try {
    const library = await deleteInstructionSet(editing);
    closeEditor();
    apply(library);
    showStatus(`Deleted ${name}`, "ok");
    addButton.focus();
  } catch (error) {
    view.confirmingDelete = false;
    button.disabled = false;
    button.textContent = "Delete";
    button.classList.remove("confirming");
    showStatus(errorMessage(error, "delete the instruction set"), "error");
  }
}

list.addEventListener("change", (event) => {
  const target = event.target as HTMLInputElement;
  if (target.name !== "active-set") return;
  const id = Number(target.value);
  const name = view.library.sets.find((set) => set.id === id)?.name ?? "";
  void run(
    () => setActiveInstructionSet(id),
    `${name} is now active`,
    "make this set active",
  );
});

list.addEventListener("input", (event) => {
  const target = event.target as HTMLInputElement | HTMLTextAreaElement;
  if (target.name === "name") view.draft.name = target.value;
  if (target.name === "instructions") view.draft.instructions = target.value;
  clearFieldError(target);
});

list.addEventListener("click", (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
    "button[data-action]",
  );
  if (!button) return;
  switch (button.dataset.action) {
    case "edit": {
      const row = button.closest<HTMLDivElement>(".set-row");
      openEditor(Number(row?.dataset.id));
      break;
    }
    case "cancel": {
      clearStatus();
      const editing = view.editing;
      finishEditing(typeof editing === "number" ? editing : undefined);
      break;
    }
    case "delete":
      void deleteEditing(button);
      break;
  }
});

list.addEventListener("focusout", (event) => {
  const target = event.target as HTMLElement;
  if (target.dataset.action !== "delete" || !view.confirmingDelete) return;
  view.confirmingDelete = false;
  target.textContent = "Delete";
  target.classList.remove("confirming");
});

list.addEventListener("submit", (event) => {
  event.preventDefault();
  void saveEditor(event.target as HTMLFormElement);
});

list.addEventListener("keydown", (event) => {
  if (event.key !== "Escape" || view.editing === null) return;
  event.preventDefault();
  void leaveEditor();
});

addButton.addEventListener("click", () => {
  openEditor("new");
});

restoreButton.addEventListener("click", () => {
  restoreDialog.returnValue = "";
  restoreDialog.showModal();
});

restoreDialog.addEventListener("close", () => {
  if (restoreDialog.returnValue !== "restore") {
    restoreButton.focus();
    return;
  }
  closeEditor();
  void run(
    restoreStarterTemplates,
    "Default templates restored",
    "restore the starter templates",
  ).then(() => restoreButton.focus());
});

/** Whether the editor is open. It handles Esc itself. */
export function isEditingSet(): boolean {
  return view.editing !== null;
}

/**
 * Closes the editor, asking first if its draft changed: Save keeps it only
 * if the save worked, Discard drops it, Cancel stays. Resolves true once the
 * editor is closed (or wasn't open).
 */
export async function leaveEditor(): Promise<boolean> {
  const editing = view.editing;
  if (editing === null) return true;
  const focusId = typeof editing === "number" ? editing : undefined;
  if (!draftChanged()) {
    clearStatus();
    finishEditing(focusId);
    return true;
  }
  switch (await askToSave(LEAVING_SET_EDITOR)) {
    case "save": {
      const form = list.querySelector<HTMLFormElement>(".set-editor");
      return form ? saveEditor(form) : false;
    }
    case "discard":
      clearStatus();
      finishEditing(focusId);
      return true;
    case "cancel":
      return false;
  }
}

/** Loads the sets and follows changes made elsewhere, such as the tray. */
export async function initInstructionSets(): Promise<void> {
  try {
    apply(await getInstructionSets());
  } catch (error) {
    showStatus(errorMessage(error, "load your instruction sets"), "error");
  }
  try {
    await onInstructionSetsChanged(apply);
  } catch {
    // Without the event the window still shows its own changes.
  }
}
