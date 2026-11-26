import { describe, test, before, beforeEach } from "node:test";
import assert from "node:assert/strict";
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

const recsUrl = pathToFileURL(
  path.join(REPO_ROOT, "vectra-web/js/recommendations.js"),
).href;
const searchUrl = pathToFileURL(
  path.join(REPO_ROOT, "vectra-web/js/search.js"),
).href;

const waitMicrotask = () => new Promise((resolve) => queueMicrotask(resolve));
const waitTick = () => new Promise((resolve) => setTimeout(resolve, 0));
const waitPaint = () => new Promise((resolve) => setTimeout(resolve, 45));

function omniboxInput(box) {
  return box?.shadowRoot?.querySelector?.("input") ?? null;
}

/** @param {import("./dom-stub.mjs").FakeNode} panel */
function panelListText(panel) {
  const list = panel.querySelector('[role="listbox"]');
  let text = list?.textContent ?? "";
  for (const card of panel.querySelectorAll("vx-product-card")) {
    text += card.getAttribute("sku") ?? "";
    text += card.getAttribute("name") ?? "";
  }
  return text;
}

/** @param {import("./dom-stub.mjs").FakeNode} panel */
function findHighlightedHeading(panel) {
  const list = panel.querySelector('[role="listbox"]');
  const kids = list?._children ?? [];
  return kids.find((c) => c.hasAttribute?.("data-highlight-substitutes")) ?? null;
}

/** @param {string} path */
function isRecommendationsPath(path) {
  return /\/api\/v1\/products\/[^/]+\/recommendations$/.test(path);
}

function recommended(overrides = {}) {
  return {
    relationId: "01J9ZKRELAAAAAAAAAAAAAA",
    id: "01J9ZKPRODAAAAAAAAAAAAA",
    sku: "SKU-1",
    name: "Producto 1",
    stock: "12",
    price: "9.99",
    location: null,
    imageUrl: null,
    priority: 1,
    note: null,
    ...overrides,
  };
}

function recommendationsPayload(overrides = {}) {
  return {
    productId: "01J9ZKAAAAAAAAAAAAAAAAAA",
    highlightSubstitutes: false,
    similar: [],
    substitutes: [],
    complements: [],
    ...overrides,
  };
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
  setSharedStyleLoader(async () => "");
  await import(
    new URL("../../../vectra-web/js/components/vx-product-card.js", import.meta.url)
      .href
  );
  await import(
    new URL("../../../vectra-web/js/components/vx-side-panel.js", import.meta.url)
      .href
  );
  await import(
    new URL("../../../vectra-web/js/components/vx-omnibox.js", import.meta.url)
      .href
  );
  webComponentsReady = true;
}

async function buildOperator({ withOmnibox = true } = {}) {
  const root = document.createElement("main");
  root.setAttribute("data-view", "operator");
  const omniboxWrap = document.createElement("div");
  omniboxWrap.setAttribute("data-region", "omnibox");
  const results = document.createElement("div");
  results.setAttribute("data-region", "results");
  const panelWrap = document.createElement("div");
  panelWrap.setAttribute("data-region", "side-panel");
  root.appendChild(omniboxWrap);
  root.appendChild(results);
  root.appendChild(panelWrap);
  document.body.appendChild(root);

  await ensureWebComponents();
  const panel = customElements._create("vx-side-panel");
  panelWrap.appendChild(panel);
  await panel.connectedCallback();

  let omnibox = null;
  if (withOmnibox) {
    omnibox = customElements._create("vx-omnibox");
    omniboxWrap.appendChild(omnibox);
    await omnibox.connectedCallback();
  }

  return { root, panel, omnibox };
}

installDomGlobals();

/** @type {typeof import("../../../vectra-web/js/recommendations.js")} */
let recsMod;
/** @type {typeof import("../../../vectra-web/js/search.js")} */
let searchMod;

