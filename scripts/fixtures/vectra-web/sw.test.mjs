import { describe, test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  extractPrecacheUrls,
  extractCacheName,
  validatePrecacheUrls,
  appRegistersServiceWorker,
  swOmitsUpdateHacks,
  loadSwHandlers,
  createMemoryCacheMock,
  registerAppServiceWorker,
} from "./sw.mjs";
import {
  applyUpdateBanner,
  bindUpdateReloadHandlers,
  startSwUpdateWatcher,
} from "../../../vectra-web/js/sw-update.js";
import { installDomGlobals } from "./dom-stub.mjs";

/** @returns {{ document: Document, window: { location: { reload: () => void }, addEventListener: (type: string, fn: () => void) => void, removeEventListener: (type: string, fn: () => void) => void }, reloadCalls: { n: number } }} */
function testDom() {
  installDomGlobals();
  const document = /** @type {Document} */ (globalThis.document);
  document.body.innerHTML = "";
  const reloadCalls = { n: 0 };
  const window = {
    location: {
      reload() {
        reloadCalls.n += 1;
      },
    },
    addEventListener() {},
    removeEventListener() {},
  };
  return { document, window, reloadCalls };
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const WEB = path.join(REPO_ROOT, "vectra-web");
const SW_PATH = path.join(WEB, "sw.js");
const APP_JS = path.join(WEB, "js/app.js");
const SW_UPDATE_JS = path.join(WEB, "js/sw-update.js");
const UPDATE_COPY = "Nueva versión disponible";

function readSw() {
  assert.ok(fs.existsSync(SW_PATH), "falta vectra-web/sw.js");
  return fs.readFileSync(SW_PATH, "utf8");
}

describe("sw_precache_is_versioned_app_shell", () => {
  test("Listo cuando T-01-14", () => {
    const source = readSw();
    assert.match(source, /vectra-shell-\{\{VERSION\}\}/);
    const cacheName = extractCacheName(source);
    assert.equal(cacheName, "vectra-shell-{{VERSION}}");
    const urls = extractPrecacheUrls(source);
    const { ok, errors } = validatePrecacheUrls(urls);
    assert.equal(ok, true, errors.join("; "));
    assert.equal(swOmitsUpdateHacks(source), true);
  });
});

describe("app_registers_service_worker", () => {
  test("app.js registra /sw.js", () => {
    assert.ok(fs.existsSync(APP_JS));
    const app = fs.readFileSync(APP_JS, "utf8");
    assert.equal(appRegistersServiceWorker(app), true);
  });
});

describe("sw_navigation_online_uses_network", () => {
  test("navigate con red devuelve la red", async () => {
    const { handlers, globals } = loadSwHandlers(readSw());
    assert.equal(typeof handlers.fetch, "function");
    const networkBody = "<html id=net></html>";
    globals.caches = createMemoryCacheMock();
    await globals.caches.open("vectra-shell-{{VERSION}}").then((c) =>
      c.addAll(["/", "/index.html"]),
    );
    globals.fetch = async () =>
      new Response(networkBody, { status: 200, headers: { "Content-Type": "text/html" } });

    let responded;
    const event = {
      request: { url: "http://localhost/", mode: "navigate" },
      respondWith(p) {
        responded = p;
      },
    };
    handlers.fetch(event);
    const response = await responded;
    const text = await response.text();
    assert.equal(text, networkBody);
  });
});

describe("sw_navigation_offline_serves_cached_shell", () => {
  test("Listo cuando T-01-14", async () => {
    const { handlers, globals } = loadSwHandlers(readSw());
    globals.caches = createMemoryCacheMock();
    const cached = "<html id=shell></html>";
    const store = await globals.caches.open("vectra-shell-{{VERSION}}");
    await store.put("/index.html", new Response(cached, { status: 200 }));
    globals.fetch = async () => {
      throw new TypeError("Failed to fetch");
    };

    let responded;
    const event = {
      request: { url: "http://localhost/", mode: "navigate" },
      respondWith(p) {
        responded = p;
      },
    };
    handlers.fetch(event);
    const response = await responded;
    const text = await response.text();
    assert.equal(text, cached);
  });
});

describe("sw_health_and_api_are_network_only", () => {
  test("No hace: sin fallback en caché", async () => {
    const { handlers, globals } = loadSwHandlers(readSw());
    globals.caches = createMemoryCacheMock();
    const store = await globals.caches.open("vectra-shell-{{VERSION}}");
    await store.put(
      "/health",
      new Response('{"status":"ok"}', { status: 200 }),
    );
    let fetchCalled = false;
    globals.fetch = async () => {
      fetchCalled = true;
      throw new TypeError("Failed to fetch");
    };

    for (const path of ["/health", "/api/v1/search"]) {
      let reject;
      const event = {
        request: new Request(`http://localhost${path}`),
        respondWith(p) {
          reject = p;
        },
      };
      handlers.fetch(event);
      await assert.rejects(reject);
    }
    assert.equal(fetchCalled, true);
  });
});

describe("sw_precache_with_api_or_media_fails", () => {
  test("unhappy fixture in-memory", () => {
    const bad = ["/", "/api/v1/search"];
    const { ok, errors } = validatePrecacheUrls(bad);
    assert.equal(ok, false);
    assert.match(errors.join(" "), /api|API|health|media/i);
    const badMedia = ["/", "/media/foo.jpg"];
    const r2 = validatePrecacheUrls(badMedia);
    assert.equal(r2.ok, false);
  });
});

describe("sw_register_without_service_worker_does_not_throw", () => {
  test("sin navigator.serviceWorker", () => {
    assert.doesNotThrow(() => registerAppServiceWorker({}));
    assert.doesNotThrow(() => registerAppServiceWorker(undefined));
  });
});

describe("sw_precache_includes_sw_update", () => {
  test("precache incluye sw-update.js", () => {
    const urls = extractPrecacheUrls(readSw());
    assert.ok(
      urls.some((u) => u.includes("/static/{{VERSION}}/js/sw-update.js")),
    );
  });
});

describe("sw_activate_deletes_old_shell_caches", () => {
  test("Listo cuando T-01-15", async () => {
    const { handlers, globals } = loadSwHandlers(readSw());
    assert.equal(typeof handlers.activate, "function");
    globals.caches = createMemoryCacheMock();
    await globals.caches.open("vectra-shell-old");
    await globals.caches.open("vectra-shell-{{VERSION}}");

    let waitUntil;
    const event = {
      waitUntil(p) {
        waitUntil = p;
      },
    };
    handlers.activate(event);
    await waitUntil;

    const keys = await globals.caches.keys();
    assert.ok(!keys.includes("vectra-shell-old"));
    assert.ok(keys.includes("vectra-shell-{{VERSION}}"));
  });
});

describe("sw_activate_keeps_current_cache", () => {
  test("unhappy: no borra la caché actual", async () => {
    const { handlers, globals } = loadSwHandlers(readSw());
    globals.caches = createMemoryCacheMock();
    await globals.caches.open("vectra-shell-{{VERSION}}");

    let waitUntil;
    handlers.activate({
      waitUntil(p) {
        waitUntil = p;
      },
    });
    await waitUntil;

    const keys = await globals.caches.keys();
    assert.deepEqual(keys, ["vectra-shell-{{VERSION}}"]);
  });
});

describe("sw_does_not_skip_waiting_or_claim", () => {
  test("No hace T-01-15 en sw.js y sw-update.js", () => {
    assert.equal(swOmitsUpdateHacks(readSw()), true);
    assert.ok(fs.existsSync(SW_UPDATE_JS));
    const updateSource = fs.readFileSync(SW_UPDATE_JS, "utf8");
    assert.equal(swOmitsUpdateHacks(updateSource), true);
  });
});

describe("sw_waiting_shows_update_banner", () => {
  test("waiting con controller muestra el aviso", () => {
    const { document } = testDom();
    const banner = document.createElement("p");
    banner.setAttribute("data-update-banner", "");
    banner.setAttribute("hidden", "");
    banner.textContent = UPDATE_COPY;
    document.body.appendChild(banner);

    applyUpdateBanner(document, true);
    assert.equal(banner.hasAttribute("hidden"), false);
    assert.match(banner.textContent ?? "", new RegExp(UPDATE_COPY));
  });
});

describe("sw_without_waiting_hides_update_banner", () => {
  test("unhappy: sin waiting oculto", () => {
    const { document } = testDom();
    const banner = document.createElement("p");
    banner.setAttribute("data-update-banner", "");
    document.body.appendChild(banner);

    applyUpdateBanner(document, false);
    assert.equal(banner.hasAttribute("hidden"), true);
  });
});

describe("sw_reload_control_reloads_without_skip_waiting", () => {
  test("Listo cuando T-01-15", () => {
    const { document, window, reloadCalls } = testDom();
    const btn = document.createElement("button");
    btn.setAttribute("data-update-reload", "");
    document.body.appendChild(btn);

    bindUpdateReloadHandlers(document, window);
    btn.dispatchEvent(new Event("click"));
    assert.equal(reloadCalls.n, 1);
    assert.equal(swOmitsUpdateHacks(fs.readFileSync(SW_UPDATE_JS, "utf8")), true);
  });
});

describe("sw_first_install_does_not_show_update_banner", () => {
  test("unhappy: sin controller no avisa", async () => {
    const { document, window } = testDom();
    const banner = document.createElement("p");
    banner.setAttribute("data-update-banner", "");
    banner.setAttribute("hidden", "");
    document.body.appendChild(banner);

    const fakeReg = {
      waiting: {},
      update: async () => {},
      addEventListener() {},
    };
    const nav = {
      serviceWorker: {
        controller: null,
        register: async () => fakeReg,
      },
    };

    await startSwUpdateWatcher({ navigator: nav, window, document });
    assert.equal(banner.hasAttribute("hidden"), true);
  });
});

describe("sw_update_check_failure_does_not_throw", () => {
  test("unhappy: update() rechaza", async () => {
    const { document, window } = testDom();
    const nav = {
      serviceWorker: {
        controller: {},
        register: async () => ({
          waiting: {},
          update: async () => {
            throw new Error("update failed");
          },
          addEventListener() {},
        }),
      },
    };

    await assert.doesNotReject(() =>
      startSwUpdateWatcher({ navigator: nav, window, document }),
    );
  });
});
