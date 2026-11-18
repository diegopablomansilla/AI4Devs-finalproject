import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const categoriesUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/admin/categories.js"),
).href;
const errorsUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/errors.js"),
).href;

const { buildTree, parentOptions, mountCategories } = await import(categoriesUrl);
const { messageFor } = await import(errorsUrl);

const FIJACIONES = {
  id: "01J9FIXACIONES0000000001",
  name: "Fijaciones",
  slug: "fijaciones",
  parentId: null,
  ancestorIds: [],
  path: ["Fijaciones"],
  active: true,
};

const PERNOS = {
  id: "01J9PERNOS000000000002",
  name: "Pernos",
  slug: "pernos",
  parentId: FIJACIONES.id,
  ancestorIds: [FIJACIONES.id],
  path: ["Fijaciones", "Pernos"],
  active: true,
};

const HERRAMIENTAS = {
  id: "01J9HERRAMIENTAS000003",
  name: "Herramientas",
  slug: "herramientas",
  parentId: null,
  ancestorIds: [],
  path: ["Herramientas"],
  active: true,
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

/**
 * @param {string} html
 */
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
    /** @param {{ target: unknown, preventDefault?: () => void }} event */
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

/**
 * @param {string} html
 * @param {string} selector
 */
function queryStub(html, selector) {
  if (
    selector.includes("[data-action=") &&
    selector.includes("[data-category-id=")
  ) {
    const actionMatch = selector.match(/\[data-action="([^"]+)"\]/);
    const idMatch = selector.match(/\[data-category-id="([^"]+)"\]/);
    if (actionMatch && idMatch) {
      const re = new RegExp(
        `data-action="${actionMatch[1]}"[^>]*data-category-id="${idMatch[1]}"|data-category-id="${idMatch[1]}"[^>]*data-action="${actionMatch[1]}"`,
      );
      if (re.test(html)) {
        return createActionTarget(actionMatch[1], idMatch[1], html, 0);
      }
    }
    return null;
  }

  const patterns = [
    {
      re: /^\[data-action="([^"]+)"\](?:\[data-category-id="([^"]+)"\])?$/,
      run(m) {
        const action = m[1];
        const id = m[2];
        const attr = id
          ? `data-action="${action}" data-category-id="${id}"`
          : `data-action="${action}"`;
        const idx = html.indexOf(attr);
        if (idx < 0) {
          return null;
        }
        return createActionTarget(action, id, html, idx);
      },
    },
    {
      re: /^form\[data-form="([^"]+)"\]$/,
      run(m) {
        const form = m[1];
        const marker = `data-form="${form}"`;
        if (!html.includes(marker)) {
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
        const match = html.match(re);
        if (!match) {
          return null;
        }
        return createFieldStub(name);
      },
    },
    {
      re: /^select\[name="parentId"\]$/,
      run() {
        if (!html.includes('name="parentId"')) {
          return null;
        }
        return createSelectStub();
      },
    },
    {
      re: /^\.vx-category-tree$/,
      run() {
        if (!html.includes('class="vx-category-tree"')) {
          return null;
        }
        return { className: "vx-category-tree", getAttribute: () => null };
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

/**
 * @param {string} html
 * @param {string} selector
 */
function queryAllStub(html, selector) {
  if (selector === "[draggable]") {
    const out = [];
    let i = 0;
    while (true) {
      const idx = html.indexOf("draggable", i);
      if (idx < 0) {
        break;
      }
      out.push({ getAttribute: (n) => (n === "draggable" ? "true" : null) });
      i = idx + 1;
    }
    return out;
  }
  if (selector === 'option[value="__root__"]') {
    return html.includes('value="__root__"') ? [{}] : [];
  }
  const single = queryStub(html, selector);
  return single ? [single] : [];
}

function createActionTarget(action, categoryId, html, idx) {
  const slice = html.slice(idx, idx + 200);
  return {
    getAttribute(name) {
      if (name === "data-action") {
        return action;
      }
      if (name === "data-category-id") {
        return categoryId ?? null;
      }
      return null;
    },
    closest(sel) {
      if (sel === "[data-action]") {
        return this;
      }
      return null;
    },
    contains(text) {
      return slice.includes(text);
    },
  };
}

function createFormTarget(formName, html = "") {
  const fields = { name: "", parentId: "" };
  let categoryId = null;
  let parentId = null;
  const catMatch = html.match(/data-category-id="([^"]+)"/);
  if (catMatch) {
    categoryId = catMatch[1];
  }
  const parentMatch = html.match(/data-parent-id="([^"]+)"/);
  if (parentMatch) {
    parentId = parentMatch[1];
  }
  return {
    getAttribute(name) {
      if (name === "data-form") {
        return formName;
      }
      if (name === "data-category-id") {
        return categoryId;
      }
      if (name === "data-parent-id") {
        return parentId;
      }
      return null;
    },
    querySelector(sel) {
      if (sel === 'vx-field[name="name"]') {
        return createFieldStub("name", fields);
      }
      if (sel === 'select[name="parentId"]') {
        return createSelectStub(fields);
      }
      if (sel === 'button[type="submit"]') {
        return { type: "submit" };
      }
      return null;
    },
    fields,
  };
}

function createFieldStub(name, bag = { name: "" }) {
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

function createSelectStub(bag = { parentId: "" }) {
  return {
    get value() {
      return bag.parentId;
    },
    set value(v) {
      bag.parentId = String(v ?? "");
    },
    querySelectorAll() {
      return [];
    },
  };
}

function apiError(code) {
  return {
    name: "ApiError",
    code,
    status: code === "duplicate_slug" ? 409 : 400,
    title: code,
    errors: [],
  };
}

describe("categories_tree_builds_from_parent_id", () => {
  test("anida por parentId", () => {
    const tree = buildTree([FIJACIONES, PERNOS, HERRAMIENTAS]);
    assert.equal(tree.length, 2);
    assert.equal(tree[0].category.name, "Fijaciones");
    assert.equal(tree[0].children.length, 1);
    assert.equal(tree[0].children[0].category.name, "Pernos");
  });
});

describe("categories_list_requests_include_inactive", () => {
  test("GET con includeInactive=true", async () => {
    const { request, calls } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });
    assert.equal(calls.length, 1);
    assert.equal(calls[0].method, "GET");
    assert.equal(
      calls[0].path,
      "/api/v1/categories?includeInactive=true",
    );
  });
});

describe("categories_empty_tree_offers_create_root", () => {
  test("muestra acción de crear raíz", async () => {
    const { request } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });
    assert.match(el.html, /data-action="create-root"/);
  });
});

