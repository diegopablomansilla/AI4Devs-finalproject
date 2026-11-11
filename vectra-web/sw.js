/* Service Worker: app shell versionado (T-01-14). Ciclo de actualización en T-01-15. */
const CACHE = "vectra-shell-{{VERSION}}";

const PRECACHE = [
  "/",
  "/index.html",
  "/manifest.webmanifest",
  "/static/{{VERSION}}/css/tokens.css",
  "/static/{{VERSION}}/css/base.css",
  "/static/{{VERSION}}/js/app.js",
  "/static/{{VERSION}}/js/router.js",
  "/static/{{VERSION}}/js/api.js",
  "/static/{{VERSION}}/js/errors.js",
  "/static/{{VERSION}}/js/keyboard-list.js",
  "/static/{{VERSION}}/js/health.js",
  "/static/{{VERSION}}/js/components/index.js",
  "/static/{{VERSION}}/js/components/shared.js",
  "/static/{{VERSION}}/js/components/overlay.js",
  "/static/{{VERSION}}/js/components/vx-button.js",
  "/static/{{VERSION}}/js/components/vx-field.js",
  "/static/{{VERSION}}/js/components/vx-chip.js",
  "/static/{{VERSION}}/js/components/vx-modal.js",
  "/static/{{VERSION}}/js/components/vx-side-panel.js",
  "/static/{{VERSION}}/js/components/vx-toast.js",
  "/static/{{VERSION}}/js/components/vx-product-card.js",
  "/static/{{VERSION}}/js/components/vx-omnibox.js",
  "/static/{{VERSION}}/icons/icon-192.png",
  "/static/{{VERSION}}/icons/icon-512.png",
  "/static/{{VERSION}}/icons/icon-192-maskable.png",
  "/static/{{VERSION}}/icons/icon-512-maskable.png",
];

function requestPathname(request) {
  try {
    return new URL(request.url).pathname;
  } catch {
    return request.url;
  }
}

function isNetworkOnlyPath(pathname) {
  return (
    pathname === "/health" ||
    pathname.startsWith("/api/") ||
    pathname.startsWith("/media/")
  );
}

self.addEventListener("install", (event) => {
  event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(PRECACHE)));
});

self.addEventListener("fetch", (event) => {
  const pathname = requestPathname(event.request);

  if (isNetworkOnlyPath(pathname)) {
    event.respondWith(fetch(event.request));
    return;
  }

  if (event.request.mode === "navigate") {
    event.respondWith(
      fetch(event.request).catch(() =>
        caches.match("/index.html").then((cached) => cached || caches.match("/")),
      ),
    );
    return;
  }

  event.respondWith(fetch(event.request));
});
