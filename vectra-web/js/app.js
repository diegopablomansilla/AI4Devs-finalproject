import "./components/index.js";
import { startRouter } from "./router.js";
import { startHealth } from "./health.js";
import { startSwUpdateWatcher } from "./sw-update.js";

/**
 * @param {{ serviceWorker?: { register: (url: string) => Promise<unknown> } }} [nav]
 */
export function registerAppServiceWorker(nav = globalThis.navigator) {
  try {
    if (nav?.serviceWorker?.register) {
      void nav.serviceWorker.register("/sw.js");
    }
  } catch {
    // sin serviceWorker o contexto inseguro: no tirar
  }
}

const outlet = document.getElementById("app");
if (outlet) {
  startRouter({ window, document, outlet });
  startHealth({ fetch: globalThis.fetch.bind(globalThis), window, document });
  registerAppServiceWorker();
  void startSwUpdateWatcher({ window, document });
}