describe("recommendations", () => {
  before(async () => {
    recsMod = await import(recsUrl);
    searchMod = await import(searchUrl);
  });

  beforeEach(() => {
    document.body._children = [];
  });

  test("recs_three_blocks_in_api_order", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () =>
        recommendationsPayload({
          similar: [
            recommended({ sku: "SIM-B", name: "B similar", priority: 2 }),
            recommended({
              id: "01J9ZKSIM222222222222222",
              sku: "SIM-A",
              name: "A similar",
              priority: 1,
            }),
          ],
          substitutes: [recommended({ sku: "SUB-1", name: "Sub 1" })],
          complements: [recommended({ sku: "CMP-1", name: "Cmp 1" })],
        }),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    const text = panelListText(panel);
    assert.match(text, /Similares/);
    assert.match(text, /Sustitutos/);
    assert.match(text, /Se usa junto con/);
    const cards = panel.querySelectorAll("vx-product-card");
    assert.equal(cards.length, 4);
    assert.equal(cards[0]?.getAttribute("sku"), "SIM-B");
    assert.equal(cards[1]?.getAttribute("sku"), "SIM-A");
    for (const card of cards) {
      assert.equal(card.getAttribute("variant"), "compact");
    }
    unmount();
  });

  test("recs_compact_cards_not_full", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () =>
        recommendationsPayload({
          similar: [recommended({ sku: "C-1" })],
        }),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    const card = panel.querySelector("vx-product-card");
    assert.equal(card?.getAttribute("variant"), "compact");
    unmount();
  });

  test("recs_empty_arrays_still_show_headings", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () => recommendationsPayload(),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    assert.ok(panel.hasAttribute("open"));
    const emptyText = panelListText(panel);
    assert.match(emptyText, /Similares/);
    assert.match(emptyText, /Sustitutos/);
    assert.match(emptyText, /Se usa junto con/);
    assert.equal(panel.querySelectorAll("vx-product-card").length, 0);
    unmount();
  });

  test("recs_inactive_sku_absent_from_payload_is_absent", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () =>
        recommendationsPayload({
          similar: [recommended({ sku: "VISIBLE" })],
        }),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    const text = panelListText(panel);
    assert.match(text, /VISIBLE/);
    assert.doesNotMatch(text, /INACTIVO-FILTRADO/);
    unmount();
  });

  test("recs_highlights_substitutes_when_flag_true", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () =>
        recommendationsPayload({
          highlightSubstitutes: true,
          substitutes: [
            recommended({ sku: "SUB-STOCK", stock: "12" }),
          ],
        }),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    const heading = findHighlightedHeading(panel);
    assert.ok(heading);
    assert.match(panelListText(panel), /Destacados/);
    assert.match(panelListText(panel), /SUB-STOCK/);
    unmount();
  });

  test("recs_does_not_highlight_substitutes_when_flag_false", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () =>
        recommendationsPayload({
          highlightSubstitutes: false,
          substitutes: [recommended({ sku: "SUB-1" })],
        }),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    assert.equal(panel.querySelector("[data-highlight-substitutes]"), null);
    assert.equal(findHighlightedHeading(panel), null);
    assert.doesNotMatch(panelListText(panel), /Destacados/);
    unmount();
  });

  test("recs_opening_recommended_product_fetches_that_id", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const paths = [];
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async (_method, path) => {
        paths.push(path);
        if (paths.length === 1) {
          return recommendationsPayload({
            similar: [
              recommended({
                id: "01J9ZKOTHERBBBBBBBBBBBB",
                sku: "PICK-ME",
              }),
            ],
          });
        }
        return recommendationsPayload({
          productId: "01J9ZKOTHERBBBBBBBBBBBB",
          similar: [recommended({ sku: "INNER" })],
        });
      },
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    const inner = panel
      .querySelectorAll("vx-product-card")
      .find((c) => c.getAttribute("data-product-id") === "01J9ZKOTHERBBBBBBBBBBBB");
    assert.ok(inner);
    await openRecommendations("01J9ZKOTHERBBBBBBBBBBBB", inner);
    await waitPaint();
    assert.equal(paths.length, 2);
    assert.match(paths[1], /01J9ZKOTHERBBBBBBBBBBBB/);
    unmount();
  });

  test("recs_close_returns_focus_to_omnibox", async () => {
    const { root, panel, omnibox } = await buildOperator({ withOmnibox: true });
    const input = omniboxInput(omnibox);
    assert.ok(input);
    input.focus();
    const opener = document.createElement("vx-product-card");
    opener.setAttribute("data-product-id", "01J9ZKAAAAAAAAAAAAAAAAAA");
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () => recommendationsPayload(),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", opener);
    await waitPaint();
    panel.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await waitMicrotask();
    await waitMicrotask();
    assert.equal(document.activeElement, input);
    unmount();
  });

  test("recs_missing_product_id_does_not_fetch", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    let calls = 0;
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () => {
        calls += 1;
        return recommendationsPayload();
      },
      showError: () => {},
    });
    await openRecommendations("", undefined);
    await openRecommendations("   ", undefined);
    assert.equal(calls, 0);
    unmount();
  });

  test("recs_404_calls_showError_and_does_not_paint", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () => {
        const err = new Error("not_found");
        err.name = "ApiError";
        err.code = "not_found";
        throw err;
      },
      showError: (e) => errors.push(e),
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitTick();
    assert.equal(errors.length, 1);
    assert.equal(errors[0].code, "not_found");
    assert.ok(!panel.hasAttribute("open"));
    unmount();
  });

  test("recs_503_calls_showError", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () => {
        const err = new Error("unavailable");
        err.name = "ApiError";
        err.code = "unavailable";
        throw err;
      },
      showError: (e) => errors.push(e),
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitTick();
    assert.equal(errors.length, 1);
    assert.equal(errors[0].code, "unavailable");
    unmount();
  });

  test("recs_network_calls_showError", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const errors = [];
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () => {
        const err = new Error("network");
        err.name = "ApiError";
        err.code = "network";
        throw err;
      },
      showError: (e) => errors.push(e),
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitTick();
    assert.equal(errors.length, 1);
    assert.equal(errors[0].code, "network");
    unmount();
  });

  test("recs_abort_does_not_paint_stale", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    let generation = 0;
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async (_m, _p, opts) => {
        generation += 1;
        const g = generation;
        if (g === 1) {
          return new Promise((resolve, reject) => {
            opts?.signal?.addEventListener("abort", () => {
              reject(new DOMException("Aborted", "AbortError"));
            });
            setTimeout(
              () =>
                resolve(
                  recommendationsPayload({
                    similar: [recommended({ sku: "STALE" })],
                  }),
                ),
              80,
            );
          });
        }
        return recommendationsPayload({
          similar: [recommended({ sku: "FRESH" })],
        });
      },
      showError: () => {},
    });
    void openRecommendations("01J9ZKAAAAA111111111111111", undefined);
    await waitMicrotask();
    await openRecommendations("01J9ZKBBBBBBBBBBBBBBBBBB", undefined);
    await waitPaint();
    await new Promise((resolve) => setTimeout(resolve, 100));
    assert.doesNotMatch(panelListText(panel), /STALE/);
    assert.match(panelListText(panel), /FRESH/);
    unmount();
  });

  test("recs_abort_does_not_call_showError", async () => {
    const { root } = await buildOperator({ withOmnibox: false });
    const errors = [];
    let calls = 0;
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async (_m, _p, opts) => {
        calls += 1;
        if (calls > 1) {
          return recommendationsPayload();
        }
        return new Promise((_resolve, reject) => {
          opts?.signal?.addEventListener("abort", () => {
            reject(new DOMException("Aborted", "AbortError"));
          });
        });
      },
      showError: (e) => errors.push(e),
    });
    const first = openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitMicrotask();
    const second = openRecommendations("01J9ZKBBBBBBBBBBBBBBBBBB", undefined);
    await Promise.allSettled([first, second]);
    assert.equal(errors.length, 0);
    unmount();
  });

  test("recs_admin_view_does_not_fetch", async () => {
    const root = document.createElement("main");
    root.setAttribute("data-view", "admin");
    document.body.appendChild(root);
    let calls = 0;
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () => {
        calls += 1;
        return recommendationsPayload();
      },
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    assert.equal(calls, 0);
    unmount();
  });

  test("recs_no_add_to_cart_control", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const { openRecommendations, unmount } = recsMod.mountRecommendations(root, {
      request: async () =>
        recommendationsPayload({
          similar: [recommended({ sku: "X" })],
        }),
      showError: () => {},
    });
    await openRecommendations("01J9ZKAAAAAAAAAAAAAAAAAA", undefined);
    await waitPaint();
    assert.doesNotMatch(panelListText(panel), /Agregar/);
    for (const btn of panel.shadowRoot?.querySelectorAll?.("button") ?? []) {
      assert.doesNotMatch(btn.textContent ?? "", /Agregar/i);
    }
    unmount();
  });
});

