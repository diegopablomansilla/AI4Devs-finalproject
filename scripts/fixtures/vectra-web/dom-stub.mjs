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

  dispatchEvent(event) {
    const list = [...(this._listeners.get(event.type) ?? [])];
    for (const fn of list) {
      fn.call(this, event);
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

  querySelector(selector) {
    if (selector === "button") {
      return this._children.find((n) => n.tagName === "BUTTON") ?? null;
    }
    if (selector === "input") {
      return this._children.find((n) => n.tagName === "INPUT") ?? null;
    }
    if (selector === '[role="alert"]') {
      return this._children.find((n) => n.getAttribute("role") === "alert") ?? null;
    }
  }
}

class FakeNode extends EventTargetStub {
  constructor(tagName = "DIV") {
    super();
    this.tagName = tagName;
    this.parentNode = null;
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

  setAttribute(name, value) {
    if (name === "disabled" && (value === "" || value === "disabled")) {
      this.disabled = true;
    }
    this._attrs[name] = String(value);
  }

  getAttribute(name) {
    if (name === "disabled" && this.disabled) {
      return "";
    }
    return this._attrs[name] ?? null;
  }

  hasAttribute(name) {
    if (name === "disabled") {
      return this.disabled;
    }
    return name in this._attrs;
  }

  removeAttribute(name) {
    if (name === "disabled") {
      this.disabled = false;
    }
    delete this._attrs[name];
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
}

export class Event {
  constructor(type, init = {}) {
    this.type = type;
    this.bubbles = init.bubbles ?? false;
    this.cancelable = init.cancelable ?? false;
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

export class HTMLElement extends FakeNode {
  constructor() {
    super("DIV");
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
    return new Ctor();
  },
};

export function installDomGlobals() {
  globalThis.CSSStyleSheet = CSSStyleSheetStub;
  globalThis.HTMLElement = HTMLElement;
  globalThis.customElements = customElements;
  globalThis.Event = Event;
  globalThis.CustomEvent = CustomEvent;
  globalThis.document = {
    createElement(tag) {
      return new FakeNode(String(tag).toUpperCase());
    },
  };
}
