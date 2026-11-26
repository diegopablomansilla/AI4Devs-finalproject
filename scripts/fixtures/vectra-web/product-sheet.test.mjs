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
const {
  renderCreateProductSheet,
  renderEditProductSheet,
  renderStockPanel,
  readProductCreate,
  readProductUpdate,
  readStockMovement,
  diffChangedProductFields,
} = await import(sheetUrl);
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
    if (options.signal?.aborted) {
      const err = new Error("The operation was aborted");
      err.name = "AbortError";
      throw err;
    }
    calls.push({
      method,
      path: url,
      body: options.body,
    });
    const handler = responses[step++];
    if (typeof handler === "function") {
      return handler({
        method,
        path: url,
        body: options.body,
        calls,
        signal: options.signal,
      });
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

function createStockFormStub(html) {
  /** @type {Record<string, string>} */
  const fieldBag = {};
  /** @type {Record<string, string>} */
  const selectBag = { type: "inbound" };
  for (const m of html.matchAll(/<vx-field[^>]+name="([^"]+)"/g)) {
    fieldBag[m[1]] = "";
  }
  return {
    getAttribute(name) {
      if (name === "data-form") {
        return "stock-movement";
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

function createTabStub(tab) {
  return {
    getAttribute(name) {
      if (name === "data-tab") {
        return tab;
      }
      if (name === "aria-disabled") {
        return null;
      }
      if (name === "aria-selected") {
        return tab === "stock" ? "false" : "true";
      }
      return null;
    },
    closest(sel) {
      if (sel === '[role="tab"][data-tab]') {
        return this;
      }
      return null;
    },
  };
}

function createEditFormStub(html) {
  const stub = createProductFormStub(html);
  const verMatch = html.match(/data-product-version="(\d+)"/);
  const version = verMatch ? verMatch[1] : "1";
  const origGet = stub.getAttribute.bind(stub);
  stub.getAttribute = (name) => {
    if (name === "data-form") {
      return "edit-product";
    }
    if (name === "data-product-version") {
      return version;
    }
    return origGet(name);
  };
  return stub;
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
      re: /^\[data-action="([^"]+)"\]\[data-product-id="([^"]+)"\]$/,
      run(m) {
        const action = m[1];
        const productId = m[2];
        const needle = `data-action="${action}" data-product-id="${productId}"`;
        const needleAlt = `data-product-id="${productId}" data-action="${action}"`;
        if (!html.includes(needle) && !html.includes(needleAlt)) {
          return null;
        }
        const stub = {
          getAttribute(name) {
            if (name === "data-action") {
              return action;
            }
            if (name === "data-product-id") {
              return productId;
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
        return stub;
      },
    },
    {
      re: /^\[data-relation-id="([^"]+)"\] \[data-action="([^"]+)"\]$/,
      run(m) {
        const relationId = m[1];
        const action = m[2];
        if (
          !html.includes(`data-relation-id="${relationId}"`) ||
          !html.includes(`data-action="${action}"`)
        ) {
          return null;
        }
        const typeMatch = html.match(
          new RegExp(
            `data-relation-id="${relationId}"[^>]*data-relation-type="([^"]+)"|data-relation-type="([^"]+)"[^>]*data-relation-id="${relationId}"`,
          ),
        );
        const relationType = typeMatch?.[1] ?? typeMatch?.[2] ?? null;
        const rowFieldBag = {};
        const rowStub = {
          getAttribute(name) {
            if (name === "data-relation-id") {
              return relationId;
            }
            if (name === "data-relation-type") {
              return relationType;
            }
            return null;
          },
          querySelector(sel) {
            const fm = sel.match(/^vx-field\[name="([^"]+)"\]$/);
            if (fm) {
              return createFieldStub(fm[1], rowFieldBag);
            }
            return null;
          },
          closest(sel) {
            if (sel === "[data-relation-id]") {
              return this;
            }
            return null;
          },
          fieldBag: rowFieldBag,
        };
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
            if (sel === "[data-relation-id]") {
              return rowStub;
            }
            return null;
          },
          rowFieldBag,
        };
      },
    },
    {
      re: /^\[data-action="([^"]+)"\]$/,
      run(m) {
        const action = m[1];
        if (!html.includes(`data-action="${action}"`)) {
          return null;
        }
        const rowMatch = html.match(
          new RegExp(
            `data-relation-id="([^"]+)"[^>]*>[\\s\\S]*?data-action="${action}"|data-action="${action}"[\\s\\S]*?data-relation-id="([^"]+)"`,
          ),
        );
        const relationId = rowMatch?.[1] ?? rowMatch?.[2] ?? null;
        const typeMatch = relationId
          ? html.match(
              new RegExp(
                `data-relation-id="${relationId}"[^>]*data-relation-type="([^"]+)"|data-relation-type="([^"]+)"[^>]*data-relation-id="${relationId}"`,
              ),
            )
          : null;
        const relationType = typeMatch?.[1] ?? typeMatch?.[2] ?? null;
        const rowFieldBag = {};
        const rowStub =
          relationId != null
            ? {
                getAttribute(name) {
                  if (name === "data-relation-id") {
                    return relationId;
                  }
                  if (name === "data-relation-type") {
                    return relationType;
                  }
                  return null;
                },
                querySelector(sel) {
                  const fm = sel.match(/^vx-field\[name="([^"]+)"\]$/);
                  if (fm) {
                    return createFieldStub(fm[1], rowFieldBag);
                  }
                  return null;
                },
                closest() {
                  return null;
                },
                fieldBag: rowFieldBag,
              }
            : null;
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
            if (sel === "[data-relation-id]" && rowStub) {
              return rowStub;
            }
            return null;
          },
          rowFieldBag,
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
      re: /^form\[data-form="edit-product"\]$/,
      run() {
        if (!html.includes('data-form="edit-product"')) {
          return null;
        }
        return createEditFormStub(html);
      },
    },
    {
      re: /^form\[data-form="stock-movement"\]$/,
      run() {
        if (!html.includes('data-form="stock-movement"')) {
          return null;
        }
        return createStockFormStub(html);
      },
    },
    {
      re: /^\[role="tab"\]\[data-tab="([^"]+)"\]$/,
      run(m) {
        const tab = m[1];
        if (!html.includes(`data-tab="${tab}"`)) {
          return null;
        }
        const disabled =
          new RegExp(
            `data-tab="${tab}"[^>]*aria-disabled="true"|aria-disabled="true"[^>]*data-tab="${tab}"`,
          ).test(html);
        const stub = createTabStub(tab);
        if (disabled) {
          stub.getAttribute = (name) => {
            if (name === "aria-disabled") {
              return "true";
            }
            return createTabStub(tab).getAttribute(name);
          };
        }
        return stub;
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

const PERN_LIST_ROW = {
  id: "01J9PERNEDIT00000001",
  sku: PERN_SKU,
  name: "Perno hexagonal M8 x 40 zincado",
  productType: "Perno",
  brand: "Fixser",
  categoryPath: ["Fijaciones"],
  price: "120.50",
  stock: "50",
  active: true,
};

const PERN_FICHA = {
  id: PERN_LIST_ROW.id,
  sku: PERN_SKU,
  name: PERN_LIST_ROW.name,
  productType: "Perno",
  description: null,
  brand: { id: FIXSER.id, name: "Fixser" },
  category: { id: FIJACIONES.id, name: "Fijaciones", path: ["Fijaciones"] },
  attributes: [],
  codes: [],
  unitOfMeasure: "unit",
  price: "120.50",
  stock: "50",
  active: true,
  version: 3,
  images: [],
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

const PERN_FICHA_STOCK_10 = { ...PERN_FICHA, stock: "10" };
const PERN_FICHA_STOCK_5 = { ...PERN_FICHA, stock: "5" };

function apiError(code, errors = []) {
  const status =
    code === "duplicate_sku" || code === "version_conflict"
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

function fillEditForm(form, categoryId, opts = {}) {
  fillPernoForm(form, categoryId, { ...opts, initialStock: undefined });
  delete form.fieldBag.initialStock;
}

function defaultEditMountResponses(
  extra = [],
  attrDefs = { items: [] },
  ficha = PERN_FICHA,
) {
  return [
    { items: [FIJACIONES] },
    { items: [FIXSER] },
    { items: [PERN_LIST_ROW], nextCursor: null },
    ficha,
    attrDefs,
    ...extra,
  ];
}

function fillStockForm(form, opts = {}) {
  form.selectBag.type = opts.type ?? "inbound";
  form.fieldBag.quantity = opts.quantity ?? "";
  if (opts.reason != null) {
    form.fieldBag.reason = opts.reason;
  } else {
    form.fieldBag.reason = "";
  }
}

const REL_TARGET_ACTIVE = {
  id: "01J9TARGETACTIVE000001",
  sku: "ARA-M8",
  name: "Arandela M8",
  productType: "",
  brand: null,
  categoryPath: [],
  price: "0",
  stock: "0",
  active: true,
  imageUrl: null,
};

const REL_TARGET_INACTIVE = {
  id: "01J9TARGETINACT000001",
  sku: "OBS-01",
  name: "Producto baja",
  productType: "",
  brand: null,
  categoryPath: [],
  price: "0",
  stock: "0",
  active: false,
  imageUrl: null,
};

const REL_SIMILAR = {
  id: "01J9RELSIM00000000001",
  type: "similar",
  sourceProductId: PERN_LIST_ROW.id,
  targetProductId: REL_TARGET_ACTIVE.id,
  priority: 2,
  note: "Similar A",
  bidirectionalGroup: "01J9BIDIRGROUP0000001",
  target: REL_TARGET_ACTIVE,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

const REL_SUBSTITUTE = {
  id: "01J9RELSUB00000000001",
  type: "substitute",
  sourceProductId: PERN_LIST_ROW.id,
  targetProductId: REL_TARGET_INACTIVE.id,
  priority: 1,
  note: null,
  bidirectionalGroup: null,
  target: REL_TARGET_INACTIVE,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

const REL_COMPLEMENT = {
  id: "01J9RELCOMP0000000001",
  type: "complement",
  sourceProductId: PERN_LIST_ROW.id,
  targetProductId: "01J9TARGETCOMP0000001",
  priority: 3,
  note: "Va junto",
  bidirectionalGroup: null,
  target: {
    id: "01J9TARGETCOMP0000001",
    sku: "TUER-M8",
    name: "Tuerca M8",
    active: true,
  },
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
};

const RELATIONS_THREE_TYPES = [REL_SIMILAR, REL_SUBSTITUTE, REL_COMPLEMENT];

function mountWithRelationsResponses(extraAfterEdit = [], relations = RELATIONS_THREE_TYPES) {
  return [
    ...defaultEditMountResponses(extraAfterEdit),
    { items: relations },
  ];
}

async function openEditRelationsTab(el, deps, productId = PERN_LIST_ROW.id) {
  const request = deps.request ?? deps;
  await openEditSheet(el, { request, showError: deps.showError ?? (() => {}) }, productId);
  const tab = el.querySelector('[role="tab"][data-tab="relaciones"]');
  assert.ok(tab, "pestaña Relaciones");
  el.dispatchClick(tab);
  for (let i = 0; i < 10; i++) {
    await new Promise((r) => setTimeout(r, 0));
  }
  return el;
}

async function openEditStockTab(el, deps, productId = PERN_LIST_ROW.id) {
  const request = deps.request ?? deps;
  await openEditSheet(el, { request, showError: deps.showError ?? (() => {}) }, productId);
  const callsBefore =
    typeof deps.calls !== "undefined" ? deps.calls.length : 0;
  const tab = el.querySelector('[role="tab"][data-tab="stock"]');
  assert.ok(tab, "pestaña Stock");
  el.dispatchClick(tab);
  for (let i = 0; i < 6; i++) {
    await new Promise((r) => setTimeout(r, 0));
  }
  if (deps.calls) {
    assert.equal(deps.calls.length, callsBefore);
  }
  return el.querySelector('form[data-form="stock-movement"]');
}

async function openEditSheet(el, deps, productId = PERN_LIST_ROW.id) {
  const request = deps.request ?? deps;
  const showError = deps.showError ?? (() => {});
  await mountProducts(el, { request, showError });
  const editBtn = el.querySelector(
    `[data-action="edit"][data-product-id="${productId}"]`,
  );
  assert.ok(editBtn, "botón Editar");
  el.dispatchClick(editBtn);
  for (let i = 0; i < 6; i++) {
    await new Promise((r) => setTimeout(r, 0));
  }
  return el.querySelector('form[data-form="edit-product"]');
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

describe("diff_changed_product_fields_lists_price", () => {
  test("precio cambiado", () => {
    const before = { ...PERN_FICHA, price: "120.50", version: 3 };
    const after = { ...PERN_FICHA, price: "130.00", version: 4 };
    const { keys, labels } = diffChangedProductFields(before, after);
    assert.deepEqual(keys, ["price"]);
    assert.deepEqual(labels, ["Precio"]);
  });
});

describe("sheet_edit_stock_is_read_only", () => {
  test("stock visible sin input editable", () => {
    const html = renderEditProductSheet(PERN_FICHA, [FIJACIONES], [FIXSER], []);
    assert.match(html, /Stock actual: 50/);
    assert.doesNotMatch(html, /name="stock"/);
    assert.doesNotMatch(html, /name="initialStock"/);
  });
});

describe("sheet_edit_opens_hydrated_datos_tab", () => {
  test("Editar producto y SKU hidratado", async () => {
    const { request, calls } = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    await openEditSheet(el, request);
    assert.match(el.html, /Editar producto/);
    assert.match(el.html, /data-form="edit-product"/);
    assert.match(el.html, new RegExp(`value="${PERN_SKU}"`));
    assert.ok(
      calls.some(
        (c) =>
          c.method === "GET" &&
          c.path === `/api/v1/products/${PERN_LIST_ROW.id}`,
      ),
    );
    assert.ok(
      calls.some(
        (c) => c.method === "GET" && c.path === "/api/v1/attribute-definitions",
      ),
    );
  });
});

describe("sheet_edit_price_patches_with_version", () => {
  test("PATCH con version", async () => {
    const updated = { ...PERN_FICHA, price: "130.00", version: 4 };
    const { request, calls } = createCallTracker(
      defaultEditMountResponses([updated, { items: [PERN_LIST_ROW], nextCursor: null }]),
    );
    const el = createMountEl();
    const form = await openEditSheet(el, request);
    fillEditForm(form, FIJACIONES.id, { price: "130.00" });
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const patch = calls.find((c) => c.method === "PATCH");
    assert.ok(patch);
    assert.equal(patch.body.version, 3);
    assert.equal(patch.body.price, "130.00");
    assert.equal(patch.body.initialStock, undefined);
    assert.equal(patch.body.stock, undefined);
    assert.match(el.html, /class="vx-product-table"/);
  });
});

describe("sheet_edit_version_conflict_reloads_and_lists_changed_fields", () => {
  test("409 recarga y role status Precio", async () => {
    const errors = [];
    const fichaV4 = {
      ...PERN_FICHA,
      version: 4,
      price: "130.00",
    };
    const { request, calls } = createCallTracker(
      defaultEditMountResponses([
        { throw: apiError("version_conflict") },
        fichaV4,
      ]),
    );
    const el = createMountEl();
    const form = await openEditSheet(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    fillEditForm(form, FIJACIONES.id, {
      name: "Nombre local sin guardar",
      price: "125.00",
    });
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["version_conflict"]);
    assert.ok(
      calls.filter(
        (c) =>
          c.method === "GET" &&
          c.path === `/api/v1/products/${PERN_LIST_ROW.id}`,
      ).length >= 2,
    );
    assert.match(el.html, /role="status"/);
    assert.match(el.html, /Precio/);
    assert.match(el.html, /value="130\.00"/);
    assert.match(el.html, /data-form="edit-product"/);
  });
});

describe("sheet_edit_does_not_merge_local_unsaved_fields", () => {
  test("nombre del servidor tras conflicto", async () => {
    const fichaV4 = {
      ...PERN_FICHA,
      version: 4,
      price: "130.00",
      name: "Perno actualizado por otra terminal",
    };
    const { request } = createCallTracker(
      defaultEditMountResponses([
        { throw: apiError("version_conflict") },
        fichaV4,
      ]),
    );
    const el = createMountEl();
    const form = await openEditSheet(el, { request, showError: () => {} });
    fillEditForm(form, FIJACIONES.id, { name: "Nombre local B" });
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.match(
      el.html,
      /value="Perno actualizado por otra terminal"/,
    );
    assert.doesNotMatch(el.html, /value="Nombre local B"/);
  });
});

describe("sheet_edit_invalid_input_sets_field_error", () => {
  test("error en sku", async () => {
    const { request } = createCallTracker(
      defaultEditMountResponses([
        {
          throw: apiError("invalid_input", [
            { field: "sku", message: "inválido" },
          ]),
        },
      ]),
    );
    const el = createMountEl();
    const root = { querySelector: (s) => el.querySelector(s) };
    const { showError } = createErrorUi({ root });
    const form = await openEditSheet(el, { request, showError });
    fillEditForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.match(
      el.html,
      /name="sku"[^>]*error="inválido"|error="inválido"[^>]*name="sku"/,
    );
    assert.match(el.html, /data-form="edit-product"/);
  });
});

describe("sheet_edit_not_found_shows_not_found", () => {
  test("GET ficha 404", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [FIJACIONES] },
      { items: [FIXSER] },
      { items: [PERN_LIST_ROW], nextCursor: null },
      { throw: apiError("not_found") },
    ]);
    const el = createMountEl();
    await openEditSheet(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["not_found"]);
    assert.match(el.html, /class="vx-product-table"/);
  });
});

describe("sheet_edit_network_shows_network", () => {
  test("PATCH network", async () => {
    const errors = [];
    const { request } = createCallTracker(
      defaultEditMountResponses([{ throw: apiError("network") }]),
    );
    const el = createMountEl();
    const form = await openEditSheet(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    fillEditForm(form, FIJACIONES.id);
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["network"]);
    assert.match(el.html, /data-form="edit-product"/);
  });
});

describe("sheet_edit_empty_required_does_not_patch", () => {
  test("sin PATCH si falta precio", async () => {
    const { request, calls } = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    const form = await openEditSheet(el, request);
    form.fieldBag.price = "";
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    assert.equal(calls.filter((c) => c.method === "PATCH").length, 0);
  });
});

describe("readStockMovement_unit", () => {
  test("cantidad vacía devuelve null", () => {
    const html = renderStockPanel("10", {});
    const form = createStockFormStub(html);
    const root = { querySelector: (s) => form.querySelector(s) };
    form.fieldBag.quantity = "";
    assert.equal(readStockMovement(root, form), null);
  });

  test("ajuste sin motivo devuelve error reason", () => {
    const html = renderStockPanel("5", {});
    const form = createStockFormStub(html);
    const root = { querySelector: (s) => form.querySelector(s) };
    form.selectBag.type = "adjustment";
    form.fieldBag.quantity = "-1";
    form.fieldBag.reason = "";
    const parsed = readStockMovement(root, form);
    assert.deepEqual(parsed, { error: "reason" });
  });
});

describe("renderStockPanel_unit", () => {
  test("formulario y stock resultante", () => {
    const html = renderStockPanel("10", {}, "30");
    assert.match(html, /data-form="stock-movement"/);
    assert.match(html, /Stock actual: 10/);
    assert.match(html, /role="status"/);
    assert.match(html, /Stock resultante: 30/);
    assert.doesNotMatch(html, /name="stock"/);
  });
});

describe("sheet_create_stock_tab_stays_disabled", () => {
  test("Stock no operable en alta", async () => {
    const { request } = createCallTracker(defaultMountResponses());
    const el = createMountEl();
    await openCreateSheet(el, request);
    assert.match(el.html, /data-tab="stock"[^>]*aria-disabled="true"|aria-disabled="true"[^>]*data-tab="stock"/);
    const tab = el.querySelector('[role="tab"][data-tab="stock"]');
    const before = el.html;
    el.dispatchClick(tab);
    await new Promise((r) => setTimeout(r, 0));
    assert.equal(el.html, before);
    assert.doesNotMatch(el.html, /data-form="stock-movement"/);
  });
});

describe("sheet_edit_stock_tab_is_operable", () => {
  test("Stock sin aria-disabled", async () => {
    const { request } = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    await openEditSheet(el, request);
    assert.doesNotMatch(
      el.html,
      /data-tab="stock"[^>]*aria-disabled="true"|aria-disabled="true"[^>]*data-tab="stock"/,
    );
    assert.doesNotMatch(
      el.html,
      /data-tab="relaciones"[^>]*aria-disabled="true"|aria-disabled="true"[^>]*data-tab="relaciones"/,
    );
  });
});

describe("sheet_stock_shows_form_without_http", () => {
  test("panel Stock sin HTTP extra", async () => {
    const tracker = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    const form = await openEditStockTab(el, tracker);
    assert.ok(form);
    assert.match(el.html, /data-form="stock-movement"/);
    assert.match(el.html, /name="quantity"/);
    assert.match(el.html, /aria-selected="true"[^>]*data-tab="stock"|data-tab="stock"[^>]*aria-selected="true"/);
  });
});

describe("sheet_stock_inbound_posts_and_shows_resulting_stock", () => {
  test("ingreso 10 + 20 → 30", async () => {
    const movement = {
      id: "01J9MOVINBOUND0000001",
      productId: PERN_LIST_ROW.id,
      type: "inbound",
      quantity: "20",
      resultingStock: "30",
      reason: null,
      terminal: null,
      createdAt: "2026-01-01T00:00:00Z",
    };
    const { request, calls } = createCallTracker(
      defaultEditMountResponses([movement], { items: [] }, PERN_FICHA_STOCK_10),
    );
    const el = createMountEl();
    const form = await openEditStockTab(el, { request, calls });
    fillStockForm(form, { type: "inbound", quantity: "20" });
    el.dispatchSubmit(form);
    for (let i = 0; i < 6; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    const post = calls.find(
      (c) => c.method === "POST" && c.path.includes("/stock-movements"),
    );
    assert.ok(post);
    assert.equal(post.body.type, "inbound");
    assert.equal(post.body.quantity, "20");
    assert.equal(post.body.reason, undefined);
    assert.equal(calls.filter((c) => c.method === "PATCH").length, 0);
    assert.match(el.html, /Stock resultante: 30/);
    assert.match(el.html, /Stock actual: 30/);
  });
});

describe("sheet_stock_empty_quantity_does_not_post", () => {
  test("sin POST sin cantidad", async () => {
    const { request, calls } = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    const form = await openEditStockTab(el, { request, calls });
    fillStockForm(form, { type: "inbound", quantity: "" });
    el.dispatchSubmit(form);
    await new Promise((r) => setTimeout(r, 0));
    assert.equal(
      calls.filter((c) => c.method === "POST" && c.path.includes("stock-movements")).length,
      0,
    );
  });
});

describe("sheet_stock_adjustment_to_zero_with_reason", () => {
  test("ajuste -5 a cero", async () => {
    const movement = {
      id: "01J9MOVADJ00000000001",
      productId: PERN_LIST_ROW.id,
      type: "adjustment",
      quantity: "-5",
      resultingStock: "0",
      reason: "Conteo físico",
      createdAt: "2026-01-01T00:00:00Z",
    };
    const { request, calls } = createCallTracker(
      defaultEditMountResponses([movement], { items: [] }, PERN_FICHA_STOCK_5),
    );
    const el = createMountEl();
    const form = await openEditStockTab(el, { request, calls });
    fillStockForm(form, {
      type: "adjustment",
      quantity: "-5",
      reason: "Conteo físico",
    });
    el.dispatchSubmit(form);
    for (let i = 0; i < 6; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    const post = calls.find((c) => c.method === "POST");
    assert.equal(post?.body?.type, "adjustment");
    assert.equal(post?.body?.reason, "Conteo físico");
    assert.match(el.html, /Stock resultante: 0/);
  });
});

describe("sheet_stock_adjustment_without_reason_shows_invalid_input", () => {
  test("sin POST ni motivo", async () => {
    const errors = [];
    const { request, calls } = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    const form = await openEditStockTab(el, {
      request,
      calls,
      showError: (e) => errors.push(e.code),
    });
    fillStockForm(form, { type: "adjustment", quantity: "-1", reason: "" });
    el.dispatchSubmit(form);
    for (let i = 0; i < 4; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.deepEqual(errors, ["invalid_input"]);
    assert.equal(
      calls.filter((c) => c.method === "POST" && c.path.includes("stock-movements")).length,
      0,
    );
    assert.match(
      el.html,
      /name="reason"[^>]*error="El motivo es obligatorio"/,
    );
  });
});

describe("sheet_stock_negative_keeps_stock_and_shows_negative_stock", () => {
  test("stock intacto en 5", async () => {
    const errors = [];
    const { request, calls } = createCallTracker(
      defaultEditMountResponses(
        [{ throw: apiError("negative_stock") }],
        { items: [] },
        PERN_FICHA_STOCK_5,
      ),
    );
    const el = createMountEl();
    const form = await openEditStockTab(el, {
      request,
      calls,
      showError: (e) => errors.push(e.code),
    });
    fillStockForm(form, {
      type: "adjustment",
      quantity: "-8",
      reason: "Conteo físico",
    });
    el.dispatchSubmit(form);
    for (let i = 0; i < 6; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.deepEqual(errors, ["negative_stock"]);
    assert.match(el.html, /Stock actual: 5/);
  });
});

describe("sheet_stock_invalid_input_sets_field_error", () => {
  test("quantity con invalid_input", async () => {
    const { request, calls } = createCallTracker(
      defaultEditMountResponses(
        [
          {
            throw: apiError("invalid_input", [
              { field: "quantity", message: "inválida" },
            ]),
          },
        ],
        { items: [] },
        PERN_FICHA_STOCK_10,
      ),
    );
    const el = createMountEl();
    const root = { querySelector: (s) => el.querySelector(s) };
    const { showError } = createErrorUi({ root });
    const form = await openEditStockTab(el, { request, calls, showError });
    fillStockForm(form, { type: "inbound", quantity: "0" });
    el.dispatchSubmit(form);
    for (let i = 0; i < 6; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.match(el.html, /name="quantity"[^>]*error="inválida"/);
    assert.match(el.html, /data-form="stock-movement"/);
  });
});

describe("sheet_stock_not_found_shows_not_found", () => {
  test("POST 404", async () => {
    const errors = [];
    const { request } = createCallTracker(
      defaultEditMountResponses([{ throw: apiError("not_found") }]),
    );
    const el = createMountEl();
    const form = await openEditStockTab(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    fillStockForm(form, { type: "inbound", quantity: "1" });
    el.dispatchSubmit(form);
    for (let i = 0; i < 6; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.deepEqual(errors, ["not_found"]);
    assert.match(el.html, /data-form="stock-movement"/);
  });
});

describe("sheet_stock_network_shows_network", () => {
  test("fallo de red", async () => {
    const errors = [];
    const { request } = createCallTracker(
      defaultEditMountResponses([{ throw: apiError("network") }]),
    );
    const el = createMountEl();
    const form = await openEditStockTab(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    fillStockForm(form, { type: "inbound", quantity: "1" });
    el.dispatchSubmit(form);
    for (let i = 0; i < 6; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.deepEqual(errors, ["network"]);
    assert.match(el.html, /data-form="stock-movement"/);
  });
});

describe("sheet_stock_has_no_edit_or_delete_movement", () => {
  test("sin acciones de mutate", async () => {
    const { request } = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    await openEditStockTab(el, { request });
    assert.doesNotMatch(el.html, /data-action="edit-movement"/i);
    assert.doesNotMatch(el.html, /data-action="delete-movement"/i);
    assert.doesNotMatch(el.html, /Borrar movimiento/i);
    assert.doesNotMatch(el.html, /Editar movimiento/i);
  });
});

describe("sheet_stock_does_not_get_or_export_history", () => {
  test("sin GET historial ni exportar", async () => {
    const { request, calls } = createCallTracker(defaultEditMountResponses());
    const el = createMountEl();
    const form = await openEditStockTab(el, { request, calls });
    fillStockForm(form, { type: "inbound", quantity: "1" });
    el.dispatchSubmit(form);
    for (let i = 0; i < 4; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.equal(
      calls.filter(
        (c) =>
          c.method === "GET" && c.path.includes("/stock-movements"),
      ).length,
      0,
    );
    assert.doesNotMatch(el.html, /Exportar/i);
    assert.doesNotMatch(el.html, /Historial/i);
  });
});

describe("sheet_edit_relations_tab_is_operable", () => {
  test("Relaciones sin aria-disabled en edición", async () => {
    const { request } = createCallTracker(mountWithRelationsResponses());
    const el = createMountEl();
    await openEditSheet(el, request);
    assert.doesNotMatch(
      el.html,
      /data-tab="relaciones"[^>]*aria-disabled="true"|aria-disabled="true"[^>]*data-tab="relaciones"/,
    );
  });
});

describe("sheet_relations_lists_grouped_by_type", () => {
  test("tres grupos y GET relations", async () => {
    const { request, calls } = createCallTracker(mountWithRelationsResponses());
    const el = createMountEl();
    await openEditRelationsTab(el, { request, calls });
    const relGet = calls.find(
      (c) => c.method === "GET" && c.path.includes("/relations"),
    );
    assert.ok(relGet);
    assert.doesNotMatch(relGet.path, /direction=incoming/);
    assert.doesNotMatch(relGet.path, /recommendations/);
    assert.match(el.html, /Similares/);
    assert.match(el.html, /Sustitutos/);
    assert.match(el.html, /Se usa junto con/);
    assert.match(el.html, /ARA-M8/);
    assert.match(el.html, /OBS-01/);
    assert.match(el.html, /TUER-M8/);
    assert.match(
      el.html,
      /aria-selected="true"[^>]*data-tab="relaciones"|data-tab="relaciones"[^>]*aria-selected="true"/,
    );
  });
});

describe("sheet_relations_empty_items_still_show_headings", () => {
  test("títulos sin filas", async () => {
    const { request } = createCallTracker(
      mountWithRelationsResponses([], []),
    );
    const el = createMountEl();
    await openEditRelationsTab(el, request);
    assert.match(el.html, /Similares/);
    assert.match(el.html, /Sustitutos/);
    assert.match(el.html, /Se usa junto con/);
    assert.doesNotMatch(el.html, /data-relation-id=/);
  });
});

describe("sheet_relations_shows_inactive_target", () => {
  test("destino inactivo visible", async () => {
    const { request } = createCallTracker(mountWithRelationsResponses());
    const el = createMountEl();
    await openEditRelationsTab(el, request);
    assert.match(el.html, /Producto baja/);
    assert.match(el.html, /OBS-01/);
  });
});

describe("sheet_relations_shows_bidirectional_indicator", () => {
  test("texto Bidireccional", async () => {
    const { request } = createCallTracker(mountWithRelationsResponses());
    const el = createMountEl();
    await openEditRelationsTab(el, request);
    assert.match(el.html, /Bidireccional/);
    const simBlock = el.html.split(`data-relation-id="${REL_SIMILAR.id}"`)[1];
    assert.ok(simBlock);
    assert.match(simBlock.split("data-relation-id=")[0], /Bidireccional/);
  });
});

describe("sheet_relations_patch_priority_updates_row", () => {
  test("PATCH substitute prioridad 1", async () => {
  let items = [...RELATIONS_THREE_TYPES];
  const tracker = createCallTracker([
      ...defaultEditMountResponses(),
      () => ({ items }),
      ({ method, path, body }) => {
        if (method === "PATCH" && path.includes("/relations/substitute/")) {
          const idx = items.findIndex((i) => i.id === REL_SUBSTITUTE.id);
          items[idx] = { ...items[idx], priority: body.priority };
          return { items: [items[idx]] };
        }
        return { items };
      },
      () => ({ items }),
    ]);
    const { request, calls } = tracker;
    const el = createMountEl();
    await openEditRelationsTab(el, { request, calls });
    const saveBtn = el.querySelector(
      `[data-relation-id="${REL_SUBSTITUTE.id}"] [data-action="save-relation"]`,
    );
    assert.ok(saveBtn);
    if (saveBtn.rowFieldBag) {
      saveBtn.rowFieldBag.priority = "1";
      saveBtn.rowFieldBag.note = REL_SUBSTITUTE.note ?? "";
    }
    el.dispatchClick(saveBtn);
    for (let i = 0; i < 10; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    const patch = calls.find((c) => c.method === "PATCH");
    assert.ok(patch);
    assert.match(patch.path, /\/relations\/substitute\//);
    assert.equal(patch.body.priority, 1);
    assert.equal(typeof patch.body.priority, "number");
    assert.match(
      el.html,
      new RegExp(
        `data-relation-id="${REL_SUBSTITUTE.id}"[\\s\\S]*name="priority"[^>]*value="1"|value="1"[^>]*name="priority"`,
      ),
    );
  });
});

describe("sheet_relations_patch_note_updates_row", () => {
  test("PATCH nota", async () => {
    let items = [...RELATIONS_THREE_TYPES];
    const { request, calls } = createCallTracker([
      ...defaultEditMountResponses(),
      () => ({ items }),
      ({ body }) => {
        items = items.map((i) =>
          i.id === REL_COMPLEMENT.id ? { ...i, note: body.note } : i,
        );
        return { items: [items.find((i) => i.id === REL_COMPLEMENT.id)] };
      },
      () => ({ items }),
    ]);
    const el = createMountEl();
    await openEditRelationsTab(el, { request, calls });
    const saveBtn = el.querySelector(
      `[data-relation-id="${REL_COMPLEMENT.id}"] [data-action="save-relation"]`,
    );
    assert.ok(saveBtn?.rowFieldBag);
    saveBtn.rowFieldBag.priority = String(REL_COMPLEMENT.priority);
    saveBtn.rowFieldBag.note = "Arandela plana M8";
    el.dispatchClick(saveBtn);
    for (let i = 0; i < 10; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    const patch = calls.find((c) => c.method === "PATCH");
    assert.equal(patch?.body?.note, "Arandela plana M8");
    assert.match(el.html, /Arandela plana M8/);
  });
});

describe("sheet_relations_delete_removes_row", () => {
  test("DELETE y fila ausente", async () => {
    let items = [...RELATIONS_THREE_TYPES];
    const { request, calls } = createCallTracker([
      ...defaultEditMountResponses(),
      () => ({ items }),
      () => undefined,
      () => ({ items: items.filter((i) => i.id !== REL_COMPLEMENT.id) }),
    ]);
    const el = createMountEl();
    await openEditRelationsTab(el, { request, calls });
    const delBtn = el.querySelector(
      `[data-relation-id="${REL_COMPLEMENT.id}"] [data-action="delete-relation"]`,
    );
    assert.ok(delBtn);
    el.dispatchClick(delBtn);
    for (let i = 0; i < 10; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    const del = calls.find((c) => c.method === "DELETE");
    assert.ok(del);
    assert.match(del.path, /\/relations\//);
    assert.doesNotMatch(el.html, new RegExp(`data-relation-id="${REL_COMPLEMENT.id}"`));
    assert.equal(
      calls.filter((c) => c.method === "DELETE").length,
      1,
    );
  });
});

describe("sheet_relations_priority_below_one_does_not_patch", () => {
  test("prioridad 0 sin PATCH", async () => {
    const { request, calls } = createCallTracker(mountWithRelationsResponses());
    const el = createMountEl();
    await openEditRelationsTab(el, { request, calls });
    const saveBtn = el.querySelector(
      `[data-relation-id="${REL_SUBSTITUTE.id}"] [data-action="save-relation"]`,
    );
    saveBtn.rowFieldBag.priority = "0";
    saveBtn.rowFieldBag.note = "";
    el.dispatchClick(saveBtn);
    for (let i = 0; i < 8; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.equal(calls.filter((c) => c.method === "PATCH").length, 0);
    assert.match(el.html, /name="priority"[^>]*error=/);
  });
});

describe("sheet_relations_invalid_input_sets_field_error", () => {
  test("note invalid_input", async () => {
    const { request, calls } = createCallTracker([
      ...mountWithRelationsResponses(),
      {
        throw: apiError("invalid_input", [
          { field: "note", message: "demasiado larga" },
        ]),
      },
    ]);
    const el = createMountEl();
    const root = { querySelector: (s) => el.querySelector(s) };
    const { showError } = createErrorUi({ root });
    await openEditRelationsTab(el, { request, calls, showError });
    const saveBtn = el.querySelector(
      `[data-relation-id="${REL_COMPLEMENT.id}"] [data-action="save-relation"]`,
    );
    saveBtn.rowFieldBag.priority = String(REL_COMPLEMENT.priority);
    saveBtn.rowFieldBag.note = "x".repeat(600);
    el.dispatchClick(saveBtn);
    for (let i = 0; i < 10; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.match(el.html, /name="note"[^>]*error="demasiado larga"/);
    assert.match(el.html, /data-panel="relations"/);
  });
});

describe("sheet_relations_not_found_shows_not_found", () => {
  test("GET 404", async () => {
    const errors = [];
    const { request } = createCallTracker([
      ...defaultEditMountResponses(),
      { throw: apiError("not_found") },
    ]);
    const el = createMountEl();
    await openEditRelationsTab(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    assert.deepEqual(errors, ["not_found"]);
  });
});

describe("sheet_relations_unavailable_shows_unavailable", () => {
  test("503", async () => {
    const errors = [];
    const { request } = createCallTracker([
      ...defaultEditMountResponses(),
      { throw: { ...apiError("unavailable"), status: 503 } },
    ]);
    const el = createMountEl();
    await openEditRelationsTab(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    assert.deepEqual(errors, ["unavailable"]);
  });
});

describe("sheet_relations_network_shows_network", () => {
  test("network en guardar", async () => {
    const errors = [];
    const tracker = createCallTracker([
      ...mountWithRelationsResponses(),
      { throw: apiError("network") },
    ]);
    const el = createMountEl();
    await openEditRelationsTab(el, {
      request: tracker.request,
      showError: (e) => errors.push(e.code),
    });
    const saveBtn = el.querySelector(
      `[data-relation-id="${REL_COMPLEMENT.id}"] [data-action="save-relation"]`,
    );
    saveBtn.rowFieldBag.priority = String(REL_COMPLEMENT.priority);
    el.dispatchClick(saveBtn);
    for (let i = 0; i < 10; i++) {
      await new Promise((r) => setTimeout(r, 0));
    }
    assert.deepEqual(errors, ["network"]);
  });
});

describe("sheet_relations_abort_does_not_paint_stale", () => {
  test("cambio de pestaña aborta GET", async () => {
    const { request, calls } = createCallTracker([
      ...defaultEditMountResponses(),
      ({ signal }) =>
        new Promise((resolve, reject) => {
          const t = setTimeout(
            () => resolve({ items: [{ ...REL_SIMILAR, note: "STALE" }] }),
            80,
          );
          signal?.addEventListener("abort", () => {
            clearTimeout(t);
            const err = new Error("aborted");
            err.name = "AbortError";
            reject(err);
          });
        }),
    ]);
    const el = createMountEl();
    await openEditSheet(el, { request, calls });
    const relTab = el.querySelector('[role="tab"][data-tab="relaciones"]');
    el.dispatchClick(relTab);
    const datosTab = el.querySelector('[role="tab"][data-tab="datos"]');
    el.dispatchClick(datosTab);
    await new Promise((r) => setTimeout(r, 120));
    assert.doesNotMatch(el.html, /STALE/);
  });
});

describe("sheet_create_relations_tab_stays_disabled", () => {
  test("alta mantiene Relaciones disabled", async () => {
    const { request, calls } = createCallTracker(defaultMountResponses());
    const el = createMountEl();
    await openCreateSheet(el, request);
    assert.match(
      el.html,
      /data-tab="relaciones"[^>]*aria-disabled="true"|aria-disabled="true"[^>]*data-tab="relaciones"/,
    );
    const tab = el.querySelector('[role="tab"][data-tab="relaciones"]');
    const before = el.html;
    el.dispatchClick(tab);
    await new Promise((r) => setTimeout(r, 0));
    assert.equal(el.html, before);
    assert.equal(
      calls.filter((c) => c.path.includes("/relations")).length,
      0,
    );
  });
});

describe("sheet_relations_has_no_create_controls", () => {
  test("sin POST ni buscador", async () => {
    const { request, calls } = createCallTracker(mountWithRelationsResponses());
    const el = createMountEl();
    await openEditRelationsTab(el, { request, calls });
    assert.doesNotMatch(el.html, /data-form="create-relation"/i);
    assert.doesNotMatch(el.html, /Buscar producto/i);
    assert.doesNotMatch(el.html, /Nueva relación/i);
    assert.equal(
      calls.filter((c) => c.method === "POST" && c.path.includes("/relations")).length,
      0,
    );
  });
});

describe("sheet_relations_does_not_get_incoming", () => {
  test("GET sin direction=incoming", async () => {
    const { request, calls } = createCallTracker(mountWithRelationsResponses());
    const el = createMountEl();
    await openEditRelationsTab(el, { request, calls });
    const relGet = calls.find((c) => c.path.includes("/relations"));
    assert.ok(relGet);
    assert.doesNotMatch(relGet.path, /incoming/);
  });
});