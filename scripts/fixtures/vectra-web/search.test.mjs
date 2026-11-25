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
const waitPaint = () => new Promise((resolve) => setTimeout(resolve, 45));

/** @param {import("./dom-stub.mjs").FakeNode | Element} scope */
async function waitSearchPaint(scope) {
  await waitPaint();
  const root =
    typeof scope?.querySelectorAll === "function"
      ? scope
      : document;
  const cards = root.querySelectorAll?.("vx-product-card") ?? [];
  for (const card of cards) {
    if (typeof card.connectedCallback === "function") {
      await card.connectedCallback();
    }
  }
}

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

/** @param {string} path */
function isByCodePath(path) {
  return path.includes("/search/by-code/");
}

/** @param {string} path */
function isTextSearchPath(path) {
  return /^\/api\/v1\/search\?/.test(path);
}

/**
 * @param {{
 *   search?: (path: string, opts?: { signal?: AbortSignal }) => unknown | Promise<unknown>,
 *   byCode?: (path: string, opts?: { signal?: AbortSignal }) => unknown | Promise<unknown>,
 * }} handlers
 */
function createRoutingRequest(handlers = {}) {
  return async (method, path, opts) => {
    assert.equal(method, "GET");
    if (isByCodePath(path)) {
      if (handlers.byCode) {
        return handlers.byCode(path, opts);
      }
      return { items: [] };
    }
    if (isTextSearchPath(path)) {
      if (handlers.search) {
        return handlers.search(path, opts);
      }
      return searchPayload("", []);
    }
    throw new Error(`unexpected path: ${path}`);
  };
}

