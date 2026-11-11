import { describe, test, before } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { installDomGlobals, customElements, CustomEvent, KeyboardEvent } from "./dom-stub.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const COMPONENTS_HTML = path.join(REPO_ROOT, "vectra-web/dev/components.html");

function readCss(name) {
  return fs.readFileSync(path.join(REPO_ROOT, "vectra-web/css", name), "utf8");
}

const waitMicrotask = () => new Promise((resolve) => queueMicrotask(resolve));

/** @param {import("./dom-stub.mjs").HTMLElement} omnibox */
function omniboxInput(omnibox) {
  return omnibox.shadowRoot?.querySelector("input") ?? null;
}

installDomGlobals();

describe("document composed bubbling", () => {
  test("evento bubbles+composed llega a document", () => {
    let seen = 0;
    document.addEventListener("vx-closed", () => {
      seen += 1;
    });
    const host = document.createElement("div");
    document.body.appendChild(host);
    const child = document.createElement("span");
    host.appendChild(child);
    child.dispatchEvent(
      new CustomEvent("vx-closed", { bubbles: true, composed: true }),
    );
    assert.equal(seen, 1);
  });
});

describe("components_html_catalog_loads_omnibox", () => {
  test("muestra vx-omnibox", () => {
    const html = fs.readFileSync(COMPONENTS_HTML, "utf8");
    assert.match(html, /<vx-omnibox\b/);
  });
});

describe("omnibox", () => {
  before(async () => {
    const sharedUrl = new URL(
      "../../../vectra-web/js/components/shared.js",
      import.meta.url,
    ).href;
    const { setSharedStyleLoader } = await import(sharedUrl);
    setSharedStyleLoader(async (url) => {
      const file = url.pathname.endsWith("base.css") ? "base.css" : "tokens.css";
      return readCss(file);
    });
    await import(
      new URL("../../../vectra-web/js/components/index.js", import.meta.url).href
    );
  });

  test("omnibox_focuses_on_connect", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const input = omniboxInput(box);
    assert.ok(input);
    assert.equal(document.activeElement, input);
  });

  test("omnibox_focuses_on_vx_closed", async () => {
    const opener = document.createElement("button");
    document.body.appendChild(opener);
    opener.focus();

    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();

    document.dispatchEvent(
      new CustomEvent("vx-closed", { bubbles: true, composed: true }),
    );
    await waitMicrotask();
    assert.equal(document.activeElement, omniboxInput(box));
  });

  test("omnibox_side_panel_close_focuses_omnibox", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();

    const opener = document.createElement("button");
    document.body.appendChild(opener);

    const panel = customElements._create("vx-side-panel");
    document.body.appendChild(panel);
    await panel.connectedCallback();
    panel.show(opener);
    panel.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await waitMicrotask();
    assert.equal(document.activeElement, omniboxInput(box));
  });

  test("omnibox_escape_clears_query", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const input = omniboxInput(box);
    assert.ok(input);
    input.value = "tornillo";
    box.setAttribute("value", "tornillo");
    input.focus();
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    assert.equal(input.value, "");
    assert.equal(box.getAttribute("value"), "");
  });

  test("omnibox_escape_on_empty_query_does_not_throw", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const input = omniboxInput(box);
    input.focus();
    assert.doesNotThrow(() => {
      input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    });
    assert.equal(input.value, "");
  });

  test("omnibox_vx_closed_when_disconnected_does_not_throw", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    document.body.removeChild(box);
    assert.doesNotThrow(() => {
      document.dispatchEvent(
        new CustomEvent("vx-closed", { bubbles: true, composed: true }),
      );
    });
  });

  test("omnibox_escape_with_open_panel_does_not_clear_query", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const input = omniboxInput(box);
    input.value = "tornillo";
    box.setAttribute("value", "tornillo");

    const opener = document.createElement("button");
    document.body.appendChild(opener);

    const panel = customElements._create("vx-side-panel");
    document.body.appendChild(panel);
    await panel.connectedCallback();
    panel.show(opener);
    panel.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

    assert.equal(input.value, "tornillo");
    assert.equal(box.getAttribute("value"), "tornillo");
  });

  test("omnibox_does_not_emit_vx_query", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    let queries = 0;
    box.addEventListener("vx-query", () => {
      queries += 1;
    });
    document.addEventListener("vx-query", () => {
      queries += 1;
    });
    const input = omniboxInput(box);
    input.value = "perno";
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true }));
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    assert.equal(queries, 0);
  });
});
