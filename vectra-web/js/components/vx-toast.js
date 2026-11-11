import { applyComponentStyles } from "./shared.js";

const LOCAL_CSS = `
:host {
  display: block;
  margin-top: var(--vx-space-2);
}
.region {
  padding: var(--vx-space-3);
  border-radius: var(--vx-radius-sm);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text);
  border: 1px solid var(--vx-color-text-muted);
}
:host([variant="error"]) .region {
  border-color: var(--vx-color-error);
  color: var(--vx-color-on-error);
  background: var(--vx-color-error);
}
`;

export class VxToast extends HTMLElement {
  static get observedAttributes() {
    return ["variant"];
  }

  constructor() {
    super();
    /** @type {HTMLElement | null} */
    this._region = null;
  }

  async connectedCallback() {
    if (this.shadowRoot) {
      this._syncVariant();
      return;
    }
    const root = this.attachShadow({ mode: "open" });
    const region = document.createElement("div");
    region.className = "region";
    region.setAttribute("role", "status");
    root.appendChild(region);
    this._region = region;
    await applyComponentStyles(root, LOCAL_CSS);
    this._syncVariant();
  }

  attributeChangedCallback() {
    this._syncVariant();
  }

  _syncVariant() {
    if (!this._region) {
      return;
    }
    const raw = this.getAttribute("variant");
    const isError = raw === "error";
    this._region.setAttribute("aria-live", isError ? "assertive" : "polite");
    const message = this.textContent.trim();
    this._region.textContent =
      message || (isError ? "Ocurrió un error" : "Información");
  }
}

customElements.define("vx-toast", VxToast);
