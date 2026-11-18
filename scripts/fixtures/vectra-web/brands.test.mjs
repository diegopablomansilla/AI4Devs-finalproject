import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const brandsUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/admin/brands.js"),
).href;
const routerUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/router.js"),
).href;

const { mountBrands } = await import(brandsUrl);
const { startRouter } = await import(routerUrl);

const FIXSER = {
  id: "01J9FIXSER000000000001",
  name: "Fixser",
  slug: "fixser",
  active: true,
};

const FIXSER_PRO = {
  ...FIXSER,
  name: "Fixser Pro",
  slug: "fixser-pro",
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
    selector.includes("[data-brand-id=")
  ) {
    const actionMatch = selector.match(/\[data-action="([^"]+)"\]/);
    const idMatch = selector.match(/\[data-brand-id="([^"]+)"\]/);
    if (actionMatch && idMatch) {
      const re = new RegExp(
        `data-action="${actionMatch[1]}"[^>]*data-brand-id="${idMatch[1]}"|data-brand-id="${idMatch[1]}"[^>]*data-action="${actionMatch[1]}"`,
      );
      if (re.test(html)) {
        return createActionTarget(actionMatch[1], idMatch[1]);
      }
    }
    return null;
  }

  const patterns = [
    {
      re: /^\[data-action="([^"]+)"\](?:\[data-brand-id="([^"]+)"\])?$/,
      run(m) {
        const action = m[1];
        const id = m[2];
        const attr = id
          ? `data-action="${action}" data-brand-id="${id}"`
          : `data-action="${action}"`;
        if (!html.includes(attr.split(" ")[0]) || html.indexOf(attr) < 0) {
          const simple = `data-action="${action}"`;
          if (!id && html.includes(simple)) {
            return createActionTarget(action, undefined);
          }
          if (id) {
            const idAttr = `data-brand-id="${id}"`;
            if (
              html.includes(`data-action="${action}"`) &&
              html.includes(idAttr)
            ) {
              return createActionTarget(action, id);
            }
          }
          return null;
        }
        return createActionTarget(action, id);
      },
    },
    {
      re: /^form\[data-form="([^"]+)"\]$/,
      run(m) {
        const form = m[1];
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
  if (selector === 'input[type="file"]') {
    const out = [];
    let i = 0;
    while (true) {
      const idx = html.indexOf('type="file"', i);
      if (idx < 0) {
        break;
      }
      out.push({});
      i = idx + 1;
    }
    return out;
  }
  if (selector === "img") {
    const out = [];
    let i = 0;
    while (true) {
      const idx = html.indexOf("<img", i);
      if (idx < 0) {
        break;
      }
      out.push({});
      i = idx + 1;
    }
    return out;
  }
  const single = queryStub(html, selector);
  return single ? [single] : [];
}

function createActionTarget(action, brandId) {
  return {
    getAttribute(name) {
      if (name === "data-action") {
        return action;
      }
      if (name === "data-brand-id") {
        return brandId ?? null;
      }
      return null;
    },
    closest(sel) {
      if (sel === "[data-action]") {
        return createActionTarget(action, brandId);
      }
      return null;
    },
  };
}

function createFormTarget(formName, html) {
  const fields = { name: "" };
  let brandId = null;
  const idMatch = html.match(/data-brand-id="([^"]+)"/);
  if (idMatch) {
    brandId = idMatch[1];
  }
  return {
    getAttribute(name) {
      if (name === "data-form") {
        return formName;
      }
      if (name === "data-brand-id") {
        return brandId;
      }
      return null;
    },
    querySelector(sel) {
      if (sel === 'vx-field[name="name"]') {
        return createFieldStub("name", fields);
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

function apiError(code) {
  return {
    name: "ApiError",
    code,
    status: code === "duplicate_slug" ? 409 : code === "not_found" ? 404 : 400,
    title: code,
    errors: [],
  };
}

describe("brands_list_requests_include_inactive", () => {
  test("GET con includeInactive=true", async () => {
    const { request, calls } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });
    assert.equal(calls.length, 1);
    assert.equal(calls[0].method, "GET");
    assert.equal(calls[0].path, "/api/v1/brands?includeInactive=true");
  });
});

describe("brands_empty_list_offers_create", () => {
  test("muestra acción de crear", async () => {
    const { request } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });
    assert.match(el.html, /data-action="create"/);
  });
});

describe("brands_create_fixser_posts_name", () => {
  test("POST con name sin slug", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      { ...FIXSER },
      { items: [FIXSER] },
    ]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="name"]').setAttribute("value", "Fixser");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.deepEqual(post.body, { name: "Fixser" });
    assert.equal(post.body.slug, undefined);
    assert.ok(el.html.includes("Fixser"));
  });
});

