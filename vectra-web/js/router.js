/**
 * Router hash: operador (#/) y administración (#/admin/<sección>).
 */

const SLUG = /^[a-z0-9-]+$/;

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
 * @param {{ view: string, section?: string }} route
 * @returns {string}
 */
function viewHtml(route) {
  if (route.view === "operator") {
    return `<main class="vx-operator" data-view="operator">
  <header class="vx-operator-header">
    <div data-region="omnibox"><vx-omnibox></vx-omnibox></div>
    <p><a href="#/admin/productos">Ir a administración</a></p>
  </header>
  <div data-region="results" class="vx-operator-results vx-surface"></div>
  <div data-region="side-panel"><vx-side-panel></vx-side-panel></div>
</main>`;
  }
  if (route.view === "admin") {
    const section = route.section ?? "";
    return `<main data-view="admin" data-section="${section}">
  <h1>Administración: ${section}</h1>
  <p><a href="#/">Volver al operador</a></p>
</main>`;
  }
  return `<main data-view="not-found">
  <h1>Página no encontrada</h1>
  <p><a href="#/">Volver al operador</a></p>
</main>`;
}

/**
 * @param {{ window: Window, document: Document, outlet: HTMLElement }} opts
 */
export function startRouter({ window, document, outlet }) {
  const paint = () => {
    const route = parseHash(window.location.hash);
    outlet.innerHTML = viewHtml(route);
  };

  window.addEventListener("hashchange", paint);
  paint();
}
