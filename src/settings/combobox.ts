// The two pickers on the provider card, after the WAI-ARIA APG combobox
// pattern: the provider is a select-only combobox (a button), the model an
// editable one (a text field whose typed value always counts).
//
// Focus never leaves the field: the active option is named with
// aria-activedescendant. The list is a manual popover, so it sits in the top
// layer, clear of the card's `overflow: hidden`, and is placed under the
// field (above it, if there's no room below).

export interface Option {
  value: string;
  label: string;
  /** A short second line, such as "On this PC". */
  detail?: string;
  /** Id of a `<symbol>` in the page's logo sprite. */
  icon?: string;
  /** Heading of the group the option is listed under. */
  group?: string;
  /** Starts a new block after a divider. */
  divided?: boolean;
}

const SVG = "http://www.w3.org/2000/svg";

/** Gap between the field and its list. */
const GAP = 4;

/** The tallest a list gets, about eight rows; longer ones scroll. */
const MAX_HEIGHT = 320;

let lists = 0;

function icon(id: string, className: string): SVGSVGElement {
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("class", className);
  svg.setAttribute("aria-hidden", "true");
  const use = document.createElementNS(SVG, "use");
  use.setAttribute("href", `#${id}`);
  svg.append(use);
  return svg;
}

/** True for a key that types a character, so the list can jump to it. */
function isPrintable(event: KeyboardEvent): boolean {
  return (
    event.key.length === 1 && !event.ctrlKey && !event.altKey && !event.metaKey
  );
}

/**
 * The list half of a combobox: renders options, tracks the active one,
 * and opens anchored to `anchor`. The field half lives in the two classes
 * below.
 */
class Popup {
  readonly element: HTMLDivElement;
  private options: Option[] = [];
  private rows: HTMLElement[] = [];
  private active = -1;
  private readonly id = `combo-${++lists}`;

  constructor(
    private readonly anchor: HTMLElement,
    private readonly field: HTMLElement,
    label: string,
    private readonly onPick: (value: string) => void,
  ) {
    this.element = document.createElement("div");
    this.element.id = this.id;
    this.element.className = "combo-list";
    this.element.setAttribute("role", "listbox");
    this.element.setAttribute("aria-label", label);
    this.element.popover = "manual";
    // Keep focus in the field while the mouse picks.
    this.element.addEventListener("mousedown", (event) =>
      event.preventDefault(),
    );
    this.element.addEventListener("click", (event) => {
      const row = (event.target as Element).closest<HTMLElement>(
        "[role=option]",
      );
      const index = row ? this.rows.indexOf(row) : -1;
      const option = this.options[index];
      if (option) this.onPick(option.value);
    });
    document.body.append(this.element);
    field.setAttribute("aria-controls", this.id);
    field.setAttribute("aria-expanded", "false");

    window.addEventListener("resize", () => this.close());
    document.addEventListener(
      "scroll",
      (event) => {
        if (this.isOpen && !this.element.contains(event.target as Node)) {
          this.close();
        }
      },
      true,
    );
    document.addEventListener("pointerdown", (event) => {
      const target = event.target as Node;
      if (
        this.isOpen &&
        !this.element.contains(target) &&
        !this.anchor.contains(target)
      ) {
        this.close();
      }
    });
  }

  get isOpen(): boolean {
    return this.element.matches(":popover-open");
  }

  get count(): number {
    return this.options.length;
  }

  get activeOption(): Option | undefined {
    return this.options[this.active];
  }

