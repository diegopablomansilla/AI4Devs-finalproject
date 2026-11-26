/**
 * Admin: árbol de categorías (#/admin/categorias).
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
 * @param {Array<{ name: string, path?: string[] }>} pathLike
 */
export function formatPath(pathLike) {
  const path = Array.isArray(pathLike) ? pathLike : pathLike?.path;
  return Array.isArray(path) ? path.join(" > ") : "";
}

/**
 * @typedef {{ id: string, name: string, parentId: string | null, path: string[], active: boolean }} CategoryItem
 * @typedef {{ category: CategoryItem, children: ReturnType<typeof buildTree> }} CategoryNode
 */

/**
 * @param {CategoryItem[]} items
 * @returns {CategoryNode[]}
 */
export function buildTree(items) {
  /** @type {Map<string, CategoryNode>} */
  const byId = new Map();
  for (const category of items) {
    byId.set(category.id, { category, children: [] });
  }

  /** @type {CategoryNode[]} */
  const roots = [];
  for (const category of items) {
    const node = byId.get(category.id);
    if (!node) {
      continue;
    }
    const parentId = category.parentId;
    if (parentId == null || !byId.has(parentId)) {
      roots.push(node);
    } else {
      byId.get(parentId).children.push(node);
    }
  }

  const sortNodes = (nodes) => {
    nodes.sort((a, b) =>
      a.category.name.localeCompare(b.category.name, "es"),
    );
    for (const node of nodes) {
      sortNodes(node.children);
    }
  };
  sortNodes(roots);
  return roots;
}

/**
 * @param {CategoryItem[]} items
 * @param {string} rootId
 * @returns {Set<string>}
 */
export function collectDescendantIds(items, rootId) {
  /** @type {Set<string>} */
  const set = new Set([rootId]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const c of items) {
      if (c.parentId && set.has(c.parentId) && !set.has(c.id)) {
        set.add(c.id);
        changed = true;
      }
    }
  }
  return set;
}

/**
 * @param {CategoryItem[]} items
 * @param {string} movingId
 * @returns {{ id: string | null, label: string }[]}
 */
export function parentOptions(items, movingId) {
  const exclude = collectDescendantIds(items, movingId);
  /** @type {{ id: string | null, label: string }[]} */
  const options = [{ id: null, label: "Raíz" }];
  for (const c of items) {
    if (exclude.has(c.id) || c.id === movingId) {
      continue;
    }
    options.push({
      id: c.id,
      label: formatPath(c.path) || c.name,
    });
  }
  options.sort((a, b) => a.label.localeCompare(b.label, "es"));
  return options;
}

/**
 * @param {CategoryNode[]} nodes
 * @param {number} depth
 */
function renderNodes(nodes, depth = 0) {
  if (nodes.length === 0) {
    return "";
  }
  const indent = depth > 0 ? ` style="margin-left: ${depth * 1.25}rem"` : "";
  return `<ul class="vx-category-branch"${indent}>
${nodes
  .map((node) => {
    const c = node.category;
    const id = escapeHtml(c.id);
    const name = escapeHtml(c.name);
    const pathStr = escapeHtml(formatPath(c.path));
    const inactive = c.active === false;
    const inactiveBadge = inactive
      ? ' <span class="vx-category-inactive">Inactiva</span>'
      : "";
    const actions = inactive
      ? `<button type="button" data-action="reactivate" data-category-id="${id}">Reactivar</button>`
      : `<button type="button" data-action="create-child" data-category-id="${id}">Crear subcategoría</button>
      <button type="button" data-action="rename" data-category-id="${id}">Renombrar</button>
      <button type="button" data-action="move" data-category-id="${id}">Mover</button>
      <button type="button" data-action="deactivate" data-category-id="${id}">Dar de baja</button>`;
    const childHtml = renderNodes(node.children, depth + 1);
    return `  <li class="vx-category-node" data-category-id="${id}" data-active="${inactive ? "false" : "true"}">
    <div class="vx-category-row">
      <span data-name>${name}</span>${inactiveBadge}
      <span class="vx-category-path" data-path>${pathStr}</span>
      <div class="vx-category-actions">${actions}</div>
    </div>
    ${childHtml}
  </li>`;
  })
  .join("\n")}
</ul>`;
}

/**
 * @param {CategoryItem[]} items
 * @param {{ kind: string, categoryId?: string, parentId?: string, defaultName?: string } | null} form
 */
function renderForm(form, items) {
  if (!form) {
    return "";
  }
  const labels = {
    "create-root": "Nueva categoría",
    "create-child": "Crear subcategoría",
    rename: "Renombrar categoría",
    move: "Mover categoría",
  };
  const title = labels[form.kind] ?? "Categoría";
  const defaultName = form.defaultName ?? "";
  const nameField = `<vx-field name="name" label="Nombre" value="${escapeHtml(defaultName)}"></vx-field>`;

  if (form.kind === "move") {
    const opts = parentOptions(items, form.categoryId);
    const optionsHtml = opts
      .map((o) => {
        const value = o.id == null ? "__root__" : o.id;
        const selected =
          form.parentId != null && o.id === form.parentId ? " selected" : "";
        return `<option value="${escapeHtml(value)}"${selected}>${escapeHtml(o.label)}</option>`;
      })
      .join("");
    return `<form data-form="move" class="vx-category-form" data-category-id="${escapeHtml(form.categoryId)}">
  <h2 class="vx-category-form-title">${escapeHtml(title)}</h2>
  <label class="vx-category-select-label">Nuevo padre
    <select name="parentId">${optionsHtml}</select>
  </label>
  <div class="vx-category-form-actions">
    <button type="submit">Guardar</button>
    <button type="button" data-action="cancel-form">Cancelar</button>
  </div>
</form>`;
  }

  const attrs = [];
  if (form.parentId) {
    attrs.push(`data-parent-id="${escapeHtml(form.parentId)}"`);
  }
  if (form.categoryId) {
    attrs.push(`data-category-id="${escapeHtml(form.categoryId)}"`);
  }
  const attrStr = attrs.length > 0 ? ` ${attrs.join(" ")}` : "";

  return `<form data-form="${escapeHtml(form.kind)}" class="vx-category-form"${attrStr}>
  <h2 class="vx-category-form-title">${escapeHtml(title)}</h2>
  ${nameField}
  <div class="vx-category-form-actions">
    <button type="submit">Guardar</button>
    <button type="button" data-action="cancel-form">Cancelar</button>
  </div>
</form>`;
}

