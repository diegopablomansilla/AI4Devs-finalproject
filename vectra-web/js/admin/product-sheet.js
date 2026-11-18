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
 * @param {Array<{ id: string, active?: boolean }>} items
 */
export function activeAttributeDefinitions(items) {
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
 * @param {string} label
 * @param {string | null | undefined} unit
 */
function attributeLabel(label, unit) {
  const u = unit != null ? String(unit).trim() : "";
  if (u.length > 0) {
    return `${label} (${u})`;
  }
  return label;
}

const DECIMAL_RE = /^-?\d+(\.\d+)?$/;

/**
 * @param {string} raw
 * @param {string} type
 */
export function coerceAttributeValue(raw, type) {
  const trimmed = String(raw ?? "").trim();
  if (trimmed.length === 0) {
    return null;
  }
  if (type === "number") {
    if (DECIMAL_RE.test(trimmed)) {
      return Number(trimmed);
    }
    return trimmed;
  }
  if (type === "boolean") {
    if (trimmed === "true") {
      return true;
    }
    if (trimmed === "false") {
      return false;
    }
    return null;
  }
  return trimmed;
}

/** @type {Record<string, string>} */
const PRODUCT_FIELD_LABELS = {
  sku: "SKU",
  name: "Nombre",
  productType: "Tipo",
  description: "Descripción",
  brandId: "Marca",
  categoryId: "Categoría",
  unitOfMeasure: "Unidad",
  price: "Precio",
  location: "Ubicación",
  attributes: "Atributos",
  codes: "Códigos",
};

/**
 * @param {unknown} value
 */
function formatAttrValue(value) {
  if (value === true) {
    return "true";
  }
  if (value === false) {
    return "false";
  }
  if (value == null) {
    return "";
  }
  return String(value);
}

/**
 * @param {Array<{ definitionId?: string, value?: unknown }> | undefined | null} attrs
 */
function sortAttributes(attrs) {
  if (!Array.isArray(attrs)) {
    return [];
  }
  return [...attrs]
    .map((a) => ({
      definitionId: String(a.definitionId ?? ""),
      value: a.value,
    }))
    .sort((a, b) => a.definitionId.localeCompare(b.definitionId));
}

/**
 * @param {Array<{ type?: string, value?: string }> | undefined | null} codes
 */
function sortCodes(codes) {
  if (!Array.isArray(codes)) {
    return [];
  }
  return [...codes]
    .map((c) => ({ type: String(c.type ?? ""), value: String(c.value ?? "") }))
    .sort((a, b) => a.type.localeCompare(b.type));
}

/**
 * @param {Record<string, unknown> | null | undefined} product
 */
function normalizeProductComparable(product) {
  if (!product) {
    return {};
  }
  const loc = product.location;
  const locationKey =
    loc && typeof loc === "object"
      ? JSON.stringify({
          aisle: /** @type {Record<string, unknown>} */ (loc).aisle ?? null,
          shelf: /** @type {Record<string, unknown>} */ (loc).shelf ?? null,
          position: /** @type {Record<string, unknown>} */ (loc).position ?? null,
        })
      : JSON.stringify({});
  const brand = product.brand;
  const brandId =
    brand && typeof brand === "object" && "id" in brand
      ? String(/** @type {{ id: string }} */ (brand).id)
      : null;
  const category = product.category;
  const categoryId =
    category && typeof category === "object" && "id" in category
      ? String(/** @type {{ id: string }} */ (category).id)
      : "";
  return {
    sku: String(product.sku ?? ""),
    name: String(product.name ?? ""),
    productType: String(product.productType ?? ""),
    description:
      product.description == null ? "" : String(product.description),
    brandId,
    categoryId,
    unitOfMeasure: String(product.unitOfMeasure ?? "unit"),
    price: String(product.price ?? ""),
    location: locationKey,
    attributes: JSON.stringify(sortAttributes(
      /** @type {Array<{ definitionId?: string, value?: unknown }>} */ (
        product.attributes
      ),
    )),
    codes: JSON.stringify(sortCodes(
      /** @type {Array<{ type?: string, value?: string }>} */ (product.codes),
    )),
  };
}

/**
 * @param {Record<string, unknown> | null | undefined} before
 * @param {Record<string, unknown> | null | undefined} after
 * @returns {{ keys: string[], labels: string[] }}
 */
export function diffChangedProductFields(before, after) {
  const a = normalizeProductComparable(before);
  const b = normalizeProductComparable(after);
  /** @type {string[]} */
  const keys = [];
  for (const key of Object.keys(PRODUCT_FIELD_LABELS)) {
    if (a[key] !== b[key]) {
      keys.push(key);
    }
  }
  return {
    keys,
    labels: keys.map((k) => PRODUCT_FIELD_LABELS[k] ?? k),
  };
}

/**
 * @param {Array<{ definitionId: string, value: unknown, type?: string }>} productAttrs
 * @param {string} definitionId
 * @param {string} type
 */
function attrValueForDefinition(productAttrs, definitionId, type) {
  const row = productAttrs.find((a) => a.definitionId === definitionId);
  if (!row) {
    return "";
  }
  const v = row.value;
  if (type === "boolean") {
    if (v === true) {
      return "true";
    }
    if (v === false) {
      return "false";
    }
    return "";
  }
  return formatAttrValue(v);
}

/**
 * @param {{
 *   id: string,
 *   label: string,
 *   type: string,
 *   unit?: string | null,
 *   options?: string[] | null,
 * }} def
 * @param {number} index
 * @param {Record<string, string>} fieldErrors
 * @param {Array<{ definitionId: string, value: unknown }>} [productAttrs]
 * @param {Set<string>} [changedKeys]
 */
function renderAttributeControl(
  def,
  index,
  fieldErrors,
  productAttrs = [],
  changedKeys = new Set(),
) {
  const name = `attributes[${index}].value`;
  const label = attributeLabel(def.label, def.unit);
  const defId = escapeHtml(def.id);
  const type = String(def.type ?? "text");
  const initial = attrValueForDefinition(productAttrs, def.id, type);
  const changedClass =
    changedKeys.has("attributes") ? " vx-product-field-changed" : "";

  if (type === "boolean") {
    const selTrue = initial === "true" ? " selected" : "";
    const selFalse = initial === "false" ? " selected" : "";
    return `<div class="vx-product-attr-row${changedClass}" data-attr-definition-id="${defId}" data-attr-type="boolean">
    <label class="vx-product-sheet-select">${escapeHtml(label)}
      <select data-attr-index="${index}" data-attr-type="boolean">
        <option value="">—</option>
        <option value="true"${selTrue}>Sí</option>
        <option value="false"${selFalse}>No</option>
      </select>
    </label>
    ${vxField(name, "", fieldErrors)}
    </div>`;
  }

  if (type === "enum") {
    const opts = Array.isArray(def.options) ? def.options : [];
    const options = ['<option value="">—</option>']
      .concat(
        opts.map((o) => {
          const s = String(o);
          const selected = initial === s ? " selected" : "";
          return `<option value="${escapeHtml(s)}"${selected}>${escapeHtml(s)}</option>`;
        }),
      )
      .join("\n");
    return `<div class="vx-product-attr-row${changedClass}" data-attr-definition-id="${defId}" data-attr-type="enum">
    <label class="vx-product-sheet-select">${escapeHtml(label)}
      <select data-attr-index="${index}" data-attr-type="enum">
${options}
      </select>
    </label>
    ${vxField(name, "", fieldErrors)}
    </div>`;
  }

  const fieldOpts =
    initial.length > 0 ? { value: initial } : {};
  return `<div class="vx-product-attr-row${changedClass}" data-attr-definition-id="${defId}" data-attr-type="${escapeHtml(type)}">
    ${vxField(name, label, fieldErrors, fieldOpts)}
    </div>`;
}

/**
 * @param {Array<{ id: string, label: string, type: string, unit?: string | null, options?: string[] | null, active?: boolean }>} attributeDefs
 * @param {Record<string, string>} fieldErrors
 * @param {Array<{ definitionId: string, value: unknown }>} [productAttrs]
 * @param {Set<string>} [changedKeys]
 */
function renderAttributesSection(
  attributeDefs,
  fieldErrors,
  productAttrs = [],
  changedKeys = new Set(),
) {
  const active = activeAttributeDefinitions(attributeDefs);
  if (active.length === 0) {
    return `<section class="vx-product-attrs" aria-label="Atributos técnicos">
    <h2 class="vx-product-section-title">Atributos técnicos</h2>
    </section>`;
  }
  const rows = active
    .map((def, i) =>
      renderAttributeControl(def, i, fieldErrors, productAttrs, changedKeys),
    )
    .join("\n");
  return `<section class="vx-product-attrs" aria-label="Atributos técnicos">
  <h2 class="vx-product-section-title">Atributos técnicos</h2>
${rows}
  </section>`;
}

/**
 * @param {Array<{ type: string, value: string }>} codes
 * @param {string} type
 */
function codeValue(codes, type) {
  const row = codes.find((c) => c.type === type);
  return row ? String(row.value ?? "") : "";
}

/**
 * @param {Record<string, string>} fieldErrors
 * @param {Array<{ type: string, value: string }>} [productCodes]
 * @param {Set<string>} [changedKeys]
 */
function renderCodesSection(
  fieldErrors,
  productCodes = [],
  changedKeys = new Set(),
) {
  const changedClass = changedKeys.has("codes")
    ? " vx-product-field-changed"
    : "";
  const mfg = codeValue(productCodes, "manufacturer");
  const oem = codeValue(productCodes, "oem");
  return `<section class="vx-product-codes${changedClass}" aria-label="Códigos de producto">
  <h2 class="vx-product-section-title">Códigos</h2>
  ${vxField("codes.manufacturer", "Código de fabricante", fieldErrors, mfg.length > 0 ? { value: mfg } : {})}
  ${vxField("codes.oem", "Código OEM", fieldErrors, oem.length > 0 ? { value: oem } : {})}
  </section>`;
}

/**
 * @param {string} fieldKey
 * @param {Set<string>} changedKeys
 */
function fieldChangedClass(fieldKey, changedKeys) {
  return changedKeys.has(fieldKey) ? " vx-product-field-changed" : "";
}

/**
 * @param {Array<{ id: string, name: string, active?: boolean }>} categories
 * @param {Array<{ id: string, name: string, active?: boolean }>} brands
 * @param {Array<{ id: string, label: string, type: string, unit?: string | null, options?: string[] | null, active?: boolean }>} [attributeDefs]
 * @param {Record<string, string>} [fieldErrors]
 */
export function renderCreateProductSheet(
  categories,
  brands,
  attributeDefs = [],
  fieldErrors = {},
) {
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
    ${renderAttributesSection(attributeDefs, fieldErrors)}
    ${renderCodesSection(fieldErrors)}
    <div class="vx-product-sheet-actions">
      <button type="submit">Guardar</button>
      <button type="button" data-action="cancel-sheet">Cancelar</button>
    </div>
  </form>
</div>`;
}

/**
 * @param {ParentNode & { querySelector?: typeof Document.prototype.querySelector, querySelectorAll?: typeof Document.prototype.querySelectorAll }} root
 * @param {Element | { querySelector?: Function, querySelectorAll?: Function }} form
 */
export function readProductCreate(root, form) {
  const query = (sel) =>
    form?.querySelector?.(sel) ?? root?.querySelector?.(sel) ?? null;
  const queryAll = (sel) => {
    const fromForm = form?.querySelectorAll?.(sel);
    if (fromForm && fromForm.length != null) {
      return fromForm;
    }
    const fromRoot = root?.querySelectorAll?.(sel);
    return fromRoot ?? [];
  };

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

  /** @type {Array<{ definitionId: string, value: unknown }>} */
  const attributes = [];
  const rows = queryAll(".vx-product-attr-row");
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    const defId = row?.getAttribute?.("data-attr-definition-id");
    const type = row?.getAttribute?.("data-attr-type") ?? "text";
    if (!defId) {
      continue;
    }
    let raw = "";
    if (type === "boolean" || type === "enum") {
      const sel = query(`select[data-attr-index="${i}"]`);
      raw = String(sel?.value ?? "").trim();
    } else {
      raw = fieldValue(`attributes[${i}].value`);
    }
    const value = coerceAttributeValue(raw, type);
    if (value === null) {
      continue;
    }
    attributes.push({ definitionId: defId, value });
  }
  if (attributes.length > 0) {
    body.attributes = attributes;
  }

  /** @type {Array<{ type: string, value: string }>} */
  const codes = [];
  const manufacturer = fieldValue("codes.manufacturer");
  if (manufacturer.length > 0) {
    codes.push({ type: "manufacturer", value: manufacturer });
  }
  const oem = fieldValue("codes.oem");
  if (oem.length > 0) {
    codes.push({ type: "oem", value: oem });
  }
  if (codes.length > 0) {
    body.codes = codes;
  }

  return body;
}

/**
 * @param {{
 *   sku: string,
 *   name: string,
 *   productType: string,
 *   description?: string | null,
 *   brand?: { id: string } | null,
 *   category: { id: string },
 *   attributes?: Array<{ definitionId: string, value: unknown }>,
 *   codes?: Array<{ type: string, value: string }>,
 *   unitOfMeasure?: string,
 *   price: string,
 *   stock: string,
 *   location?: { aisle?: string | null, shelf?: string | null, position?: string | null } | null,
 *   version: number,
 * }} product
 * @param {Array<{ id: string, name: string, active?: boolean }>} categories
 * @param {Array<{ id: string, name: string, active?: boolean }>} brands
 * @param {Array<{ id: string, label: string, type: string, unit?: string | null, options?: string[] | null, active?: boolean }>} attributeDefs
 * @param {Record<string, string>} fieldErrors
 * @param {string[]} [changedFieldKeys]
 * @param {string[]} [conflictLabels]
 */
export function renderEditProductSheet(
  product,
  categories,
  brands,
  attributeDefs = [],
  fieldErrors = {},
  changedFieldKeys = [],
  conflictLabels = [],
) {
  const activeCats = activeLookupItems(categories);
  const activeBrands = activeLookupItems(brands);
  const categoryId = product.category?.id ?? "";
  const brandId = product.brand?.id ?? "";
  const changedKeys = new Set(changedFieldKeys);
  const productAttrs = Array.isArray(product.attributes) ? product.attributes : [];
  const productCodes = Array.isArray(product.codes) ? product.codes : [];
  const loc = product.location ?? {};
  const unit = String(product.unitOfMeasure ?? "unit");

  const unitOptions = UNIT_OF_MEASURE_OPTIONS.map((opt) => {
    const selected = opt.value === unit ? " selected" : "";
    return `<option value="${opt.value}"${selected}>${escapeHtml(opt.label)}</option>`;
  }).join("\n");

  const conflictStatus =
    conflictLabels.length > 0
      ? `<p class="vx-product-conflict-status" role="status">Otra terminal modificó: ${conflictLabels.map(escapeHtml).join(", ")}</p>`
      : "";

  const desc =
    product.description == null ? "" : String(product.description);
  const aisle = loc.aisle == null ? "" : String(loc.aisle);
  const shelf = loc.shelf == null ? "" : String(loc.shelf);
  const position = loc.position == null ? "" : String(loc.position);

  return `<div class="vx-product-sheet">
  <h1 class="vx-product-sheet-title">Editar producto</h1>
  ${conflictStatus}
  <div class="vx-product-tabs" role="tablist" aria-label="Ficha de producto">
    <button type="button" class="vx-product-tab" role="tab" aria-selected="true" data-tab="datos">Datos</button>
    <button type="button" class="vx-product-tab" role="tab" aria-selected="false" aria-disabled="true" data-tab="stock">Stock</button>
    <button type="button" class="vx-product-tab" role="tab" aria-selected="false" aria-disabled="true" data-tab="relaciones">Relaciones</button>
  </div>
  <form data-form="edit-product" class="vx-product-sheet-form" data-product-version="${escapeHtml(String(product.version))}">
    <p class="vx-product-stock-readonly">Stock actual: ${escapeHtml(String(product.stock ?? ""))}</p>
    ${vxField("sku", "SKU", fieldErrors, { required: true, value: product.sku })}
    ${vxField("name", "Nombre", fieldErrors, { required: true, value: product.name })}
    ${vxField("productType", "Tipo", fieldErrors, { required: true, value: product.productType })}
    ${vxField("description", "Descripción", fieldErrors, { value: desc })}
    <label class="vx-product-sheet-select${fieldChangedClass("categoryId", changedKeys)}">Categoría
      <select name="categoryId" data-sheet-field="categoryId">
${renderRequiredSelectOptions(activeCats, "Elegir categoría", categoryId)}
      </select>
    </label>
    ${vxField("categoryId", "", fieldErrors)}
    <label class="vx-product-sheet-select${fieldChangedClass("brandId", changedKeys)}">Marca
      <select name="brandId" data-sheet-field="brandId">
${renderRequiredSelectOptions(activeBrands, "Sin marca", brandId)}
      </select>
    </label>
    ${vxField("brandId", "", fieldErrors)}
    <label class="vx-product-sheet-select${fieldChangedClass("unitOfMeasure", changedKeys)}">Unidad
      <select name="unitOfMeasure" data-sheet-field="unitOfMeasure">
${unitOptions}
      </select>
    </label>
    ${vxField("unitOfMeasure", "", fieldErrors)}
    <div class="vx-product-field-wrap${fieldChangedClass("price", changedKeys)}">
    ${vxField("price", "Precio", fieldErrors, { required: true, value: product.price })}
    </div>
    ${vxField("location.aisle", "Pasillo", fieldErrors, { value: aisle })}
    ${vxField("location.shelf", "Estante", fieldErrors, { value: shelf })}
    ${vxField("location.position", "Posición", fieldErrors, { value: position })}
    ${renderAttributesSection(attributeDefs, fieldErrors, productAttrs, changedKeys)}
    ${renderCodesSection(fieldErrors, productCodes, changedKeys)}
    <div class="vx-product-sheet-actions">
      <button type="submit">Guardar</button>
      <button type="button" data-action="cancel-sheet">Cancelar</button>
    </div>
  </form>
</div>`;
}

/**
 * @param {Array<{ definitionId: string, value: unknown }> | undefined} formAttributes
 * @param {Record<string, unknown>} snapshot
 * @param {Array<{ id: string, active?: boolean }>} attributeDefs
 */
export function mergePatchAttributes(formAttributes, snapshot, attributeDefs) {
  const activeIds = new Set(
    activeAttributeDefinitions(attributeDefs).map((d) => d.id),
  );
  const snapAttrs = Array.isArray(snapshot.attributes) ? snapshot.attributes : [];
  const preserved = snapAttrs
    .filter((a) => !activeIds.has(a.definitionId))
    .map((a) => ({ definitionId: a.definitionId, value: a.value }));
  const fromForm = Array.isArray(formAttributes) ? formAttributes : [];
  if (fromForm.length === 0 && preserved.length === 0) {
    return undefined;
  }
  return [...fromForm, ...preserved];
}

/**
 * @param {Array<{ type: string, value: string }> | undefined} formCodes
 * @param {Record<string, unknown>} snapshot
 */
export function mergePatchCodes(formCodes, snapshot) {
  const paintedTypes = new Set(["manufacturer", "oem"]);
  const snapCodes = Array.isArray(snapshot.codes) ? snapshot.codes : [];
  const preserved = snapCodes.filter((c) => !paintedTypes.has(c.type));
  const fromForm = Array.isArray(formCodes) ? formCodes : [];
  if (fromForm.length === 0 && preserved.length === 0) {
    return undefined;
  }
  return [...fromForm, ...preserved];
}

/**
 * @param {ParentNode & { querySelector?: typeof Document.prototype.querySelector, querySelectorAll?: typeof Document.prototype.querySelectorAll }} root
 * @param {Element | { querySelector?: Function, querySelectorAll?: Function, getAttribute?: Function }} form
 */
export function readProductUpdate(root, form) {
  const versionRaw = form?.getAttribute?.("data-product-version");
  const version = Number(versionRaw);
  if (!Number.isFinite(version) || version < 1) {
    return null;
  }

  const body = readProductCreate(root, form);
  if (!body) {
    return null;
  }
  delete body.initialStock;
  body.version = version;
  return body;
}