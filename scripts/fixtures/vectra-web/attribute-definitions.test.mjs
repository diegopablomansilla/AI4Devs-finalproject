import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const attrsUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/admin/attribute-definitions.js"),
).href;
const routerUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/router.js"),
).href;

const { mountAttributeDefinitions } = await import(attrsUrl);
const { startRouter } = await import(routerUrl);

const GRADO = {
  id: "01J9GRADO0000000000001",
  key: "grado",
  label: "Grado",
  type: "enum",
  unit: null,
  options: ["2", "5", "8"],
  facetable: true,
  searchable: true,
  order: 0,
  active: true,
};

const MEDIDA = {
  id: "01J9MEDIDA000000000001",
  key: "medida",
  label: "Medida",
  type: "text",
  unit: "mm",
  options: null,
  facetable: false,
  searchable: true,
  order: 10,
  active: true,
};

const GRADO_SAE = {
  ...GRADO,
  label: "Grado SAE",
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
    return handler ?? { items: [] };
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
    dispatchSubmit(form) {
      const event = {
        target: form,
        preventDefault() {},
      };
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

function queryStub(html, selector) {
  if (
    selector.includes("[data-action=") &&
    selector.includes("[data-attribute-id=")
  ) {
    const actionMatch = selector.match(/\[data-action="([^"]+)"\]/);
    const idMatch = selector.match(/\[data-attribute-id="([^"]+)"\]/);
    if (actionMatch && idMatch) {
      if (
        html.includes(`data-action="${actionMatch[1]}"`) &&
        html.includes(`data-attribute-id="${idMatch[1]}"`)
      ) {
        return createActionTarget(actionMatch[1], idMatch[1]);
      }
    }
    return null;
  }

  const patterns = [
    {
      re: /^\[data-action="([^"]+)"\](?:\[data-attribute-id="([^"]+)"\])?$/,
      run(m) {
        const action = m[1];
        const id = m[2];
        if (!html.includes(`data-action="${action}"`)) {
          return null;
        }
        if (id && !html.includes(`data-attribute-id="${id}"`)) {
          return null;
        }
        return createActionTarget(action, id);
      },
    },
    {
      re: /^form\[data-form="([^"]+)"\]$/,
      run(m) {
        const form = m[1];
        if (!html.includes(`data-form="${form}"`)) {
          return null;
        }
        const blockMatch = html.match(
          new RegExp(
            `<form[^>]*data-form="${form}"[^>]*>[\\s\\S]*?<\\/form>`,
          ),
        );
        const block = blockMatch ? blockMatch[0] : html;
        return createFormTarget(form, block);
      },
    },
    {
      re: /^vx-field\[name="([^"]+)"\]$/,
      run(m) {
        const name = m[1];
        const re = new RegExp(`<vx-field[^>]+name="${name}"[^>]*>`);
        if (!re.test(html)) {
          return null;
        }
        return createFieldStub(name);
      },
    },
    {
      re: /^select\[name="type"\]$/,
      run() {
        if (!html.includes('name="type"')) {
          return null;
        }
        return createSelectStub("text");
      },
    },
    {
      re: /^input\[name="([^"]+)"\]$/,
      run(m) {
        const name = m[1];
        if (!html.includes(`name="${name}"`)) {
          return null;
        }
        const checked = new RegExp(
          `<input[^>]*name="${name}"[^>]*checked`,
        ).test(html);
        return createCheckboxStub(name, checked);
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
  if (selector === "[data-facet-preview], .vx-facet-preview") {
    const out = [];
    if (
      /data-facet-preview|vx-facet-preview|vista previa de facetas/i.test(
        html,
      )
    ) {
      out.push({});
    }
    return out;
  }
  const single = queryStub(html, selector);
  return single ? [single] : [];
}

function createActionTarget(action, attributeId) {
  return {
    getAttribute(name) {
      if (name === "data-action") {
        return action;
      }
      if (name === "data-attribute-id") {
        return attributeId ?? null;
      }
      return null;
    },
    closest(sel) {
      if (sel === "[data-action]") {
        return createActionTarget(action, attributeId);
      }
      return null;
    },
  };
}

function createFieldStub(name, bag = {}) {
  return {
    getAttribute(attr) {
      if (attr === "name") {
        return name;
      }
      if (attr === "value") {
        return bag[name] ?? "";
      }
      return null;
    },
    setAttribute(attr, value) {
      if (attr === "value") {
        bag[name] = String(value ?? "");
      }
    },
  };
}

function createCheckboxStub(name, checked = false) {
  const state = { checked: !!checked };
  return {
    name,
    get checked() {
      return state.checked;
    },
    set checked(v) {
      state.checked = !!v;
    },
    getAttribute(attr) {
      if (attr === "name") {
        return name;
      }
      if (attr === "checked") {
        return state.checked ? "" : null;
      }
      return null;
    },
  };
}

function createSelectStub(value) {
  const state = { value: String(value) };
  return {
    name: "type",
    get value() {
      return state.value;
    },
    set value(v) {
      state.value = String(v);
    },
    getAttribute(attr) {
      if (attr === "name") {
        return "type";
      }
      if (attr === "value") {
        return state.value;
      }
      return null;
    },
    closest(sel) {
      if (sel === "form") {
        return null;
      }
      return null;
    },
  };
}

function createFormTarget(formName, html) {
  /** @type {Record<string, string>} */
  const fields = {};
  const keyMatch = html.match(/name="key"[^>]*value="([^"]*)"/);
  const labelMatch = html.match(/name="label"[^>]*value="([^"]*)"/);
  const optionsMatch = html.match(/name="options"[^>]*value="([^"]*)"/);
  const unitMatch = html.match(/name="unit"[^>]*value="([^"]*)"/);
  const orderMatch = html.match(/name="order"[^>]*value="([^"]*)"/);
  if (keyMatch) {
    fields.key = keyMatch[1];
  }
  if (labelMatch) {
    fields.label = labelMatch[1];
  }
  if (optionsMatch) {
    fields.options = optionsMatch[1];
  }
  if (unitMatch) {
    fields.unit = unitMatch[1];
  }
  if (orderMatch) {
    fields.order = orderMatch[1];
  }

  let attributeId = null;
  const idMatch = html.match(/data-attribute-id="([^"]+)"/);
  if (idMatch) {
    attributeId = idMatch[1];
  }

  const typeSelected =
    html.match(/<option value="([^"]+)" selected>/)?.[1] ?? "text";
  const select = createSelectStub(typeSelected);
  const facetable = /name="facetable"[^>]*checked/.test(html);
  const searchable = /name="searchable"[^>]*checked/.test(html);
  const facetableBox = createCheckboxStub("facetable", facetable);
  const searchableBox = createCheckboxStub("searchable", searchable);

  return {
    getAttribute(name) {
      if (name === "data-form") {
        return formName;
      }
      if (name === "data-attribute-id") {
        return attributeId;
      }
      return null;
    },
    querySelector(sel) {
      if (sel.startsWith('vx-field[name="')) {
        const m = sel.match(/vx-field\[name="([^"]+)"\]/);
        if (!m) {
          return null;
        }
        const name = m[1];
        if (name === "options" && !html.includes('name="options"')) {
          return null;
        }
        return createFieldStub(name, fields);
      }
      if (sel === 'select[name="type"]') {
        return select;
      }
      if (sel === 'input[name="facetable"]') {
        return facetableBox;
      }
      if (sel === 'input[name="searchable"]') {
        return searchableBox;
      }
      return null;
    },
    fields,
    select,
    facetableBox,
    searchableBox,
  };
}

