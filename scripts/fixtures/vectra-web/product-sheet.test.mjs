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

const ID_LARGO = "01J9LARGO00000000000001";
const ID_MEDIDA = "01J9MEDIDA0000000000001";
const ID_ZINCADO = "01J9ZINCADO00000000001";
const ID_GRADO = "01J9GRADO000000000001";
const ID_INACTIVO = "01J9INACTIVO00000000001";

const SAMPLE_ATTR_DEFS = {
  items: [
    {
      id: ID_LARGO,
      key: "largo",
      label: "Largo",
      type: "number",
      unit: "mm",
      active: true,
      order: 10,
    },
    {
      id: ID_MEDIDA,
      key: "medida",
      label: "Medida",
      type: "text",
      unit: null,
      active: true,
      order: 0,
    },
    {
      id: ID_ZINCADO,
      key: "zincado",
      label: "Zincado",
      type: "boolean",
      unit: null,
      active: true,
      order: 20,
    },
    {
      id: ID_GRADO,
      key: "grado",
      label: "Grado",
      type: "enum",
      options: ["2", "5", "8"],
      active: true,
      order: 30,
    },
    {
      id: ID_INACTIVO,
      key: "obsoleto",
      label: "Obsoleto",
      type: "text",
      active: false,
      order: 99,
    },
  ],
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
  /** @type {Record<string, string>} */
  const attrSelectBag = {};
  for (const m of html.matchAll(/<vx-field[^>]+name="([^"]+)"/g)) {
    fieldBag[m[1]] = "";
  }
  for (const m of html.matchAll(/data-attr-index="(\d+)"/g)) {
    attrSelectBag[`attr-${m[1]}`] = "";
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
      const attrSelectMatch = sel.match(/^select\[data-attr-index="(\d+)"\]$/);
      if (attrSelectMatch) {
        return createSelectStub(`attr-${attrSelectMatch[1]}`, attrSelectBag);
      }
      return null;
    },
    querySelectorAll(sel) {
      if (sel === ".vx-product-attr-row") {
        /** @type {Array<{ defId: string, type: string }>} */
        const parsed = [];
        const re =
          /class="vx-product-attr-row"[^>]*data-attr-definition-id="([^"]+)"[^>]*data-attr-type="([^"]+)"/g;
        let m;
        while ((m = re.exec(html)) !== null) {
          parsed.push({ defId: m[1], type: m[2] });
        }
        return parsed.map((row) => ({
          getAttribute(name) {
            if (name === "data-attr-definition-id") {
              return row.defId;
            }
            if (name === "data-attr-type") {
              return row.type;
            }
            return null;
          },
        }));
      }
      return [];
    },
    fieldBag,
    selectBag,
    attrSelectBag,
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
  if (opts.medida != null) {
    form.fieldBag["attributes[1].value"] = opts.medida;
  }
  if (opts.largo != null) {
    form.fieldBag["attributes[0].value"] = opts.largo;
  }
  if (opts.manufacturer != null) {
    form.fieldBag["codes.manufacturer"] = opts.manufacturer;
  }
  if (opts.oem != null) {
    form.fieldBag["codes.oem"] = opts.oem;
  }
}

function defaultMountResponses(extra = [], attrDefs = { items: [] }) {
  return [
    { items: [FIJACIONES] },
    { items: [FIXSER] },
    { items: [], nextCursor: null },
    attrDefs,
    ...extra,
  ];
}