describe("brands_rename_patches_name", () => {
  test("PATCH renombra", async () => {
    const { request, calls } = createCallTracker([
      { items: [FIXSER] },
      FIXSER_PRO,
      { items: [FIXSER_PRO] },
    ]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="rename"][data-brand-id="${FIXSER.id}"]`,
    );
    el.dispatchClick(btn);
    const form = el.querySelector('form[data-form="rename"]');
    form.querySelector('vx-field[name="name"]').setAttribute("value", "Fixser Pro");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const patch = calls.find(
      (c) => c.method === "PATCH" && c.path.includes(FIXSER.id),
    );
    assert.ok(patch);
    assert.deepEqual(patch.body, { name: "Fixser Pro" });
    assert.match(el.html, /Fixser Pro/);
  });
});

describe("brands_deactivate_deletes", () => {
  test("DELETE", async () => {
    const inactive = { ...FIXSER, active: false };
    const { request, calls } = createCallTracker([
      { items: [FIXSER] },
      null,
      { items: [inactive] },
    ]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-brand-id="${FIXSER.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const del = calls.find((c) => c.method === "DELETE");
    assert.ok(del);
    assert.match(del.path, /FIXSER/);
    assert.match(el.html, /Inactiva|data-active="false"/i);
  });
});

describe("brands_reactivate_patches_active_true", () => {
  test("PATCH active true", async () => {
    const inactive = { ...FIXSER, active: false };
    const active = { ...FIXSER, active: true };
    const { request, calls } = createCallTracker([
      { items: [inactive] },
      active,
      { items: [active] },
    ]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });

    const btn = el.querySelector(
      `[data-action="reactivate"][data-brand-id="${FIXSER.id}"]`,
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

describe("brands_duplicate_slug_shows_duplicate_slug", () => {
  test("showError duplicate_slug", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [] },
      { throw: apiError("duplicate_slug") },
    ]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: (e) => errors.push(e.code) });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="name"]').setAttribute("value", "Dup");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["duplicate_slug"]);
  });
});

describe("brands_not_found_shows_not_found", () => {
  test("showError not_found", async () => {
    const errors = [];
    const { request } = createCallTracker([
      { items: [FIXSER] },
      { throw: apiError("not_found") },
    ]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: (e) => errors.push(e.code) });

    const btn = el.querySelector(
      `[data-action="deactivate"][data-brand-id="${FIXSER.id}"]`,
    );
    el.dispatchClick(btn);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    assert.deepEqual(errors, ["not_found"]);
    assert.match(el.html, /Fixser/);
  });
});

describe("brands_network_shows_network", () => {
  test("GET falla con network", async () => {
    const errors = [];
    const { request } = createCallTracker([{ throw: apiError("network") }]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: (e) => errors.push(e.code) });
    assert.deepEqual(errors, ["network"]);
  });
});

describe("brands_empty_name_does_not_post", () => {
  test("sin POST si name vacío", async () => {
    const { request, calls } = createCallTracker([{ items: [] }]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const form = el.querySelector('form[data-form="create"]');
    form.querySelector('vx-field[name="name"]').setAttribute("value", "   ");
    el.dispatchSubmit(form);

    await new Promise((r) => setTimeout(r, 0));

    assert.equal(calls.filter((c) => c.method === "POST").length, 0);
  });
});

describe("brands_no_logo_ui", () => {
  test("sin file ni img de logo", async () => {
    const { request } = createCallTracker([{ items: [FIXSER] }]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });
    assert.equal(el.querySelectorAll('input[type="file"]').length, 0);
    assert.doesNotMatch(el.html, /logo/i);
    assert.doesNotMatch(el.html, /<img\b/);
  });
});

describe("brands_does_not_send_slug", () => {
  test("POST y PATCH sin slug", async () => {
    const { request, calls } = createCallTracker([
      { items: [] },
      FIXSER,
      { items: [FIXSER] },
      FIXSER_PRO,
      { items: [FIXSER_PRO] },
    ]);
    const el = createMountEl();
    await mountBrands(el, { request, showError: () => {} });

    el.dispatchClick(el.querySelector('[data-action="create"]'));
    const createForm = el.querySelector('form[data-form="create"]');
    createForm.querySelector('vx-field[name="name"]').setAttribute("value", "Fixser");
    el.dispatchSubmit(createForm);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const post = calls.find((c) => c.method === "POST");
    assert.ok(post);
    assert.equal(Object.hasOwn(post.body, "slug"), false);

    const renameBtn = el.querySelector(
      `[data-action="rename"][data-brand-id="${FIXSER.id}"]`,
    );
    el.dispatchClick(renameBtn);
    const renameForm = el.querySelector('form[data-form="rename"]');
    renameForm.querySelector('vx-field[name="name"]').setAttribute("value", "Fixser Pro");
    el.dispatchSubmit(renameForm);

    await new Promise((r) => setTimeout(r, 0));
    await new Promise((r) => setTimeout(r, 0));

    const patch = calls.find((c) => c.method === "PATCH");
    assert.ok(patch);
    assert.equal(Object.hasOwn(patch.body, "slug"), false);
  });
});

describe("brands_router_mounts_on_marcas", () => {
  test("GET al pintar #/admin/marcas", async () => {
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
      location: { hash: "#/admin/marcas" },
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
      mountBrands,
    });
    await new Promise((r) => setTimeout(r, 0));
    assert.ok(calls.some((c) => c.path.includes("/api/v1/brands")));
  });
});