  /**
   * Lists `options`, checking the one whose value is `chosen`. `more` is a
   * closing note, such as how many were left out.
   */
  render(options: Option[], chosen: string, more = ""): void {
    this.options = options;
    this.active = -1;
    const children: HTMLElement[] = [];
    this.rows = [];
    let group: string | undefined;
    options.forEach((option, index) => {
      if (option.divided && index > 0) {
        const divider = document.createElement("div");
        divider.className = "combo-divider";
        divider.setAttribute("role", "separator");
        children.push(divider);
      }
      if (option.group !== undefined && option.group !== group) {
        group = option.group;
        const heading = document.createElement("div");
        heading.className = "combo-group";
        heading.setAttribute("role", "presentation");
        heading.textContent = group;
        children.push(heading);
      }
      const row = document.createElement("div");
      row.id = `${this.id}-${index}`;
      row.className = "combo-option";
      row.setAttribute("role", "option");
      const selected = option.value === chosen;
      row.setAttribute("aria-selected", String(selected));
      if (option.icon) row.append(icon(option.icon, "combo-icon"));
      const text = document.createElement("span");
      text.className = "combo-label";
      text.textContent = option.label;
      row.append(text);
      if (option.detail) {
        const detail = document.createElement("span");
        detail.className = "combo-detail";
        detail.textContent = option.detail;
        row.append(detail);
      }
      if (selected) row.append(icon("check", "combo-check"));
      this.rows.push(row);
      children.push(row);
    });
    if (more) {
      const note = document.createElement("div");
      note.className = "combo-note";
      note.setAttribute("role", "presentation");
      note.textContent = more;
      children.push(note);
    }
    this.element.replaceChildren(...children);
    this.field.removeAttribute("aria-activedescendant");
  }

  open(): void {
    if (this.options.length === 0 && !this.element.firstChild) return;
    if (!this.isOpen) this.element.showPopover();
    this.field.setAttribute("aria-expanded", "true");
    this.place();
  }

  close(): void {
    if (this.isOpen) this.element.hidePopover();
    this.field.setAttribute("aria-expanded", "false");
    this.setActive(-1);
  }

  /** Makes option `index` active (-1 for none) and scrolls it into view. */
  setActive(index: number): void {
    this.rows[this.active]?.removeAttribute("data-active");
    this.active = index;
    const row = this.rows[index];
    if (row) {
      row.dataset.active = "";
      row.scrollIntoView({ block: "nearest" });
      this.field.setAttribute("aria-activedescendant", row.id);
    } else {
      this.field.removeAttribute("aria-activedescendant");
    }
  }

  /** Moves the active option by `step`, stopping at either end. */
  move(step: number): void {
    if (this.options.length === 0) return;
    const from =
      this.active === -1 && step < 0 ? this.options.length : this.active;
    const to = Math.min(Math.max(from + step, 0), this.options.length - 1);
    this.setActive(to);
  }

  first(): void {
    this.setActive(this.options.length > 0 ? 0 : -1);
  }

  last(): void {
    this.setActive(this.options.length - 1);
  }

  /** Makes the option with `value` active, if it's listed. */
  activate(value: string): void {
    const index = this.options.findIndex((option) => option.value === value);
    if (index !== -1) this.setActive(index);
  }

  /** Jumps to the next option whose label starts with `prefix`. */
  typeAhead(prefix: string): void {
    const wanted = prefix.toLowerCase();
    const count = this.options.length;
    const start =
      prefix.length === 1 ? this.active + 1 : Math.max(this.active, 0);
    for (let step = 0; step < count; step++) {
      const index = (start + step) % count;
      if (this.options[index]?.label.toLowerCase().startsWith(wanted)) {
        this.setActive(index);
        return;
      }
    }
  }

  /** Under the anchor at its width, or above it if there's no room below. */
  private place(): void {
    const box = this.anchor.getBoundingClientRect();
    const list = this.element;
    list.style.left = `${box.left}px`;
    list.style.width = `${box.width}px`;
    list.style.maxHeight = "";
    const below = window.innerHeight - box.bottom - GAP * 2;
    const above = box.top - GAP * 2;
    const wanted = Math.min(list.scrollHeight + 4, MAX_HEIGHT);
    if (wanted <= below || below >= above) {
      list.style.top = `${box.bottom + GAP}px`;
      list.style.bottom = "";
      list.style.maxHeight = `${Math.min(Math.max(below, 120), MAX_HEIGHT)}px`;
      list.dataset.side = "below";
    } else {
      list.style.top = "";
      list.style.bottom = `${window.innerHeight - box.top + GAP}px`;
      list.style.maxHeight = `${Math.min(above, MAX_HEIGHT)}px`;
      list.dataset.side = "above";
    }
  }
}

