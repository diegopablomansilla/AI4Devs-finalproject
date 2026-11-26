# Proposal

## Ticket padre

- **Ticket:** [T-01-15](../../../docs/7-tickets.md) — Actualización del Service Worker
- **Change:** `T-01-15`
- **HU:** [HU-01-01](../../../docs/6-historias.md#hu-01-01--aplicación-instalable-pwa) (escenario «Actualización»)
- **Depende de:** T-01-14 (cerrado en `develop`; precache `vectra-shell-{{VERSION}}`, registro en `app.js`, sin `skipWaiting` / `clients.claim`)
- **Hace:** detección de versión nueva en espera, aviso "Nueva versión disponible", activación al recargar y borrado de cachés viejas.
- **No hace:** actualizaciones forzadas sin recargar.
- **Listo cuando:** cambiar la versión del servidor y recargar aplica los nuevos estáticos.

## Why

T-01-14 deja un Service Worker que precachea el *app shell* versionado, pero no hay ciclo de actualización: el worker nuevo queda *waiting* sin aviso, no hay `activate` que borre `vectra-shell-*` anteriores, y T-01-14 aplaza `skipWaiting` / `clients.claim` a este ticket. HU-01-01 pide que una versión nueva de estáticos se descargue y se aplique **al recargar**. Sin este change, cada bump de `version` acumula cachés y el operador en PWA *standalone* (sin chrome de recarga) no se entera.

## What Changes

- `vectra-web/sw.js`: listener `activate` que borra cachés `vectra-shell-*` distintas de `CACHE` (`vectra-shell-{{VERSION}}`). MUST NOT llamar `skipWaiting` ni `clients.claim` (**No hace**: forzar sin recargar). `install` / `fetch` de T-01-14 no cambian.
- `vectra-web/js/sw-update.js` (nuevo): tras `register("/sw.js")`, detecta `registration.waiting` y `updatefound` → `installed` con un *controller* ya activo; muestra el aviso. Llama `registration.update()` al recuperar el foco (la SPA no navega de documento). MUST NOT hacer `location.reload()` solo por detectar *waiting*.
- Banner persistente `[data-update-banner]` en operador y admin con el texto exacto **«Nueva versión disponible»** (`aria-live="polite"`). Incluye un control **Recargar** (`data-update-reload`) que solo hace `location.reload()` — necesario en *standalone*, donde no hay chrome de recarga. MUST NOT postMessage `skipWaiting`. Distinto del banner PA-04.
- Precache y harness: agregar `/static/{{VERSION}}/js/sw-update.js`. `app.js` arranca el *watcher*.
- TDD: `sw.test.mjs` (VM `activate` + *waiting* fake) + `router.test.mjs` (markup del banner) + harness. MUST NOT cambiar `docs/openapi.yaml`. MUST NOT agregar Playwright, Lighthouse CLI ni npm.

El **Listo cuando** se automatiza: `activate` deja solo la caché de la versión nueva; el *watcher* muestra el aviso ante un worker en espera; Recargar = `reload` sin `skipWaiting`. El e2e en navegador real queda en T-09-10.

## Non-goals

- Actualizaciones forzadas sin recargar (`skipWaiting`, `clients.claim`, `reload` automático al detectar *waiting*) — **No hace** del ticket
- Precache del catálogo / `/api/*` / `/health` / `/media/*` — **T-01-14** (ya cubierto; no se reabre)
- HTTPS, CA local, `vectra.local` — **T-00-16**, **T-00-17**, **T-00-18** (v2)
- Playwright / e2e / Lighthouse en CI — **T-09-10**
- Banner «Sin conexión con el Nodo Maestro» — **T-01-14** (no se cambia el copy ni la señal `/health`)
- Banner «Búsqueda en modo reducido» — **T-03-31**
- Cambiar estrategias `fetch` de T-01-14, MIME / `no-cache` de `GET /sw.js`, o `docs/openapi.yaml`
- jsdom, job nuevo de CI, Web Components nuevos, tokens CSS nuevos

## Capabilities

### New Capabilities

- Ninguna. El ciclo de actualización del SW es parte de la SPA `vectra-web`. El Gateway ya sustituye `{{VERSION}}` en `GET /sw.js` (T-01-14).

### Modified Capabilities

- `vectra-web`: `activate` + aviso de versión + control Recargar; módulo `sw-update.js`; ampliar la suite Node (happy + unhappy). El requirement «Service Worker del app shell» deja de aplazar el ciclo a T-01-15 y pasa a prohibir `skipWaiting` / `clients.claim` como No hace de este ticket.

## Impact

- **Repo:** `vectra-web/sw.js`; `vectra-web/js/sw-update.js` (nuevo); `vectra-web/js/app.js`; `vectra-web/js/router.js`; `vectra-web/css/base.css` (estilos del banner con tokens existentes); `scripts/fixtures/vectra-web/sw.test.mjs`; `scripts/fixtures/vectra-web/sw.mjs` (mock `caches.keys` / `delete`; dejar de tratar `activate` como hack); `scripts/fixtures/vectra-web/router.test.mjs`; `scripts/check-vectra-web.sh`
- **APIs / OpenAPI:** sin cambios; reutiliza `GET /sw.js` `no-cache` y `/static/<versión>/…` de T-00-19 / T-01-14
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-15`; en `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-15 (criterio de actualización de HU-01-01)
- **Alcance:** cabe en ~2 h (`activate` + *watcher* + banner Recargar + tests Node); no se divide
