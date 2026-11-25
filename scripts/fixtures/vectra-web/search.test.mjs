import { describe, test, before, beforeEach } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  installDomGlobals,
  customElements,
  CustomEvent,
  KeyboardEvent,
} from "./dom-stub.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");

const searchUrl = pathToFileURL(
  path.join(REPO_ROOT, "vectra-web/js/search.js"),
).href;
const routerUrl = pathToFileURL(
  path.join(REPO_ROOT, "vectra-web/js/router.js"),
).href;

const waitMicrotask = () => new Promise((resolve) => queueMicrotask(resolve));
const waitTick = () => new Promise((resolve) => setTimeout(resolve, 0));
const waitPaint = () => new Promise((resolve) => setTimeout(resolve, 20));

function readCss(name) {
  return fs.readFileSync(path.join(REPO_ROOT, "vectra-web/css", name), "utf8");
}

let webComponentsReady = false;

async function ensureWebComponents() {
  if (webComponentsReady) {
    return;
  }
  const sharedUrl = new URL(
    "../../../vectra-web/js/components/shared.js",
    import.meta.url,
  ).href;
  const { setSharedStyleLoader } = await import(sharedUrl);
  setSharedStyleLoader(async (url) => {
    const file = url.pathname.endsWith("base.css") ? "base.css" : "tokens.css";
    return readCss(file);
  });
  await import(
    new URL("../../../vectra-web/js/components/vx-product-card.js", import.meta.url)
      .href
  );
  await import(
    new URL("../../../vectra-web/js/components/vx-omnibox.js", import.meta.url)
      .href
  );
  webComponentsReady = true;
}

function fullHit(overrides = {}) {
  return {
    name: "Perno de Acero",
    sku: "PER-STEEL",
    brand: "Fixser",
    price: "120.00",
    stock: "40",
    location: "P3 · E-B · 12",
    imageUrl: null,
    highlight: {
      name: "<mark>Perno</mark> de Acero",
      description: null,
    },
    ...overrides,
  };
}

function searchPayload(query, hits) {
  return {
    query,
    hits,
    estimatedTotalHits: hits.length,
    offset: 0,
    limit: 20,
    processingTimeMs: 1,
    degraded: false,
    facets: null,
  };
}

/** @returns {{ root: import("./dom-stub.mjs").FakeNode, results: import("./dom-stub.mjs").FakeNode, omnibox?: import("./dom-stub.mjs").FakeNode }} */
async function buildOperator({ withOmnibox = true } = {}) {
  const root = document.createElement("main");
  root.setAttribute("data-view", "operator");
  const omniboxWrap = document.createElement("div");
  omniboxWrap.setAttribute("data-region", "omnibox");
  const results = document.createElement("div");
  results.setAttribute("data-region", "results");
  root.appendChild(omniboxWrap);
  root.appendChild(results);
  document.body.appendChild(root);

  await ensureWebComponents();

  let omnibox = null;
  if (withOmnibox) {
    omnibox = customElements._create("vx-omnibox");
    omniboxWrap.appendChild(omnibox);
    await omnibox.connectedCallback();
  }

  return { root, results, omnibox };
}

function dispatchTextQuery(root, text) {
  root.dispatchEvent(
    new CustomEvent("vx-query", {
      bubbles: true,
      composed: true,
      detail: { mode: "text", text },
    }),
  );
}

/** @param {import("./dom-stub.mjs").FakeNode} results */
function resultsItemText(results) {
  const list = results.querySelector('[role="listbox"]');
  if (!list?._children?.length) {
    return "";
  }
  return list._children.map((c) => c.textContent ?? "").join(" ");
}

installDomGlobals();

/** @type {typeof import("../../../vectra-web/js/search.js")} */
let searchMod;
/** @type {typeof import("../../../vectra-web/js/router.js")} */
let routerMod;