function apiError(code) {
  return {
    name: "ApiError",
    code,
    status:
      code === "conflict"
        ? 409
        : code === "not_found"
          ? 404
          : code === "network"
            ? 0
            : 400,
    title: code,
    errors: [],
  };
}

async function tick() {
  await new Promise((r) => setTimeout(r, 0));
  await new Promise((r) => setTimeout(r, 0));
}

describe("attrs_list_requests_include_inactive", () => {
  test("GET con includeInactive=true", async () => {
    const { request, calls } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });
    assert.equal(calls.length, 1);
    assert.equal(calls[0].method, "GET");
    assert.equal(
      calls[0].path,
      "/api/v1/attribute-definitions?includeInactive=true",
    );
  });
});

describe("attrs_list_preserves_api_order", () => {
  test("grado antes que medida según order", async () => {
    const { request } = createCallTracker([{ items: [GRADO, MEDIDA] }]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });
    const gradoIdx = el.html.indexOf('data-key="grado"');
    const medidaIdx = el.html.indexOf('data-key="medida"');
    assert.ok(gradoIdx >= 0);
    assert.ok(medidaIdx >= 0);
    assert.ok(gradoIdx < medidaIdx);
  });
});

describe("attrs_empty_list_offers_create", () => {
  test("muestra acción de crear", async () => {
    const { request } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });
    assert.match(el.html, /data-action="create"/);
  });
});

