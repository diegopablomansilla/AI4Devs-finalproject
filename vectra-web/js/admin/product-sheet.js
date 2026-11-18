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
 */
function renderAttributeControl(def, index, fieldErrors) {
  const name = `attributes[${index}].value`;
  const label = attributeLabel(def.label, def.unit);
  const defId = escapeHtml(def.id);
  const type = String(def.type ?? "text");

  if (type === "boolean") {
    return `<div class="vx-product-attr-row" data-attr-definition-id="${defId}" data-attr-type="boolean">
    <label class="vx-product-sheet-select">${escapeHtml(label)}
      <select data-attr-index="${index}" data-attr-type="boolean">
        <option value="">—</option>
        <option value="true">Sí</option>
        <option value="false">No</option>
      </select>
    </label>
    ${vxField(name, "", fieldErrors)}
    </div>`;
  }

  if (type === "enum") {
    const opts = Array.isArray(def.options) ? def.options : [];
    const options = ['<option value="">—</option>']
      .concat(
        opts.map(
          (o) =>
            `<option value="${escapeHtml(String(o))}">${escapeHtml(String(o))}</option>`,
        ),
      )
      .join("\n");
    return `<div class="vx-product-attr-row" data-attr-definition-id="${defId}" data-attr-type="enum">
    <label class="vx-product-sheet-select">${escapeHtml(label)}
      <select data-attr-index="${index}" data-attr-type="enum">
${options}
      </select>
    </label>
    ${vxField(name, "", fieldErrors)}
    </div>`;
  }

  return `<div class="vx-product-attr-row" data-attr-definition-id="${defId}" data-attr-type="${escapeHtml(type)}">
    ${vxField(name, label, fieldErrors)}
    </div>`;
}

/**
 * @param {Array<{ id: string, label: string, type: string, unit?: string | null, options?: string[] | null, active?: boolean }>} attributeDefs
 * @param {Record<string, string>} fieldErrors
 */
function renderAttributesSection(attributeDefs, fieldErrors) {
  const active = activeAttributeDefinitions(attributeDefs);
  if (active.length === 0) {
    return `<section class="vx-product-attrs" aria-label="Atributos técnicos">
    <h2 class="vx-product-section-title">Atributos técnicos</h2>
    </section>`;
  }
  const rows = active
    .map((def, i) => renderAttributeControl(def, i, fieldErrors))
    .join("\n");
  return `<section class="vx-product-attrs" aria-label="Atributos técnicos">
  <h2 class="vx-product-section-title">Atributos técnicos</h2>
${rows}
  </section>`;
}

/**
 * @param {Record<string, string>} fieldErrors
 */
function renderCodesSection(fieldErrors) {
  return `<section class="vx-product-codes" aria-label="Códigos de producto">
  <h2 class="vx-product-section-title">Códigos</h2>
  ${vxField("codes.manufacturer", "Código de fabricante", fieldErrors)}
  ${vxField("codes.oem", "Código OEM", fieldErrors)}
  </section>`;
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