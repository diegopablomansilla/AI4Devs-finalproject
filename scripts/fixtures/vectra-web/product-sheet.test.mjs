import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const productsUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/admin/products.js"),
).href;
const sheetUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/admin/product-sheet.js"),
).href;
const errorsUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/errors.js"),
).href;

const { mountProducts } = await import(productsUrl);
const { renderCreateProductSheet, readProductCreate } = await import(sheetUrl);
const { createErrorUi } = await import(errorsUrl);

const FIJACIONES = {
  id: "01J9FIXACIONES0000000001",
  name: "Fijaciones",
  active: true,
};

const FIXSER = {
  id: "01J9FIXSER000000000001",
  name: "Fixser",
  active: true,
};

const PERN_SKU = "PER-M8X40-ZN";

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
      const event = { target, preventDefault() {} };
      for (const { type, fn } of listeners) {
        if (type === "click") {
          fn(event);
        }
      }
    },
    dispatchSubmit(form) {
      const event = { target: form, preventDefault() {} };
      for (const { type, fn } of listeners) {
        if (type === "submit") {
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

function createFieldStub(name, bag) {
  return {
    getAttribute(attr) {
      if (attr === "name") {
        return name;
      }
      if (attr === "value") {
        return bag[name] ?? "";
      }
      if (attr === "error") {
        return bag[`error:${name}`] ?? "";
      }
      return null;
    },
    setAttribute(attr, value) {
      if (attr === "value") {
        bag[name] = String(value ?? "");
      }
      if (attr === "error") {
        bag[`error:${name}`] = String(value ?? "");
      }
    },
  };
}

function createSelectStub(name, bag) {
  return {
    get value() {
      return bag[name] ?? "";
    },
    set value(v) {
      bag[name] = String(v);
    },
    getAttribute(attr) {
      if (attr === "name") {
        return name;
      }
      return null;
    },
  };
}

function createProductFormStub(html) {
  /** @type {Record<string, string>} */
  const fieldBag = {};
  /** @type {Record<string, string>} */
  const selectBag = {
    categoryId: "",
    brandId: "",
    unitOfMeasure: "unit",
  };
  for (const m of html.matchAll(/<vx-field[^>]+name="([^"]+)"/g)) {
    fieldBag[m[1]] = "";
  }
  return {
    getAttribute(name) {
      if (name === "data-form") {
        return "create-product";
      }
      return null;
    },
    querySelector(sel) {
      const fieldMatch = sel.match(/^vx-field\[name="([^"]+)"\]$/);
      if (fieldMatch) {
        return createFieldStub(fieldMatch[1], fieldBag);
      }
      const selectMatch = sel.match(/^select\[name="([^"]+)"\]$/);
      if (selectMatch) {
        return createSelectStub(selectMatch[1], selectBag);
      }
      return null;
    },
    fieldBag,
    selectBag,
  };
}

function queryStub(html, selector) {
  const patterns = [
    {
      re: /^\[data-action="([^"]+)"\]$/,
      run(m) {
        const action = m[1];
        if (!html.includes(`data-action="${action}"`)) {
          return null;
        }
        return {
          getAttribute(name) {
            if (name === "data-action") {
              return action;
            }
            return null;
          },
          closest(sel) {
            if (sel === "[data-action]") {
              return this;
            }
            return null;
          },
        };
      },
    },
    {
      re: /^form\[data-form="create-product"\]$/,
      run() {
        if (!html.includes('data-form="create-product"')) {
          return null;
        }
        return createProductFormStub(html);
      },
    },
    {
      re: /^vx-field\[name="([^"]+)"\]$/,
      run(m) {
        const name = m[1];
        if (!html.includes(`name="${name}"`)) {
          return null;
        }
        const bag = {};
        return createFieldStub(name, bag);
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
  if (selector === 'input[type="file"]' || selector === "img") {
    return [];
  }
  if (
    selector === '[role="tab"][data-tab="stock"]' ||
    selector === '[role="tab"][data-tab="relaciones"]'
  ) {
    const tab = selector.match(/data-tab="([^"]+)"/)?.[1];
    if (tab && html.includes(`data-tab="${tab}"`)) {
      return [{}];
    }
    return [];
  }
  const single = queryStub(html, selector);
  return single ? [single] : [];
}

function apiError(code, errors = []) {
  const status =
    code === "duplicate_sku"
      ? 409
      : code === "not_found"
        ? 404
        : code === "network"
          ? 0
          : 400;
  return {
    name: "ApiError",
    code,
    status,
    title: code,
    errors,
  };
}

function fillPernoForm(form, categoryId, opts = {}) {
  form.fieldBag.sku = opts.sku ?? PERN_SKU;
  form.fieldBag.name = opts.name ?? "Perno hexagonal M8 x 40 zincado";
  form.fieldBag.productType = opts.productType ?? "Perno";
  form.fieldBag.price = opts.price ?? "120.50";
  form.fieldBag.initialStock = opts.initialStock ?? "50";
  form.selectBag.categoryId = categoryId;
  if (opts.brandId) {
    form.selectBag.brandId = opts.brandId;
  }
}

function defaultMountResponses(extra = []) {
  return [
    { items: [FIJACIONES] },
    { items: [FIXSER] },
    { items: [], nextCursor: null },
    ...extra,
  ];
}

async function openCreateSheet(el, deps) {
  const request = deps.request ?? deps;
  const showError = deps.showError ?? (() => {});
  await mountProducts(el, { request, showError });
  el.dispatchClick(el.querySelector('[data-action="create"]'));
  return el.querySelector('form[data-form="create-product"]');
}

describe("sheet_new_product_opens_datos_tab", () => {
  test("pestaña Datos y campos obligatorios", async () => {
    const { request } = createCallTracker(defaultMountResponses());
    const el = createMountEl();
    await openCreateSheet(el, request);
    assert.match(el.html, /role="tablist"/);
    assert.match(el.html, /data-tab="datos"/);
    assert.match(el.html, /aria-selected="true"[^>]*data-tab="datos"|data-tab="datos"[^>]*aria-selected="true"/);
    assert.match(el.html, /name="sku"/);
    assert.match(el.html, /name="productType"/);
    assert.match(el.html, /name="categoryId"/);
    assert.match(el.html, /name="initialStock"/);
  });
});

describe("sheet_stock_and_relations_tabs_are_disabled", () => {
  test("sin pestaña Imágenes", async () => {
    const { request } = createCallTracker(defaultMountResponses());
    const el = createMountEl();
    await openCreateSheet(el, request);
    assert.match(el.html, /data-tab="stock"/);
    assert.match(el.html, /data-tab="relaciones"/);
    assert.match(el.html, /aria-disabled="true"/);
    assert.doesNotMatch(el.html, /data-tab="imagenes"/i);
    assert.doesNotMatch(el.html, /Imágenes/);
  });
});

describe("sheet_create_perno_posts_product_create", () => {
  test("POST ProductCreate", async () => {
    const created = {
      id: "01J9PERNNEW0000000001",
      sku: PERN_SKU,
      version: 1,
    };
    const { request, calls } = createCallTracker(
      defaultMountResponses([created, { items: [], nextCursor: null }]),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, request);
    fillPernoForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.equal(post.path, "/api/v1/products");
    assert.equal(post.body.sku, PERN_SKU);
    assert.equal(post.body.price, "120.50");
    assert.equal(post.body.initialStock, "50");
    assert.equal(post.body.categoryId, FIJACIONES.id);
    assert.equal(post.body.attributes, undefined);
    assert.equal(post.body.codes, undefined);
  });
});

describe("sheet_create_returns_to_list_with_new_sku", () => {
  test("vuelve al listado con el SKU", async () => {
    const created = {
      id: "01J9PERNNEW0000000002",
      sku: PERN_SKU,
      version: 1,
    };
    const summary = {
      id: created.id,
      sku: PERN_SKU,
      name: "Perno hexagonal M8 x 40 zincado",
      productType: "Perno",
      price: "120.50",
      stock: "50",
      active: true,
    };
    const { request } = createCallTracker(
      defaultMountResponses([
        created,
        { items: [summary], nextCursor: null },
      ]),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, request);
    fillPernoForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.match(el.html, /class="vx-product-table"/);
    assert.match(el.html, new RegExp(PERN_SKU));
  });
});

describe("sheet_cancel_does_not_post", () => {
  test("cancelar sin POST", async () => {
    const { request, calls } = createCallTracker(defaultMountResponses());
    const el = createMountEl();
    await openCreateSheet(el, request);
    el.dispatchClick(el.querySelector('[data-action="cancel-sheet"]'));
    assert.equal(calls.filter((c) => c.method === "POST").length, 0);
    assert.match(el.html, /class="vx-product-admin"/);
  });
});

describe("sheet_empty_required_does_not_post", () => {
  test("sin POST si faltan obligatorios", async () => {
    const { request, calls } = createCallTracker(defaultMountResponses());
    const el = createMountEl();
    const form = await openCreateSheet(el, request);
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    assert.equal(calls.filter((c) => c.method === "POST").length, 0);
  });
});

describe("sheet_duplicate_sku_shows_duplicate_sku", () => {
  test("toast duplicate_sku y ficha abierta", async () => {
    const errors = [];
    const { request } = createCallTracker(
      defaultMountResponses([{ throw: apiError("duplicate_sku") }]),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    fillPernoForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["duplicate_sku"]);
    assert.match(el.html, /data-form="create-product"/);
  });
});

describe("sheet_invalid_input_sets_sku_field_error", () => {
  test("error en vx-field sku", async () => {
    const { request } = createCallTracker(
      defaultMountResponses([
        {
          throw: apiError("invalid_input", [
            { field: "sku", message: "requerido" },
          ]),
        },
      ]),
    );
    const el = createMountEl();
    const root = { querySelector: (s) => el.querySelector(s) };
    const { showError } = createErrorUi({ root });
    const form = await openCreateSheet(el, { request, showError });
    fillPernoForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.match(el.html, /name="sku"[^>]*error="requerido"|error="requerido"[^>]*name="sku"/);
    assert.match(el.html, /data-form="create-product"/);
  });
});

describe("sheet_invalid_input_sets_category_id_field_error", () => {
  test("error en vx-field categoryId", async () => {
    const { request } = createCallTracker(
      defaultMountResponses([
        {
          throw: apiError("invalid_input", [
            { field: "categoryId", message: "inválido" },
          ]),
        },
      ]),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, { request, showError: () => {} });
    fillPernoForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.match(
      el.html,
      /name="categoryId"[^>]*error="inválido"|error="inválido"[^>]*name="categoryId"/,
    );
  });
});

describe("sheet_network_shows_network", () => {
  test("POST falla con network", async () => {
    const errors = [];
    const { request } = createCallTracker(
      defaultMountResponses([{ throw: apiError("network") }]),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    fillPernoForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["network"]);
    assert.match(el.html, /data-form="create-product"/);
  });
});

describe("sheet_no_attributes_or_codes", () => {
  test("sin controles ni payload de atributos", async () => {
    const created = { id: "01J9X", sku: PERN_SKU, version: 1 };
    const { request, calls } = createCallTracker(
      defaultMountResponses([created, { items: [], nextCursor: null }]),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, { request, showError: () => {} });
    fillPernoForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.doesNotMatch(el.html, /name="attributes/);
    assert.doesNotMatch(el.html, /codes/);
    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.equal(post.body.attributes, undefined);
    assert.equal(post.body.codes, undefined);
  });
});

describe("readProductCreate_unit", () => {
  test("readProductCreate devuelve null sin obligatorios", () => {
    const html = renderCreateProductSheet([FIJACIONES], [FIXSER]);
    const form = createProductFormStub(html);
    const root = { querySelector: (s) => form.querySelector(s) };
    assert.equal(readProductCreate(root, form), null);
  });
});