import { bindKeyboardList } from "./keyboard-list.js";
import { appendSanitizedHtml } from "./highlight.js";
import { mountRecommendations } from "./recommendations.js";

/**
 * @param {string} q
 */
export function looksLikeCode(q) {
  return q.length >= 1 && q.length <= 64 && /^[A-Za-z0-9-]+$/.test(q);
}

/**
 * @param {Element} el
 */
function clearElement(el) {
  while (el.lastChild) {
    el.removeChild(el.lastChild);
  }
}

/**
 * @param {HTMLElement | Element} root
 * @param {{ request: (method: string, path: string, options?: { signal?: AbortSignal }) => Promise<unknown>, showError: (err: unknown) => void }} deps
 */
export function mountSearch(root, { request, showError }) {
  const { openRecommendations, unmount: unmountRecommendations } =
    mountRecommendations(root, { request, showError });

  /** @type {AbortController | null} */
  let inflight = null;
  /** @type {(() => void) | null} */
  let unbindList = null;
  let generation = 0;

  /**
   * @param {HTMLElement} card
   */
  function openResult(card) {
    const productId = card.getAttribute?.("data-product-id");
    if (!productId) {
      return;
    }
    void openRecommendations(productId, card);
  }

  function resultsRegion() {
    return typeof root.querySelector === "function"
      ? root.querySelector('[data-region="results"]')
      : null;
  }

  function unbindKeyboard() {
    if (unbindList) {
      unbindList();
      unbindList = null;
    }
  }

  function clearResults() {
    const results = resultsRegion();
    if (!results) {
      return;
    }
    unbindKeyboard();
    clearElement(results);
  }

  /**
   * @param {unknown} value
   * @returns {string}
   */
  function asAttrString(value) {
    if (value === null || value === undefined) {
      return "";
    }
    return String(value);
  }

  /**
   * @param {Record<string, unknown>} hit
   * @param {{ useHighlight?: boolean }} [opts]
   * @returns {HTMLElement | null}
   */
  function createProductCardForHit(hit, opts = {}) {
    const { useHighlight = true } = opts;
    const card = document.createElement("vx-product-card");
    card.setAttribute("variant", "full");

    const sku = asAttrString(hit.sku);
    const name = asAttrString(hit.name);
    if (sku) {
      card.setAttribute("sku", sku);
    }
    if (name) {
      card.setAttribute("name", name);
    }

    const brand = hit.brand;
    if (brand != null && asAttrString(brand).trim() !== "") {
      card.setAttribute("brand", asAttrString(brand));
    }
    const price = hit.price;
    if (price != null && asAttrString(price).trim() !== "") {
      card.setAttribute("price", asAttrString(price));
    }
    const stock = hit.stock;
    if (stock != null && asAttrString(stock).trim() !== "") {
      card.setAttribute("stock", asAttrString(stock));
    }
    const location = hit.location;
    if (location != null && asAttrString(location).trim() !== "") {
      card.setAttribute("location", asAttrString(location));
    }
    const imageUrl = hit.imageUrl;
    if (imageUrl != null && asAttrString(imageUrl).trim() !== "") {
      card.setAttribute("image-url", asAttrString(imageUrl).trim());
    }

    const productId = asAttrString(hit.id);
    if (productId) {
      card.setAttribute("data-product-id", productId);
    }

    const highlightName = useHighlight ? hit?.highlight?.name : null;
    if (highlightName != null && String(highlightName).trim() !== "") {
      const slot = document.createElement("span");
      slot.setAttribute("slot", "name");
      appendSanitizedHtml(slot, String(highlightName));
      card.appendChild(slot);
    }

    return card;
  }

  /**
   * @param {Array<Record<string, unknown>>} exactItems
   * @param {Array<Record<string, unknown>>} searchHits
   */
  function paintResults(exactItems, searchHits) {
    const results = resultsRegion();
    if (!results) {
      return;
    }
    unbindKeyboard();
    clearElement(results);

    const pinnedIds = new Set();
    for (const item of exactItems) {
      if (item && typeof item === "object" && item.id != null) {
        pinnedIds.add(asAttrString(item.id));
      }
    }

    const cards = [];
    const hasExact = exactItems.length > 0;

    const list = document.createElement("div");
    list.setAttribute("role", "listbox");
    list.setAttribute("tabindex", "0");

    if (hasExact) {
      const heading = document.createElement("p");
      heading.className = "vx-search-exact-label";
      heading.textContent = "Código exacto";
      list.appendChild(heading);
      for (const item of exactItems) {
        if (!item || typeof item !== "object") {
          continue;
        }
        const card = createProductCardForHit(item, { useHighlight: false });
        if (!card) {
          continue;
        }
        list.appendChild(card);
        cards.push(card);
      }
    }

    for (const hit of searchHits) {
      if (!hit || typeof hit !== "object") {
        continue;
      }
      const id = asAttrString(hit.id);
      if (id && pinnedIds.has(id)) {
        continue;
      }
      const card = createProductCardForHit(
        /** @type {Record<string, unknown>} */ (hit),
      );
      if (!card) {
        continue;
      }
      list.appendChild(card);
      cards.push(card);
    }

    if (!cards.length) {
      return;
    }

    results.appendChild(list);

    const omnibox =
      typeof root.querySelector === "function"
        ? root.querySelector("vx-omnibox")
        : null;
    const omniboxInput = omnibox?.shadowRoot?.querySelector?.("input") ?? null;
    const controllers = [list];
    if (omniboxInput) {
      controllers.push(omniboxInput);
    }
    unbindList = bindKeyboardList({
      list,
      getItems: () => cards,
      controllers,
      restoreFocus: omnibox ?? null,
      onOpen: (card) => {
        openResult(card);
      },
    });

    list.addEventListener("click", (event) => {
      /** @type {EventTarget | null} */
      let target = event.target ?? null;
      for (let i = 0; i < 8 && target; i += 1) {
        if (
          /** @type {HTMLElement} */ (target).localName === "vx-product-card"
        ) {
          openResult(/** @type {HTMLElement} */ (target));
          break;
        }
        target = /** @type {HTMLElement} */ (target).parentNode ?? null;
      }
    });
  }

  /**
   * @param {CustomEvent} event
   */
  async function onQuery(event) {
    const { mode, text } = event.detail ?? {};
    if (mode !== "text") {
      if (inflight) {
        inflight.abort();
        inflight = null;
      }
      return;
    }

    const q = String(text ?? "").trim();
    if (q === "") {
      if (inflight) {
        inflight.abort();
        inflight = null;
      }
      clearResults();
      return;
    }

    if (inflight) {
      inflight.abort();
    }
    const controller = new AbortController();
    inflight = controller;
    const gen = ++generation;
    const searchPath = `/api/v1/search?q=${encodeURIComponent(q.slice(0, 200))}`;
    const codeLike = looksLikeCode(q);
    const byCodePath = codeLike
      ? `/api/v1/search/by-code/${encodeURIComponent(q)}`
      : null;

    /** @type {Promise<unknown>[]} */
    const requests = [
      request("GET", searchPath, { signal: controller.signal }),
    ];
    if (byCodePath) {
      requests.push(
        request("GET", byCodePath, { signal: controller.signal }),
      );
    }

    try {
      const settled = await Promise.allSettled(requests);
      if (controller.signal.aborted || gen !== generation) {
        return;
      }

      const searchOutcome = settled[0];
      const byCodeOutcome = settled[1];

      let exactItems = [];
      let searchHits = [];
      let paintedSearch = false;
      let paintedExact = false;

      if (byCodeOutcome) {
        if (byCodeOutcome.status === "fulfilled") {
          const data = /** @type {{ items?: unknown[] }} */ (byCodeOutcome.value);
          exactItems = Array.isArray(data?.items)
            ? /** @type {Array<Record<string, unknown>>} */ (data.items)
            : [];
          paintedExact = exactItems.length > 0;
        } else if (byCodeOutcome.reason?.name !== "AbortError") {
          showError(byCodeOutcome.reason);
        }
      }

      if (searchOutcome.status === "fulfilled") {
        const data = /** @type {{ hits?: unknown[] }} */ (searchOutcome.value);
        searchHits = Array.isArray(data?.hits)
          ? /** @type {Array<Record<string, unknown>>} */ (data.hits)
          : [];
        paintedSearch = true;
      } else if (searchOutcome.reason?.name !== "AbortError") {
        showError(searchOutcome.reason);
      }

      if (!paintedSearch && !paintedExact) {
        return;
      }

      paintResults(exactItems, paintedSearch ? searchHits : []);
    } catch (err) {
      if (err?.name === "AbortError" || controller.signal.aborted) {
        return;
      }
      showError(err);
    } finally {
      if (inflight === controller) {
        inflight = null;
      }
    }
  }

  const handler = (event) => {
    void onQuery(event);
  };

  root.addEventListener("vx-query", handler);

  return () => {
    root.removeEventListener("vx-query", handler);
    if (inflight) {
      inflight.abort();
      inflight = null;
    }
    unbindKeyboard();
    generation += 1;
    unmountRecommendations();
  };
}