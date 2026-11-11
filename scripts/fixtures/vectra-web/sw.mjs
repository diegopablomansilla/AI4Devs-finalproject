import vm from "node:vm";

const PRECACHE_BLOCK =
  /(?:const|let|var)\s+PRECACHE\s*=\s*\[([\s\S]*?)\];/;

/**
 * @param {string} swSource
 * @returns {string[]}
 */
export function extractPrecacheUrls(swSource) {
  const block = swSource.match(PRECACHE_BLOCK);
  if (!block) {
    return [];
  }
  const inner = block[1];
  /** @type {string[]} */
  const urls = [];
  const re = /["'`]([^"'`]+)["'`]/g;
  let m;
  while ((m = re.exec(inner)) !== null) {
    urls.push(m[1]);
  }
  return urls;
}

/**
 * @param {string} swSource
 */
export function extractCacheName(swSource) {
  const m = swSource.match(
    /(?:const|let|var)\s+(?:CACHE|CACHE_NAME)\s*=\s*["'`]([^"'`]+)["'`]/,
  );
  return m?.[1] ?? "";
}

/**
 * @param {string[]} urls
 */
export function validatePrecacheUrls(urls) {
  /** @type {string[]} */
  const errors = [];
  if (urls.length === 0) {
    errors.push("falta lista PRECACHE");
    return { ok: false, errors };
  }
  for (const u of urls) {
    if (u.includes("/api/") || u === "/health" || u.startsWith("/health?")) {
      errors.push(`precache incluye API o health: ${u}`);
    }
    if (u.startsWith("/media/")) {
      errors.push(`precache incluye media: ${u}`);
    }
  }
  const needs = ["/", "/index.html", "/manifest.webmanifest"];
  for (const need of needs) {
    if (!urls.includes(need)) {
      errors.push(`falta precache ${need}`);
    }
  }
  if (!urls.some((u) => u.includes("/static/{{VERSION}}/js/app.js"))) {
    errors.push("falta precache js/app.js versionado");
  }
  return { ok: errors.length === 0, errors };
}

/**
 * @param {string} navSource from app.js
 */
export function appRegistersServiceWorker(navSource) {
  return (
    navSource.includes('serviceWorker.register("/sw.js")') ||
    navSource.includes("serviceWorker.register('/sw.js')")
  );
}

/**
 * @param {string} swSource
 */
export function swOmitsUpdateHacks(swSource) {
  return !swSource.includes("skipWaiting") && !swSource.includes("clients.claim");
}

/**
 * @param {string} swSource
 * @returns {{ install?: (ev: unknown) => void, fetch?: (ev: unknown) => void }}
 */
export function loadSwHandlers(swSource) {
  /** @type {Record<string, (ev: unknown) => void>} */
  const handlers = {};
  const globals = {
    self: {
      addEventListener(type, fn) {
        handlers[type] = fn;
      },
    },
    caches: null,
    fetch: null,
    URL,
    Request,
    Response,
  };
  vm.runInNewContext(swSource, globals, { filename: "sw.js" });
  return { handlers, globals };
}

/**
 * @returns {{ open: (name: string) => Promise<{ addAll: (urls: string[]) => Promise<void>, put: (req: unknown, res: unknown) => Promise<void>, match: (req: string) => Promise<unknown> }>, match: (req: string) => Promise<unknown> }}
 */
export function createMemoryCacheMock() {
  /** @type {Map<string, Map<string, unknown>>} */
  const stores = new Map();

  return {
    async open(name) {
      if (!stores.has(name)) {
        stores.set(name, new Map());
      }
      const map = stores.get(name);
      return {
        async addAll(urls) {
          for (const u of urls) {
            map.set(u, { url: u, body: `cached:${u}` });
          }
        },
        async put(req, res) {
          const key = typeof req === "string" ? req : req.url;
          map.set(key, res);
        },
        async match(req) {
          const key = typeof req === "string" ? req : req.url;
          return map.get(key) ?? null;
        },
      };
    },
    async match(req) {
      const key = typeof req === "string" ? req : req.url;
      for (const map of stores.values()) {
        const hit = map.get(key);
        if (hit) {
          return hit;
        }
      }
      return null;
    },
  };
}

/**
 * @param {{ navigator?: { serviceWorker?: { register: (path: string) => Promise<unknown> } } }} nav
 */
export function registerAppServiceWorker(nav) {
  try {
    const sw = nav?.serviceWorker;
    if (sw && typeof sw.register === "function") {
      void sw.register("/sw.js");
    }
  } catch {
    // MUST NOT throw when serviceWorker is missing or register fails
  }
}