/**
 * @param {CategoryNode[]} tree
 * @param {CategoryItem[]} items
 * @param {{ kind: string, categoryId?: string, parentId?: string, defaultName?: string } | null} form
 */
function renderShell(tree, items, form) {
  const toolbar =
    items.length === 0
      ? `<p class="vx-category-empty">No hay categorías.</p>
  <button type="button" data-action="create-root">Nueva categoría</button>`
      : `<button type="button" data-action="create-root">Nueva categoría</button>`;

  return `<div class="vx-category-admin">
  <h1 class="vx-category-title">Categorías</h1>
  <div class="vx-category-toolbar">${toolbar}</div>
  ${renderForm(form, items)}
  <div class="vx-category-tree">${renderNodes(tree)}</div>
</div>`;
}

/**
 * @param {ParentNode & { innerHTML?: string, querySelector?: typeof Document.prototype.querySelector }} el
 * @param {{ request: Function, showError?: Function }} deps
 */
export async function mountCategories(el, { request, showError = () => {} }) {
  if (!el || typeof request !== "function") {
    return;
  }

  /** @type {CategoryItem[]} */
  let items = [];
  /** @type {{ kind: string, categoryId?: string, parentId?: string, defaultName?: string } | null} */
  let activeForm = null;

  const paint = () => {
    const tree = buildTree(items);
    el.innerHTML = renderShell(tree, items, activeForm);
  };

  const reload = async () => {
    const data = await request(
      "GET",
      "/api/v1/categories?includeInactive=true",
    );
    items = Array.isArray(data?.items) ? data.items : [];
    paint();
  };

  const readNameFromForm = (form) => {
    const field = form.querySelector?.('vx-field[name="name"]');
    const raw = field?.getAttribute?.("value") ?? "";
    return String(raw).trim();
  };

  const readParentFromMoveForm = (form) => {
    const select = form.querySelector?.('select[name="parentId"]');
    const raw = select?.value ?? "";
    if (raw === "__root__" || raw === "") {
      return null;
    }
    return raw;
  };

  const findCategory = (id) => items.find((c) => c.id === id);

  el.addEventListener("click", (ev) => {
    const target = ev.target;
    const actionEl =
      target?.closest?.("[data-action]") ?? target;
    const action = actionEl?.getAttribute?.("data-action");
    if (!action) {
      return;
    }

    if (action === "cancel-form") {
      activeForm = null;
      paint();
      return;
    }

    const categoryId = actionEl.getAttribute?.("data-category-id");

    if (action === "create-root") {
      activeForm = { kind: "create-root" };
      paint();
      return;
    }

    if (action === "create-child" && categoryId) {
      activeForm = { kind: "create-child", parentId: categoryId };
      paint();
      return;
    }

    if (action === "rename" && categoryId) {
      const cat = findCategory(categoryId);
      activeForm = {
        kind: "rename",
        categoryId,
        defaultName: cat?.name ?? "",
      };
      paint();
      return;
    }

    if (action === "move" && categoryId) {
      const cat = findCategory(categoryId);
      activeForm = {
        kind: "move",
        categoryId,
        parentId: cat?.parentId ?? null,
      };
      paint();
      return;
    }

    if (action === "deactivate" && categoryId) {
      void (async () => {
        try {
          await request("DELETE", `/api/v1/categories/${categoryId}`);
          activeForm = null;
          await reload();
        } catch (err) {
          showError(err);
        }
      })();
      return;
    }

    if (action === "reactivate" && categoryId) {
      void (async () => {
        try {
          await request("PATCH", `/api/v1/categories/${categoryId}`, {
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
        if (formKind === "create-root") {
          const name = readNameFromForm(form);
          if (name.length === 0) {
            return;
          }
          await request("POST", "/api/v1/categories", { body: { name } });
        } else if (formKind === "create-child") {
          const name = readNameFromForm(form);
          if (name.length === 0) {
            return;
          }
          const parentId =
            form.getAttribute?.("data-parent-id") ??
            activeForm?.parentId ??
            null;
          const body = { name };
          if (parentId) {
            body.parentId = parentId;
          }
          await request("POST", "/api/v1/categories", { body });
        } else if (formKind === "rename") {
          const name = readNameFromForm(form);
          if (name.length === 0) {
            return;
          }
          const categoryId = form.getAttribute?.("data-category-id");
          if (!categoryId) {
            return;
          }
          await request("PATCH", `/api/v1/categories/${categoryId}`, {
            body: { name },
          });
        } else if (formKind === "move") {
          const categoryId = form.getAttribute?.("data-category-id");
          if (!categoryId) {
            return;
          }
          const parentId = readParentFromMoveForm(form);
          await request("PATCH", `/api/v1/categories/${categoryId}`, {
            body: { parentId },
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