describe("categories_create_subcategory_posts_name_and_parent", () => {
  test("POST con name y parentId", async () => {
    const { request, calls } = createCallTracker([
      { items: [FIJACIONES] },
      { id: PERNOS.id, name: "Pernos", parentId: FIJACIONES.id },
      { items: [FIJACIONES, PERNOS] },
    ]);
    const el = createMountEl();
    const mountPromise = mountCategories(el, { request, showError: () => {} });
    await mountPromise;

    const btn = el.querySelector(
      `[data-action="create-child"][data-category-id="${FIJACIONES.id}"]`,
    );
    assert.ok(btn);
    el.dispatchClick(btn);

    const form = el.querySelector('form[data-form="create-child"]');
    assert.ok(form);
    const field = form.querySelector('vx-field[name="name"]');
    field.setAttribute("value", "Pernos");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.deepEqual(post.body, {
      name: "Pernos",
      parentId: FIJACIONES.id,
    });
    assert.ok(el.html.includes("Pernos"));
  });
});

describe("categories_create_root_posts_without_parent", () => {
  test("POST sin parentId", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      { ...FIJACIONES },
      { items: [FIJACIONES] },
    ]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });

    const btn = el.querySelector('[data-action="create-root"]');
    el.dispatchClick(btn);
    const form = el.querySelector('form[data-form="create-root"]');
    const field = form.querySelector('vx-field[name="name"]');
    field.setAttribute("value", "Fijaciones");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.deepEqual(post.body, { name: "Fijaciones" });
    assert.equal(post.body.parentId, undefined);
  });
});

describe("categories_rename_patches_name", () => {
  test("PATCH renombra", async () => {
    const tornilleria = {
      ...FIJACIONES,
      name: "Tornillería",
      path: ["Tornillería"],
    };
    const pernosRenamed = {
      ...PERNOS,
      path: ["Tornillería", "Pernos"],
    };
    const { request, calls } = createCallTracker([
      { items: [FIJACIONES, PERNOS] },
      tornilleria,
      { items: [tornilleria, pernosRenamed] },
    ]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="rename"][data-category-id="${FIJACIONES.id}"]`,
    );
    el.dispatchClick(btn);
    const form = el.querySelector('form[data-form="rename"]');
    const field = form.querySelector('vx-field[name="name"]');
    field.setAttribute("value", "Tornillería");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const patch = calls.find(
      (c) => c.method === "PATCH" && c.path.includes(FIJACIONES.id),
    );
    assert.ok(patch);
    assert.deepEqual(patch.body, { name: "Tornillería" });
    assert.match(el.html, /Tornillería/);
  });
});

describe("categories_move_patches_parent_id", () => {
  test("PATCH con parentId", async () => {
    const pernosMoved = {
      ...PERNOS,
      parentId: HERRAMIENTAS.id,
      path: ["Herramientas", "Pernos"],
    };
    const { request, calls } = createCallTracker([
      { items: [FIJACIONES, PERNOS, HERRAMIENTAS] },
      pernosMoved,
      { items: [FIJACIONES, HERRAMIENTAS, pernosMoved] },
    ]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="move"][data-category-id="${PERNOS.id}"]`,
    );
    el.dispatchClick(btn);
    const form = el.querySelector('form[data-form="move"]');
    const select = form.querySelector('select[name="parentId"]');
    select.value = HERRAMIENTAS.id;
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const patch = calls.find(
      (c) => c.method === "PATCH" && c.path.includes(PERNOS.id),
    );
    assert.ok(patch);
    assert.deepEqual(patch.body, { parentId: HERRAMIENTAS.id });
  });
});

