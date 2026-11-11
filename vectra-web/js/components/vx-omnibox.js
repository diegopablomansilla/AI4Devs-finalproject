import { applyComponentStyles } from "./shared.js";

const LOCAL_CSS = `
:host {
  display: block;
  width: 100%;
}
input {
  box-sizing: border-box;
  width: 100%;
  min-height: var(--vx-touch-min);
  min-width: var(--vx-touch-min);
  padding: var(--vx-space-2) var(--vx-space-3);
  border-radius: var(--vx-radius-md);
  border: 1px solid var(--vx-color-text-muted);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text);
  font: inherit;
}
input:focus-visible {
  outline: 2px solid var(--vx-color-focus-ring);
  outline-offset: 2px;
}
`;

export class VxOmnibox extends HTMLElement {
  static get observedAttributes() {
    return ["value"];
  }

  constructor() {
    super();
    /** @type {HTMLInputElement | null} */
    this._input = null;
    /** @type {(() => void) | null} */
    this._onClosed = null;
    this._built = false;
  }

  async connectedCallback() {
    if (!this.shadowRoot) {
      this.setAttribute("role", "search");
      this.setAttribute("aria-label", "Buscar");
      const root = this.attachShadow({ mode: "open" });
      this._input = document.createElement("input");
      this._input.type = "search";
      this._input.setAttribute("aria-label", "Buscar");
      this._input.autocomplete = "off";
      root.appendChild(this._input);
      await applyComponentStyles(root, LOCAL_CSS);
      this._input.addEventListener("input", () => this._syncValueFromInput());
      this._input.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
          this._clearQuery();
          e.preventDefault();
        }
      });
      this._built = true;
      this._syncValueToInput();
    }
    this._input?.focus();
    if (!this._onClosed) {
      this._onClosed = () => {
        queueMicrotask(() => {
          if (this.isConnected && this._input) {
            this._input.focus();
          }
        });
      };
      document.addEventListener("vx-closed", this._onClosed);
    }
  }

  disconnectedCallback() {
    if (this._onClosed) {
      document.removeEventListener("vx-closed", this._onClosed);
      this._onClosed = null;
    }
  }

  attributeChangedCallback() {
    this._syncValueToInput();
  }

  _syncValueToInput() {
    if (!this._built || !this._input) {
      return;
    }
    const attr = this.getAttribute("value");
    if (attr !== null) {
      this._input.value = attr;
    }
  }

  _syncValueFromInput() {
    if (!this._input) {
      return;
    }
    const v = this._input.value;
    if (v === "") {
      this.setAttribute("value", "");
    } else {
      this.setAttribute("value", v);
    }
  }

  _clearQuery() {
    if (!this._input) {
      return;
    }
    this._input.value = "";
    this.setAttribute("value", "");
  }

}

if (typeof customElements !== "undefined") {
  customElements.define("vx-omnibox", VxOmnibox);
}
