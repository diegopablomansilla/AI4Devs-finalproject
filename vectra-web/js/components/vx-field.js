import { applyComponentStyles } from "./shared.js";

const LOCAL_CSS = `
:host {
  display: block;
  margin-bottom: var(--vx-space-3);
}
label {
  display: block;
  margin-bottom: var(--vx-space-1);
  color: var(--vx-color-text);
}
input {
  box-sizing: border-box;
  width: 100%;
  min-height: var(--vx-touch-min);
  padding: var(--vx-space-2);
  border-radius: var(--vx-radius-sm);
  border: 1px solid var(--vx-color-text-muted);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text);
  font: inherit;
}
input:focus-visible {
  outline: 2px solid var(--vx-color-focus-ring);
  outline-offset: 2px;
}
input:disabled {
  opacity: 0.55;
}
.help {
  margin-top: var(--vx-space-1);
  color: var(--vx-color-text-muted);
  font-size: 0.875rem;
}
.error {
  margin-top: var(--vx-space-1);
  color: var(--vx-color-error);
  font-size: 0.875rem;
}
`;

export class VxField extends HTMLElement {
  static get observedAttributes() {
    return ["label", "help", "error", "disabled", "type", "name", "value", "required"];
  }

  constructor() {
    super();
    this._built = false;
  }

  async connectedCallback() {
    if (!this.shadowRoot) {
      const root = this.attachShadow({ mode: "open" });
      this._label = document.createElement("label");
      this._input = document.createElement("input");
      this._help = document.createElement("p");
      this._help.className = "help";
      this._error = document.createElement("p");
      this._error.className = "error";
      this._error.setAttribute("role", "alert");
      root.appendChild(this._label);
      root.appendChild(this._input);
      root.appendChild(this._help);
      root.appendChild(this._error);
      await applyComponentStyles(root, LOCAL_CSS);
      this._input.addEventListener("change", () => {
        this.setAttribute("value", this._input.value);
        this.dispatchEvent(
          new Event("change", { bubbles: true, composed: true }),
        );
      });
      this._built = true;
    }
    this._sync();
  }

  attributeChangedCallback() {
    this._sync();
  }

  _sync() {
    if (!this._built) {
      return;
    }
    const labelText = this.getAttribute("label") ?? "";
    const helpText = this.getAttribute("help") ?? "";
    const errorText = this.getAttribute("error") ?? "";
    const fieldId = this.getAttribute("name") || `vx-field-${Math.random().toString(36).slice(2, 9)}`;

    this._label.textContent = labelText;
    this._label.setAttribute("for", fieldId);
    this._input.id = fieldId;
    this._input.name = this.getAttribute("name") ?? fieldId;
    this._input.type = this.getAttribute("type") || "text";
    if (this.hasAttribute("value")) {
      this._input.value = this.getAttribute("value");
    }
    this._input.disabled = this.hasAttribute("disabled");
    this._input.required = this.hasAttribute("required");

    this._help.textContent = helpText;
    this._help.hidden = helpText.length === 0;
    this._help.id = `${fieldId}-help`;

    this._error.textContent = errorText;
    this._error.hidden = errorText.length === 0;
    this._error.id = `${fieldId}-error`;

    const described = [];
    if (helpText) {
      described.push(this._help.id);
    }
    if (errorText) {
      described.push(this._error.id);
    }
    if (described.length > 0) {
      this._input.setAttribute("aria-describedby", described.join(" "));
    } else {
      this._input.removeAttribute("aria-describedby");
    }

    if (errorText) {
      this._input.setAttribute("aria-invalid", "true");
    } else {
      this._input.removeAttribute("aria-invalid");
    }
  }
}

customElements.define("vx-field", VxField);
