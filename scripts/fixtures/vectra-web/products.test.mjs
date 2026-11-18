import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const productsUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/admin/products.js"),
).href;
const routerUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/router.js"),
).href;

const { mountProducts } = await import(productsUrl);
const { startRouter } = await import(routerUrl);

const FIJACIONES = {
  id: "01J9FIXACIONES0000000001",
  name: "Fijaciones",
  slug: "fijaciones",
  parentId: null,
  ancestorIds: [],
  path: ["Fijaciones"],
  active: true,
};

const FIXSER = {
  id: "01J9FIXSER000000000001",
  name: "Fixser",
  slug: "fixser",
  active: true,
};

const PERN001 = {
  id: "01J9PERN0010000000001",
  sku: "PERN-001",
  name: "Perno M8",
  productType: "perno",
  brand: "Fixser",
  categoryPath: ["Fijaciones", "Pernos"],
  price: "10.00",
  stock: "5",
  active: true,
};

const PERN002 = {
  ...PERN001,
  id: "01J9PERN0020000000002",
  sku: "PERN-002",
  name: "Perno M10",
};

function createCallTracker(responses) {
  /** @type {Array<{ method: string, path: string, body?: unknown }>} */
  const calls = [];
  let step = 0;

  async function request(method, url, options = {}) {
    calls.push({
      method,
      path: url,
      body: options.body,
    });
    const handler = responses[step++];
    if (typeof handler === "function") {
      return handler({ method, path: url, body: options.body, calls });
    }
    if (handler && typeof handler.then === "function") {
      return handler;
    }
    if (handler && handler.throw) {
      throw handler.throw;
    }
    return handler ?? { items: [], nextCursor: null };
  }

  return { request, calls };
}

function createMountEl(html = "") {
  let content = html;
  /** @type {Array<{ type: string, fn: Function }>} */
  const listeners = [];

  const el = {
    get innerHTML() {
      return content;
    },
    set innerHTML(value) {
      content = String(value);
    },
    addEventListener(type, fn) {
      listeners.push({ type, fn });
    },
    querySelector(selector) {
      return queryStub(content, selector);
    },
    querySelectorAll(selector) {
      return queryAllStub(content, selector);
    },
    dispatchClick(target) {
      const event = {
        target,
        preventDefault() {},
      };
      for (const { type, fn } of listeners) {
        if (type === "click") {
          fn(event);
        }
      }
    },
    dispatchChange(target) {
      const event = {
        target,
        preventDefault() {},
      };
      for (const { type, fn } of listeners) {
        if (type === "change") {
          fn(event);
        }
      }
    },
    get html() {
      return content;
    },
  };
  return el;
}

function queryStub(html, selector) {
  if (
    selector.includes("[data-action=") &&
    selector.includes("[data-product-id=")
  ) {
    const actionMatch = selector.match(/\[data-action="([^"]+)"\]/);
    const idMatch = selector.match(/\[data-product-id="([^"]+)"\]/);
    if (actionMatch && idMatch) {
      if (
        html.includes(`data-action="${actionMatch[1]}"`) &&
        html.includes(`data-product-id="${idMatch[1]}"`)
      ) {
        return createActionTarget(actionMatch[1], idMatch[1]);
      }
    }
    return null;
  }

  const patterns = [
    {
      re: /^\[data-action="([^"]+)"\]$/,
      run(m) {
        const action = m[1];
        if (!html.includes(`data-action="${action}"`)) {
          return null;
        }
        return createActionTarget(action, undefined);
      },
    },
    {
      re: /^\[data-action="([^"]+)"\]\[data-product-id="([^"]+)"\]$/,
      run(m) {
        return createActionTarget(m[1], m[2]);
      },
    },
    {
      re: /^select\[data-filter="([^"]+)"\]$/,
      run(m) {
        const filter = m[1];
        if (!html.includes(`data-filter="${filter}"`)) {
          return null;
        }
        const selectedMatch = html.match(
          new RegExp(
            `<select[^>]*data-filter="${filter}"[^>]*>[\\s\\S]*?<option value="([^"]+)" selected`,
          ),
        );
        const initial = selectedMatch?.[1] ?? "";
        return createFilterSelectStub(filter, initial);
      },
    },
    {
      re: /^\.vx-product-table$/,
      run() {
        if (!html.includes('class="vx-product-table"')) {
          return null;
        }
        return { tagName: "TABLE", className: "vx-product-table" };
      },
    },
  ];

  for (const { re, run } of patterns) {
    const m = selector.match(re);
    if (m) {
      return run(m);
    }
  }
  return null;
}

function queryAllStub(html, selector) {
  if (selector === 'input[type="file"]') {
    return [];
  }
  if (selector === ".vx-product-table input, .vx-product-table vx-field") {
    return [];
  }
  if (selector === "form") {
    if (/<form\b/i.test(html)) {
      return [{}];
    }
    return [];
  }
  const single = queryStub(html, selector);
  return single ? [single] : [];
}

