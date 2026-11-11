import { applyComponentStyles } from "./shared.js";

const LOCAL_CSS = `
:host {
  display: inline-block;
}
button {
  box-sizing: border-box;
  min-height: var(--vx-touch-min);
  min-width: var(--vx-touch-min);
  padding: var(--vx-space-1) var(--vx-space-3);
  border-radius: var(--vx-radius-md);
  border: 1px solid var(--vx-color-text-muted);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text);
  font: inherit;
  cursor: pointer;
}
button:focus-visible {
  outline: 2px solid var(--vx-color-focus-ring);
  outline-offset: 2px;
}
button:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
:host([selected]) button {
  border-color: var(--vx-color-accent);
  background: var(--vx-color-accent);
  color: var(--vx-color-on-accent);
}
`;

export class VxChip extends HTMLElement {
  static get observedAttributes() {
    return ["selected", "disabled", "count"];
  }

  constructor() {
    super();
    this._button = null;
  }

  async connectedCallback() {
    if (this.shadowRoot) {
      this._render();
      return;
    }
    const root = this.attachShadow({ mode: "open" });
    this._button = document.createElement("button");
    this._button.type = "button";
    this._button.addEventListener("click", () => this._onClick());
    root.appendChild(this._button);
    await applyComponentStyles(root, LOCAL_CSS);
    this._render();
  }

  attributeChangedCallback() {
    this._render();
  }

  _labelText() {
    const base = this.textContent.trim() || "Filtro";
    const count = this.getAttribute("count");
    if (count !== null && count !== "") {
      return `${base} (${count})`;
    }
    return base;
  }

  _render() {
    if (!this._button) {
      return;
    }
    this._button.disabled = this.hasAttribute("disabled");
    this._button.setAttribute(
      "aria-pressed",
      this.hasAttribute("selected") ? "true" : "false",
    );
    this._button.textContent = this._labelText();
  }

  _onClick() {
    if (this.hasAttribute("disabled")) {
      return;
    }
    const next = !this.hasAttribute("selected");
    if (next) {
      this.setAttribute("selected", "");
    } else {
      this.removeAttribute("selected");
    }
    this._render();
    this.dispatchEvent(
      new CustomEvent("vx-chip-toggle", {
        bubbles: true,
        detail: { selected: next },
      }),
    );
  }
}

customElements.define("vx-chip", VxChip);
