/**
 * Admin: definiciones de atributos técnicos (#/admin/atributos).
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

/**
 * @typedef {{
 *   id: string,
 *   key: string,
 *   label: string,
 *   type: string,
 *   unit?: string | null,
 *   options?: string[] | null,
 *   facetable: boolean,
 *   searchable: boolean,
 *   order: number,
 *   active: boolean,
 * }} AttrItem
 */

/**
 * @typedef {{
 *   kind: 'create' | 'edit',
 *   attributeId?: string,
 *   type: string,
 *   defaults?: Partial<AttrItem> & { optionsText?: string },
 * }} AttrForm
 */

const TYPE_OPTIONS = [
  { value: "text", label: "Texto" },
  { value: "number", label: "Número" },
  { value: "boolean", label: "Booleano" },
  { value: "enum", label: "Enumerado" },
];

/**
 * @param {string} raw
 * @returns {string[]}
 */
export function parseOptionsText(raw) {
  return String(raw ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

/**
 * @param {AttrItem[]} items
 */
function renderRows(items) {
  if (items.length === 0) {
    return "";
  }
  return `<ul class="vx-attr-list">
${items
  .map((item) => {
    const id = escapeHtml(item.id);
    const label = escapeHtml(item.label ?? "");
    const key = escapeHtml(item.key ?? "");
    const type = escapeHtml(item.type ?? "");
    const unit = escapeHtml(item.unit ?? "");
    const inactive = item.active === false;
    const inactiveBadge = inactive
      ? ' <span class="vx-attr-inactive">Inactiva</span>'
      : "";
    const actions = inactive
      ? `<button type="button" data-action="reactivate" data-attribute-id="${id}">Reactivar</button>`
      : `<button type="button" data-action="edit" data-attribute-id="${id}">Editar</button>
      <button type="button" data-action="deactivate" data-attribute-id="${id}">Dar de baja</button>`;
    const unitSpan = unit
      ? `<span class="vx-attr-unit" data-unit>${unit}</span>`
      : "";
    return `  <li class="vx-attr-row" data-attribute-id="${id}" data-key="${key}" data-active="${inactive ? "false" : "true"}">
    <span data-label>${label}</span>${inactiveBadge}
    <span class="vx-attr-key" data-key>${key}</span>
    <span class="vx-attr-type" data-type>${type}</span>
    ${unitSpan}
    <div class="vx-attr-actions">${actions}</div>
  </li>`;
  })
  .join("\n")}
</ul>`;
}

/**
 * @param {AttrForm | null} form
 */
function renderForm(form) {
  if (!form) {
    return "";
  }
  const titles = {
    create: "Nueva definición",
    edit: "Editar definición",
  };
  const title = titles[form.kind] ?? "Atributo";
  const d = form.defaults ?? {};
  const type = form.type || d.type || "text";
  const keyVal = escapeHtml(d.key ?? "");
  const labelVal = escapeHtml(d.label ?? "");
  const unitVal = escapeHtml(d.unit ?? "");
  const orderVal =
    d.order !== undefined && d.order !== null ? String(d.order) : "";
  const optionsText =
    d.optionsText ??
    (Array.isArray(d.options) ? d.options.join(", ") : "");
  const facetable = d.facetable === true;
  const searchable = d.searchable !== false;
  const idAttr = form.attributeId
    ? ` data-attribute-id="${escapeHtml(form.attributeId)}"`
    : "";

  const typeSelect = `<label class="vx-attr-type-label">Tipo
  <select name="type" data-field="type">
${TYPE_OPTIONS.map(
  (opt) =>
    `    <option value="${opt.value}"${opt.value === type ? " selected" : ""}>${opt.label}</option>`,
).join("\n")}
  </select>
</label>`;

  const optionsField =
    type === "enum"
      ? `<vx-field name="options" label="Opciones (separadas por coma)" value="${escapeHtml(optionsText)}"></vx-field>`
      : "";

  return `<form data-form="${escapeHtml(form.kind)}" class="vx-attr-form"${idAttr}>
  <h2 class="vx-attr-form-title">${escapeHtml(title)}</h2>
  <vx-field name="key" label="Clave" value="${keyVal}"></vx-field>
  <vx-field name="label" label="Etiqueta" value="${labelVal}"></vx-field>
  ${typeSelect}
  <vx-field name="unit" label="Unidad" value="${unitVal}"></vx-field>
  ${optionsField}
  <label class="vx-attr-check"><input type="checkbox" name="facetable"${facetable ? " checked" : ""}> Facetable</label>
  <label class="vx-attr-check"><input type="checkbox" name="searchable"${searchable ? " checked" : ""}> Searchable</label>
  <vx-field name="order" label="Orden" value="${escapeHtml(orderVal)}"></vx-field>
  <div class="vx-attr-form-actions">
    <button type="submit">Guardar</button>
    <button type="button" data-action="cancel-form">Cancelar</button>
  </div>
</form>`;
}

/**
 * @param {AttrItem[]} items
 * @param {AttrForm | null} form
 */
function renderShell(items, form) {
  const emptyMsg =
    items.length === 0
      ? `<p class="vx-attr-empty">No hay atributos.</p>`
      : "";
  const toolbar = `<button type="button" data-action="create">Nueva definición</button>`;

  return `<div class="vx-attr-admin">
  <h1 class="vx-attr-title">Atributos</h1>
  <div class="vx-attr-toolbar">${toolbar}</div>
  ${emptyMsg}
  ${renderForm(form)}
  <div class="vx-attr-table">${renderRows(items)}</div>
</div>`;
}

/**
 * @param {ParentNode & { querySelector?: Function }} form
 * @param {string} name
 */
function readFieldValue(form, name) {
  const field = form.querySelector?.(`vx-field[name="${name}"]`);
  const raw = field?.getAttribute?.("value") ?? "";
  return String(raw).trim();
}

/**
 * @param {ParentNode & { querySelector?: Function }} form
 * @param {string} name
 */
function readCheckbox(form, name) {
  const input = form.querySelector?.(`input[name="${name}"]`);
  if (!input) {
    return false;
  }
  if (typeof input.checked === "boolean") {
    return input.checked;
  }
  return input.getAttribute?.("checked") != null;
}

/**
 * @param {ParentNode & { querySelector?: Function }} form
 */
function readType(form) {
  const select = form.querySelector?.('select[name="type"]');
  const raw =
    select?.value ?? select?.getAttribute?.("value") ?? "text";
  return String(raw);
}

/**
 * @param {ParentNode & { querySelector?: Function }} form
 * @param {string} type
 */
function buildBodyFromForm(form, type) {
  const key = readFieldValue(form, "key");
  const label = readFieldValue(form, "label");
  const unit = readFieldValue(form, "unit");
  const orderRaw = readFieldValue(form, "order");
  const facetable = readCheckbox(form, "facetable");
  const searchable = readCheckbox(form, "searchable");

  /** @type {Record<string, unknown>} */
  const body = {
    key,
    label,
    type,
    facetable,
    searchable,
  };

  if (unit.length > 0) {
    body.unit = unit;
  }

  if (type === "enum") {
    const optionsText = readFieldValue(form, "options");
    body.options = parseOptionsText(optionsText);
  }

  if (orderRaw.length > 0 && /^-?\d+$/.test(orderRaw)) {
    body.order = Number(orderRaw);
  }

  return { key, label, body };
}

/**
 * @param {ParentNode & { innerHTML?: string, querySelector?: typeof Document.prototype.querySelector, addEventListener?: Function }} el
 * @param {{ request: Function, showError?: Function }} deps
 */
export async function mountAttributeDefinitions(
  el,
  { request, showError = () => {} },
) {
  if (!el || typeof request !== "function") {
    return;
  }

  /** @type {AttrItem[]} */
  let items = [];
  /** @type {AttrForm | null} */
  let activeForm = null;

  const paint = () => {
    el.innerHTML = renderShell(items, activeForm);
  };

  const reload = async () => {
    const data = await request(
      "GET",
      "/api/v1/attribute-definitions?includeInactive=true",
    );
    items = Array.isArray(data?.items) ? data.items : [];
    paint();
  };

  const findItem = (id) => items.find((a) => a.id === id);

  el.addEventListener("click", (ev) => {
    const target = ev.target;
    const actionEl = target?.closest?.("[data-action]") ?? target;
    const action = actionEl?.getAttribute?.("data-action");
    if (!action) {
      return;
    }

    if (action === "cancel-form") {
      activeForm = null;
      paint();
      return;
    }

    const attributeId = actionEl.getAttribute?.("data-attribute-id");

    if (action === "create") {
      activeForm = {
        kind: "create",
        type: "text",
        defaults: { facetable: false, searchable: true },
      };
      paint();
      return;
    }

    if (action === "edit" && attributeId) {
      const item = findItem(attributeId);
      activeForm = {
        kind: "edit",
        attributeId,
        type: item?.type ?? "text",
        defaults: {
          key: item?.key ?? "",
          label: item?.label ?? "",
          type: item?.type ?? "text",
          unit: item?.unit ?? "",
          options: item?.options ?? [],
          facetable: item?.facetable === true,
          searchable: item?.searchable !== false,
          order: item?.order,
        },
      };
      paint();
      return;
    }

    if (action === "deactivate" && attributeId) {
      void (async () => {
        try {
          await request(
            "DELETE",
            `/api/v1/attribute-definitions/${attributeId}`,
          );
          activeForm = null;
          await reload();
        } catch (err) {
          showError(err);
        }
      })();
      return;
    }

    if (action === "reactivate" && attributeId) {
      void (async () => {
        try {
          await request(
            "PATCH",
            `/api/v1/attribute-definitions/${attributeId}`,
            { body: { active: true } },
          );
          activeForm = null;
          await reload();
        } catch (err) {
          showError(err);
        }
      })();
    }
  });

  el.addEventListener("change", (ev) => {
    const target = ev.target;
    const name = target?.getAttribute?.("name") ?? target?.name;
    if (name !== "type" || !activeForm) {
      return;
    }
    const nextType =
      target?.value ?? target?.getAttribute?.("value") ?? "text";
    const formEl =
      target?.closest?.("form") ??
      el.querySelector?.(`form[data-form="${activeForm.kind}"]`);
    const prevDefaults = { ...(activeForm.defaults ?? {}) };
    if (formEl) {
      prevDefaults.key = readFieldValue(formEl, "key");
      prevDefaults.label = readFieldValue(formEl, "label");
      prevDefaults.unit = readFieldValue(formEl, "unit");
      prevDefaults.order = readFieldValue(formEl, "order");
      prevDefaults.facetable = readCheckbox(formEl, "facetable");
      prevDefaults.searchable = readCheckbox(formEl, "searchable");
      if (activeForm.type === "enum") {
        prevDefaults.optionsText = readFieldValue(formEl, "options");
      }
    }
    activeForm = {
      ...activeForm,
      type: String(nextType),
      defaults: {
        ...prevDefaults,
        type: String(nextType),
      },
    };
    paint();
  });

  el.addEventListener("submit", (ev) => {
    const form = ev.target;
    const formKind = form?.getAttribute?.("data-form");
    if (!formKind) {
      return;
    }
    ev.preventDefault?.();

    void (async () => {
      try {
        const type =
          activeForm?.type ?? readType(form) ?? "text";
        const { key, label, body } = buildBodyFromForm(form, type);
        if (key.length === 0 || label.length === 0) {
          return;
        }

        if (formKind === "create") {
          await request("POST", "/api/v1/attribute-definitions", {
            body,
          });
        } else if (formKind === "edit") {
          const attributeId = form.getAttribute?.("data-attribute-id");
          if (!attributeId) {
            return;
          }
          await request(
            "PATCH",
            `/api/v1/attribute-definitions/${attributeId}`,
            { body },
          );
        } else {
          return;
        }
        activeForm = null;
        await reload();
      } catch (err) {
        showError(err);
      }
    })();
  });

  try {
    await reload();
  } catch (err) {
    showError(err);
    paint();
  }
}