function createActionTarget(action, productId) {
  return {
    getAttribute(name) {
      if (name === "data-action") {
        return action;
      }
      if (name === "data-product-id") {
        return productId ?? null;
      }
      return null;
    },
    closest(sel) {
      if (sel === "[data-action]") {
        return createActionTarget(action, productId);
      }
      return null;
    },
  };
}

function createFilterSelectStub(filter, initial = "") {
  const state = { value: String(initial) };
  return {
    get value() {
      return state.value;
    },
    set value(v) {
      state.value = String(v);
    },
    getAttribute(name) {
      if (name === "data-filter") {
        return filter;
      }
      return null;
    },
  };
}

function apiError(code, status = 400) {
  const resolved =
    code === "not_found"
      ? 404
      : code === "version_conflict"
        ? 409
        : code === "network"
          ? 0
          : status;
  return {
    name: "ApiError",
    code,
    status: resolved,
    title: code,
    errors: [],
  };
}

describe("products_list_requests_active_true", () => {
  test("GET productos con active=true", async () => {
    const { request, calls } = createCallTracker([
      { items: [FIJACIONES] },
      { items: [FIXSER] },
      { items: [PERN001], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });
    const listGet = calls.find(
      (c) => c.method === "GET" && c.path.startsWith("/api/v1/products"),
    );
    assert.ok(listGet);
    assert.match(listGet.path, /active=true/);
    assert.doesNotMatch(listGet.path, /cursor=/);
  });
});

describe("products_lookups_request_categories_and_brands", () => {
  test("GET de maestros", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });
    assert.ok(
      calls.some((c) => c.path === "/api/v1/categories?includeInactive=true"),
    );
    assert.ok(
      calls.some((c) => c.path === "/api/v1/brands?includeInactive=true"),
    );
  });
});

describe("products_empty_list_does_not_post", () => {
  test("sin POST si vacío", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });
    assert.equal(calls.filter((c) => c.method === "POST").length, 0);
  });
});

describe("products_list_offers_edit", () => {
  test("Editar en cada fila", async () => {
    const { request } = createCallTracker([
      { items: [FIJACIONES] },
      { items: [FIXSER] },
      { items: [PERN001], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });
    assert.match(el.html, /data-action="edit"/);
    assert.match(el.html, new RegExp(`data-product-id="${PERN001.id}"`));
  });
});

describe("products_empty_list_offers_create", () => {
  test("ofrece Nuevo producto", async () => {
    const { request } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });
    assert.match(el.html, /data-action="create"/);
    assert.match(el.html, /Nuevo producto/);
  });
});

describe("products_filter_category_sends_category_id", () => {
  test("filtra por categoría", async () => {
    const { request, calls } = createCallTracker([
      { items: [FIJACIONES] },
      { items: [] },
      { items: [PERN001], nextCursor: null },
      { items: [PERN001], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });

    const catSelect = el.querySelector('select[data-filter="category"]');
    catSelect.value = FIJACIONES.id;
    el.dispatchChange(catSelect);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const filtered = calls
      .filter((c) => c.method === "GET" && c.path.includes("/api/v1/products"))
      .pop();
    assert.ok(filtered);
    assert.match(filtered.path, new RegExp(`categoryId=${FIJACIONES.id}`));
    assert.match(filtered.path, /active=true/);
  });
});

describe("products_filter_inactive_shows_reactivate", () => {
  test("inactivos y reactivar", async () => {
    const inactive = { ...PERN001, active: false };
    const { request, calls } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [PERN001], nextCursor: null },
      { items: [inactive], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });

    const activeSelect = el.querySelector('select[data-filter="active"]');
    activeSelect.value = "false";
    el.dispatchChange(activeSelect);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const filtered = calls
      .filter((c) => c.method === "GET" && c.path.includes("/api/v1/products"))
      .pop();
    assert.ok(filtered);
    assert.match(filtered.path, /active=false/);
    assert.match(el.html, /data-action="reactivate"/);
    assert.doesNotMatch(el.html, /data-action="deactivate"/);
  });
});

describe("products_load_more_appends_with_cursor", () => {
  test("concatena páginas", async () => {
    const cursor = "01J9CURSOR00000000001";
    const { request, calls } = createCallTracker([
      { items: [] },
      { items: [] },
      {
        items: [PERN001],
        nextCursor: cursor,
      },
      {
        items: [PERN002],
        nextCursor: null,
      },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });

    const more = el.querySelector('[data-action="load-more"]');
    assert.ok(more);
    el.dispatchClick(more);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const lastGet = calls
      .filter((c) => c.method === "GET" && c.path.includes("/api/v1/products"))
      .pop();
    assert.ok(lastGet);
    assert.match(lastGet.path, new RegExp(`cursor=${cursor}`));
    assert.match(el.html, /PERN-001/);
    assert.match(el.html, /PERN-002/);
  });
});

