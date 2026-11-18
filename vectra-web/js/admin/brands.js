/**
 * Admin: listado de marcas (#/admin/marcas).
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
 * @typedef {{ id: string, name: string, slug: string, active: boolean }} BrandItem
 */

/**
 * @param {BrandItem[]} items
 */
function sortBrands(items) {
  return [...items].sort((a, b) =>
    a.name.localeCompare(b.name, "es"),
  );
}

/**
 * @param {BrandItem[]} items
 */
function renderRows(items) {
  const sorted = sortBrands(items);
  if (sorted.length === 0) {
    return "";
  }
  return `<ul class="vx-brand-list">
${sorted
  .map((brand) => {
    const id = escapeHtml(brand.id);
    const name = escapeHtml(brand.name);
    const slug = escapeHtml(brand.slug ?? "");
    const inactive = brand.active === false;
    const inactiveBadge = inactive
      ? ' <span class="vx-brand-inactive">Inactiva</span>'
      : "";
    const actions = inactive
      ? `<button type="button" data-action="reactivate" data-brand-id="${id}">Reactivar</button>`
      : `<button type="button" data-action="rename" data-brand-id="${id}">Editar</button>
      <button type="button" data-action="deactivate" data-brand-id="${id}">Dar de baja</button>`;
    return `  <li class="vx-brand-row" data-brand-id="${id}" data-active="${inactive ? "false" : "true"}">
    <span data-name>${name}</span>${inactiveBadge}
    <span class="vx-brand-slug" data-slug>${slug}</span>
    <div class="vx-brand-actions">${actions}</div>
  </li>`;
  })
  .join("\n")}
</ul>`;
}

/**
 * @param {{ kind: string, brandId?: string, defaultName?: string } | null} form
 */
function renderForm(form) {
  if (!form) {
    return "";
  }
  const labels = {
    create: "Nueva marca",
    rename: "Editar marca",
  };
  const title = labels[form.kind] ?? "Marca";
  const defaultName = form.defaultName ?? "";
  const nameField = `<vx-field name="name" label="Nombre" value="${escapeHtml(defaultName)}"></vx-field>`;
  const brandAttr = form.brandId
    ? ` data-brand-id="${escapeHtml(form.brandId)}"`
    : "";

  return `<form data-form="${escapeHtml(form.kind)}" class="vx-brand-form"${brandAttr}>
  <h2 class="vx-brand-form-title">${escapeHtml(title)}</h2>
  ${nameField}
  <div class="vx-brand-form-actions">
    <button type="submit">Guardar</button>
    <button type="button" data-action="cancel-form">Cancelar</button>
  </div>
</form>`;
}

/**
 * @param {BrandItem[]} items
 * @param {{ kind: string, brandId?: string, defaultName?: string } | null} form
 */
function renderShell(items, form) {
  const emptyMsg =
    items.length === 0
      ? `<p class="vx-brand-empty">No hay marcas.</p>`
      : "";
  const toolbar = `<button type="button" data-action="create">Nueva marca</button>`;

  return `<div class="vx-brand-admin">
  <h1 class="vx-brand-title">Marcas</h1>
  <div class="vx-brand-toolbar">${toolbar}</div>
  ${emptyMsg}
  ${renderForm(form)}
  <div class="vx-brand-table">${renderRows(items)}</div>
</div>`;
}

/**
 * @param {ParentNode & { innerHTML?: string, querySelector?: typeof Document.prototype.querySelector }} el
 * @param {{ request: Function, showError?: Function }} deps
 */
export async function mountBrands(el, { request, showError = () => {} }) {
  if (!el || typeof request !== "function") {
    return;
  }

  /** @type {BrandItem[]} */
  let items = [];
  /** @type {{ kind: string, brandId?: string, defaultName?: string } | null} */
  let activeForm = null;

  const paint = () => {
    el.innerHTML = renderShell(items, activeForm);
  };

  const reload = async () => {
    const data = await request(
      "GET",
      "/api/v1/brands?includeInactive=true",
    );
    items = Array.isArray(data?.items) ? data.items : [];
    paint();
  };

  const readNameFromForm = (form) => {
    const field = form.querySelector?.('vx-field[name="name"]');
    const raw = field?.getAttribute?.("value") ?? "";
    return String(raw).trim();
  };

  const findBrand = (id) => items.find((b) => b.id === id);

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

    const brandId = actionEl.getAttribute?.("data-brand-id");

    if (action === "create") {
      activeForm = { kind: "create" };
      paint();
      return;
    }

    if (action === "rename" && brandId) {
      const brand = findBrand(brandId);
      activeForm = {
        kind: "rename",
        brandId,
        defaultName: brand?.name ?? "",
      };
      paint();
      return;
    }

    if (action === "deactivate" && brandId) {
      void (async () => {
        try {
          await request("DELETE", `/api/v1/brands/${brandId}`);
          activeForm = null;
          await reload();
        } catch (err) {
          showError(err);
        }
      })();
      return;
    }

    if (action === "reactivate" && brandId) {
      void (async () => {
        try {
          await request("PATCH", `/api/v1/brands/${brandId}`, {
            body: { active: true },
          });
          activeForm = null;
          await reload();
        } catch (err) {
          showError(err);
        }
      })();
    }
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
        const name = readNameFromForm(form);
        if (name.length === 0) {
          return;
        }
        if (formKind === "create") {
          await request("POST", "/api/v1/brands", { body: { name } });
        } else if (formKind === "rename") {
          const brandId = form.getAttribute?.("data-brand-id");
          if (!brandId) {
            return;
          }
          await request("PATCH", `/api/v1/brands/${brandId}`, {
            body: { name },
          });
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