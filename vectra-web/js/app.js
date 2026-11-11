import "./components/index.js";
import { startRouter } from "./router.js";
import { startHealth } from "./health.js";

const outlet = document.getElementById("app");
if (outlet) {
  startRouter({ window, document, outlet });
  startHealth({ fetch: globalThis.fetch.bind(globalThis), window, document });
}
