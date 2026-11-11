import { applyComponentStyles } from "./shared.js";

const LOCAL_CSS = `
:host {
  display: block;
}
article.card {
  box-sizing: border-box;
  padding: var(--vx-space-3);
  border-radius: var(--vx-radius-md);
  display: grid;
  gap: var(--vx-space-2);
}
:host([data-out-of-stock]) article.card {
  opacity: 0.85;
  border: 1px solid var(--vx-color-text-muted);
}
.name {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--vx-color-text);
}
.meta {
  margin: 0;
  font-size: 0.9rem;
  color: var(--vx-color-text-muted);
}
.out-of-stock {
  margin: 0;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--vx-color-error);
}
.media img {
  display: block;
  width: 100%;
  max-height: 8rem;
  object-fit: cover;
  border-radius: var(--vx-radius-sm);
}
.media [data-field="image-placeholder"] {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 5rem;
  border-radius: var(--vx-radius-sm);
  background: var(--vx-color-surface-solid);
  color: var(--vx-color-text-muted);
  font-size: 0.85rem;
}
`;

/**
 * @param {string | null} stock
 * @returns {boolean}
 */
function isZeroStock(stock) {
  if (stock === null || stock === "") {
    return false;
  }
  const normalized = String(stock).trim().replace(",", ".");
  const n = Number.parseFloat(normalized);
  return !Number.isNaN(n) && n === 0;
}

/**
 * @param {string | null} variant
 * @returns {"full" | "compact"}
 */
function effectiveVariant(variant) {
  return variant === "compact" ? "compact" : "full";
}

/**
 * @param {string | null} value
 * @returns {string}
 */
function displayText(value) {
  return value === null || value === undefined ? "" : String(value);
}

export class VxProductCard extends HTMLElement {
  static get observedAttributes() {
    return [
      "variant",
      "sku",
      "name",
      "brand",
      "price",
      "stock",
      "location",
      "image-url",
    ];
  }

  constructor() {
    super();
    /** @type {HTMLElement | null} */
    this._article = null;
  }

  async connectedCallback() {
    if (this.shadowRoot) {
      this._render();
      return;
    }
    const root = this.attachShadow({ mode: "open" });
    this._article = document.createElement("article");
    this._article.className = "vx-surface card";
    root.appendChild(this._article);
    await applyComponentStyles(root, LOCAL_CSS);
    this._render();
  }

  attributeChangedCallback() {
    this._render();
  }

  _hasSlottedName() {
    return Boolean(this.querySelector('[slot="name"]'));
  }

  /**
   * @param {HTMLElement} nameEl
   */
  _fillNameEl(nameEl) {
    const slotChild = this.querySelector('[slot="name"]');
    if (slotChild) {
      const html = slotChild.innerHTML;
      if (html && typeof nameEl.innerHTML === "string") {
        nameEl.innerHTML = html;
      } else {
        nameEl.textContent = slotChild.textContent ?? "";
      }
      return;
    }
    nameEl.textContent = displayText(this.getAttribute("name"));
  }

  _render() {
    if (!this._article) {
      return;
    }

    const variant = effectiveVariant(this.getAttribute("variant"));
    const compact = variant === "compact";
    const stock = this.getAttribute("stock");
    const outOfStock = isZeroStock(stock);

    if (outOfStock) {
      this.setAttribute("data-out-of-stock", "");
    } else {
      this.removeAttribute("data-out-of-stock");
    }

    const nameText = this._hasSlottedName()
      ? (this.querySelector('[slot="name"]')?.textContent ?? "")
      : displayText(this.getAttribute("name"));
    const sku = displayText(this.getAttribute("sku"));
    const brand = displayText(this.getAttribute("brand"));
    const price = displayText(this.getAttribute("price"));
    const stockText = displayText(stock);
    const location = displayText(this.getAttribute("location"));
    const imageUrl = (this.getAttribute("image-url") ?? "").trim();
    const altName = nameText || displayText(this.getAttribute("name")) || sku || "Producto";

    if (this._article._children) {
      this._article._children.length = 0;
    }

    // Build via DOM API so name uses textContent (never innerHTML from attribute)
    const nameEl = document.createElement("p");
    nameEl.className = "name";
    nameEl.setAttribute("data-field", "name");
    this._fillNameEl(nameEl);
    this._article.appendChild(nameEl);

    if (sku) {
      const el = document.createElement("p");
      el.className = "meta";
      el.setAttribute("data-field", "sku");
      el.textContent = `SKU: ${sku}`;
      this._article.appendChild(el);
    }
    if (!compact && brand) {
      const el = document.createElement("p");
      el.className = "meta";
      el.setAttribute("data-field", "brand");
      el.textContent = brand;
      this._article.appendChild(el);
    }
    if (price) {
      const el = document.createElement("p");
      el.className = "meta";
      el.setAttribute("data-field", "price");
      el.textContent = `$ ${price}`;
      this._article.appendChild(el);
    }
    if (stockText) {
      const el = document.createElement("p");
      el.className = "meta";
      el.setAttribute("data-field", "stock");
      el.textContent = `Stock: ${stockText}`;
      this._article.appendChild(el);
    }
    if (!compact && location) {
      const el = document.createElement("p");
      el.className = "meta";
      el.setAttribute("data-field", "location");
      el.textContent = location;
      this._article.appendChild(el);
    }
    if (outOfStock) {
      const el = document.createElement("p");
      el.className = "out-of-stock";
      el.setAttribute("data-field", "out-of-stock-label");
      el.textContent = "Sin stock";
      this._article.appendChild(el);
    }
    if (!compact) {
      const media = document.createElement("div");
      media.className = "media";
      if (imageUrl) {
        const img = document.createElement("img");
        img.setAttribute("data-field", "image");
        img.setAttribute("src", imageUrl);
        img.setAttribute("alt", altName);
        media.appendChild(img);
      } else {
        const ph = document.createElement("div");
        ph.setAttribute("data-field", "image-placeholder");
        ph.className = "placeholder";
        ph.textContent = "Sin imagen";
        media.appendChild(ph);
      }
      this._article.appendChild(media);
    }
  }
}

customElements.define("vx-product-card", VxProductCard);
