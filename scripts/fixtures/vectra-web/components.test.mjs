import { describe, test, before } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { installDomGlobals, customElements } from "./dom-stub.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");

const COMPONENTS_HTML = path.join(REPO_ROOT, "vectra-web/dev/components.html");

function read(p) {
  try {
    return fs.readFileSync(p, "utf8");
  } catch {
    return "";
  }
}

function readCss(name) {
  return read(path.join(REPO_ROOT, "vectra-web/css", name));
}

function adoptHasSheets(el) {
  return (
    el.shadowRoot &&
    Array.isArray(el.shadowRoot.adoptedStyleSheets) &&
    el.shadowRoot.adoptedStyleSheets.length >= 2
  );
}

installDomGlobals();

describe("components_html_catalog_loads_three_tags", () => {
  test("muestra vx-button, vx-field, vx-chip", () => {
    const html = read(COMPONENTS_HTML);
    assert.ok(html.length > 0, "falta vectra-web/dev/components.html");
    assert.match(html, /<vx-button\b/);
    assert.match(html, /<vx-field\b/);
    assert.match(html, /<vx-chip\b/);
    assert.match(html, /\.\.\/js\/components\/index\.js/);
  });
});

describe("vx components", () => {
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

  test("vx_button_shadow_and_adopted_stylesheets", async () => {
    const el = customElements._create("vx-button");
    await el.connectedCallback();
    assert.ok(el.shadowRoot);
    assert.equal(adoptHasSheets(el), true);
    const btn = el.shadowRoot.querySelector("button");
    assert.ok(btn);
    const css = el.shadowRoot.adoptedStyleSheets
      .map((s) => s._text ?? "")
      .join("\n");
    assert.match(css, /--vx-touch-min/);
  });

  test("vx_button_loading_sets_aria_busy_and_disabled", async () => {
    const el = customElements._create("vx-button");
    el.setAttribute("loading", "");
    await el.connectedCallback();
    const btn = el.shadowRoot.querySelector("button");
    assert.equal(btn.disabled, true);
    assert.equal(btn.getAttribute("aria-busy"), "true");
  });

  test("vx_field_label_help_error_and_aria", async () => {
    const el = customElements._create("vx-field");
    el.setAttribute("label", "SKU");
    el.setAttribute("help", "Código de barras");
    el.setAttribute("error", "Requerido");
    await el.connectedCallback();
    const input = el.shadowRoot.querySelector("input");
    assert.equal(input.getAttribute("aria-invalid"), "true");
    const described = input.getAttribute("aria-describedby") ?? "";
    assert.ok(described.length > 0);
    assert.ok(el.shadowRoot.querySelector('[role="alert"]'));
  });

  test("vx_field_error_without_aria_invalid_fails", async () => {
    const el = customElements._create("vx-field");
    el.setAttribute("error", "Mal");
    await el.connectedCallback();
    const input = el.shadowRoot.querySelector("input");
    assert.equal(input.getAttribute("aria-invalid"), "true");
  });

  test("vx_chip_selected_aria_pressed_and_toggle_event", async () => {
    const el = customElements._create("vx-chip");
    el.setAttribute("selected", "");
    await el.connectedCallback();
    let fired = false;
    let detail;
    el.addEventListener("vx-chip-toggle", (e) => {
      fired = true;
      detail = e.detail;
    });
    el.shadowRoot.querySelector("button").click();
    assert.equal(fired, true);
    assert.equal(typeof detail?.selected, "boolean");
    assert.equal(el.shadowRoot.querySelector("button").getAttribute("aria-pressed"), "false");
  });

  test("vx_chip_disabled_does_not_toggle", async () => {
    const el = customElements._create("vx-chip");
    el.setAttribute("disabled", "");
    await el.connectedCallback();
    let fired = false;
    el.addEventListener("vx-chip-toggle", () => {
      fired = true;
    });
    el.shadowRoot.querySelector("button").click();
    assert.equal(fired, false);
  });
});
