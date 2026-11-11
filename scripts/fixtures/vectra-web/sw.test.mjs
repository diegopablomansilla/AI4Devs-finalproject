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

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const WEB = path.join(REPO_ROOT, "vectra-web");
const SW_PATH = path.join(WEB, "sw.js");
const APP_JS = path.join(WEB, "js/app.js");

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