/**
 * A select-only combobox on a button: picks one of a few fixed options.
 * Keys follow the APG: arrows, Home and End, Enter or Space to pick, Esc to
 * close, and typing jumps to a matching option.
 */
export class Select {
  private readonly popup: Popup;
  private options: Option[] = [];
  private chosen = "";
  private typed = "";
  private typedTimer: number | undefined;

  constructor(
    private readonly button: HTMLButtonElement,
    label: string,
    private readonly onChange: (value: string) => void,
  ) {
    button.setAttribute("role", "combobox");
    button.setAttribute("aria-haspopup", "listbox");
    this.popup = new Popup(button, button, label, (value) => {
      this.pick(value);
    });
    button.addEventListener("click", () => {
      if (this.popup.isOpen) {
        this.popup.close();
      } else {
        this.openAt(this.chosen);
      }
    });
    button.addEventListener("keydown", (event) => this.onKey(event));
    button.addEventListener("blur", () => this.popup.close());
  }

  get value(): string {
    return this.chosen;
  }

  setOptions(options: Option[]): void {
    this.options = options;
    this.popup.render(options, this.chosen);
  }

  /** Shows `value` as chosen, without telling `onChange`. */
  set(value: string): void {
    const option = this.options.find((candidate) => candidate.value === value);
    if (!option) return;
    this.chosen = value;
    this.button.replaceChildren();
    if (option.icon) this.button.append(icon(option.icon, "combo-icon"));
    const text = document.createElement("span");
    text.className = "combo-value";
    text.textContent = option.label;
    this.button.append(text, icon("chevron", "combo-chevron"));
    this.popup.render(this.options, value);
  }

  private pick(value: string): void {
    this.popup.close();
    if (value === this.chosen) return;
    this.set(value);
    this.onChange(value);
  }

  private openAt(value: string): void {
    this.popup.open();
    this.popup.activate(value);
  }

  private onKey(event: KeyboardEvent): void {
    const open = this.popup.isOpen;
    const key = event.key;
    if (!open) {
      if (["ArrowDown", "ArrowUp", "Enter", " "].includes(key)) {
        this.openAt(this.chosen);
      } else if (key === "Home" || key === "End") {
        this.popup.open();
        if (key === "Home") this.popup.first();
        else this.popup.last();
      } else if (isPrintable(event)) {
        this.popup.open();
        this.typeAhead(event.key);
      } else {
        return;
      }
      event.preventDefault();
      return;
    }
    switch (key) {
      case "ArrowDown":
        if (event.altKey) return;
        this.popup.move(1);
        break;
      case "ArrowUp":
        if (event.altKey) {
          this.pickActive();
          break;
        }
        this.popup.move(-1);
        break;
      case "Home":
        this.popup.first();
        break;
      case "End":
        this.popup.last();
        break;
      case "Enter":
      case " ":
        if (key === " " && this.typed !== "") {
          this.typeAhead(" ");
          break;
        }
        this.pickActive();
        break;
      case "Escape":
        this.popup.close();
        break;
      case "Tab":
        this.pickActive();
        return;
      default:
        if (!isPrintable(event)) return;
        this.typeAhead(event.key);
    }
    event.preventDefault();
  }

  private pickActive(): void {
    const option = this.popup.activeOption;
    if (option) this.pick(option.value);
    else this.popup.close();
  }

  private typeAhead(character: string): void {
    window.clearTimeout(this.typedTimer);
    this.typed += character;
    this.popup.typeAhead(this.typed);
    this.typedTimer = window.setTimeout(() => {
      this.typed = "";
    }, 500);
  }
}

/** How many model rows are drawn at most; typing narrows the rest. */
const MAX_ROWS = 200;

/**
 * An editable combobox on a text field: suggests from a long list, filtered
 * as you type, and the typed text always counts as the value. Nothing is
 * picked on blur.
 */
