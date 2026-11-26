import { describe, test, before, beforeEach } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  installDomGlobals,
  customElements,
  CustomEvent,
  Event,
} from "./dom-stub.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");

function healthJson(overrides = {}) {
  const base = {
    status: "ok",
    version: "0.1.0",
    uptimeSeconds: 1,
    components: {
      storage: { status: "up" },
      meilisearch: { status: "up" },
      llm: { status: "up" },
      speech: { status: "up" },
      textEmbeddings: { status: "disabled" },
      sync: { status: "disabled" },
    },
  };
  return { ...base, ...overrides, components: { ...base.components, ...(overrides.components ?? {}) } };
}

function jsonResponse(body, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function readCss(name) {
  return fs.readFileSync(path.join(REPO_ROOT, "vectra-web/css", name), "utf8");
}

const waitMicrotask = () => new Promise((resolve) => queueMicrotask(resolve));
const waitTick = () => new Promise((resolve) => setTimeout(resolve, 0));

function stubWindow() {
  /** @type {Map<string, Array<(...args: unknown[]) => void>>} */
  const listeners = new Map();
  return {
    addEventListener(type, fn) {
      const list = listeners.get(type) ?? [];
      list.push(fn);
      listeners.set(type, list);
    },
    removeEventListener(type, fn) {
      const list = listeners.get(type) ?? [];
      listeners.set(
        type,
        list.filter((entry) => entry !== fn),
      );
    },
  };
}

/** @param {import("./dom-stub.mjs").HTMLElement} omnibox */
function omniboxModeButton(omnibox, mode) {
  return omnibox.shadowRoot?.querySelector(`[data-mode="${mode}"]`) ?? null;
}

installDomGlobals();

describe("health", () => {
  /** @type {typeof import("../../../vectra-web/js/health.js")} */
  let healthMod;

  /** @type {() => void} */
  let stopHealthPoller = () => {};

  /** @param {Parameters<typeof import("../../../vectra-web/js/health.js").startHealth>[0]} opts */
  function startHealthTracked(opts) {
    stopHealthPoller();
    stopHealthPoller = healthMod.startHealth(opts);
  }

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
    healthMod = await import(
      new URL("../../../vectra-web/js/health.js", import.meta.url).href
    );
  });

  beforeEach(() => {
    try {
      stopHealthPoller();
    } catch {
      /* stub window may lack removeEventListener */
    }
    stopHealthPoller = () => {};
    document.body._children = [];
  });

  test("health_maps_llm_down_to_ai_false", () => {
    const body = healthJson({
      status: "degraded",
      components: { llm: { status: "down" } },
    });
    const snap = healthMod.capabilitiesFromHealth(body);
    assert.equal(snap.capabilities.ai, false);
    assert.equal(snap.capabilities.textSearch, true);
    assert.equal(snap.status, "degraded");
  });

  test("health_llm_down_disables_ai_mode_text_stays", async () => {
    const degraded = healthJson({
      status: "degraded",
      components: { llm: { status: "down" } },
    });
    let call = 0;
    const fetchFn = async () => {
      call += 1;
      return jsonResponse(degraded);
    };

    const indicator = document.createElement("p");
    indicator.setAttribute("data-health", "");
    indicator.setAttribute("aria-live", "polite");
    const omnibox = customElements._create("vx-omnibox");
    const outlet = document.createElement("div");
    outlet.appendChild(indicator);
    outlet.appendChild(omnibox);
    document.body.appendChild(outlet);

    await omnibox.connectedCallback();

    const intervals = [];
    const fakeSetInterval = (fn, ms) => {
      intervals.push({ fn, ms });
      return 1;
    };

    startHealthTracked({
      fetch: fetchFn,
      window: stubWindow(),
      document,
      setInterval: fakeSetInterval,
      clearInterval: () => {},
    });

    await waitTick();
    assert.equal(call, 1);
    assert.equal(omnibox.hasAttribute("ai-unavailable"), true);
    const aiBtn = omniboxModeButton(omnibox, "ai");
    assert.equal(aiBtn?.disabled, true);
    const textBtn = omniboxModeButton(omnibox, "text");
    assert.equal(textBtn?.disabled, false);
  });

  test("health_polls_every_10s", () => {
    const intervals = [];
    const fetchFn = async () => jsonResponse(healthJson());
    startHealthTracked({
      fetch: fetchFn,
      window: stubWindow(),
      document,
      setInterval: (fn, ms) => {
        intervals.push({ fn, ms });
        return 1;
      },
      clearInterval: () => {},
    });
    assert.equal(intervals.length, 1);
    assert.equal(intervals[0].ms, 10000);
    assert.equal(healthMod.HEALTH_POLL_MS, 10000);
  });

  test("health_refetches_on_focus", async () => {
    /** @type {Array<() => void>} */
    const focusHandlers = [];
    const win = stubWindow();
    const baseAdd = win.addEventListener.bind(win);
    win.addEventListener = (type, fn) => {
      if (type === "focus") {
        focusHandlers.push(fn);
      }
      baseAdd(type, fn);
    };
    let calls = 0;
    const fetchFn = async () => {
      calls += 1;
      return jsonResponse(healthJson());
    };
    startHealthTracked({
      fetch: fetchFn,
      window: win,
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    const before = calls;
    for (const fn of focusHandlers) {
      fn();
    }
    await waitTick();
    assert.ok(calls > before);
  });

  test("health_indicator_shows_degraded", async () => {
    const degraded = healthJson({
      status: "degraded",
      components: { llm: { status: "down" } },
    });
    const indicator = document.createElement("p");
    indicator.setAttribute("data-health", "");
    document.body.appendChild(indicator);

    startHealthTracked({
      fetch: async () => jsonResponse(degraded),
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.match(indicator.textContent ?? "", /degradado/i);
  });

  test("health_speech_not_up_disables_voice", async () => {
    const body = healthJson({
      components: { speech: { status: "disabled" } },
    });
    const omnibox = customElements._create("vx-omnibox");
    document.body.appendChild(omnibox);
    await omnibox.connectedCallback();

    startHealthTracked({
      fetch: async () => jsonResponse(body),
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.equal(omnibox.hasAttribute("voice-unavailable"), true);
    assert.equal(omniboxModeButton(omnibox, "voice")?.disabled, true);
  });

  test("health_vision_absent_hides_image", async () => {
    const omnibox = customElements._create("vx-omnibox");
    document.body.appendChild(omnibox);
    await omnibox.connectedCallback();

    startHealthTracked({
      fetch: async () => jsonResponse(healthJson()),
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.equal(omnibox.hasAttribute("vision"), false);
    assert.equal(omniboxModeButton(omnibox, "image")?.hidden, true);
  });

  test("health_network_failure_disables_optional_keeps_text", async () => {
    const omnibox = customElements._create("vx-omnibox");
    document.body.appendChild(omnibox);
    await omnibox.connectedCallback();
    const banner = document.createElement("p");
    banner.setAttribute("data-offline-banner", "");
    banner.setAttribute("hidden", "");
    banner.textContent = "Sin conexión con el Nodo Maestro";
    document.body.appendChild(banner);

    /** @type {{ status?: string; capabilities?: { textSearch?: boolean } }} */
    let lastDetail = {};
    document.addEventListener("vx-health", (e) => {
      lastDetail = e.detail ?? {};
    });

    await assert.doesNotReject(async () => {
      startHealthTracked({
        fetch: async () => {
          throw new TypeError("Failed to fetch");
        },
        window: stubWindow(),
        document,
        setInterval: () => 1,
        clearInterval: () => {},
      });
      await waitTick();
    });
    assert.equal(lastDetail.status, "unknown");
    assert.equal(lastDetail.capabilities?.textSearch, true);
    assert.equal(omnibox.hasAttribute("ai-unavailable"), true);
    assert.equal(omniboxModeButton(omnibox, "text")?.disabled, false);
    assert.equal(banner.hasAttribute("hidden"), false);
    assert.match(banner.textContent ?? "", /Sin conexión con el Nodo Maestro/);
  });

  test("health_503_starting_parses_snapshot", async () => {
    const starting = healthJson({
      status: "starting",
      components: { llm: { status: "starting" }, storage: { status: "starting" } },
    });
    const indicator = document.createElement("p");
    indicator.setAttribute("data-health", "");
    document.body.appendChild(indicator);

    startHealthTracked({
      fetch: async () => jsonResponse(starting, 503),
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.match(indicator.textContent ?? "", /arrancando/i);
    const banner = document.querySelector("[data-offline-banner]");
    if (banner) {
      assert.equal(banner.hasAttribute("hidden"), true);
    }
  });

  test("health_invalid_body_does_not_throw", async () => {
    await assert.doesNotReject(async () => {
      startHealthTracked({
        fetch: async () => new Response("<html>", { status: 200 }),
        window: stubWindow(),
        document,
        setInterval: () => 1,
        clearInterval: () => {},
      });
      await waitTick();
    });
  });

  test("health_does_not_show_reduced_search_banner", async () => {
    const body = healthJson({
      status: "degraded",
      components: { meilisearch: { status: "down" } },
    });
    const outlet = document.createElement("div");
    outlet.innerHTML = '<div id="app"></div>';
    document.body.appendChild(outlet);

    startHealthTracked({
      fetch: async () => jsonResponse(body),
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.doesNotMatch(document.body.textContent ?? "", /modo reducido/i);
    assert.doesNotMatch(
      document.body.textContent ?? "",
      /Búsqueda en modo reducido/i,
    );
    const banner = document.querySelector("[data-offline-banner]");
    if (banner) {
      assert.equal(banner.hasAttribute("hidden"), true);
    }
  });
});

describe("health_network_failure_shows_offline_banner", () => {
  test("Listo cuando T-01-14", async () => {
    installDomGlobals();
    const healthMod = await import(
      new URL("../../../vectra-web/js/health.js", import.meta.url).href
    );
    const banner = document.createElement("p");
    banner.setAttribute("data-offline-banner", "");
    banner.setAttribute("hidden", "");
    banner.textContent = "Sin conexión con el Nodo Maestro";
    document.body.appendChild(banner);
    healthMod.startHealth({
      fetch: async () => {
        throw new TypeError("Failed to fetch");
      },
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.equal(banner.hasAttribute("hidden"), false);
  });
});

describe("health_reachable_hides_offline_banner", () => {
  test("200 ok oculta el banner", async () => {
    installDomGlobals();
    const healthMod = await import(
      new URL("../../../vectra-web/js/health.js", import.meta.url).href
    );
    const banner = document.createElement("p");
    banner.setAttribute("data-offline-banner", "");
    banner.removeAttribute("hidden");
    document.body.appendChild(banner);
    healthMod.startHealth({
      fetch: async () => jsonResponse(healthJson()),
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.equal(banner.hasAttribute("hidden"), true);
  });
});

describe("health_degraded_does_not_show_offline_banner", () => {
  test("degraded sin banner PA-04", async () => {
    installDomGlobals();
    const healthMod = await import(
      new URL("../../../vectra-web/js/health.js", import.meta.url).href
    );
    const banner = document.createElement("p");
    banner.setAttribute("data-offline-banner", "");
    banner.setAttribute("hidden", "");
    document.body.appendChild(banner);
    const body = healthJson({
      status: "degraded",
      components: { meilisearch: { status: "down" }, llm: { status: "up" } },
    });
    healthMod.startHealth({
      fetch: async () => jsonResponse(body),
      window: stubWindow(),
      document,
      setInterval: () => 1,
      clearInterval: () => {},
    });
    await waitTick();
    assert.equal(banner.hasAttribute("hidden"), true);
  });
});