function productSummary(overrides = {}) {
  return {
    id: "01J9ZKAAAAAAAAAAAAAAAAAA",
    sku: "PER-STEEL",
    name: "Perno de Acero",
    productType: "Perno",
    brand: "Fixser",
    price: "120.00",
    stock: "40",
    active: true,
    imageUrl: null,
    ...overrides,
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
  const parts = [];
  for (const child of list._children) {
    if (child.localName === "vx-product-card") {
      parts.push(child.textContent ?? "");
      const sku = child.getAttribute?.("sku");
      if (sku) {
        parts.push(sku);
      }
    } else {
      parts.push(child.textContent ?? "");
    }
  }
  return parts.join(" ");
}

/** @param {import("./dom-stub.mjs").FakeNode} results */
function resultsRegionText(results) {
  return results.textContent ?? "";
}

/** @param {import("./dom-stub.mjs").FakeNode} results */
function listProductCards(results) {
  const list = results.querySelector('[role="listbox"]');
  if (!list?._children?.length) {
    return [];
  }
  return list._children.filter((c) => c.localName === "vx-product-card");
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
    assert.ok(calls.every((c) => !isByCodePath(c.path)));
    unmount();
  });

  test("search_cancels_previous_and_paints_last_only", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    /** @type {AbortSignal[]} */
    const signals = [];
    /** @type {Array<() => void>} */
    const resolveFirst = [];
    let searchN = 0;
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async (path, opts) => {
          searchN += 1;
          signals.push(opts.signal);
          if (searchN === 1) {
            return new Promise((resolve, reject) => {
              opts.signal.addEventListener("abort", () => {
                reject(new DOMException("Aborted", "AbortError"));
              });
              resolveFirst.push(() =>
                resolve(
                  searchPayload("p", [{ name: "Solo P", sku: "P-OLD" }]),
                ),
              );
            });
          }
          assert.match(path, /perno/);
          return searchPayload("perno m8", [
            { name: "Perno M8", sku: "PER-M8" },
          ]);
        },
        byCode: async (_path, opts) => {
          signals.push(opts.signal);
          return { items: [] };
        },
      }),
      showError: () => {},
    });

    dispatchTextQuery(root, "p");
    await waitMicrotask();
    dispatchTextQuery(root, "perno m8");
    await waitPaint();

    assert.ok(signals.length >= 2);
    for (const sig of signals.slice(0, -1)) {
      assert.equal(sig.aborted, true);
    }
    assert.equal(signals.at(-1).aborted, false);
    const painted = resultsItemText(results);
    assert.match(painted, /PER-M8/);
    assert.doesNotMatch(painted, /Solo P| P-OLD/);

    resolveFirst[0]?.();
    await waitTick();
    assert.doesNotMatch(resultsItemText(results), /Solo P/);
    unmount();
  });

  test("search_renders_name_and_sku_as_text", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async () => searchPayload("perno", [fullHit()]),
      }),
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
      request: createRoutingRequest({
        search: async () => searchPayload("perno", [fullHit()]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitSearchPaint(results);
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
      request: createRoutingRequest({
        search: async () =>
          searchPayload("x", [
            fullHit({ stock: "0", name: "Agotado", sku: "SIN-1" }),
          ]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitSearchPaint(results);
    const card = results.querySelector("vx-product-card");
    assert.ok(card);
    assert.equal(card.hasAttribute("data-out-of-stock"), true);
    assert.match(card.textContent ?? "", /Sin stock/i);
    unmount();
  });

  test("search_missing_image_url_shows_placeholder", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async () => searchPayload("x", [fullHit({ imageUrl: null })]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitSearchPaint(results);
    const card = results.querySelector("vx-product-card");
    const rootShadow = card?.shadowRoot;
    assert.equal(rootShadow?.querySelector("img"), null);
    assert.ok(rootShadow?.querySelector('[data-field="image-placeholder"]'));
    unmount();
  });

  test("search_image_url_sets_card_src", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async () =>
          searchPayload("x", [
            fullHit({ imageUrl: "https://example.test/perno.jpg" }),
          ]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitSearchPaint(results);
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
      request: createRoutingRequest({
        search: async () =>
          searchPayload("perno", [
            { name: "Perno", sku: "PER-1" },
            { name: "Otro", sku: "PER-2" },
          ]),
      }),
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
    let searchN = 0;
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async (_path, opts) => {
          searchN += 1;
          if (searchN === 1) {
            return new Promise((resolve, reject) => {
              opts.signal.addEventListener("abort", () => {
                reject(new DOMException("Aborted", "AbortError"));
              });
              resolveLate = () =>
                resolve(
                  searchPayload("p", [{ name: "Tarde", sku: "LATE" }]),
                );
            });
          }
          return searchPayload("pe", [{ name: "Pe", sku: "PE" }]);
        },
        byCode: async () => ({ items: [] }),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "p");
    await waitMicrotask();
    dispatchTextQuery(root, "pe");
    await waitPaint();
    resolveLate?.();
    await waitPaint();
    assert.match(resultsItemText(results), /PE/);
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
    /** @type {string[]} */
    const paths = [];
    const unmount = searchMod.mountSearch(root, {
      request: async (_method, path) => {
        paths.push(path);
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
    assert.equal(paths.length, 0);
    assert.ok(paths.every((p) => !isByCodePath(p)));
    unmount();
  });

  test("search_empty_query_clears_without_fetch", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
  /** @type {string[]} */
    const paths = [];
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async (path) => {
          paths.push(path);
          return searchPayload("perno", [{ name: "Perno", sku: "P1" }]);
        },
        byCode: async (path) => {
          paths.push(path);
          return { items: [] };
        },
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitPaint();
    assert.match(resultsItemText(results), /P1/);
    const callsBeforeClear = paths.length;
    assert.equal(callsBeforeClear, 2);
    dispatchTextQuery(root, "   ");
    await waitTick();
    assert.equal(paths.length, callsBeforeClear);
    assert.equal(results.querySelector('[role="listbox"]'), null);
    unmount();
  });

  test("search_problem_json_calls_showError", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async () => {
          const err = new Error("unavailable");
          err.name = "ApiError";
          err.code = "unavailable";
          throw err;
        },
        byCode: async () => ({ items: [] }),
      }),
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
      request: createRoutingRequest({
        search: async () => searchPayload("xyz", []),
      }),
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
      request: createRoutingRequest({
        search: async () =>
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
      }),
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
      request: createRoutingRequest({
        search: async () =>
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
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "zinc");
    await waitSearchPaint(results);
    assert.equal(results.querySelector("mark"), null);
    assert.match(resultsItemText(results), /Perno/);
    unmount();
  });

  test("search_html_in_plain_fields_is_text", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async () =>
          searchPayload("x", [
            fullHit({
              brand: '<img onerror="alert(1)">',
              sku: "SKU-1",
            }),
          ]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitSearchPaint(results);
    const card = results.querySelector("vx-product-card");
    assert.ok(card);
    assert.equal(results.querySelector("img"), null);
    assert.match(card.textContent ?? "", /<img/);
    unmount();
  });

  test("search_malformed_hit_does_not_throw", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async () =>
          searchPayload("x", [null, "bad", fullHit({ sku: "GOOD", name: "Ok" })]),
      }),
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
      request: createRoutingRequest({
        search: async () =>
          searchPayload("x", [
            {
              name: "Arandela",
              sku: "A-1",
              price: "1",
              stock: "1",
              highlight: null,
            },
          ]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "x");
    await waitSearchPaint(results);
    assert.match(resultsItemText(results), /Arandela/);
    assert.equal(results.querySelector("mark"), null);
    unmount();
  });

  test("search_highlight_img_onerror_no_img_element", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async () =>
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
      }),
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

  test("search_code_like_query_calls_by_code_in_parallel", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    /** @type {string[]} */
    const paths = [];
    /** @type {Array<{ resolve: () => void, path: string }>} */
    const pending = [];
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        search: async (path, opts) =>
          new Promise((resolve) => {
            paths.push(path);
            pending.push({
              path,
              resolve: () => resolve(searchPayload("OEM-PER-8840", [])),
            });
            if (pending.length === 2) {
              queueMicrotask(() => {
                for (const p of pending) p.resolve();
              });
            }
          }),
        byCode: async (path, opts) =>
          new Promise((resolve) => {
            paths.push(path);
            pending.push({
              path,
              resolve: () => resolve({ items: [] }),
            });
            if (pending.length === 2) {
              queueMicrotask(() => {
                for (const p of pending) p.resolve();
              });
            }
          }),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "OEM-PER-8840");
    await waitMicrotask();
    assert.equal(paths.length, 2);
    assert.ok(paths.some((p) => isTextSearchPath(p)));
    assert.ok(paths.some((p) => isByCodePath(p)));
    assert.match(paths.find(isByCodePath) ?? "", /OEM-PER-8840/);
    await waitTick();
    unmount();
  });

  test("search_shared_oem_pins_both_exact_matches_first", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const exact1 = productSummary({
      id: "01J9ZK111111111111111111",
      sku: "PER-ACERO-01",
      name: "Perno de Acero",
    });
    const exact2 = productSummary({
      id: "01J9ZK222222222222222222",
      sku: "PER-ACERO-02",
      name: "Perno alternativo OEM 8840",
    });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        byCode: async () => ({ items: [exact1, exact2] }),
        search: async () =>
          searchPayload("OEM-PER-8840", [
            {
              ...fullHit({
                name: exact1.name,
                sku: exact1.sku,
              }),
              id: exact1.id,
            },
            {
              ...fullHit({
                name: exact2.name,
                sku: exact2.sku,
              }),
              id: exact2.id,
            },
            fullHit({ sku: "PER-OTHER", name: "Otro" }),
          ]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "OEM-PER-8840");
    await waitPaint();
    assert.match(resultsRegionText(results), /Código exacto/);
    const cards = listProductCards(results);
    assert.equal(cards.length, 3);
    assert.equal(cards[0].getAttribute("sku"), "PER-ACERO-01");
    assert.equal(cards[1].getAttribute("sku"), "PER-ACERO-02");
    const skus = cards.map((c) => c.getAttribute("sku"));
    assert.equal(skus.filter((s) => s === "PER-ACERO-01").length, 1);
    assert.equal(skus.filter((s) => s === "PER-ACERO-02").length, 1);
    unmount();
  });

  test("search_arrow_down_selects_first_exact_match", async () => {
    const { root, omnibox, results } = await buildOperator();
    const input = omnibox?.shadowRoot?.querySelector("input");
    assert.ok(input);
    const exact = productSummary({
      id: "01J9ZKEXACT111111111111",
      sku: "PER-EXACT",
      name: "Exacto primero",
    });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        byCode: async () => ({ items: [exact] }),
        search: async () =>
          searchPayload("HX-0840Z", [
            { name: "Texto", sku: "PER-TEXT", id: "01J9ZK333333333333333333" },
          ]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "HX-0840Z");
    await waitPaint();
    input.dispatchEvent(
      new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }),
    );
    const cards = listProductCards(results);
    assert.ok(cards.length >= 2);
    assert.equal(cards[0].getAttribute("sku"), "PER-EXACT");
    assert.ok(cards[0].hasAttribute("aria-selected"));
    unmount();
  });

  test("search_phrase_with_spaces_does_not_call_by_code", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const paths = [];
    const unmount = searchMod.mountSearch(root, {
      request: async (_m, path) => {
        paths.push(path);
        return searchPayload("perno m8", []);
      },
      showError: () => {},
    });
    dispatchTextQuery(root, "perno m8");
    await waitTick();
    assert.equal(paths.length, 1);
    assert.ok(isTextSearchPath(paths[0]));
    unmount();
  });

  test("search_code_longer_than_64_does_not_call_by_code", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const code = "A".repeat(65);
    const paths = [];
    const unmount = searchMod.mountSearch(root, {
      request: async (_m, path) => {
        paths.push(path);
        return searchPayload(code, []);
      },
      showError: () => {},
    });
    dispatchTextQuery(root, code);
    await waitTick();
    assert.equal(paths.length, 1);
    assert.ok(isTextSearchPath(paths[0]));
    unmount();
  });

  test("search_by_code_empty_items_does_not_pin", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        byCode: async () => ({ items: [] }),
        search: async () =>
          searchPayload("HX-0840Z", [fullHit({ sku: "PER-HX", name: "Desde search" })]),
      }),
      showError: () => {},
    });
    dispatchTextQuery(root, "HX-0840Z");
    await waitPaint();
    assert.doesNotMatch(resultsRegionText(results), /Código exacto/);
    assert.match(resultsItemText(results), /PER-HX/);
    unmount();
  });

  test("search_by_code_error_still_paints_search_hits", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        byCode: async () => {
          const err = new Error("unavailable");
          err.name = "ApiError";
          err.code = "unavailable";
          throw err;
        },
        search: async () =>
          searchPayload("HX-0840Z", [fullHit({ sku: "PER-OK", name: "Desde Meili" })]),
      }),
      showError: (e) => errors.push(e),
    });
    dispatchTextQuery(root, "HX-0840Z");
    await waitPaint();
    assert.equal(errors.length, 1);
    assert.doesNotMatch(resultsRegionText(results), /Código exacto/);
    assert.match(resultsItemText(results), /PER-OK/);
    unmount();
  });

  test("search_search_error_still_paints_exact_matches", async () => {
    const { root, results } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const exact = productSummary({
      id: "01J9ZK444444444444444444",
      sku: "PER-EXACT-ONLY",
      name: "Solo exacto",
    });
    const unmount = searchMod.mountSearch(root, {
      request: createRoutingRequest({
        byCode: async () => ({ items: [exact] }),
        search: async () => {
          const err = new Error("network");
          err.name = "ApiError";
          err.code = "network";
          throw err;
        },
      }),
      showError: (e) => errors.push(e),
    });
    dispatchTextQuery(root, "HX-0840Z");
    await waitPaint();
    assert.equal(errors.length, 1);
    assert.match(resultsRegionText(results), /Código exacto/);
    assert.equal(
      listProductCards(results)[0]?.getAttribute("sku"),
      "PER-EXACT-ONLY",
    );
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