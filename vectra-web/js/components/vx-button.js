import { applyComponentStyles } from "./shared.js";

const LOCAL_CSS = `
:host {
  display: inline-block;
}
button {
  box-sizing: border-box;
  min-height: var(--vx-touch-min);
  min-width: var(--vx-touch-min);
  padding: var(--vx-space-2) var(--vx-space-3);
  border-radius: var(--vx-radius-sm);
  border: 1px solid transparent;
  font: inherit;
  color: var(--vx-color-on-accent);
  background: var(--vx-color-accent);
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
:host([variant="ghost"]) button {
  color: var(--vx-color-text);
  background: transparent;
  border-color: var(--vx-color-text-muted);
}
:host([variant="danger"]) button {
  color: var(--vx-color-on-error);
  background: var(--vx-color-error);
}
`;

export class VxButton extends HTMLElement {
  static get observedAttributes() {
    return ["variant", "disabled", "loading", "type"];
  }

  constructor() {
    super();
    /** @type {HTMLButtonElement | null} */
    this._button = null;
    this._ready = false;
  }

  async connectedCallback() {
    if (this.shadowRoot) {
      this._renderState();
      return;
    }
    const root = this.attachShadow({ mode: "open" });
    this._button = document.createElement("button");
    root.appendChild(this._button);
    await applyComponentStyles(root, LOCAL_CSS);
    this._ready = true;
    this._renderState();
  }

  attributeChangedCallback() {
    this._renderState();
  }

  _renderState() {
    if (!this._button) {
      return;
    }
    const loading = this.hasAttribute("loading");
    const disabled = this.hasAttribute("disabled") || loading;
    this._button.disabled = disabled;
    this._button.type = this.getAttribute("type") || "button";
    if (loading) {
      this._button.setAttribute("aria-busy", "true");
    } else {
      this._button.removeAttribute("aria-busy");
    }
    const label = this.textContent.trim();
    this._button.textContent = label || "Botón";
  }
}

customElements.define("vx-button", VxButton);
