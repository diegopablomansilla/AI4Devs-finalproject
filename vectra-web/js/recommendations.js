import { bindKeyboardList } from "./keyboard-list.js";

/**
 * @param {Element} el
 */
function clearElement(el) {
  while (el.lastChild) {
    el.removeChild(el.lastChild);
  }
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
 * @param {HTMLElement | Element} root
 * @returns {boolean}
 */
function isOperatorRoot(root) {
  if (root.getAttribute?.("data-view") === "operator") {
    return true;
  }
  return (
    typeof root.querySelector === "function" &&
    root.querySelector('[data-view="operator"]') != null
  );
}

/**
 * @param {Record<string, unknown>} item
 * @returns {HTMLElement}
 */
function createCompactCard(item) {
  const card = document.createElement("vx-product-card");
  card.setAttribute("variant", "compact");

  const id = asAttrString(item.id);
  if (id) {
    card.setAttribute("data-product-id", id);
  }
  const sku = asAttrString(item.sku);
  if (sku) {
    card.setAttribute("sku", sku);
  }
  const name = asAttrString(item.name);
  if (name) {
    card.setAttribute("name", name);
  }
  const stock = item.stock;
  if (stock != null && asAttrString(stock).trim() !== "") {
    card.setAttribute("stock", asAttrString(stock));
  }
  const price = item.price;
  if (price != null && asAttrString(price).trim() !== "") {
    card.setAttribute("price", asAttrString(price));
  }
  const location = item.location;
  if (location != null && asAttrString(location).trim() !== "") {
    card.setAttribute("location", asAttrString(location));
  }
  const imageUrl = item.imageUrl;
  if (imageUrl != null && asAttrString(imageUrl).trim() !== "") {
    card.setAttribute("image-url", asAttrString(imageUrl).trim());
  }

  return card;
}

/**
 * @param {HTMLElement | Element} root
 * @param {{ request: (method: string, path: string, options?: { signal?: AbortSignal }) => Promise<unknown>, showError: (err: unknown) => void }} deps
 */
export function mountRecommendations(root, { request, showError }) {
  /** @type {AbortController | null} */
  let inflight = null;
  /** @type {(() => void) | null} */
  let unbindPanelList = null;
  let generation = 0;

  function sidePanelRegion() {
    return typeof root.querySelector === "function"
      ? root.querySelector('[data-region="side-panel"]')
      : null;
  }

  function sidePanel() {
    const region = sidePanelRegion();
    if (!region) {
      return null;
    }
    const fromQuery = region.querySelector?.("vx-side-panel");
    if (fromQuery) {
      return fromQuery;
    }
    const kids = region._children ?? [];
    return kids.find((c) => c.localName === "vx-side-panel") ?? null;
  }

  function unbindPanelKeyboard() {
    if (unbindPanelList) {
      unbindPanelList();
      unbindPanelList = null;
    }
  }

  /**
   * @param {Record<string, unknown>} data
   * @param {HTMLElement | undefined} opener
   */
  function paintPanelContent(data, opener) {
    const panel = sidePanel();
    if (!panel) {
      return;
    }

    unbindPanelKeyboard();
    clearElement(panel);

    const list = document.createElement("div");
    list.className = "vx-recs-list";
    list.setAttribute("role", "listbox");
    list.setAttribute("tabindex", "0");

    /** @type {HTMLElement[]} */
    const cards = [];

    /** @type {{ key: string, title: string, canHighlight?: boolean }[]} */
    const blocks = [
      { key: "similar", title: "Similares" },
      { key: "substitutes", title: "Sustitutos", canHighlight: true },
      { key: "complements", title: "Se usa junto con" },
    ];

    const highlightSubstitutes = data.highlightSubstitutes === true;

    for (const block of blocks) {
      const heading = document.createElement("p");
      heading.className = "vx-recs-heading";
      if (block.canHighlight && highlightSubstitutes) {
        heading.setAttribute("data-highlight-substitutes", "");
        heading.textContent = `${block.title} — Destacados`;
      } else {
        heading.textContent = block.title;
      }
      list.appendChild(heading);

      const items = Array.isArray(data[block.key]) ? data[block.key] : [];
      for (const item of items) {
        if (!item || typeof item !== "object") {
          continue;
        }
        const card = createCompactCard(
          /** @type {Record<string, unknown>} */ (item),
        );
        list.appendChild(card);
        cards.push(card);
      }
    }

    panel.appendChild(list);
    panel.show(opener);

    const omnibox =
      typeof root.querySelector === "function"
        ? root.querySelector("vx-omnibox")
        : null;

    unbindPanelList = bindKeyboardList({
      list,
      getItems: () => cards,
      restoreFocus: omnibox ?? null,
      onOpen: (card) => {
        const pid = card.getAttribute?.("data-product-id");
        if (pid) {
          void openRecommendations(pid, card);
        }
      },
    });
  }

  /**
   * @param {string} productId
   * @param {HTMLElement | undefined} opener
   */
  async function openRecommendations(productId, opener) {
    const id = String(productId ?? "").trim();
    if (!id || !isOperatorRoot(root)) {
      return;
    }

    if (inflight) {
      inflight.abort();
    }
    const controller = new AbortController();
    inflight = controller;
    const gen = ++generation;
    const path = `/api/v1/products/${encodeURIComponent(id)}/recommendations`;

    try {
      const data = await request("GET", path, { signal: controller.signal });
      if (controller.signal.aborted || gen !== generation) {
        return;
      }
      if (!data || typeof data !== "object") {
        return;
      }
      paintPanelContent(
        /** @type {Record<string, unknown>} */ (data),
        opener,
      );
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

  /**
   * @param {Event} event
   */
  function onPanelRegionClick(event) {
    const region = sidePanelRegion();
    if (!region) {
      return;
    }
    /** @type {EventTarget | null} */
    let target = event.target ?? null;
    let card = null;
    for (let i = 0; i < 8 && target; i += 1) {
      if (
        /** @type {HTMLElement} */ (target).localName === "vx-product-card"
      ) {
        card = /** @type {HTMLElement} */ (target);
        break;
      }
      target = /** @type {HTMLElement} */ (target).parentNode ?? null;
    }
    const panel = sidePanel();
    let inPanel = false;
    if (panel) {
      let walk = card.parentNode;
      while (walk) {
        if (walk === panel) {
          inPanel = true;
          break;
        }
        walk = walk.parentNode ?? null;
      }
    }
    if (!card || !inPanel) {
      return;
    }
    const pid = card.getAttribute("data-product-id");
    if (pid) {
      void openRecommendations(pid, card);
    }
  }

  const region = sidePanelRegion();
  region?.addEventListener("click", onPanelRegionClick);

  const unmount = () => {
    region?.removeEventListener("click", onPanelRegionClick);
    if (inflight) {
      inflight.abort();
      inflight = null;
    }
    unbindPanelKeyboard();
    generation += 1;
    const panel = sidePanel();
    if (panel) {
      panel.hide?.();
      clearElement(panel);
    }
  };

  return { openRecommendations, unmount };
}