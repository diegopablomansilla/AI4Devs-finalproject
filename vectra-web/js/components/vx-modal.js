import { applyComponentStyles } from "./shared.js";
import { createOverlayController } from "./overlay.js";

const LOCAL_CSS = `
:host {
  display: none;
  position: fixed;
  inset: 0;
  z-index: 100;
  align-items: center;
  justify-content: center;
}
:host([open]) {
  display: flex;
}
.backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
}
.dialog {
  position: relative;
  z-index: 1;
  max-width: 24rem;
  width: calc(100% - var(--vx-space-4));
  padding: var(--vx-space-4);
  border-radius: var(--vx-radius-md);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text);
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
  margin-right: var(--vx-space-2);
}
button:focus-visible {
  outline: 2px solid var(--vx-color-focus-ring);
  outline-offset: 2px;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--vx-space-2);
  margin-top: var(--vx-space-3);
}
`;

export class VxModal extends HTMLElement {
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
    const dialog = document.createElement("div");
    dialog.className = "dialog";
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    const close = document.createElement("button");
    close.type = "button";
    close.setAttribute("data-close", "");
    close.textContent = "Cerrar";
    const confirm = document.createElement("button");
    confirm.type = "button";
    confirm.textContent = "Confirmar";
    const actions = document.createElement("div");
    actions.className = "actions";
    actions.appendChild(close);
    actions.appendChild(confirm);
    const slot = document.createElement("slot");
    dialog.appendChild(slot);
    dialog.appendChild(actions);
    root.appendChild(backdrop);
    root.appendChild(dialog);
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

customElements.define("vx-modal", VxModal);
