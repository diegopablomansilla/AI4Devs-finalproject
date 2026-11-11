/** Minimal DOM for custom element tests in Node (no jsdom). */

export class CSSStyleSheetStub {
  constructor() {
    this.cssRules = [];
    this._text = "";
  }

  replaceSync(text) {
    this._text = text;
    return this;
  }

  async replace(text) {
    this._text = text;
    return this;
  }
}

class EventTargetStub {
  constructor() {
    this._listeners = new Map();
  }

  addEventListener(type, fn) {
    const list = this._listeners.get(type) ?? [];
    list.push(fn);
    this._listeners.set(type, list);
  }

  removeEventListener(type, fn) {
    const list = this._listeners.get(type) ?? [];
    this._listeners.set(
      type,
      list.filter((f) => f !== fn),
    );
  }

  /** @param {FakeNode | DocumentStub | FakeShadowRoot} node */
  _fireListeners(node, event) {
    const listeners = node._listeners?.get(event.type) ?? [];
    for (const fn of [...listeners]) {
      fn.call(node, event);
    }
  }

  dispatchEvent(event) {
    this._fireListeners(this, event);
    if (!event.bubbles) {
      return !event.defaultPrevented;
    }
    let current = this;
    const doc = globalThis.document;
    for (;;) {
      /** @type {FakeNode | DocumentStub | FakeShadowRoot | null | undefined} */
      let parent = current.parentNode ?? null;
      if (parent && parent.host) {
        if (!event.composed) {
          break;
        }
        parent = parent.host;
      } else if (!parent && current === doc?.body && doc) {
        parent = doc;
      }
      if (!parent) {
        break;
      }
      current = parent;
      this._fireListeners(current, event);
    }
    return !event.defaultPrevented;
  }
}

class FakeShadowRoot extends EventTargetStub {
  constructor(host) {
    super();
    this.host = host;
    this.adoptedStyleSheets = [];
    this.innerHTML = "";
    this._children = [];
  }

  appendChild(node) {
    this._children.push(node);
    if (node.parentNode !== this) {
      node.parentNode = this;
    }
    return node;
  }

  _descendants() {
    /** @type {FakeNode[]} */
    const out = [];
    const walk = (node) => {
      const kids = node._children ?? [];
      for (const child of kids) {
        out.push(child);
        walk(child);
      }
    };
    walk(this);
    return out;
  }

  _matchSelector(node, selector) {
    const sel = String(selector);
    if (selector === "button") {
      return node.tagName === "BUTTON";
    }
    if (sel.includes("button") && sel.includes(":not")) {
      return node.tagName === "BUTTON" && !node.disabled;
    }
    if (selector === "input") {
      return node.tagName === "INPUT";
    }
    if (selector === '[role="alert"]') {
      return node.getAttribute("role") === "alert";
    }
    if (selector === '[role="dialog"]') {
      return node.getAttribute("role") === "dialog";
    }
    if (selector === "[data-close]") {
      return node.tagName === "BUTTON" && node.hasAttribute("data-close");
    }
    if (selector === "[aria-live]") {
      return node.hasAttribute("aria-live");
    }
    if (selector === "img") {
      return node.tagName === "IMG";
    }
    if (selector === '[data-field="image-placeholder"]') {
      return node.getAttribute("data-field") === "image-placeholder";
    }
    const dataFieldMatch = selector.match(/^\[data-field="([^"]+)"\]$/);
    if (dataFieldMatch) {
      return node.getAttribute("data-field") === dataFieldMatch[1];
    }
    if (selector === "[data-field]") {
      return node.hasAttribute("data-field");
    }
    const dataModeMatch = selector.match(/^\[data-mode="([^"]+)"\]$/);
    if (dataModeMatch) {
      return node.getAttribute("data-mode") === dataModeMatch[1];
    }
    return false;
  }

  querySelector(selector) {
    return this._descendants().find((n) => this._matchSelector(n, selector)) ?? null;
  }

  querySelectorAll(selector) {
    return this._descendants().filter((n) => this._matchSelector(n, selector));
  }
}

class FakeNode extends EventTargetStub {
  constructor(tagName = "DIV") {
    super();
    this.tagName = tagName;
    this.parentNode = null;
    this.isConnected = true;
    this._attrs = {};
    this._text = "";
    this.shadowRoot = null;
    this.className = "";
    this.id = "";
    this.disabled = false;
    this.type = "button";
    this.hidden = false;
    this.value = "";
  }

  get textContent() {
    return this._text;
  }

  set textContent(value) {
    this._text = String(value ?? "");
  }

  focus() {
    if (globalThis.document?._setActiveElement) {
      globalThis.document._setActiveElement(this);
    }
  }

  _notifyAttributeChanged(name, oldValue, newValue) {
    if (typeof this.attributeChangedCallback !== "function") {
      return;
    }
    const ctor = /** @type {{ observedAttributes?: string[] }} */ (this.constructor);
    const observed = ctor.observedAttributes;
    if (!observed || !observed.includes(name)) {
      return;
    }
    this.attributeChangedCallback(name, oldValue, newValue);
  }

