import { applyComponentStyles } from "./shared.js";

const LOCAL_CSS = `
:host {
  display: block;
  width: 100%;
}
.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: var(--vx-space-2);
  margin-bottom: var(--vx-space-2);
}
.toolbar button {
  box-sizing: border-box;
  min-height: var(--vx-touch-min);
  min-width: var(--vx-touch-min);
  padding: var(--vx-space-2) var(--vx-space-3);
  border-radius: var(--vx-radius-sm);
  border: 1px solid var(--vx-color-text-muted);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text);
  font: inherit;
  cursor: pointer;
}
.toolbar button:focus-visible {
  outline: 2px solid var(--vx-color-focus-ring);
  outline-offset: 2px;
}
.toolbar button[aria-pressed="true"] {
  color: var(--vx-color-on-accent);
  background: var(--vx-color-accent);
  border-color: transparent;
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

const VALID_MODES = new Set(["text", "voice", "ai", "image"]);

const MODE_LABELS = {
  text: "Texto",
  voice: "Voz",
  ai: "IA",
  image: "Imagen",
};

export class VxOmnibox extends HTMLElement {
  static get observedAttributes() {
    return ["value", "mode", "vision"];
  }

  constructor() {
    super();
    /** @type {HTMLInputElement | null} */
    this._input = null;
    /** @type {Map<string, HTMLButtonElement>} */
    this._modeButtons = new Map();
    /** @type {(() => void) | null} */
    this._onClosed = null;
    /** @type {"text" | "voice" | "ai" | "image"} */
    this._pinned = "text";
    this._built = false;
  }

  async connectedCallback() {
    if (!this.shadowRoot) {
      this.setAttribute("role", "search");
      this.setAttribute("aria-label", "Buscar");
      const root = this.attachShadow({ mode: "open" });

      const toolbar = document.createElement("div");
      toolbar.className = "toolbar";
      toolbar.setAttribute("role", "group");
      toolbar.setAttribute("aria-label", "Modo de búsqueda");

      for (const mode of ["text", "voice", "ai", "image"]) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.setAttribute("data-mode", mode);
        btn.textContent = MODE_LABELS[mode];
        btn.setAttribute("aria-pressed", "false");
        btn.addEventListener("click", () => this._onModeClick(mode));
        this._modeButtons.set(mode, btn);
        toolbar.appendChild(btn);
      }

      this._input = document.createElement("input");
      this._input.type = "search";
      this._input.setAttribute("aria-label", "Buscar");
      this._input.autocomplete = "off";

      root.appendChild(toolbar);
      root.appendChild(this._input);
      await applyComponentStyles(root, LOCAL_CSS);

      this._input.addEventListener("input", () => this._onInput());
      this._input.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
          this._clearQuery();
          e.preventDefault();
        }
      });

      this._built = true;
      this._syncPinnedFromModeAttribute();
      this._updateVisionUi();
      this._syncValueToInput();
      this._applyMode();
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

  attributeChangedCallback(name) {
    if (name === "value") {
      this._syncValueToInput();
      if (this._built) {
        this._applyMode();
      }
    } else if (name === "mode" && this._built) {
      this._syncPinnedFromModeAttribute();
      this._applyMode();
    } else if (name === "vision" && this._built) {
      this._updateVisionUi();
      this._applyMode();
    }
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

  _effectiveMode() {
    const text = this._input?.value ?? "";
    if (text.startsWith("?")) {
      return "ai";
    }
    if (!VALID_MODES.has(this._pinned)) {
      return "text";
    }
    return this._pinned;
  }

  _syncPinnedFromModeAttribute() {
    const m = this.getAttribute("mode");
    if (!m) {
      this._pinned = "text";
      return;
    }
    if (VALID_MODES.has(m)) {
      this._pinned = m;
    } else {
      this._pinned = "text";
    }
  }

  _updateVisionUi() {
    const imageBtn = this._modeButtons.get("image");
    if (!imageBtn) {
      return;
    }
    const show = this.hasAttribute("vision");
    imageBtn.hidden = !show;
    imageBtn.tabIndex = show ? 0 : -1;
    if (!show && this._pinned === "image") {
      this._pinned = "text";
    }
  }

  _applyMode() {
    const effective = this._effectiveMode();
    this.setAttribute("mode", effective);
    for (const [mode, btn] of this._modeButtons) {
      const active = mode === effective;
      btn.setAttribute("aria-pressed", active ? "true" : "false");
    }
  }

  _emitQuery() {
    const mode = this._effectiveMode();
    const text = this._input?.value ?? "";
    this.dispatchEvent(
      new CustomEvent("vx-query", {
        bubbles: true,
        composed: true,
        detail: { mode, text },
      }),
    );
  }

  _onInput() {
    if (this._pinned === "voice" || this._pinned === "image") {
      this._pinned = "text";
    }
    this._syncValueFromInput();
    this._applyMode();
    this._emitQuery();
  }

  _onModeClick(mode) {
    if (mode === "image" && !this.hasAttribute("vision")) {
      return;
    }
    this._pinned = mode;
    this._applyMode();
    this._emitQuery();
  }

  _clearQuery() {
    if (!this._input) {
      return;
    }
    this._input.value = "";
    this.setAttribute("value", "");
    this._applyMode();
    this._emitQuery();
  }
}

if (typeof customElements !== "undefined") {
  customElements.define("vx-omnibox", VxOmnibox);
}