describe("search", () => {
  before(async () => {
    searchMod = await import(searchUrl);
    routerMod = await import(routerUrl);
  });

  beforeEach(() => {
    document.body._children = [];
  });

  test("search_text_query_calls_get_search_with_q", async () => {
    const { root } = await buildOperator();
    const calls = [];
    const unmount = searchMod.mountSearch(root, {
      request: async (method, path) => {
        calls.push({ method, path });
        return searchPayload("perno m8", []);
      },
      showError: () => {},
    });
    dispatchTextQuery(root, "perno m8");
    await waitTick();
    assert.equal(calls.length, 1);
    assert.equal(calls[0].method, "GET");
    assert.match(calls[0].path, /^\/api\/v1\/search\?q=/);
    assert.match(calls[0].path, /perno/);
    assert.doesNotMatch(calls[0].path, /facets/);
    unmount();
  });

  test("search_cancels_previous_and_paints_last_only", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    /** @type {AbortSignal[]} */
    const signals = [];
    /** @type {Array<() => void>} */
    const resolveFirst = [];
    let n = 0;
    const unmount = searchMod.mountSearch(root, {
      request: async (_method, path, opts) => {
        n += 1;
        signals.push(opts.signal);
        if (n === 1) {
          return new Promise((resolve) => {
            resolveFirst.push(() =>
              resolve(
                searchPayload("p", [
                  { name: "Solo P", sku: "P-OLD" },
                ]),
              ),
            );
          });
        }
        assert.match(path, /perno%20m8|perno\+m8|perno m8/);
        return searchPayload("perno m8", [
          { name: "Perno M8", sku: "PER-M8" },
        ]);
      },
      showError: () => {},
    });

    dispatchTextQuery(root, "p");
    await waitMicrotask();
    dispatchTextQuery(root, "perno m8");
    await waitTick();

    assert.equal(signals[0].aborted, true);
    assert.equal(signals[1].aborted, false);
    const painted = resultsItemText(results);
    assert.match(painted, /Perno M8/);
    assert.doesNotMatch(painted, /Solo P/);

    resolveFirst[0]?.();
    await waitTick();
    assert.doesNotMatch(resultsItemText(results), /Solo P/);
    unmount();
  });

  test("search_renders_name_and_sku_as_text", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () => searchPayload("perno", [fullHit()]),
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitPaint();
    const painted = resultsItemText(results);
    assert.match(painted, /Perno de Acero/);
    assert.match(painted, /PER-STEEL/);
    assert.ok(results.querySelector("vx-product-card"));
    const listbox = results.querySelector('[role="listbox"]');
    assert.ok(listbox);
    unmount();
  });

  test("search_renders_full_product_card", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () => searchPayload("perno", [fullHit()]),
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitPaint();
    const card = results.querySelector("vx-product-card");
    assert.ok(card);
    assert.equal(card.getAttribute("sku"), "PER-STEEL");
    assert.equal(card.getAttribute("brand"), "Fixser");
    assert.equal(card.getAttribute("price"), "120.00");
    assert.equal(card.getAttribute("location"), "P3 · E-B · 12");
    const rootShadow = card.shadowRoot;
    assert.ok(rootShadow?.querySelector('[data-field="brand"]'));
    assert.ok(rootShadow?.querySelector('[data-field="location"]'));
    unmount();
  });

  test("search_zero_stock_marks_out_of_stock", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("x", [fullHit({ stock: "0", name: "Agotado", sku: "SIN-1" })]),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitPaint();
    const card = results.querySelector("vx-product-card");
    assert.ok(card);
    assert.equal(card.hasAttribute("data-out-of-stock"), true);
    assert.match(card.textContent ?? "", /Sin stock/i);
    unmount();
  });

  test("search_missing_image_url_shows_placeholder", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () => searchPayload("x", [fullHit({ imageUrl: null })]),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitPaint();
    const card = results.querySelector("vx-product-card");
    const rootShadow = card?.shadowRoot;
    assert.equal(rootShadow?.querySelector("img"), null);
    assert.ok(rootShadow?.querySelector('[data-field="image-placeholder"]'));
    unmount();
  });

  test("search_image_url_sets_card_src", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("x", [
          fullHit({ imageUrl: "https://example.test/perno.jpg" }),
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitPaint();
    const img = results
      .querySelector("vx-product-card")
      ?.shadowRoot?.querySelector("img");
    assert.ok(img);
    assert.equal(img.getAttribute("src"), "https://example.test/perno.jpg");
    unmount();
  });

  test("search_arrow_down_from_omnibox_selects_first_hit", async () => {
    const { root, omnibox } = await buildOperator();
    const input = omnibox?.shadowRoot?.querySelector("input");
    assert.ok(input);
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("perno", [
          { name: "Perno", sku: "PER-1" },
          { name: "Otro", sku: "PER-2" },
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitTick();
    input.dispatchEvent(
      new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }),
    );
    const listbox = root.querySelector('[role="listbox"]');
    assert.ok(listbox);
    assert.ok(listbox.getAttribute("aria-activedescendant"));
    unmount();
  });

  test("search_aborted_response_does_not_paint", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    let resolveLate = null;
    let n = 0;
    const unmount = searchMod.mountSearch(root, {
      request: async () => {
        n += 1;
        if (n === 1) {
          return new Promise((resolve) => {
            resolveLate = () =>
              resolve(
                searchPayload("p", [{ name: "Tarde", sku: "LATE" }]),
              );
          });
        }
        return searchPayload("pe", [{ name: "Pe", sku: "PE" }]);
      },
      showError: () => {},
    });
    dispatchTextQuery(root, "p");
    await waitMicrotask();
    dispatchTextQuery(root, "pe");
    await waitTick();
    resolveLate?.();
    await waitTick();
    assert.match(resultsItemText(results), /Pe/);
    assert.doesNotMatch(resultsItemText(results), /Tarde/);
    unmount();
  });

  test("search_abort_error_does_not_call_showError", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const unmount = searchMod.mountSearch(root, {
      request: async (_m, _p, opts) =>
        new Promise((_resolve, reject) => {
          opts.signal.addEventListener("abort", () => {
            const err = new DOMException("Aborted", "AbortError");
            reject(err);
          });
        }),
      showError: (e) => errors.push(e),
    });
    dispatchTextQuery(root, "a");
    await waitMicrotask();
    dispatchTextQuery(root, "b");
    await waitTick();
    assert.equal(errors.length, 0);
    unmount();
  });

  test("search_non_text_mode_does_not_call_search", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    let calls = 0;
    const unmount = searchMod.mountSearch(root, {
      request: async () => {
        calls += 1;
        return searchPayload("x", []);
      },
      showError: () => {},
    });
    for (const mode of ["ai", "voice", "image"]) {
      root.dispatchEvent(
        new CustomEvent("vx-query", {
          bubbles: true,
          composed: true,
          detail: { mode, text: mode === "ai" ? "?hola" : "x" },
        }),
      );
    }
    await waitTick();
    assert.equal(calls, 0);
    unmount();
  });

  test("search_empty_query_clears_without_fetch", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    let calls = 0;
    const unmount = searchMod.mountSearch(root, {
      request: async () => {
        calls += 1;
        return searchPayload("perno", [{ name: "Perno", sku: "P1" }]);
      },
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitTick();
    assert.match(resultsItemText(results), /Perno/);
    dispatchTextQuery(root, "   ");
    await waitTick();
    assert.equal(calls, 1);
    assert.equal(results.querySelector('[role="listbox"]'), null);
    unmount();
  });

  test("search_problem_json_calls_showError", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const unmount = searchMod.mountSearch(root, {
      request: async () => {
        const err = new Error("unavailable");
        err.name = "ApiError";
        err.code = "unavailable";
        throw err;
      },
      showError: (e) => errors.push(e),
    });
    dispatchTextQuery(root, "perno");
    await waitTick();
    assert.equal(errors.length, 1);
    assert.equal(errors[0].code, "unavailable");
    unmount();
  });

  test("search_empty_hits_shows_empty_list", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const unmount = searchMod.mountSearch(root, {
      request: async () => searchPayload("xyz", []),
      showError: (e) => errors.push(e),
    });
    dispatchTextQuery(root, "xyz");
    await waitTick();
    assert.equal(results.querySelector('[role="listbox"]'), null);
    assert.equal(errors.length, 0);
    unmount();
  });

  test("search_highlight_name_paints_mark", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("perno", [
          {
            name: "Perno M8",
            sku: "PER-M8",
            brand: "Fixser",
            price: "10",
            stock: "1",
            highlight: { name: "<mark>Perno</mark> M8", description: null },
          },
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitPaint();
    const list = results.querySelector('[role="listbox"]');
    assert.ok(list);
    assert.ok(results.querySelector("mark"));
    assert.match(resultsItemText(results), /PER-M8/);
    unmount();
  });

  test("search_highlight_description_does_not_paint_on_card", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("zinc", [
          {
            name: "Perno",
            sku: "P-1",
            brand: "X",
            price: "1",
            stock: "2",
            highlight: {
              name: null,
              description: "<mark>zincado</mark>",
            },
          },
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "zinc");
    await waitPaint();
    assert.equal(results.querySelector("mark"), null);
    assert.match(resultsItemText(results), /Perno/);
    unmount();
  });

  test("search_html_in_plain_fields_is_text", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("x", [
          fullHit({
            brand: '<img onerror="alert(1)">',
            sku: "SKU-1",
          }),
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitPaint();
    const card = results.querySelector("vx-product-card");
    assert.ok(card);
    assert.equal(results.querySelector("img"), null);
    assert.match(card.textContent ?? "", /<img/);
    unmount();
  });

  test("search_malformed_hit_does_not_throw", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("x", [
          null,
          "bad",
          fullHit({ sku: "GOOD", name: "Ok" }),
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitPaint();
    const cards = results.querySelectorAll("vx-product-card");
    assert.equal(cards.length, 1);
    assert.equal(cards[0].getAttribute("sku"), "GOOD");
    unmount();
  });

  test("search_highlight_null_falls_back_to_name", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("x", [
          {
            name: "Arandela",
            sku: "A-1",
            price: "1",
            stock: "1",
            highlight: null,
          },
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitPaint();
    assert.match(resultsItemText(results), /Arandela/);
    assert.equal(results.querySelector("mark"), null);
    unmount();
  });

  test("search_highlight_img_onerror_no_img_element", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: async () =>
        searchPayload("x", [
          {
            name: "Safe product",
            sku: "SKU-1",
            highlight: {
              name: '<img onerror="alert(1)">',
              description: null,
            },
          },
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitPaint();
    const list = results.querySelector('[role="listbox"]');
    assert.ok(list);
    assert.equal(results.querySelector("img"), null);
    assert.match(resultsItemText(results), /&lt;img/);
    unmount();
  });

  test("search_admin_view_does_not_call_search", async () => {
    let calls = 0;
    let html = "";
    const hashListeners = [];
    const win = {
      location: {
        _hash: "#/",
        get hash() {
          return this._hash;
        },
        set hash(v) {
          this._hash = v;
          for (const fn of hashListeners) fn();
        },
      },
      addEventListener(type, fn) {
        if (type === "hashchange") hashListeners.push(fn);
      },
    };
    const outlet = {
      get innerHTML() {
        return html;
      },
      set innerHTML(v) {
        html = v;
      },
      addEventListener(type, fn) {
        if (type === "vx-query") this._vxQuery = fn;
      },
      removeEventListener(type, fn) {
        if (this._vxQuery === fn) this._vxQuery = null;
      },
      dispatchEvent(event) {
        this._vxQuery?.(event);
        return true;
      },
      querySelector(sel) {
        if (sel === '[data-region="results"]') {
          return /data-region="results"/.test(html) ? { _children: [] } : null;
        }
        if (sel === "vx-omnibox") {
          return null;
        }
        return null;
      },
    };
    const doc = { getElementById: () => null };
    routerMod.startRouter({
      window: win,
      document: doc,
      outlet,
      request: async () => {
        calls += 1;
        return searchPayload("", []);
      },
      showError: () => {},
      mountSearch: searchMod.mountSearch,
    });
    win.location.hash = "#/admin/productos";
    dispatchTextQuery(outlet, "perno");
    await waitTick();
    assert.equal(calls, 0);
  });
});