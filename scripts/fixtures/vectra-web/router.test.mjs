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

describe("router_operator_header_has_data_health", () => {
  test("encabezado operador con data-health y aria-live", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-health\b/);
    assert.match(html, /aria-live="polite"/);
    const headerStart = html.indexOf("vx-operator-header");
    assert.ok(headerStart >= 0);
    const headerSlice = html.slice(headerStart, headerStart + 600);
    assert.match(headerSlice, /data-health\b/);
  });
});

describe("router_admin_header_has_data_health", () => {
  test("encabezado admin con data-health y aria-live", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-health\b/);
    assert.match(html, /aria-live="polite"/);
    const headerStart = html.indexOf("vx-admin-header");
    assert.ok(headerStart >= 0);
    const headerSlice = html.slice(headerStart, headerStart + 600);
    assert.match(headerSlice, /data-health\b/);
  });
});

const OFFLINE_COPY = "Sin conexión con el Nodo Maestro";
const UPDATE_COPY = "Nueva versión disponible";

describe("router_operator_header_has_offline_banner", () => {
  test("encabezado operador con data-offline-banner y aria-live assertive", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-offline-banner\b/);
    assert.match(html, /aria-live="assertive"/);
    assert.match(html, new RegExp(OFFLINE_COPY));
    const headerStart = html.indexOf("vx-operator-header");
    const headerSlice = html.slice(headerStart, headerStart + 800);
    assert.match(headerSlice, /data-offline-banner\b/);
  });
});

describe("router_operator_header_has_update_banner", () => {
  test("encabezado operador con data-update-banner y Recargar", () => {
    const { outlet, window, document } = createTestDom("#/");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-update-banner\b/);
    assert.match(html, /aria-live="polite"/);
    assert.match(html, new RegExp(UPDATE_COPY));
    assert.match(html, /data-update-reload\b/);
    assert.match(html, />Recargar</);
    const headerStart = html.indexOf("vx-operator-header");
    const headerSlice = html.slice(headerStart, headerStart + 900);
    assert.match(headerSlice, /data-update-banner\b/);
  });
});

describe("router_admin_header_has_update_banner", () => {
  test("encabezado admin con data-update-banner y Recargar", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-update-banner\b/);
    assert.match(html, /aria-live="polite"/);
    assert.match(html, new RegExp(UPDATE_COPY));
    assert.match(html, /data-update-reload\b/);
    const headerStart = html.indexOf("vx-admin-header");
    const headerSlice = html.slice(headerStart, headerStart + 900);
    assert.match(headerSlice, /data-update-banner\b/);
  });
});

describe("router_admin_header_has_offline_banner", () => {
  test("encabezado admin con data-offline-banner y aria-live assertive", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-offline-banner\b/);
    assert.match(html, /aria-live="assertive"/);
    assert.match(html, new RegExp(OFFLINE_COPY));
    const headerStart = html.indexOf("vx-admin-header");
    const headerSlice = html.slice(headerStart, headerStart + 800);
    assert.match(headerSlice, /data-offline-banner\b/);
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

const ADMIN_HREFS = [
  "#/admin/productos",
  "#/admin/categorias",
  "#/admin/marcas",
  "#/admin/atributos",
  "#/admin/stock",
  "#/admin/relaciones",
  "#/admin/sinonimos",
  "#/admin/notas-tecnicas",
];

describe("router_admin_layout_has_menu_and_back_link", () => {
  test("#/admin/productos", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /class="vx-admin"/);
    assert.match(html, /data-view="admin"/);
    assert.match(html, /href="#\/"/);
    assert.match(html, /Volver al operador/);
    assert.match(html, /aria-label="Administración"/);
    assert.match(html, /data-region="admin-content"/);
    assert.match(html, /<vx-field[^>]+name="terminal"/);
  });
});

describe("router_admin_menu_lists_eight_sections", () => {
  test("enlaces del menú", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    for (const href of ADMIN_HREFS) {
      assert.match(html, new RegExp(`href="${href.replace("#", "#")}"`));
    }
  });
});

describe("router_admin_nav_to_categorias_does_not_reload", () => {
  test("#/admin/productos a #/admin/categorias", () => {
    const { outlet, window, document, reloadCount } = createTestDom(
      "#/admin/productos",
    );
    startRouter({ window, document, outlet });
    window.location.hash = "#/admin/categorias";
    assert.ok(outlet.querySelector('[data-view="admin"]'));
    assert.ok(outlet.querySelector('[data-section="categorias"]'));
    assert.equal(reloadCount.value, 0);
  });
});

describe("router_admin_back_to_operator_in_one_step", () => {
  test("desde admin a #/", () => {
    const { outlet, window, document, reloadCount } = createTestDom(
      "#/admin/marcas",
    );
    startRouter({ window, document, outlet });
    window.location.hash = "#/";
    assert.ok(outlet.querySelector('[data-view="operator"]'));
    assert.equal(outlet.querySelector('[data-view="admin"]'), null);
    assert.equal(reloadCount.value, 0);
  });
});

describe("router_admin_current_section_has_aria_current", () => {
  test("productos activo", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(
      html,
      /href="#\/admin\/productos"[^>]*aria-current="page"/,
    );
    assert.doesNotMatch(
      html,
      /href="#\/admin\/categorias"[^>]*aria-current="page"/,
    );
  });

  test("categorias activo", () => {
    const { outlet, window, document } = createTestDom("#/admin/categorias");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(
      html,
      /href="#\/admin\/categorias"[^>]*aria-current="page"/,
    );
    assert.doesNotMatch(
      html,
      /href="#\/admin\/productos"[^>]*aria-current="page"/,
    );
  });
});

describe("router_admin_content_region_has_no_business_ui", () => {
  test("sin ABM", () => {
    const { outlet, window, document } = createTestDom("#/admin/productos");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    const contentStart = html.indexOf('data-region="admin-content"');
    assert.ok(contentStart >= 0);
    const contentSlice = html.slice(contentStart);
    assert.doesNotMatch(contentSlice, /<vx-product-card\b/);
    assert.doesNotMatch(contentSlice, /<form\b/i);
  });
});

describe("router_admin_unknown_slug_has_menu_without_current", () => {
  test("#/admin/foo", () => {
    const { outlet, window, document } = createTestDom("#/admin/foo");
    startRouter({ window, document, outlet });
    const html = outlet.innerHTML;
    assert.match(html, /data-section="foo"/);
    for (const href of ADMIN_HREFS) {
      assert.match(html, new RegExp(`href="${href.replace("#", "#")}"`));
    }
    assert.doesNotMatch(html, /aria-current="page"/);
  });
});

describe("router_admin_does_not_call_api", () => {
  test("sin fetch al pintar admin", () => {
    const calls = [];
    const original = globalThis.fetch;
    globalThis.fetch = (...args) => {
      calls.push(args);
      return original?.(...args) ?? Promise.resolve({ ok: true });
    };
    try {
      const { outlet, window, document } = createTestDom("#/admin/productos");
      startRouter({ window, document, outlet });
      window.location.hash = "#/admin/stock";
      assert.equal(calls.length, 0);
    } finally {
      globalThis.fetch = original;
    }
  });
});
