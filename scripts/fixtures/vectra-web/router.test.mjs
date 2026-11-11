import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const routerUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/router.js"),
).href;

const { parseHash, startRouter } = await import(routerUrl);

/** @returns {{ outlet: object, window: object, reloadCount: { value: number } }} */
function createTestDom(initialHash = "#/") {
  let html = "";
  const reloadCount = { value: 0 };
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
    reload() {
      reloadCount.value += 1;
    },
  };

  const outlet = {
    get innerHTML() {
      return html;
    },
    set innerHTML(value) {
      html = value;
    },
    querySelector(selector) {
      if (selector === 'input[type="password"]') {
        return /<input[^>]+type="password"/i.test(html) ? {} : null;
      }
      if (selector === 'input[type="text"][name="username"]') {
        return /name="username"/i.test(html) ? {} : null;
      }
      const viewMatch = selector.match(/^\[data-view="([^"]+)"\]$/);
      if (viewMatch) {
        const view = viewMatch[1];
        const re = new RegExp(`data-view="${view}"`);
        return re.test(html) ? { dataset: { view } } : null;
      }
      const sectionMatch = selector.match(/^\[data-section="([^"]+)"\]$/);
      if (sectionMatch) {
        const section = sectionMatch[1];
        const re = new RegExp(`data-section="${section}"`);
        return re.test(html) ? { dataset: { section } } : null;
      }
      return null;
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

  const doc = {
    getElementById() {
      return null;
    },
  };

  return { outlet, window: win, document: doc, reloadCount };
}

describe("router_operator_to_admin_productos_does_not_reload", () => {
  test("hash #/ to #/admin/productos", () => {
    const { outlet, window, document, reloadCount } = createTestDom("#/");
    startRouter({ window, document, outlet });

    assert.ok(outlet.querySelector('[data-view="operator"]'));
    assert.equal(reloadCount.value, 0);

    window.location.hash = "#/admin/productos";
    assert.ok(outlet.querySelector('[data-view="admin"]'));
    assert.ok(outlet.querySelector('[data-section="productos"]'));
    assert.equal(outlet.querySelector('[data-view="operator"]'), null);
    assert.equal(reloadCount.value, 0);
  });
});

describe("router_admin_productos_to_operator_does_not_reload", () => {
  test("hash #/admin/productos to #/", () => {
    const { outlet, window, document, reloadCount } = createTestDom(
      "#/admin/productos",
    );
    startRouter({ window, document, outlet });

    assert.ok(outlet.querySelector('[data-view="admin"]'));
    window.location.hash = "#/";
    assert.ok(outlet.querySelector('[data-view="operator"]'));
    assert.equal(outlet.querySelector('[data-view="admin"]'), null);
    assert.equal(reloadCount.value, 0);
  });
});

describe("router_empty_hash_is_operator", () => {
  test("hash vacío", () => {
    const { outlet, window, document } = createTestDom("");
    startRouter({ window, document, outlet });
    assert.ok(outlet.querySelector('[data-view="operator"]'));
  });

  test("hash #", () => {
    const { outlet, window, document } = createTestDom("#");
    startRouter({ window, document, outlet });
    assert.ok(outlet.querySelector('[data-view="operator"]'));
  });
});

describe("router_unknown_hash_is_not_found", () => {
  test("#/no-existe", () => {
    const { outlet, window, document, reloadCount } = createTestDom("#/");
    startRouter({ window, document, outlet });
    window.location.hash = "#/no-existe";
    assert.ok(outlet.querySelector('[data-view="not-found"]'));
    assert.equal(outlet.querySelector('[data-view="operator"]'), null);
    assert.equal(outlet.querySelector('[data-view="admin"]'), null);
    assert.equal(reloadCount.value, 0);
  });
});

describe("router_admin_without_section_is_not_found", () => {
  test("#/admin", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    window.location.hash = "#/admin";
    assert.ok(outlet.querySelector('[data-view="not-found"]'));
  });

  test("#/admin/", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    window.location.hash = "#/admin/";
    assert.ok(outlet.querySelector('[data-view="not-found"]'));
  });
});

describe("router_invalid_admin_slug_is_not_found", () => {
  test("#/admin/productos/extra", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    window.location.hash = "#/admin/productos/extra";
    assert.ok(outlet.querySelector('[data-view="not-found"]'));
    assert.equal(outlet.querySelector('[data-section="productos"]'), null);
  });

  test("#/admin/Productos", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    window.location.hash = "#/admin/Productos";
    assert.ok(outlet.querySelector('[data-view="not-found"]'));
    assert.equal(outlet.querySelector('[data-section="productos"]'), null);
  });

  test("#/admin/prod uctos", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    window.location.hash = "#/admin/prod uctos";
    assert.ok(outlet.querySelector('[data-view="not-found"]'));
  });
});

describe("administración no pide credenciales", () => {
  test("#/admin/productos sin login", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    assert.ok(outlet.querySelector('[data-view="admin"]'));
    assert.equal(outlet.querySelector('input[type="password"]'), null);
    assert.equal(outlet.querySelector('input[type="text"][name="username"]'), null);
  });
});

describe("parseHash", () => {
  test("operator paths", () => {
    assert.deepEqual(parseHash(""), { view: "operator" });
    assert.deepEqual(parseHash("#"), { view: "operator" });
    assert.deepEqual(parseHash("#/"), { view: "operator" });
  });

  test("admin productos", () => {
    assert.deepEqual(parseHash("#/admin/productos"), {
      view: "admin",
      section: "productos",
    });
  });
});

describe("router_operator_layout_has_omnibox_results_panel", () => {
  test("vista operador declara regiones y vx-omnibox", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-view="operator"/);
    assert.match(html, /data-region="omnibox"/);
    assert.match(html, /<vx-omnibox\b/);
    assert.match(html, /data-region="results"/);
    assert.match(html, /data-region="side-panel"/);
    assert.match(html, /<vx-side-panel\b/);
    assert.doesNotMatch(html, /<vx-side-panel[^>]*\sopen\b/);
    assert.match(html, /href="#\/admin\/productos"/);
  });
});

describe("router_admin_view_has_no_omnibox", () => {
  test("#/admin/productos sin vx-omnibox", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-view="admin"/);
    assert.doesNotMatch(html, /<vx-omnibox\b/);
    assert.doesNotMatch(html, /data-region="omnibox"/);
  });
});
