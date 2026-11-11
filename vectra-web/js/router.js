/**
 * Router hash: operador (#/) y administración (#/admin/<sección>).
 */

const SLUG = /^[a-z0-9-]+$/;

export const TERMINAL_STORAGE_KEY = "vectra.terminal";
const TERMINAL_MAX_LEN = 64;

/** @type {{ slug: string, label: string }[]} */
export const ADMIN_SECTIONS = [
  { slug: "productos", label: "Productos" },
  { slug: "categorias", label: "Categorías" },
  { slug: "marcas", label: "Marcas" },
  { slug: "atributos", label: "Atributos" },
  { slug: "stock", label: "Stock" },
  { slug: "relaciones", label: "Relaciones" },
  { slug: "sinonimos", label: "Sinónimos" },
  { slug: "notas-tecnicas", label: "Notas técnicas" },
];

/**
 * @param {string} text
 * @returns {string}
 */
function escapeHtmlAttr(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;");
}

/**
 * @param {string} section
 * @returns {string}
 */
function adminSectionTitle(section) {
  const found = ADMIN_SECTIONS.find((s) => s.slug === section);
  return found ? found.label : section;
}

/**
 * @param {string} hash
 * @returns {{ view: 'operator' | 'admin' | 'not-found', section?: string }}
 */
export function parseHash(hash) {
  let h = hash ?? "";
  if (h.startsWith("#")) {
    h = h.slice(1);
  }
  if (h === "" || h === "/") {
    return { view: "operator" };
  }
  if (!h.startsWith("/")) {
    return { view: "not-found" };
  }

  const parts = h.split("/").filter((p) => p.length > 0);
  if (parts.length === 0) {
    return { view: "operator" };
  }

  if (parts[0] === "admin") {
    if (parts.length !== 2) {
      return { view: "not-found" };
    }
    const section = parts[1];
    if (!SLUG.test(section)) {
      return { view: "not-found" };
    }
    return { view: "admin", section };
  }

  return { view: "not-found" };
}

/**
 * @param {Storage | { getItem: (k: string) => string | null, setItem?: Function, removeItem?: Function }} storage
 * @returns {string}
 */
export function readTerminalName(storage) {
  if (!storage || typeof storage.getItem !== "function") {
    return "";
  }
  try {
    const raw = storage.getItem(TERMINAL_STORAGE_KEY);
    if (raw == null) {
      return "";
    }
    return String(raw).trim();
  } catch {
    return "";
  }
}

/**
 * @param {Storage | { getItem: (k: string) => string | null, setItem: (k: string, v: string) => void, removeItem: (k: string) => void }} storage
 * @param {string} raw
 * @returns {{ ok: true, value: string } | { ok: false, error: string }}
 */
export function writeTerminalName(storage, raw) {
  const trimmed = String(raw ?? "").trim();
  if (trimmed.length > TERMINAL_MAX_LEN) {
    return { ok: false, error: "Usá como máximo 64 caracteres" };
  }
  if (!storage) {
    return { ok: true, value: trimmed };
  }
  try {
    if (trimmed.length === 0) {
      if (typeof storage.removeItem === "function") {
        storage.removeItem(TERMINAL_STORAGE_KEY);
      }
      return { ok: true, value: "" };
    }
    if (typeof storage.setItem === "function") {
      storage.setItem(TERMINAL_STORAGE_KEY, trimmed);
    }
    return { ok: true, value: trimmed };
  } catch {
    return { ok: true, value: readTerminalName(storage) };
  }
}

/**
 * @param {HTMLElement | null | undefined} field
 * @param {Storage | null | undefined} storage
 */
export function bindTerminalName(field, storage) {
  if (!field || typeof field.addEventListener !== "function") {
    return;
  }
  field.addEventListener("change", () => {
    const value = field.getAttribute("value") ?? "";
    const result = writeTerminalName(storage, value);
    if (!result.ok) {
      field.setAttribute("error", result.error);
      return;
    }
    field.removeAttribute("error");
  });
}

/**
 * @param {{ view: string, section?: string }} route
 * @param {string} terminalValue
 * @returns {string}
 */
function viewHtml(route, terminalValue) {
  if (route.view === "operator") {
    return `<main class="vx-operator" data-view="operator">
  <header class="vx-operator-header">
    <p class="vx-offline-banner" data-offline-banner hidden aria-live="assertive">Sin conexión con el Nodo Maestro</p>
    <p class="vx-health-status" data-health aria-live="polite"></p>
    <div data-region="omnibox"><vx-omnibox></vx-omnibox></div>
    <p><a href="#/admin/productos">Ir a administración</a></p>
  </header>
  <div data-region="results" class="vx-operator-results vx-surface"></div>
  <div data-region="side-panel"><vx-side-panel></vx-side-panel></div>
</main>`;
  }
  if (route.view === "admin") {
    const section = route.section ?? "";
    const title = escapeHtmlAttr(adminSectionTitle(section));
    const safeSection = escapeHtmlAttr(section);
    const safeTerminal = escapeHtmlAttr(terminalValue);
    const valueAttr =
      safeTerminal.length > 0 ? ` value="${safeTerminal}"` : "";
    const navLinks = ADMIN_SECTIONS.map(({ slug, label }) => {
      const current =
        slug === section ? ' aria-current="page"' : "";
      return `    <a href="#/admin/${slug}"${current}>${label}</a>`;
    }).join("\n");
    return `<main class="vx-admin" data-view="admin" data-section="${safeSection}">
  <header class="vx-admin-header">
    <p class="vx-offline-banner" data-offline-banner hidden aria-live="assertive">Sin conexión con el Nodo Maestro</p>
    <p class="vx-health-status" data-health aria-live="polite"></p>
    <p><a href="#/">Volver al operador</a></p>
    <vx-field name="terminal" label="Nombre de esta terminal"${valueAttr}></vx-field>
  </header>
  <nav class="vx-admin-nav" aria-label="Administración">
${navLinks}
  </nav>
  <div data-region="admin-content" class="vx-admin-content vx-surface">
    <h1>${title}</h1>
  </div>
</main>`;
  }
  return `<main data-view="not-found">
  <h1>Página no encontrada</h1>
  <p><a href="#/">Volver al operador</a></p>
</main>`;
}

/**
 * @param {{ window: Window, document: Document, outlet: HTMLElement, storage?: Storage }} opts
 */
export function startRouter({ window, document, outlet, storage }) {
  const resolvedStorage =
    storage ??
    (typeof window.localStorage !== "undefined" ? window.localStorage : null);

  const paint = () => {
    const route = parseHash(window.location.hash);
    const terminalValue = readTerminalName(resolvedStorage);
    outlet.innerHTML = viewHtml(route, terminalValue);
    if (route.view === "admin" && typeof outlet.querySelector === "function") {
      const field = outlet.querySelector('vx-field[name="terminal"]');
      bindTerminalName(field, resolvedStorage);
    }
  };

  window.addEventListener("hashchange", paint);
  paint();
}
