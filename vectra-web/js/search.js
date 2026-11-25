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
      const row = document.createElement("div");
      const nameWrap = document.createElement("span");
      const highlightName = hit?.highlight?.name;
      if (highlightName != null && String(highlightName).trim() !== "") {
        appendSanitizedHtml(nameWrap, highlightName);
      } else {
        nameWrap.textContent = hit?.name ?? "";
      }
      row.appendChild(nameWrap);
      row.appendChild(document.createTextNode(" · "));
      row.appendChild(document.createTextNode(hit?.sku ?? ""));
      const highlightDesc = hit?.highlight?.description;
      if (highlightDesc != null && String(highlightDesc).trim() !== "") {
        const descWrap = document.createElement("div");
        appendSanitizedHtml(descWrap, highlightDesc);
        row.appendChild(descWrap);
      }
      list.appendChild(row);
      items.push(row);
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