import "./components/index.js";
import { startRouter } from "./router.js";

const outlet = document.getElementById("app");
if (outlet) {
  startRouter({ window, document, outlet });
}