async function openCreateSheet(el, deps) {
  const request = deps.request ?? deps;
  const showError = deps.showError ?? (() => {});
  await mountProducts(el, { request, showError });
  el.dispatchClick(el.querySelector('[data-action="create"]'));
  await new Promise((r) => setTimeout(r, 0));
  await new Promise((r) => setTimeout(r, 0));
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
    const attrs = post.body.attributes;
    assert.ok(attrs === undefined || (Array.isArray(attrs) && attrs.length === 0));
    const codes = post.body.codes;
    assert.ok(codes === undefined || (Array.isArray(codes) && codes.length === 0));
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

describe("sheet_attribute_controls_match_type_and_unit", () => {
  test("controles por tipo y unidad", async () => {
    const { request, calls } = createCallTracker(
      defaultMountResponses([], SAMPLE_ATTR_DEFS),
    );
    const el = createMountEl();
    await openCreateSheet(el, request);
    const attrGet = calls.find(
      (c) => c.method === "GET" && c.path === "/api/v1/attribute-definitions",
    );
    assert.ok(attrGet);
    assert.doesNotMatch(attrGet.path, /includeInactive/);
    assert.match(el.html, /Largo \(mm\)/);
    assert.match(el.html, /name="attributes\[1\]\.value"/);
    assert.match(el.html, /data-attr-type="boolean"/);
    assert.match(el.html, />2</);
    assert.match(el.html, />5</);
    assert.match(el.html, />8</);
    assert.doesNotMatch(el.html, /<input[^>]+type="number"/i);
    assert.match(el.html, /name="codes\.manufacturer"/);
    assert.match(el.html, /name="codes\.oem"/);
  });
});

describe("sheet_no_barcode_code", () => {
  test("sin barcode en UI ni POST", async () => {
    const created = { id: "01J9X", sku: PERN_SKU, version: 1 };
    const { request, calls } = createCallTracker(
      defaultMountResponses(
        [created, { items: [], nextCursor: null }],
        SAMPLE_ATTR_DEFS,
      ),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, { request, showError: () => {} });
    fillPernoForm(form, FIJACIONES.id, {
      medida: "M8",
      largo: "40",
      manufacturer: "HX-0840Z",
    });
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.doesNotMatch(el.html, /barcode/i);
    assert.doesNotMatch(el.html, /código de barras/i);
    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.ok(
      !post.body.codes?.some((c) => c.type === "barcode"),
    );
  });
});

describe("sheet_create_perno_posts_attributes_and_codes", () => {
  test("POST con medida, largo y fabricante", async () => {
    const created = { id: "01J9PERNATTR001", sku: PERN_SKU, version: 1 };
    const { request, calls } = createCallTracker(
      defaultMountResponses(
        [created, { items: [], nextCursor: null }],
        SAMPLE_ATTR_DEFS,
      ),
    );
    const el = createMountEl();
    const form = await openCreateSheet(el, request);
    fillPernoForm(form, FIJACIONES.id, {
      medida: "M8",
      largo: "40",
      manufacturer: "HX-0840Z",
    });
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.equal(post.body.attributes.length, 2);
    assert.equal(post.body.attributes[0].definitionId, ID_LARGO);
    assert.equal(post.body.attributes[0].value, 40);
    assert.equal(post.body.attributes[1].definitionId, ID_MEDIDA);
    assert.equal(post.body.attributes[1].value, "M8");
    assert.deepEqual(post.body.codes, [
      { type: "manufacturer", value: "HX-0840Z" },
    ]);
  });
});

describe("sheet_empty_attributes_omitted_from_post", () => {
  test("sin attributes en POST si están vacíos", async () => {
    const created = { id: "01J9PERNATTR002", sku: PERN_SKU, version: 1 };
    const { request, calls } = createCallTracker(
      defaultMountResponses(
        [created, { items: [], nextCursor: null }],
        SAMPLE_ATTR_DEFS,
      ),
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
    const attrs = post.body.attributes;
    assert.ok(attrs === undefined || attrs.length === 0);
  });
});

describe("sheet_cuarenta_on_largo_sets_attribute_field_error", () => {
  test("invalid_input en largo", async () => {
    const errors = [];
    const { request } = createCallTracker(
      defaultMountResponses(
        [
          {
            throw: apiError("invalid_input", [
              {
                field: "attributes[0].value",
                message: "expected number value",
              },
            ]),
          },
        ],
        { items: [{ ...SAMPLE_ATTR_DEFS.items[0] }] },
      ),
    );
    const el = createMountEl();
    const root = { querySelector: (s) => el.querySelector(s) };
    const { showError } = createErrorUi({ root });
    const form = await openCreateSheet(el, {
      request,
      showError: (e) => {
        errors.push(e.code);
        showError(e);
      },
    });
    fillPernoForm(form, FIJACIONES.id, { largo: "cuarenta" });
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["invalid_input"]);
    assert.match(
      el.html,
      /name="attributes\[0\]\.value"[^>]*error="expected number value"|error="expected number value"[^>]*name="attributes\[0\]\.value"/,
    );
    assert.match(el.html, /data-form="create-product"/);
  });
});

describe("sheet_inactive_attribute_defs_are_hidden", () => {
  test("definición inactiva sin control", async () => {
    const { request } = createCallTracker(
      defaultMountResponses([], SAMPLE_ATTR_DEFS),
    );
    const el = createMountEl();
    await openCreateSheet(el, request);
    assert.doesNotMatch(el.html, /Obsoleto/);
    assert.doesNotMatch(el.html, new RegExp(ID_INACTIVO));
  });
});

describe("sheet_attribute_defs_network_shows_network", () => {
  test("GET defs con network", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [FIJACIONES] },
      { items: [FIXSER] },
      { items: [], nextCursor: null },
      { throw: apiError("network") },
    ]);
    const el = createMountEl();
    await mountProducts(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    el.dispatchClick(el.querySelector('[data-action="create"]'));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["network"]);
    assert.match(el.html, /data-form="create-product"/);
  });
});

describe("readProductCreate_unit", () => {
  test("readProductCreate devuelve null sin obligatorios", () => {
    const html = renderCreateProductSheet([FIJACIONES], [FIXSER], []);
    const form = createProductFormStub(html);
    const root = { querySelector: (s) => form.querySelector(s) };
    assert.equal(readProductCreate(root, form), null);
  });
});