export class Suggest {
  private readonly popup: Popup;
  private names: string[] = [];
  private recommended: string[] = [];
  /** True once the user has typed since the list opened. */
  private filtering = false;
  /** True while a pick sets the field, so its `input` event is ignored. */
  private picking = false;

  constructor(
    private readonly input: HTMLInputElement,
    toggle: HTMLButtonElement,
    label: string,
  ) {
    input.setAttribute("role", "combobox");
    input.setAttribute("aria-autocomplete", "list");
    const anchor = input.parentElement ?? input;
    this.popup = new Popup(anchor, input, label, (value) => this.pick(value));
    input.addEventListener("input", () => {
      if (this.picking) return;
      this.filtering = true;
      this.show();
    });
    input.addEventListener("keydown", (event) => this.onKey(event));
    input.addEventListener("blur", () => this.popup.close());
    input.addEventListener("click", () => {
      if (!this.popup.isOpen) this.openAll();
    });
    // The arrow button opens the full list; it never takes focus.
    toggle.addEventListener("mousedown", (event) => event.preventDefault());
    toggle.addEventListener("click", () => {
      if (this.popup.isOpen) {
        this.popup.close();
      } else {
        input.focus();
        this.openAll();
      }
    });
  }

  /** Lists `names`, with `recommended` first under their own heading. */
  setNames(names: string[], recommended: string[]): void {
    this.names = names;
    this.recommended = recommended.filter((name) => names.includes(name));
    if (this.popup.isOpen) this.show();
  }

  private openAll(): void {
    this.filtering = false;
    this.show();
  }

  /** Renders the names that match the typed text, and opens or closes. */
  private show(): void {
    const typed = this.input.value.trim();
    const query = this.filtering ? typed.toLowerCase() : "";
    const matches = (name: string) => name.toLowerCase().includes(query);
    const top = this.recommended.filter(matches);
    const rest = this.names.filter(
      (name) => matches(name) && !this.recommended.includes(name),
    );
    const grouped = top.length > 0;
    const options: Option[] = [
      ...top.map((name) => ({
        value: name,
        label: name,
        group: "Recommended",
      })),
      ...rest.slice(0, MAX_ROWS).map((name) => ({
        value: name,
        label: name,
        group: grouped ? "All models" : undefined,
      })),
    ];
    const listed = options.length > 0;
    if (!listed && typed !== "") {
      options.push({
        value: typed,
        label: `Use "${typed}"`,
        detail: "Not in the list",
      });
    }
    const hidden = rest.length - MAX_ROWS;
    const more = hidden > 0 ? `${hidden} more. Type to narrow the list.` : "";
    this.popup.render(options, listed ? typed : "", more);
    if (options.length === 0) {
      this.popup.close();
      return;
    }
    this.popup.open();
    if (!this.filtering) this.popup.activate(typed);
  }

  private pick(value: string): void {
    this.popup.close();
    this.input.value = value;
    // Tells the form, as typing would; the list stays closed.
    this.picking = true;
    this.input.dispatchEvent(new Event("input", { bubbles: true }));
    this.picking = false;
    this.input.setSelectionRange(value.length, value.length);
  }

  private onKey(event: KeyboardEvent): void {
    const open = this.popup.isOpen;
    switch (event.key) {
      case "ArrowDown":
        if (!open) {
          this.openAll();
          if (!event.altKey && !this.popup.activeOption) this.popup.first();
        } else if (!event.altKey) {
          this.popup.move(1);
        }
        break;
      case "ArrowUp":
        if (!open) return;
        if (event.altKey) {
          this.popup.close();
          break;
        }
        this.popup.move(-1);
        break;
      case "Enter": {
        // A closed list leaves Enter to the form, so it saves.
        if (!open) return;
        const option = this.popup.activeOption;
        if (option) this.pick(option.value);
        else this.popup.close();
        break;
      }
      case "Escape":
        // A closed list leaves Esc to the window, so it closes.
        if (!open) return;
        this.popup.close();
        break;
      default:
        return;
    }
    event.preventDefault();
  }
}