describe("attrs_create_grado_enum_facetable", () => {
  test("POST grado enum 2/5/8 facetable", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      GRADO,
      { items: [GRADO] },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const typeSelect = el.querySelector('select[name="type"]');
    typeSelect.value = "enum";
    el.dispatchChange(typeSelect);

    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="key"]').setAttribute("value", "grado");
    form
      .querySelector('vx-field[name="label"]')
      .setAttribute("value", "Grado");
    form
      .querySelector('vx-field[name="options"]')
      .setAttribute("value", "2, 5, 8");
    form.querySelector('input[name="facetable"]').checked = true;
    form.querySelector('input[name="searchable"]').checked = true;
    el.dispatchSubmit(form);

    await tick();

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.equal(post.path, "/api/v1/attribute-definitions");
    assert.equal(post.body.key, "grado");
    assert.equal(post.body.label, "Grado");
    assert.equal(post.body.type, "enum");
    assert.deepEqual(post.body.options, ["2", "5", "8"]);
    assert.equal(post.body.facetable, true);
    assert.match(el.html, /grado/i);
  });
});

describe("attrs_options_visible_only_for_enum", () => {
  test("options aparece solo con enum", async () => {
    const { request } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    assert.doesNotMatch(el.html, /name="options"/);

    const typeSelect = el.querySelector('select[name="type"]');
    typeSelect.value = "enum";
    el.dispatchChange(typeSelect);
    assert.match(el.html, /name="options"/);

    typeSelect.value = "text";
    el.dispatchChange(typeSelect);
    assert.doesNotMatch(el.html, /name="options"/);
  });
});

describe("attrs_edit_patches_label", () => {
  test("PATCH label", async () => {
    const { request, calls } = createCallTracker([
      { items: [GRADO] },
      GRADO_SAE,
      { items: [GRADO_SAE] },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="edit"][data-attribute-id="${GRADO.id}"]`,
    );
    el.dispatchClick(btn);
    const form = el.querySelector('form[data-form="edit"]');
    form
      .querySelector('vx-field[name="label"]')
      .setAttribute("value", "Grado SAE");
    el.dispatchSubmit(form);

    await tick();

    const patch = calls.find(
      (c) => c.method === "PATCH" && c.path.includes(GRADO.id),
    );
    assert.ok(patch);
    assert.equal(patch.body.label, "Grado SAE");
    assert.match(el.html, /Grado SAE/);
  });
});

describe("attrs_deactivate_deletes", () => {
  test("DELETE", async () => {
    const inactive = { ...GRADO, active: false };
    const { request, calls } = createCallTracker([
      { items: [GRADO] },
      null,
      { items: [inactive] },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-attribute-id="${GRADO.id}"]`,
    );
    el.dispatchClick(btn);
    await tick();

    const del = calls.find((c) => c.method === "DELETE");
    assert.ok(del);
    assert.match(del.path, /GRADO/);
    assert.match(el.html, /Inactiva|data-active="false"/i);
  });
});

