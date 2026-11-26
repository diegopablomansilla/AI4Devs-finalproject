import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const routerUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/router.js"),
).href;

const {
  TERMINAL_STORAGE_KEY,
  readTerminalName,
  writeTerminalName,
  bindTerminalName,
  startRouter,
} = await import(routerUrl);

function createMemoryStorage(initial = {}) {
  /** @type {Record<string, string>} */
  const map = { ...initial };
  return {
    getItem(key) {
      return Object.prototype.hasOwnProperty.call(map, key) ? map[key] : null;
    },
    setItem(key, value) {
      map[key] = String(value);
    },
    removeItem(key) {
      delete map[key];
    },
    _map: map,
  };
}

function createRouterDom(initialHash, storage) {
  let html = "";
  const hashChangeListeners = [];
  const location = {
    _hash: initialHash,
    get hash() {
      return this._hash;
    },
    set hash(value) {
      this._hash = value;
      for (const fn of hashChangeListeners) {
        fn();
      }
    },
  };
  const hostRef = { storage, field: null };
  const outlet = {
    get innerHTML() {
      return html;
    },
    set innerHTML(value) {
      html = value;
    },
    querySelector(selector) {
      if (selector !== 'vx-field[name="terminal"]') {
        return null;
      }
      if (!/<vx-field[^>]+name="terminal"/.test(html)) {
        return null;
      }
      return createTerminalField(storage);
    },
  };
  const win = {
    location,
    addEventListener(type, fn) {
      if (type === "hashchange") {
        hashChangeListeners.push(fn);
      }
    },
  };
  const doc = { getElementById: () => null };
  return { outlet, window: win, document: doc, storage, getHtml: () => html };
}

/** @param {ReturnType<typeof createMemoryStorage>} storage */
function createTerminalField(storage) {
  const attrs = { value: "", error: "" };
  const listeners = [];
  const field = {
    getAttribute(name) {
      if (name === "value") {
        return attrs.value;
      }
      if (name === "error") {
        return attrs.error;
      }
      if (name === "name") {
        return "terminal";
      }
      return null;
    },
    setAttribute(name, value) {
      if (name === "value") {
        attrs.value = value ?? "";
      }
      if (name === "error") {
        attrs.error = value ?? "";
      }
    },
    removeAttribute(name) {
      if (name === "error") {
        attrs.error = "";
      }
    },
    addEventListener(type, fn) {
      if (type === "change") {
        listeners.push(fn);
      }
    },
    dispatchChange(value) {
      attrs.value = value;
      for (const fn of listeners) {
        fn();
      }
    },
  };
  bindTerminalName(field, storage);
  return field;
}

describe("admin_terminal_name_persists_to_storage", () => {
  test("writeTerminalName guarda Caja-1", () => {
    const storage = createMemoryStorage();
    const result = writeTerminalName(storage, "Caja-1");
    assert.equal(result.ok, true);
    assert.equal(storage.getItem(TERMINAL_STORAGE_KEY), "Caja-1");
  });

  test("bind con change persiste", () => {
    const storage = createMemoryStorage();
    const { outlet, window, document } = createRouterDom(
      "#/admin/productos",
      storage,
    );
    startRouter({ window, document, outlet, storage });
    const field = outlet.querySelector('vx-field[name="terminal"]');
    assert.ok(field);
    field.dispatchChange("Mostrador 2");
    assert.equal(storage.getItem(TERMINAL_STORAGE_KEY), "Mostrador 2");
    assert.equal(field.getAttribute("error"), "");
  });
});

describe("admin_terminal_name_loads_from_storage", () => {
  test("value en el HTML", () => {
    const storage = createMemoryStorage({ [TERMINAL_STORAGE_KEY]: "Mostrador" });
    const { outlet, window, document, getHtml } = createRouterDom(
      "#/admin/productos",
      storage,
    );
    startRouter({ window, document, outlet, storage });
    assert.match(getHtml(), /value="Mostrador"/);
    assert.equal(readTerminalName(storage), "Mostrador");
  });
});

describe("admin_terminal_name_over_64_does_not_save", () => {
  test("rechaza 65 caracteres", () => {
    const storage = createMemoryStorage({ [TERMINAL_STORAGE_KEY]: "x" });
    const long = "a".repeat(65);
    const result = writeTerminalName(storage, long);
    assert.equal(result.ok, false);
    assert.match(result.error, /64/);
    assert.equal(storage.getItem(TERMINAL_STORAGE_KEY), "x");
  });

  test("bind muestra error", () => {
    const storage = createMemoryStorage({ [TERMINAL_STORAGE_KEY]: "x" });
    const field = createTerminalField(storage);
    field.dispatchChange("b".repeat(65));
    assert.match(field.getAttribute("error"), /64/);
    assert.equal(storage.getItem(TERMINAL_STORAGE_KEY), "x");
  });
});

describe("admin_terminal_name_empty_clears_storage", () => {
  test("removeItem", () => {
    const storage = createMemoryStorage({ [TERMINAL_STORAGE_KEY]: "Caja-1" });
    const result = writeTerminalName(storage, "   ");
    assert.equal(result.ok, true);
    assert.equal(storage.getItem(TERMINAL_STORAGE_KEY), null);
  });
});

describe("admin_terminal_storage_failure_does_not_throw", () => {
  test("getItem lanza", () => {
    const storage = {
      getItem() {
        throw new Error("quota");
      },
      setItem() {},
      removeItem() {},
    };
    assert.equal(readTerminalName(storage), "");
    assert.doesNotThrow(() => writeTerminalName(storage, "ok"));
  });

  test("setItem lanza", () => {
    const storage = {
      getItem: () => null,
      setItem() {
        throw new Error("quota");
      },
      removeItem() {},
    };
    assert.doesNotThrow(() => {
      const result = writeTerminalName(storage, "Caja");
      assert.equal(result.ok, true);
    });
  });

  test("bind con storage roto no lanza", () => {
    const storage = {
      getItem() {
        throw new Error("quota");
      },
      setItem() {
        throw new Error("quota");
      },
      removeItem() {
        throw new Error("quota");
      },
    };
    const field = createTerminalField(storage);
    assert.doesNotThrow(() => field.dispatchChange("x"));
  });
});