describe("search recommendations open", () => {
  before(async () => {
    searchMod = await import(searchUrl);
  });

  beforeEach(() => {
    document.body._children = [];
  });

  test("search_enter_opens_recommendations_panel", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const paths = [];
    const unmount = searchMod.mountSearch(root, {
      request: async (_m, path) => {
        if (isRecommendationsPath(path)) {
          paths.push(path);
          return recommendationsPayload({
            similar: [recommended({ sku: "FROM-ENTER" })],
          });
        }
        return {
          query: "perno",
          hits: [
            {
              id: "01J9ZKAAAAAAAAAAAAAAAAAA",
              name: "Perno",
              sku: "PER-1",
              stock: "1",
              price: "10",
            },
          ],
          estimatedTotalHits: 1,
          offset: 0,
          limit: 20,
          processingTimeMs: 1,
          degraded: false,
          facets: null,
        };
      },
      showError: () => {},
    });
    root.dispatchEvent(
      new CustomEvent("vx-query", {
        bubbles: true,
        detail: { mode: "text", text: "perno" },
      }),
    );
    await waitPaint();
    const list = root.querySelector('[role="listbox"]');
    list?.focus();
    list?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    list?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await waitPaint();
    assert.equal(paths.length, 1);
    assert.ok(panel.hasAttribute("open"));
    assert.match(panelListText(panel), /FROM-ENTER/);
    unmount();
  });

  test("search_click_opens_recommendations_panel", async () => {
    const { root, panel } = await buildOperator({ withOmnibox: false });
    const results = root.querySelector('[data-region="results"]');
    const unmount = searchMod.mountSearch(root, {
      request: async (_m, path) => {
        if (isRecommendationsPath(path)) {
          return recommendationsPayload({
            similar: [recommended({ sku: "FROM-CLICK" })],
          });
        }
        return {
          query: "perno",
          hits: [
            {
              id: "01J9ZKCLICKAAAAAAAAAAAAA",
              name: "Perno clic",
              sku: "PER-CLK",
              stock: "2",
              price: "10",
            },
          ],
          estimatedTotalHits: 1,
          offset: 0,
          limit: 20,
          processingTimeMs: 1,
          degraded: false,
          facets: null,
        };
      },
      showError: () => {},
    });
    root.dispatchEvent(
      new CustomEvent("vx-query", {
        bubbles: true,
        detail: { mode: "text", text: "perno" },
      }),
    );
    await waitPaint();
    const card = results.querySelector("vx-product-card");
    assert.ok(card);
    assert.ok(card.getAttribute("data-product-id"));
    card.click();
    await waitPaint();
    assert.ok(panel.hasAttribute("open"));
    assert.match(panelListText(panel), /FROM-CLICK/);
    unmount();
  });
});