  setAttribute(name, value) {
    const oldValue = this.hasAttribute(name) ? this.getAttribute(name) : null;
    if (name === "disabled" && (value === "" || value === "disabled")) {
      this.disabled = true;
    }
    if (name === "open" && (value === "" || value === "open")) {
      this._attrs.open = "";
    } else {
      this._attrs[name] = String(value);
    }
    const newValue = this.getAttribute(name);
    if (oldValue !== newValue) {
      this._notifyAttributeChanged(name, oldValue, newValue);
    }
  }

  getAttribute(name) {
    if (name === "disabled" && this.disabled) {
      return "";
    }
    if (name === "open" && "open" in this._attrs) {
      return "";
    }
    return this._attrs[name] ?? null;
  }

  hasAttribute(name) {
    if (name === "disabled") {
      return this.disabled;
    }
    if (name === "open") {
      return "open" in this._attrs;
    }
    return name in this._attrs;
  }

  removeAttribute(name) {
    const oldValue = this.hasAttribute(name) ? this.getAttribute(name) : null;
    if (name === "disabled") {
      this.disabled = false;
    }
    if (name === "open") {
      delete this._attrs.open;
    }
    delete this._attrs[name];
    const newValue = this.getAttribute(name);
    if (oldValue !== newValue) {
      this._notifyAttributeChanged(name, oldValue, newValue);
    }
  }

  appendChild(node) {
    if (!this._children) {
      this._children = [];
    }
    this._children.push(node);
    node.parentNode = this;
    node.isConnected = true;
    return node;
  }

  removeChild(node) {
    const kids = this._children ?? [];
    const idx = kids.indexOf(node);
    if (idx !== -1) {
      kids.splice(idx, 1);
    }
    node.parentNode = null;
    node.isConnected = false;
    return node;
  }

  attachShadow() {
    this.shadowRoot = new FakeShadowRoot(this);
    return this.shadowRoot;
  }

  click() {
    if (this.disabled) {
      return;
    }
    const event = new Event("click", { bubbles: true });
    this.dispatchEvent(event);
    if (this.parentNode && this.parentNode.host) {
      this.parentNode.host.dispatchEvent(new Event("click", { bubbles: true }));
    }
  }

  querySelector(selector) {
    const sel = String(selector);
    if (sel === '[slot="name"]') {
      const kids = this._children ?? [];
      return kids.find((c) => c.getAttribute("slot") === "name") ?? null;
    }
    return null;
  }
}

export class Event {
  constructor(type, init = {}) {
    this.type = type;
    this.bubbles = init.bubbles ?? false;
    this.cancelable = init.cancelable ?? false;
    this.composed = init.composed ?? false;
    this.defaultPrevented = false;
    this.detail = init.detail;
  }

  preventDefault() {
    this.defaultPrevented = true;
  }
}

export class CustomEvent extends Event {
  constructor(type, init = {}) {
    super(type, init);
    this.detail = init.detail;
  }
}

export class KeyboardEvent extends Event {
  constructor(type, init = {}) {
    super(type, init);
    this.key = init.key ?? "";
    this.shiftKey = init.shiftKey ?? false;
  }
}

export class HTMLElement extends FakeNode {
  constructor() {
    super("DIV");
  }
}

class DocumentStub extends EventTargetStub {
  constructor() {
    super();
    this.activeElement = null;
    this.body = new FakeNode("BODY");
    this.body.parentNode = this;
    this.visibilityState = "visible";
  }

  createElement(tag) {
    const name = String(tag);
    const node = new FakeNode(name.toUpperCase());
    node.localName = name.toLowerCase();
    return node;
  }

  _setActiveElement(el) {
    this.activeElement = el;
  }

  appendChild(node) {
    return this.body.appendChild(node);
  }

  _descendants() {
    /** @type {FakeNode[]} */
    const out = [];
    const walk = (node) => {
      const kids = node._children ?? [];
      for (const child of kids) {
        out.push(child);
        walk(child);
      }
    };
    walk(this.body);
    return out;
  }

  _matchSelector(node, selector) {
    const sel = String(selector);
    if (sel === "[data-health]") {
      return node.hasAttribute("data-health");
    }
    if (sel === "vx-omnibox") {
      return node.localName === "vx-omnibox";
    }
    return false;
  }

  querySelector(selector) {
    return this._descendants().find((n) => this._matchSelector(n, selector)) ?? null;
  }

  querySelectorAll(selector) {
    return this._descendants().filter((n) => this._matchSelector(n, selector));
  }
}

const registry = new Map();

export const customElements = {
  define(name, ctor) {
    registry.set(name, ctor);
  },
  get(name) {
    return registry.get(name);
  },
  _create(name) {
    const Ctor = registry.get(name);
    if (!Ctor) {
      throw new Error(`unknown element ${name}`);
    }
    const el = new Ctor();
    el.localName = name;
    el.tagName = String(name).toUpperCase();
    return el;
  },
};

export function installDomGlobals() {
  const doc = new DocumentStub();
  globalThis.CSSStyleSheet = CSSStyleSheetStub;
  globalThis.HTMLElement = HTMLElement;
  globalThis.customElements = customElements;
  globalThis.Event = Event;
  globalThis.CustomEvent = CustomEvent;
  globalThis.KeyboardEvent = KeyboardEvent;
  globalThis.document = doc;
}
