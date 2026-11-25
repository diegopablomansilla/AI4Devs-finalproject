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

function readCss(name) {
  return fs.readFileSync(path.join(REPO_ROOT, "vectra-web/css", name), "utf8");
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

  let omnibox = null;
  if (withOmnibox) {
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
      new URL("../../../vectra-web/js/components/vx-omnibox.js", import.meta.url)
        .href
    );
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
      request: async () =>
        searchPayload("perno", [
          {
            name: "Perno de Acero",
            sku: "PER-STEEL",
            highlight: {
              name: "<mark>Perno</mark> de Acero",
              description: null,
            },
          },
        ]),
      showError: () => {},
    });
    dispatchTextQuery(root, "perno");
    await waitTick();
    const painted = resultsItemText(results);
    assert.match(painted, /Perno de Acero/);
    assert.match(painted, /PER-STEEL/);
    assert.doesNotMatch(painted, /<mark>/);
    assert.equal(
      results.querySelector("vx-product-card"),
      null,
    );
    const listbox = results.querySelector('[role="listbox"]');
    assert.ok(listbox);
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

  test("search_highlight_html_is_plain_text", async () => {
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
    await waitTick();
    const painted = resultsItemText(results);
    assert.match(painted, /Safe product/);
    assert.doesNotMatch(painted, /<img/);
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