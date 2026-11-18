/**
 * Admin: listado de productos (#/admin/productos).
 */

import {
  readProductCreate,
  renderCreateProductSheet,
} from "./product-sheet.js";

/**
 * @param {string} text
 */
function escapeHtml(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;");
}

/**
 * @typedef {{
 *   id: string,
 *   sku: string,
 *   name: string,
 *   productType: string,
 *   brand?: string | null,
 *   categoryPath?: string[],
 *   price: string,
 *   stock: string,
 *   active: boolean,
 * }} ProductSummary
 */

/**
 * @typedef {{ id: string, name: string, active?: boolean }} LookupItem
 */

/**
 * @param {{ categoryId?: string, brandId?: string, active?: boolean, cursor?: string | null }} filters
 */
export function productsQuery(filters) {
  const params = new URLSearchParams();
  const active = filters.active === false ? "false" : "true";
  params.set("active", active);
  if (filters.categoryId) {
    params.set("categoryId", filters.categoryId);
  }
  if (filters.brandId) {
    params.set("brandId", filters.brandId);
  }
  if (filters.cursor) {
    params.set("cursor", filters.cursor);
  }
  return `/api/v1/products?${params.toString()}`;
}

/**
 * @param {string[] | undefined | null} path
 */
function formatCategoryPath(path) {
  if (!Array.isArray(path) || path.length === 0) {
    return "";
  }
  return path.map((p) => String(p)).join(" > ");
}

/**
 * @param {ProductSummary} product
 */
function renderRow(product) {
  const id = escapeHtml(product.id);
  const inactive = product.active === false;
  const inactiveBadge = inactive
    ? ' <span class="vx-product-inactive">Inactivo</span>'
    : "";
  const actions = inactive
    ? `<button type="button" data-action="reactivate" data-product-id="${id}">Reactivar</button>`
    : `<button type="button" data-action="deactivate" data-product-id="${id}">Dar de baja</button>`;
  const brand = product.brand == null ? "" : escapeHtml(product.brand);
  const categoryPath = escapeHtml(formatCategoryPath(product.categoryPath));

  return `  <tr class="vx-product-row" data-product-id="${id}" data-active="${inactive ? "false" : "true"}">
    <td data-sku>${escapeHtml(product.sku)}</td>
    <td data-name>${escapeHtml(product.name)}${inactiveBadge}</td>
    <td data-type>${escapeHtml(product.productType ?? "")}</td>
    <td data-brand>${brand}</td>
    <td data-category>${categoryPath}</td>
    <td data-price>${escapeHtml(product.price ?? "")}</td>
    <td data-stock>${escapeHtml(product.stock ?? "")}</td>
    <td class="vx-product-actions">${actions}</td>
  </tr>`;
}

/**
 * @param {LookupItem[]} items
 * @param {string} selectedId
 */
function renderLookupOptions(items, selectedId) {
  const sorted = [...items].sort((a, b) =>
    a.name.localeCompare(b.name, "es"),
  );
  const options = ['<option value="">Todas</option>'];
  for (const item of sorted) {
    const id = escapeHtml(item.id);
    const selected = item.id === selectedId ? " selected" : "";
    options.push(
      `<option value="${id}"${selected}>${escapeHtml(item.name)}</option>`,
    );
  }
  return options.join("\n");
}

/**
 * @param {ProductSummary[]} items
 * @param {LookupItem[]} categories
 * @param {LookupItem[]} brands
 * @param {{ categoryId: string, brandId: string, active: boolean }} filters
 * @param {string | null} nextCursor
 */
function renderShell(items, categories, brands, filters, nextCursor) {
  const emptyMsg =
    items.length === 0
      ? `<p class="vx-product-empty">No hay productos.</p>`
      : "";
  const activeSelected = filters.active === false ? "false" : "true";
  const loadMore =
    nextCursor != null
      ? `<button type="button" data-action="load-more">Cargar más</button>`
      : "";
  const toolbar = `<div class="vx-product-toolbar"><button type="button" data-action="create">Nuevo producto</button></div>`;
  const rows =
    items.length === 0
      ? ""
      : items.map((p) => renderRow(p)).join("\n");

  return `<div class="vx-product-admin">
  <h1 class="vx-product-title">Productos</h1>
  ${toolbar}
  <div class="vx-product-filters">
    <label class="vx-product-filter">Categoría
      <select data-filter="category">
${renderLookupOptions(categories, filters.categoryId)}
      </select>
    </label>
    <label class="vx-product-filter">Marca
      <select data-filter="brand">
${renderLookupOptions(brands, filters.brandId)}
      </select>
    </label>
    <label class="vx-product-filter">Estado
      <select data-filter="active">
        <option value="true"${activeSelected === "true" ? " selected" : ""}>Activos</option>
        <option value="false"${activeSelected === "false" ? " selected" : ""}>Inactivos</option>
      </select>
    </label>
  </div>
  ${emptyMsg}
  <div class="vx-product-table-wrap">
    <table class="vx-product-table">
      <thead>
        <tr>
          <th scope="col">SKU</th>
          <th scope="col">Nombre</th>
          <th scope="col">Tipo</th>
          <th scope="col">Marca</th>
          <th scope="col">Categoría</th>
          <th scope="col">Precio</th>
          <th scope="col">Stock</th>
          <th scope="col">Acciones</th>
        </tr>
      </thead>
      <tbody>
${rows}
      </tbody>
    </table>
  </div>
  <div class="vx-product-load-more">${loadMore}</div>
</div>`;
}

