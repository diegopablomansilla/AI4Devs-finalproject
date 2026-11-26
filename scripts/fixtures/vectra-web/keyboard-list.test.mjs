import { describe, test, before } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  installDomGlobals,
  customElements,
  KeyboardEvent,
} from "./dom-stub.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const COMPONENTS_HTML = path.join(REPO_ROOT, "vectra-web/dev/components.html");

function readCss(name) {
  return fs.readFileSync(path.join(REPO_ROOT, "vectra-web/css", name), "utf8");
}

/** @param {import("./dom-stub.mjs").FakeNode} omnibox */
function omniboxInput(omnibox) {
  return omnibox.shadowRoot?.querySelector("input") ?? null;
}

function makeListWithItems(count = 2) {
  const list = document.createElement("div");
  list.setAttribute("role", "listbox");
  list.setAttribute("tabindex", "0");
  const items = [];
  for (let i = 0; i < count; i += 1) {
    const item = document.createElement("div");
    item.textContent = `item-${i}`;
    list.appendChild(item);
    items.push(item);
  }
  document.body.appendChild(list);
  return { list, items };
}

installDomGlobals();

let bindKeyboardList;

describe("components_html_catalog_loads_keyboard_list", () => {
  test("muestra lista de ejemplo y keyboard-list", () => {
    const html = fs.readFileSync(COMPONENTS_HTML, "utf8");
    assert.match(html, /keyboard-list\.js/);
    assert.match(html, /id="demo-keyboard-list"/);
    assert.match(html, /id="demo-keyboard-list"[\s\S]*<vx-product-card/);
  });
});

describe("keyboard_list", () => {
  before(async () => {
    const mod = await import(
      new URL("../../../vectra-web/js/keyboard-list.js", import.meta.url).href
    );
    bindKeyboardList = mod.bindKeyboardList;
  });

  test("keyboard_list_arrow_down_selects_first_and_updates_activedescendant", () => {
    const { list, items } = makeListWithItems(2);
    bindKeyboardList({
      list,
      getItems: () => items,
    });
    list.focus();
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    assert.equal(items[0].getAttribute("aria-selected"), "true");
    assert.equal(list.getAttribute("aria-activedescendant"), items[0].getAttribute("id"));
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    assert.equal(items[1].getAttribute("aria-selected"), "true");
    assert.equal(list.getAttribute("aria-activedescendant"), items[1].getAttribute("id"));
  });

  test("keyboard_list_enter_opens_active_item", () => {
    const { list, items } = makeListWithItems(2);
    let opened = null;
    bindKeyboardList({
      list,
      getItems: () => items,
      onOpen: (item) => {
        opened = item;
      },
    });
    list.focus();
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    assert.equal(opened, items[0]);
  });

  test("keyboard_list_escape_restores_omnibox_without_clearing", async () => {
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
      new URL("../../../vectra-web/js/components/vx-omnibox.js", import.meta.url).href
    );

    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const input = omniboxInput(box);
    assert.ok(input);
    input.value = "perno";
    box.setAttribute("value", "perno");

    const { list, items } = makeListWithItems(2);
    bindKeyboardList({
      list,
      getItems: () => items,
      controllers: [list, input],
      restoreFocus: box,
    });
    list.focus();
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    assert.equal(document.activeElement, input);
    assert.equal(input.value, "perno");
  });

  test("keyboard_list_arrow_from_omnibox_selects_first", async () => {
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
      new URL("../../../vectra-web/js/components/vx-omnibox.js", import.meta.url).href
    );

    const box = customElements._create("vx-omnibox");
    document.body.appendChild(box);
    await box.connectedCallback();
    const input = omniboxInput(box);
    assert.ok(input);
    input.focus();

    const { list, items } = makeListWithItems(2);
    bindKeyboardList({
      list,
      getItems: () => items,
      controllers: [list, input],
    });
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    assert.equal(items[0].getAttribute("aria-selected"), "true");
    assert.equal(list.getAttribute("aria-activedescendant"), items[0].getAttribute("id"));
  });

  test("keyboard_list_plain_options_are_reusable", () => {
    const list = document.createElement("div");
    document.body.appendChild(list);
    const a = document.createElement("span");
    a.textContent = "Arandela";
    const b = document.createElement("span");
    b.textContent = "Bulón";
    const items = [a, b];
    let opened = null;
    bindKeyboardList({
      list,
      getItems: () => items,
      onOpen: (item) => {
        opened = item;
      },
    });
    list.focus();
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    assert.equal(opened, a);
    assert.equal(a.getAttribute("role"), "option");
  });

  test("keyboard_list_empty_does_not_throw", () => {
    const list = document.createElement("div");
    document.body.appendChild(list);
    bindKeyboardList({ list, getItems: () => [] });
    list.focus();
    assert.doesNotThrow(() => {
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true }));
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    });
    assert.equal(list.getAttribute("aria-activedescendant"), null);
  });

  test("keyboard_list_enter_without_selection_does_not_open", () => {
    const { list, items } = makeListWithItems(2);
    let count = 0;
    bindKeyboardList({
      list,
      getItems: () => items,
      onOpen: () => {
        count += 1;
      },
    });
    list.focus();
    list.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    assert.equal(count, 0);
  });

  test("keyboard_list_escape_without_omnibox_does_not_throw", () => {
    const { list, items } = makeListWithItems(1);
    bindKeyboardList({
      list,
      getItems: () => items,
      restoreFocus: null,
    });
    list.focus();
    assert.doesNotThrow(() => {
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    });
  });

  test("keyboard_list_does_not_call_search", () => {
    let fetchCalls = 0;
    const prev = globalThis.fetch;
    globalThis.fetch = () => {
      fetchCalls += 1;
      return Promise.resolve(new Response());
    };
    try {
      const { list, items } = makeListWithItems(2);
      bindKeyboardList({
        list,
        getItems: () => items,
        onOpen: () => {},
      });
      list.focus();
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
      list.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
      assert.equal(fetchCalls, 0);
    } finally {
      globalThis.fetch = prev;
    }
  });
});
