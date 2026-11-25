import { bindKeyboardList } from "./keyboard-list.js";
import { appendSanitizedHtml } from "./highlight.js";

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
  /** @type {AbortController | null} */
  let inflight = null;
  /** @type {(() => void) | null} */
  let unbindList = null;
  let generation = 0;

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
   * @returns {HTMLElement | null}
   */
  function createProductCardForHit(hit) {
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

    const highlightName = hit?.highlight?.name;
    if (highlightName != null && String(highlightName).trim() !== "") {
      const slot = document.createElement("span");
      slot.setAttribute("slot", "name");
      appendSanitizedHtml(slot, String(highlightName));
      card.appendChild(slot);
    }

    return card;
  }

  /**
   * @param {Array<{ name?: string, sku?: string, highlight?: { name?: string | null, description?: string | null } | null }>} hits
   */
  function paintHits(hits) {
    const results = resultsRegion();
    if (!results) {
      return;
    }
    unbindKeyboard();
    clearElement(results);
    if (!hits.length) {
      return;
    }

    const list = document.createElement("div");
    list.setAttribute("role", "listbox");
    list.setAttribute("tabindex", "0");
    const items = [];
    for (const hit of hits) {
      if (!hit || typeof hit !== "object") {
        continue;
      }
      const card = createProductCardForHit(
        /** @type {Record<string, unknown>} */ (hit),
      );
      if (!card) {
        continue;
      }
      list.appendChild(card);
      items.push(card);
    }
    if (!items.length) {
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
      getItems: () => items,
      controllers,
      restoreFocus: omnibox ?? null,
      onOpen: () => {},
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
    const path = `/api/v1/search?q=${encodeURIComponent(q.slice(0, 200))}`;

    try {
      const data = await request("GET", path, { signal: controller.signal });
      if (controller.signal.aborted || gen !== generation) {
        return;
      }
      const hits = Array.isArray(/** @type {{ hits?: unknown[] }} */ (data).hits)
        ? /** @type {{ hits: Array<{ name?: string, sku?: string }> }} */ (data)
            .hits
        : [];
      paintHits(hits);
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
  };
}