/**
 * @param {ParentNode & { innerHTML?: string, querySelector?: typeof Document.prototype.querySelector }} el
 * @param {{ request: Function, showError?: Function }} deps
 */
export async function mountProducts(el, { request, showError = () => {} }) {
  if (!el || typeof request !== "function") {
    return;
  }

  /** @type {ProductSummary[]} */
  let items = [];
  /** @type {string | null} */
  let nextCursor = null;
  /** @type {LookupItem[]} */
  let categories = [];
  /** @type {LookupItem[]} */
  let brands = [];
  /** @type {{ categoryId: string, brandId: string, active: boolean }} */
  let filters = { categoryId: "", brandId: "", active: true };
  /** @type {"list" | "create"} */
  let mode = "list";
  /** @type {Record<string, string>} */
  let fieldErrors = {};

  const paint = () => {
    if (mode === "create") {
      el.innerHTML = renderCreateProductSheet(categories, brands, fieldErrors);
      return;
    }
    el.innerHTML = renderShell(
      items,
      categories,
      brands,
      filters,
      nextCursor,
    );
  };

  const fetchLookups = async () => {
    try {
      const [catData, brandData] = await Promise.all([
        request("GET", "/api/v1/categories?includeInactive=true"),
        request("GET", "/api/v1/brands?includeInactive=true"),
      ]);
      categories = Array.isArray(catData?.items) ? catData.items : [];
      brands = Array.isArray(brandData?.items) ? brandData.items : [];
    } catch (err) {
      showError(err);
      categories = [];
      brands = [];
    }
  };

  const fetchPage = async (cursor = null) => {
    const path = productsQuery({
      categoryId: filters.categoryId || undefined,
      brandId: filters.brandId || undefined,
      active: filters.active,
      cursor,
    });
    return request("GET", path);
  };

  const reloadFirstPage = async () => {
    try {
      const data = await fetchPage(null);
      items = Array.isArray(data?.items) ? data.items : [];
      nextCursor = data?.nextCursor ?? null;
      paint();
    } catch (err) {
      showError(err);
      paint();
    }
  };

  const loadMore = async () => {
    if (nextCursor == null) {
      return;
    }
    try {
      const data = await fetchPage(nextCursor);
      const pageItems = Array.isArray(data?.items) ? data.items : [];
      items = items.concat(pageItems);
      nextCursor = data?.nextCursor ?? null;
      paint();
    } catch (err) {
      showError(err);
    }
  };

  el.addEventListener("change", (ev) => {
    const target = ev.target;
    const filter = target?.getAttribute?.("data-filter");
    if (!filter) {
      return;
    }
    /** @type {string} */
    const rawValue = target?.value ?? target?.getAttribute?.("value") ?? "";
    if (filter === "category") {
      filters = { ...filters, categoryId: String(rawValue) };
    } else if (filter === "brand") {
      filters = { ...filters, brandId: String(rawValue) };
    } else if (filter === "active") {
      filters = { ...filters, active: String(rawValue) === "false" ? false : true };
    } else {
      return;
    }
    void reloadFirstPage();
  });

  el.addEventListener("click", (ev) => {
    const target = ev.target;
    const actionEl = target?.closest?.("[data-action]") ?? target;
    const action = actionEl?.getAttribute?.("data-action");
    if (!action) {
      return;
    }

    if (action === "create") {
      mode = "create";
      fieldErrors = {};
      paint();
      return;
    }

    if (action === "cancel-sheet") {
      mode = "list";
      fieldErrors = {};
      paint();
      return;
    }

    if (action === "load-more") {
      void loadMore();
      return;
    }

    const productId = actionEl.getAttribute?.("data-product-id");
    if (!productId) {
      return;
    }

    if (action === "deactivate") {
      void (async () => {
        try {
          await request("DELETE", `/api/v1/products/${productId}`);
          await reloadFirstPage();
        } catch (err) {
          showError(err);
        }
      })();
      return;
    }

    if (action === "reactivate") {
      void (async () => {
        try {
          const ficha = await request(
            "GET",
            `/api/v1/products/${productId}`,
          );
          const version = ficha?.version;
          await request("PATCH", `/api/v1/products/${productId}`, {
            body: { version, active: true },
          });
          await reloadFirstPage();
        } catch (err) {
          showError(err);
        }
      })();
    }
  });

  el.addEventListener("submit", (ev) => {
    const form = ev.target;
    const formKind = form?.getAttribute?.("data-form");
    if (formKind !== "create-product") {
      return;
    }
    ev.preventDefault?.();

    const body = readProductCreate(el, form);
    if (!body) {
      return;
    }

    void (async () => {
      try {
        await request("POST", "/api/v1/products", { body });
        mode = "list";
        fieldErrors = {};
        await reloadFirstPage();
      } catch (err) {
        fieldErrors = {};
        const list = Array.isArray(err?.errors) ? err.errors : [];
        for (const item of list) {
          const field = item?.field;
          if (field) {
            fieldErrors[field] = String(item.message ?? "");
          }
        }
        paint();
        showError(err);
      }
    })();
  });

  await fetchLookups();
  await reloadFirstPage();
}