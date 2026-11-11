/**
 * @param {Document} doc
 * @param {boolean} show
 */
export function applyUpdateBanner(doc, show) {
  for (const el of doc.querySelectorAll("[data-update-banner]")) {
    if (show) {
      el.removeAttribute("hidden");
    } else {
      el.setAttribute("hidden", "");
    }
  }
}

/**
 * @param {Document} doc
 * @param {{ location?: { reload?: () => void } }} win
 */
export function bindUpdateReloadHandlers(doc, win) {
  for (const btn of doc.querySelectorAll("[data-update-reload]")) {
    if (btn.getAttribute("data-vx-update-bound") === "1") {
      continue;
    }
    btn.setAttribute("data-vx-update-bound", "1");
    btn.addEventListener("click", () => {
      win.location?.reload?.();
    });
  }
}

/**
 * @param {ServiceWorkerRegistrationLike | null | undefined} registration
 * @param {unknown} controller
 */
function pendingUpdateVisible(registration, controller) {
  return Boolean(registration?.waiting && controller);
}

/**
 * @typedef {{ waiting?: unknown, update?: () => Promise<void>, addEventListener?: (type: string, fn: () => void) => void, installing?: { state: string, addEventListener?: (type: string, fn: () => void) => void } | null }} ServiceWorkerRegistrationLike
 */

/**
 * @param {{
 *   navigator?: { serviceWorker?: { controller: unknown, register: (url: string) => Promise<ServiceWorkerRegistrationLike> } },
 *   window: Window,
 *   document: Document,
 * }} opts
 */
export async function startSwUpdateWatcher({
  navigator: nav = globalThis.navigator,
  window: win,
  document: doc,
}) {
  let registration = /** @type {ServiceWorkerRegistrationLike | null} */ (null);
  let showUpdate = false;

  const syncUi = () => {
    applyUpdateBanner(doc, showUpdate);
    bindUpdateReloadHandlers(doc, win);
  };

  const reevaluate = () => {
    const sw = nav?.serviceWorker;
    showUpdate = pendingUpdateVisible(registration, sw?.controller);
    syncUi();
  };

  const wireUpdateFound = () => {
    if (!registration?.addEventListener) {
      return;
    }
    registration.addEventListener("updatefound", () => {
      const installing = registration.installing;
      if (!installing?.addEventListener) {
        return;
      }
      installing.addEventListener("statechange", () => {
        if (installing.state === "installed") {
          reevaluate();
        }
      });
    });
  };

  const checkForUpdate = () => {
    void (async () => {
      try {
        const sw = nav?.serviceWorker;
        if (!sw?.register) {
          return;
        }
        if (!registration) {
          registration = await sw.register("/sw.js");
          wireUpdateFound();
        }
        await registration.update?.();
        reevaluate();
      } catch {
        // sin serviceWorker o register/update fallido: no tirar
      }
    })();
  };

  try {
    const sw = nav?.serviceWorker;
    if (!sw?.register) {
      return () => {};
    }
    registration = await sw.register("/sw.js");
    if (registration.waiting && sw.controller) {
      showUpdate = true;
    }
    wireUpdateFound();
    syncUi();
  } catch {
    return () => {};
  }

  const onFocus = () => {
    checkForUpdate();
  };

  const onVisibility = () => {
    if (doc.visibilityState === "visible") {
      checkForUpdate();
    }
  };

  const onHashChange = () => {
    syncUi();
  };

  win.addEventListener("focus", onFocus);
  doc.addEventListener("visibilitychange", onVisibility);
  win.addEventListener("hashchange", onHashChange);

  return () => {
    win.removeEventListener("focus", onFocus);
    doc.removeEventListener("visibilitychange", onVisibility);
    win.removeEventListener("hashchange", onHashChange);
  };
}
