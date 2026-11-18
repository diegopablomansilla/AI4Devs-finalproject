/**
 * Admin: ficha de alta de producto (pestaña Datos).
 */

/**
 * @param {string} text
 */
function escapeHtml(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;");
}

/** @type {{ value: string, label: string }[]} */
export const UNIT_OF_MEASURE_OPTIONS = [
  { value: "unit", label: "Unidad" },
  { value: "pair", label: "Par" },
  { value: "box", label: "Caja" },
  { value: "meter", label: "Metro" },
  { value: "kilogram", label: "Kilogramo" },
  { value: "liter", label: "Litro" },
];

/**
 * @param {Array<{ id: string, name: string, active?: boolean }>} items
 */
export function activeLookupItems(items) {
  if (!Array.isArray(items)) {
    return [];
  }
  return items.filter((item) => item.active !== false);
}

/**
 * @param {Array<{ id: string, name: string }>} items
 * @param {string} placeholder
 * @param {string} selectedId
 */
function renderRequiredSelectOptions(items, placeholder, selectedId) {
  const sorted = [...items].sort((a, b) =>
    a.name.localeCompare(b.name, "es"),
  );
  const options = [`<option value="">${escapeHtml(placeholder)}</option>`];
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
 * @param {string} name
 * @param {string} label
 * @param {Record<string, string>} fieldErrors
 * @param {{ required?: boolean, value?: string }} [opts]
 */
function vxField(name, label, fieldErrors, opts = {}) {
  const err = fieldErrors[name] ?? "";
  const errAttr = err ? ` error="${escapeHtml(err)}"` : "";
  const required = opts.required ? " required" : "";
  const value = opts.value != null ? ` value="${escapeHtml(opts.value)}"` : "";
  return `<vx-field name="${escapeHtml(name)}" label="${escapeHtml(label)}"${required}${value}${errAttr}></vx-field>`;
}

/**
 * @param {Array<{ id: string, name: string, active?: boolean }>} categories
 * @param {Array<{ id: string, name: string, active?: boolean }>} brands
 * @param {Record<string, string>} [fieldErrors]
 */
export function renderCreateProductSheet(categories, brands, fieldErrors = {}) {
  const activeCats = activeLookupItems(categories);
  const activeBrands = activeLookupItems(brands);
  const unitOptions = UNIT_OF_MEASURE_OPTIONS.map((opt) => {
    const selected = opt.value === "unit" ? " selected" : "";
    return `<option value="${opt.value}"${selected}>${escapeHtml(opt.label)}</option>`;
  }).join("\n");

  return `<div class="vx-product-sheet">
  <h1 class="vx-product-sheet-title">Nuevo producto</h1>
  <div class="vx-product-tabs" role="tablist" aria-label="Ficha de producto">
    <button type="button" class="vx-product-tab" role="tab" aria-selected="true" data-tab="datos">Datos</button>
    <button type="button" class="vx-product-tab" role="tab" aria-selected="false" aria-disabled="true" data-tab="stock">Stock</button>
    <button type="button" class="vx-product-tab" role="tab" aria-selected="false" aria-disabled="true" data-tab="relaciones">Relaciones</button>
  </div>
  <form data-form="create-product" class="vx-product-sheet-form">
    ${vxField("sku", "SKU", fieldErrors, { required: true })}
    ${vxField("name", "Nombre", fieldErrors, { required: true })}
    ${vxField("productType", "Tipo", fieldErrors, { required: true })}
    ${vxField("description", "Descripción", fieldErrors)}
    <label class="vx-product-sheet-select">Categoría
      <select name="categoryId" data-sheet-field="categoryId">
${renderRequiredSelectOptions(activeCats, "Elegir categoría", "")}
      </select>
    </label>
    ${vxField("categoryId", "", fieldErrors)}
    <label class="vx-product-sheet-select">Marca
      <select name="brandId" data-sheet-field="brandId">
${renderRequiredSelectOptions(activeBrands, "Sin marca", "")}
      </select>
    </label>
    ${vxField("brandId", "", fieldErrors)}
    <label class="vx-product-sheet-select">Unidad
      <select name="unitOfMeasure" data-sheet-field="unitOfMeasure">
${unitOptions}
      </select>
    </label>
    ${vxField("unitOfMeasure", "", fieldErrors)}
    ${vxField("price", "Precio", fieldErrors, { required: true })}
    ${vxField("location.aisle", "Pasillo", fieldErrors)}
    ${vxField("location.shelf", "Estante", fieldErrors)}
    ${vxField("location.position", "Posición", fieldErrors)}
    ${vxField("initialStock", "Stock inicial", fieldErrors)}
    <div class="vx-product-sheet-actions">
      <button type="submit">Guardar</button>
      <button type="button" data-action="cancel-sheet">Cancelar</button>
    </div>
  </form>
</div>`;
}

/**
 * @param {ParentNode & { querySelector?: typeof Document.prototype.querySelector }} root
 * @param {Element | { querySelector?: Function }} form
 */
export function readProductCreate(root, form) {
  const query = (sel) =>
    form?.querySelector?.(sel) ?? root?.querySelector?.(sel) ?? null;

  const fieldValue = (name) => {
    const field = query(`vx-field[name="${name}"]`);
    const raw = field?.getAttribute?.("value") ?? "";
    return String(raw).trim();
  };

  const sku = fieldValue("sku");
  const name = fieldValue("name");
  const productType = fieldValue("productType");
  const price = fieldValue("price");
  const categorySelect = query('select[name="categoryId"]');
  const categoryId = String(categorySelect?.value ?? "").trim();

  if (
    sku.length === 0 ||
    name.length === 0 ||
    productType.length === 0 ||
    categoryId.length === 0 ||
    price.length === 0
  ) {
    return null;
  }

  /** @type {Record<string, unknown>} */
  const body = {
    sku,
    name,
    productType,
    categoryId,
    price,
    unitOfMeasure: String(query('select[name="unitOfMeasure"]')?.value ?? "unit"),
  };

  const description = fieldValue("description");
  if (description.length > 0) {
    body.description = description;
  }

  const brandId = String(query('select[name="brandId"]')?.value ?? "").trim();
  if (brandId.length > 0) {
    body.brandId = brandId;
  }

  const aisle = fieldValue("location.aisle");
  const shelf = fieldValue("location.shelf");
  const position = fieldValue("location.position");
  if (aisle.length > 0 || shelf.length > 0 || position.length > 0) {
    body.location = {
      aisle: aisle.length > 0 ? aisle : null,
      shelf: shelf.length > 0 ? shelf : null,
      position: position.length > 0 ? position : null,
    };
  }

  const initialStock = fieldValue("initialStock");
  if (initialStock.length > 0) {
    body.initialStock = initialStock;
  }

  return body;
}