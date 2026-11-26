import { applyComponentStyles } from "./shared.js";
import { createOverlayController } from "./overlay.js";

const LOCAL_CSS = `
:host {
  display: none;
  position: fixed;
  inset: 0;
  z-index: 100;
  justify-content: flex-end;
}
:host([open]) {
  display: flex;
}
.backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
}
.panel {
  position: relative;
  z-index: 1;
  height: 100%;
  width: min(22rem, 100%);
  padding: var(--vx-space-4);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text);
  box-shadow: -4px 0 24px rgba(0, 0, 0, 0.35);
}
button {
  box-sizing: border-box;
  min-height: var(--vx-touch-min);
  min-width: var(--vx-touch-min);
  padding: var(--vx-space-2) var(--vx-space-3);
  border-radius: var(--vx-radius-sm);
  border: 1px solid var(--vx-color-text-muted);
  font: inherit;
  color: var(--vx-color-text);
  background: transparent;
  cursor: pointer;
}
button:focus-visible {
  outline: 2px solid var(--vx-color-focus-ring);
  outline-offset: 2px;
}
`;

export class VxSidePanel extends HTMLElement {
  constructor() {
    super();
    /** @type {ReturnType<typeof createOverlayController> | null} */
    this._controller = null;
  }

  async connectedCallback() {
    if (this.shadowRoot) {
      return;
    }
    const root = this.attachShadow({ mode: "open" });
    const backdrop = document.createElement("div");
    backdrop.className = "backdrop";
    const panel = document.createElement("div");
    panel.className = "panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");
    const close = document.createElement("button");
    close.type = "button";
    close.setAttribute("data-close", "");
    close.textContent = "Cerrar panel";
    const slot = document.createElement("slot");
    panel.appendChild(slot);
    panel.appendChild(close);
    root.appendChild(backdrop);
    root.appendChild(panel);
    await applyComponentStyles(root, LOCAL_CSS);

    this._controller = createOverlayController(this, {
      getDialogRoot: () => this.shadowRoot,
    });
    close.addEventListener("click", () => this.hide());
  }

  /**
   * @param {HTMLElement | undefined} opener
   */
  show(opener) {
    this._controller?.show(opener);
  }

  hide() {
    this._controller?.hide();
  }
}

customElements.define("vx-side-panel", VxSidePanel);
