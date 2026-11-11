import { describe, test, before } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  installDomGlobals,
  customElements,
  CustomEvent,
  KeyboardEvent,
  Event,
} from "./dom-stub.mjs";

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

/** @param {import("./dom-stub.mjs").HTMLElement} omnibox */
function omniboxModeButton(omnibox, mode) {
  return omnibox.shadowRoot?.querySelector(`[data-mode="${mode}"]`) ?? null;
}

/** @param {import("./dom-stub.mjs").HTMLElement} omnibox */
function fireOmniboxInput(omnibox, value) {
  const input = omniboxInput(omnibox);
  assert.ok(input);
  input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true }));
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

  test("omnibox_question_prefix_sets_ai_mode_and_emits_vx_query", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    /** @type {{ mode?: string; text?: string }} */
    let lastDetail = {};
    box.addEventListener("vx-query", (e) => {
      lastDetail = e.detail ?? {};
    });
    fireOmniboxInput(box, "?hola");
    assert.equal(box.getAttribute("mode"), "ai");
    const aiBtn = omniboxModeButton(box, "ai");
    assert.equal(aiBtn?.getAttribute("aria-pressed"), "true");
    assert.equal(lastDetail.mode, "ai");
    assert.equal(lastDetail.text, "?hola");
  });

  test("omnibox_text_mode_emits_vx_query_with_mode_text", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
  /** @type {{ mode?: string; text?: string }} */
    let lastDetail = {};
    box.addEventListener("vx-query", (e) => {
      lastDetail = e.detail ?? {};
    });
    fireOmniboxInput(box, "tornillo");
    assert.equal(box.getAttribute("mode"), "text");
    assert.equal(lastDetail.mode, "text");
    assert.equal(lastDetail.text, "tornillo");
  });

  test("omnibox_click_ai_sets_mode_and_indicator", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const input = omniboxInput(box);
    input.value = "broca";
    /** @type {{ mode?: string; text?: string }} */
    let lastDetail = {};
    box.addEventListener("vx-query", (e) => {
      lastDetail = e.detail ?? {};
    });
    const aiBtn = omniboxModeButton(box, "ai");
    aiBtn?.click();
    assert.equal(box.getAttribute("mode"), "ai");
    assert.equal(aiBtn?.getAttribute("aria-pressed"), "true");
    assert.equal(lastDetail.mode, "ai");
    assert.equal(lastDetail.text, "broca");
  });

  test("omnibox_selector_shows_text_voice_ai", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    assert.ok(omniboxModeButton(box, "text"));
    assert.ok(omniboxModeButton(box, "voice"));
    assert.ok(omniboxModeButton(box, "ai"));
    assert.equal(box.getAttribute("mode"), "text");
    const imageBtn = omniboxModeButton(box, "image");
    assert.ok(imageBtn);
    assert.equal(imageBtn.hidden, true);
  });

  test("omnibox_image_mode_visible_with_vision", async () => {
    const box = customElements._create("vx-omnibox");
    box.setAttribute("vision", "");
    document.body.appendChild(box);
    await box.connectedCallback();
    const imageBtn = omniboxModeButton(box, "image");
    assert.ok(imageBtn);
    assert.equal(imageBtn.hidden, false);
    /** @type {{ mode?: string; text?: string }} */
    let lastDetail = {};
    box.addEventListener("vx-query", (e) => {
      lastDetail = e.detail ?? {};
    });
    imageBtn?.click();
    assert.equal(box.getAttribute("mode"), "image");
    assert.equal(lastDetail.mode, "image");
  });

  test("omnibox_question_not_at_start_stays_text", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    /** @type {{ mode?: string; text?: string }} */
    let lastDetail = {};
    box.addEventListener("vx-query", (e) => {
      lastDetail = e.detail ?? {};
    });
    fireOmniboxInput(box, "hola?");
    assert.equal(box.getAttribute("mode"), "text");
    assert.equal(lastDetail.mode, "text");
  });

  test("omnibox_image_mode_hidden_without_vision", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const imageBtn = omniboxModeButton(box, "image");
    assert.equal(imageBtn?.hidden, true);
    imageBtn?.click();
    assert.notEqual(box.getAttribute("mode"), "image");
  });

  test("omnibox_mode_does_not_call_search", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    let fetchCalls = 0;
    const prevFetch = globalThis.fetch;
    globalThis.fetch = () => {
      fetchCalls += 1;
      return Promise.resolve({ ok: true });
    };
    try {
      fireOmniboxInput(box, "?hola");
      omniboxModeButton(box, "voice")?.click();
      omniboxModeButton(box, "ai")?.click();
      assert.equal(fetchCalls, 0);
    } finally {
      globalThis.fetch = prevFetch;
    }
  });

  test("omnibox_escape_on_ai_prefix_resets_to_text", async () => {
    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    fireOmniboxInput(box, "?hola");
    assert.equal(box.getAttribute("mode"), "ai");
    const input = omniboxInput(box);
    input.focus();
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    assert.equal(input.value, "");
    assert.equal(box.getAttribute("mode"), "text");
  });

  test("omnibox_unknown_mode_falls_back_to_text", async () => {
    const box = customElements._create("vx-omnibox");
    box.setAttribute("mode", "nope");
    document.body.appendChild(box);
    await box.connectedCallback();
    assert.equal(box.getAttribute("mode"), "text");
    assert.doesNotThrow(() => {
      box.setAttribute("mode", "invalid");
      box.attributeChangedCallback("mode", "nope", "invalid");
    });
    assert.equal(box.getAttribute("mode"), "text");
  });
});
