// A field's own error line: shown under the field and named by the field's
// aria-describedby, so a screen reader reads it when the field takes focus.

type Field = HTMLInputElement | HTMLTextAreaElement;

/** The `.field-error` element among those `field` is described by. */
function errorLineOf(field: Field): HTMLElement | undefined {
  const ids = field.getAttribute("aria-describedby")?.split(/\s+/) ?? [];
  for (const id of ids) {
    const line = document.getElementById(id);
    if (line?.classList.contains("field-error")) return line;
  }
  return undefined;
}

export function showFieldError(field: Field, message: string): void {
  field.setAttribute("aria-invalid", "true");
  const line = errorLineOf(field);
  if (line) line.textContent = message;
}

export function clearFieldError(field: Field): void {
  field.removeAttribute("aria-invalid");
  const line = errorLineOf(field);
  if (line) line.textContent = "";
}
