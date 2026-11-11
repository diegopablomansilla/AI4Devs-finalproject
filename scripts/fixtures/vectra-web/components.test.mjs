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

describe("components_html_catalog_loads_overlay_tags", () => {
  test("muestra vx-modal, vx-side-panel, vx-toast", () => {
    const html = read(COMPONENTS_HTML);
    assert.match(html, /<vx-modal\b/);
    assert.match(html, /<vx-side-panel\b/);
    assert.match(html, /<vx-toast\b/);
  });
});

describe("dom stub focus", () => {
  test("focus actualiza document.activeElement", () => {
    const node = document.createElement("button");
    node.focus();
    assert.equal(document.activeElement, node);
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

  test("vx_modal_escape_emits_vx_closed", async () => {
    const modal = customElements._create("vx-modal");
    await modal.connectedCallback();
    let closed = 0;
    modal.addEventListener("vx-closed", (e) => {
      closed += 1;
      assert.equal(e.bubbles, true);
      assert.equal(e.composed, true);
    });
    modal.show();
    assert.equal(modal.hasAttribute("open"), true);
    modal.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    assert.equal(modal.hasAttribute("open"), false);
    assert.equal(closed, 1);
  });

  test("vx_modal_tab_stays_inside", async () => {
    const modal = customElements._create("vx-modal");
    await modal.connectedCallback();
    modal.show();
    const buttons = modal.shadowRoot.querySelectorAll("button");
    assert.ok(buttons.length >= 2, "modal necesita al menos dos botones enfocables");
    const last = buttons[buttons.length - 1];
    last.focus();
    assert.equal(document.activeElement, last);
    modal.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", bubbles: true }));
    assert.notEqual(document.activeElement, last);
    assert.ok(buttons.includes(document.activeElement));
  });

  test("vx_side_panel_close_restores_focus_to_opener", async () => {
    const opener = document.createElement("button");
    opener.id = "open-panel";
    document.body.appendChild(opener);
    opener.focus();
    assert.equal(document.activeElement, opener);

    const panel = customElements._create("vx-side-panel");
    await panel.connectedCallback();
    panel.show(opener);
    panel.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    assert.equal(document.activeElement, opener);
  });

  test("overlay_already_closed_escape_does_not_emit", async () => {
    const modal = customElements._create("vx-modal");
    await modal.connectedCallback();
    let closed = 0;
    modal.addEventListener("vx-closed", () => {
      closed += 1;
    });
    modal.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    assert.equal(closed, 0);
  });

  test("vx_toast_info_aria_live_polite", async () => {
    const toast = customElements._create("vx-toast");
    toast.setAttribute("variant", "info");
    await toast.connectedCallback();
    const live = toast.shadowRoot.querySelector("[aria-live]");
    assert.ok(live);
    assert.equal(live.getAttribute("aria-live"), "polite");
    toast.setAttribute("variant", "error");
    toast.attributeChangedCallback("variant", "info", "error");
    assert.equal(live.getAttribute("aria-live"), "assertive");
  });

  test("vx_toast_without_aria_live_fails", async () => {
    const toast = customElements._create("vx-toast");
    await toast.connectedCallback();
    const live = toast.shadowRoot.querySelector("[aria-live]");
    assert.ok(live, "vx-toast debe exponer aria-live");
    assert.ok(
      live.getAttribute("aria-live") === "polite" ||
        live.getAttribute("aria-live") === "assertive",
    );
  });

  test("vx_toast_unknown_variant_falls_back_to_info", async () => {
    const toast = customElements._create("vx-toast");
    toast.setAttribute("variant", "warning");
    await toast.connectedCallback();
    const live = toast.shadowRoot.querySelector("[aria-live]");
    assert.equal(live.getAttribute("aria-live"), "polite");
  });
});