describe("products_deactivate_deletes", () => {
  test("DELETE producto", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [PERN001], nextCursor: null },
      null,
      { items: [], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-product-id="${PERN001.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const del = calls.find((c) => c.method === "DELETE");
    assert.ok(del);
    assert.match(del.path, /PERN001/);
  });
});

describe("products_reactivate_gets_then_patches_active_true", () => {
  test("GET ficha y PATCH active", async () => {
    const inactive = { ...PERN001, active: false };
    const ficha = {
      ...inactive,
      version: 3,
      category: { id: FIJACIONES.id, name: "Fijaciones" },
      attributes: [],
      codes: [],
      images: [],
    };
    const { request, calls } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [inactive], nextCursor: null },
      { items: [inactive], nextCursor: null },
      ficha,
      { ...ficha, active: true, version: 4 },
      { items: [{ ...PERN001, active: true }], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });

    const activeSelect = el.querySelector('select[data-filter="active"]');
    activeSelect.value = "false";
    el.dispatchChange(activeSelect);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const btn = el.querySelector(
      `[data-action="reactivate"][data-product-id="${PERN001.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const getFicha = calls.find(
      (c) => c.method === "GET" && c.path === `/api/v1/products/${PERN001.id}`,
    );
    assert.ok(getFicha);
    const patch = calls.find(
      (c) => c.method === "PATCH" && c.path === `/api/v1/products/${PERN001.id}`,
    );
    assert.ok(patch);
    assert.deepEqual(patch.body, { version: 3, active: true });
    assert.equal(patch.body.sku, undefined);
    assert.equal(patch.body.name, undefined);
  });
});

describe("products_network_shows_network", () => {
  test("fallo de red en listado", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [] },
      { items: [] },
      { throw: apiError("network") },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: (e) => errors.push(e.code) });
    assert.deepEqual(errors, ["network"]);
  });
});

describe("products_not_found_shows_not_found", () => {
  test("DELETE desconocido", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [PERN001], nextCursor: null },
      { throw: apiError("not_found", 404) },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: (e) => errors.push(e.code) });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-product-id="${PERN001.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["not_found"]);
    assert.match(el.html, /PERN-001/);
  });
});

describe("products_invalid_input_shows_invalid_input", () => {
  test("listado inválido", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [] },
      { items: [] },
      { throw: apiError("invalid_input") },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: (e) => errors.push(e.code) });
    assert.deepEqual(errors, ["invalid_input"]);
  });
});

describe("products_version_conflict_shows_version_conflict", () => {
  test("PATCH conflict", async () => {
    const errors = [];
    const inactive = { ...PERN001, active: false };
    const ficha = {
      ...inactive,
      version: 1,
      category: { id: FIJACIONES.id, name: "Fijaciones" },
      attributes: [],
      codes: [],
      images: [],
    };
    const { request } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [inactive], nextCursor: null },
      { items: [inactive], nextCursor: null },
      ficha,
      { throw: apiError("version_conflict", 409) },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: (e) => errors.push(e.code) });

    const activeSelect = el.querySelector('select[data-filter="active"]');
    activeSelect.value = "false";
    el.dispatchChange(activeSelect);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const btn = el.querySelector(
      `[data-action="reactivate"][data-product-id="${PERN001.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["version_conflict"]);
  });
});

describe("products_no_inline_edit", () => {
  test("sin edición en línea ni alta", async () => {
    const { request } = createCallTracker([
      { items: [] },
      { items: [] },
      { items: [PERN001], nextCursor: null },
    ]);
    const el = createMountEl();
    await mountProducts(el, { request, showError: () => {} });
    assert.doesNotMatch(el.html, /contenteditable/i);
    assert.equal(el.querySelectorAll("form").length, 0);
    assert.equal(
      el.querySelectorAll(".vx-product-table input, .vx-product-table vx-field")
        .length,
      0,
    );
    assert.doesNotMatch(el.html, /<vx-product-card\b/);
  });
});

describe("products_router_mounts_on_productos", () => {
  test("GET al pintar #/admin/productos", async () => {
    const calls = [];
    const request = async (method, url) => {
      calls.push({ method, path: url });
      if (url.startsWith("/api/v1/products")) {
        return { items: [], nextCursor: null };
      }
      return { items: [] };
    };
    const region = createMountEl();
    let html = "";
    const outlet = {
      get innerHTML() {
        return html;
      },
      set innerHTML(value) {
        html = String(value);
      },
      querySelector(sel) {
        if (sel === '[data-region="admin-content"]') {
          return region;
        }
        if (sel === 'vx-field[name="terminal"]') {
          return { getAttribute: () => "", setAttribute() {} };
        }
        return null;
      },
    };
    const window = {
      location: { hash: "#/admin/productos" },
      localStorage: {
        getItem: () => null,
        setItem() {},
        removeItem() {},
      },
      addEventListener() {},
    };
    const document = {};
    startRouter({
      window,
      document,
      outlet,
      request,
      showError: () => {},
      mountProducts,
    });
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    assert.ok(calls.some((c) => c.path.startsWith("/api/v1/products")));
  });
});