describe("attrs_reactivate_patches_active_true", () => {
  test("PATCH active true", async () => {
    const inactive = { ...GRADO, active: false };
    const active = { ...GRADO, active: true };
    const { request, calls } = createCallTracker([
      { items: [inactive] },
      active,
      { items: [active] },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="reactivate"][data-attribute-id="${GRADO.id}"]`,
    );
    el.dispatchClick(btn);
    await tick();

    const patch = calls.find(
      (c) => c.method === "PATCH" && c.body?.active === true,
    );
    assert.ok(patch);
  });
});

describe("attrs_enum_without_options_shows_invalid_input", () => {
  test("POST enum vacío y showError invalid_input", async () => {
    const errors = [];
    const { request, calls } = createCallTracker([
      { items: [] },
      { throw: apiError("invalid_input") },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, {
      request,
      showError: (e) => errors.push(e.code),
    });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const typeSelect = el.querySelector('select[name="type"]');
    typeSelect.value = "enum";
    el.dispatchChange(typeSelect);

    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="key"]').setAttribute("value", "grado");
    form
      .querySelector('vx-field[name="label"]')
      .setAttribute("value", "Grado");
    form.querySelector('vx-field[name="options"]').setAttribute("value", "");
    el.dispatchSubmit(form);
    await tick();

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.equal(post.body.type, "enum");
    assert.deepEqual(post.body.options, []);
    assert.deepEqual(errors, ["invalid_input"]);
  });
});

describe("attrs_duplicate_key_shows_conflict", () => {
  test("showError conflict", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [] },
      { throw: apiError("conflict") },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, {
      request,
      showError: (e) => errors.push(e.code),
    });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="key"]').setAttribute("value", "grado");
    form
      .querySelector('vx-field[name="label"]')
      .setAttribute("value", "Grado");
    el.dispatchSubmit(form);
    await tick();

    assert.deepEqual(errors, ["conflict"]);
  });
});

describe("attrs_not_found_shows_not_found", () => {
  test("showError not_found", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [GRADO] },
      { throw: apiError("not_found") },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, {
      request,
      showError: (e) => errors.push(e.code),
    });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-attribute-id="${GRADO.id}"]`,
    );
    el.dispatchClick(btn);
    await tick();

    assert.deepEqual(errors, ["not_found"]);
    assert.match(el.html, /Grado|grado/);
  });
});

describe("attrs_network_shows_network", () => {
  test("GET falla con network", async () => {
    const errors = [];
    const { request } = createCallTracker([{ throw: apiError("network") }]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    assert.deepEqual(errors, ["network"]);
  });
});

describe("attrs_empty_key_or_label_does_not_post", () => {
  test("sin POST si key o label vacío", async () => {
    const { request, calls } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="key"]').setAttribute("value", "   ");
    form
      .querySelector('vx-field[name="label"]')
      .setAttribute("value", "Grado");
    el.dispatchSubmit(form);
    await tick();

    assert.equal(calls.filter((c) => c.method === "POST").length, 0);
  });
});

describe("attrs_non_enum_does_not_send_options", () => {
  test("POST text sin options", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      MEDIDA,
      { items: [MEDIDA] },
    ]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="key"]').setAttribute("value", "medida");
    form
      .querySelector('vx-field[name="label"]')
      .setAttribute("value", "Medida");
    el.dispatchSubmit(form);
    await tick();

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.equal(post.body.type, "text");
    assert.equal(Object.hasOwn(post.body, "options"), false);
  });
});

describe("attrs_no_facet_preview", () => {
  test("sin preview de facetas ni /search", async () => {
    const { request, calls } = createCallTracker([{ items: [GRADO] }]);
    const el = createMountEl();
    await mountAttributeDefinitions(el, { request, showError: () => {} });

    assert.doesNotMatch(el.html, /data-facet-preview|vx-facet-preview|vista previa de facetas/i);
    assert.equal(
      calls.filter((c) => String(c.path).includes("/search")).length,
      0,
    );
  });
});

describe("attrs_router_mounts_on_atributos", () => {
  test("GET al pintar #/admin/atributos", async () => {
    const calls = [];
    const request = async (method, url) => {
      calls.push({ method, path: url });
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
          return createFieldStub("terminal");
        }
        return null;
      },
    };
    const window = {
      location: { hash: "#/admin/atributos" },
      localStorage: {
        getItem: () => null,
        setItem() {},
        removeItem() {},
      },
      addEventListener() {},
    };
    startRouter({
      window,
      document: {},
      outlet,
      request,
      showError: () => {},
      mountAttributeDefinitions,
    });
    await tick();
    assert.ok(
      calls.some((c) => c.path.includes("/api/v1/attribute-definitions")),
    );
  });
});