describe("categories_deactivate_deletes", () => {
  test("DELETE", async () => {
    const pernosInactive = { ...PERNOS, active: false };
    const { request, calls } = createCallTracker([
      { items: [FIJACIONES, PERNOS] },
      null,
      { items: [FIJACIONES, pernosInactive] },
    ]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-category-id="${PERNOS.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const del = calls.find((c) => c.method === "DELETE");
    assert.ok(del);
    assert.match(del.path, /PERNOS/);
    assert.match(el.html, /Inactiva|data-active="false"/i);
  });
});

describe("categories_reactivate_patches_active_true", () => {
  test("PATCH active true", async () => {
    const inactive = { ...PERNOS, active: false };
    const active = { ...PERNOS, active: true };
    const { request, calls } = createCallTracker([
      { items: [FIJACIONES, inactive] },
      active,
      { items: [FIJACIONES, active] },
    ]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="reactivate"][data-category-id="${PERNOS.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const patch = calls.find(
      (c) => c.method === "PATCH" && c.body?.active === true,
    );
    assert.ok(patch);
  });
});

describe("categories_cycle_shows_category_cycle", () => {
  test("showError con category_cycle", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [FIJACIONES, PERNOS] },
      { throw: apiError("category_cycle") },
    ]);
    const el = createMountEl();
    const showError = (err) => errors.push(err.code);
    await mountCategories(el, { request, showError });

    const btn = el.querySelector(
      `[data-action="move"][data-category-id="${FIJACIONES.id}"]`,
    );
    el.dispatchClick(btn);
    const form = el.querySelector('form[data-form="move"]');
    const select = form.querySelector('select[name="parentId"]');
    select.value = PERNOS.id;
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["category_cycle"]);
    assert.match(el.html, /Fijaciones/);
  });
});

describe("categories_duplicate_slug_shows_duplicate_slug", () => {
  test("showError duplicate_slug", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [FIJACIONES] },
      { throw: apiError("duplicate_slug") },
    ]);
    const el = createMountEl();
    await mountCategories(el, {
      request,
      showError: (e) => errors.push(e.code),
    });

    const btn = el.querySelector('[data-action="create-root"]');
    el.dispatchClick(btn);
    const form = el.querySelector('form[data-form="create-root"]');
    form.querySelector('vx-field[name="name"]').setAttribute("value", "Dup");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["duplicate_slug"]);
  });
});

describe("categories_deactivate_conflict_shows_conflict", () => {
  test("showError conflict", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [FIJACIONES, PERNOS] },
      { throw: apiError("conflict") },
    ]);
    const el = createMountEl();
    await mountCategories(el, {
      request,
      showError: (e) => errors.push(e.code),
    });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-category-id="${FIJACIONES.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["conflict"]);
  });
});

describe("categories_network_shows_network", () => {
  test("GET falla con network", async () => {
    const errors = [];
    const { request } = createCallTracker([{ throw: apiError("network") }]);
    const el = createMountEl();
    await mountCategories(el, {
      request,
      showError: (e) => errors.push(e.code),
    });
    assert.deepEqual(errors, ["network"]);
  });
});

describe("categories_empty_name_does_not_post", () => {
  test("sin POST si name vacío", async () => {
    const { request, calls } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create-root"]'));
    const form = el.querySelector('form[data-form="create-root"]');
    form.querySelector('vx-field[name="name"]').setAttribute("value", "   ");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));

    assert.equal(calls.filter((c) => c.method === "POST").length, 0);
  });
});

describe("categories_no_drag_and_drop", () => {
  test("sin draggable", async () => {
    const { request } = createCallTracker([{ items: [FIJACIONES, PERNOS] }]);
    const el = createMountEl();
    await mountCategories(el, { request, showError: () => {} });
    assert.equal(el.querySelectorAll("[draggable]").length, 0);
    assert.doesNotMatch(el.html, /draggable="true"/);
  });
});

describe("categories_parent_options_exclude_self_and_descendants", () => {
  test("parentOptions omite self y descendientes", () => {
    const items = [FIJACIONES, PERNOS, HERRAMIENTAS];
    const opts = parentOptions(items, FIJACIONES.id);
    const ids = opts.map((o) => o.id);
    assert.ok(ids.includes(null));
    assert.ok(ids.includes(HERRAMIENTAS.id));
    assert.ok(!ids.includes(FIJACIONES.id));
    assert.ok(!ids.includes(PERNOS.id));
  });
});

describe("categories_toast_uses_spanish_message", () => {
  test("messageFor no devuelve code crudo", () => {
    assert.equal(messageFor("category_cycle"), "La categoría formaría un ciclo");
    assert.notEqual(messageFor("category_cycle"), "category_cycle");
  });
});