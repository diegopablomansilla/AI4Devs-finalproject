# Proposal

## Ticket padre

- **Ticket:** [T-01-14](../../../docs/7-tickets.md) — Service Worker: *app shell* y sin conexión
- **Change:** `T-01-14`
- **HU:** [HU-01-01](../../../docs/6-historias.md#hu-01-01--aplicación-instalable-pwa)
- **Depende de:** T-01-13 (cerrado en `develop`; *manifest* e íconos; `GET /manifest.webmanifest` sustituye `{{VERSION}}`); T-00-19 (cerrado; `GET /sw.js` con `no-cache`); T-01-12 (cerrado; *poller* `/health`)
- **Hace:** `sw.js` con precache versionado del *app shell*; navegación con red primero y respaldo en caché; `/api/*` y `/health` solo red; la SPA muestra el aviso "Sin conexión con el Nodo Maestro" cuando falla la red (PA-04).
- **No hace:** caché de datos del catálogo.
- **Listo cuando:** con el servidor detenido, la app instalada abre el *shell* y muestra el aviso.

## Why

HU-01-01 y PA-04 piden que, sin Nodo Maestro, Vectra siga mostrando el *app shell* desde caché con un aviso claro — no el catálogo. T-01-13 ya deja el *manifest* instalable y T-00-19 ya reserva `GET /sw.js`, pero no hay Service Worker ni registro: Lighthouse no marca instalable y, al detener Vortex, la SPA no arranca. T-01-15 (actualización del SW) necesita este precache y este aviso como base.

## What Changes

- `vectra-web/sw.js`: precache versionado del *app shell* (`CACHE` `vectra-shell-{{VERSION}}`); `install` escribe esa lista; `fetch` usa red-primero en navegación con respaldo en caché; `/api/*` y `/health` solo red; el resto de GET no precacheados solo red (no cachear `/media/*`).
- El Gateway sustituye `{{VERSION}}` en el cuerpo de `GET /sw.js` (igual que `index.html` y el *manifest*); `Cache-Control: no-cache` y MIME `text/javascript` no cambian.
- `js/app.js` registra `navigator.serviceWorker.register("/sw.js")`. MUST NOT llamar `skipWaiting` ni `clients.claim` (T-01-15). Si `serviceWorker` no existe, MUST NOT tirar.
- Banner persistente `[data-offline-banner]` en operador y admin con el texto exacto **«Sin conexión con el Nodo Maestro»** (`aria-live="assertive"`). Visible solo cuando `GET /health` falla por red; oculto si el nodo responde (200 o 503 `starting`). El indicador `[data-health]` y `errors.js` (`network`) no cambian de copy. MUST NOT mostrar el banner de búsqueda reducida (T-03-31).
- Se invierte el No hace de T-01-13: existe `sw.js` y `app.js` registra el SW.
- TDD: `scripts/check-vectra-web.sh` + `sw.test.mjs` (parse + VM con `caches`/`fetch` fake) + `health.test.mjs` (banner) + `sw_js_replaces_version` en el crate `api`. MUST NOT cambiar `docs/openapi.yaml`. MUST NOT agregar Playwright, Lighthouse CLI ni npm.

El **Listo cuando** («servidor detenido, app instalada abre el *shell* y muestra el aviso») se automatiza con las estrategias del SW y el banner; e2e en navegador real queda en T-09-10.

## Non-goals

- Caché de catálogo (`/api/*`, `/media/*`) — **No hace** del ticket (PA-04)
- Actualización del SW, aviso «Nueva versión disponible», `skipWaiting` / `clients.claim`, borrado de cachés viejas — **T-01-15**
- HTTPS, CA local, `vectra.local` — **T-00-16**, **T-00-17**, **T-00-18** (v2; en MVP `localhost` ya es contexto seguro)
- Playwright / e2e / Lighthouse en CI — **T-09-10**; este ticket no agrega `package.json` ni lighthouse
- Banner «Búsqueda en modo reducido» — **T-03-31**
- Cambiar el mensaje `network` de `errors.js` o el texto de `[data-health]`
- jsdom, job nuevo de CI, mutar OpenAPI o Web Components nuevos

## Capabilities

### New Capabilities

- Ninguna. El Service Worker y el aviso offline son parte de la SPA `vectra-web`; la sustitución en `/sw.js` es el Gateway `vortex-api`.

### Modified Capabilities

- `vectra-web`: agregar `sw.js` (precache versionado, estrategias de *fetch*), registro en `app.js`, banner PA-04 en *router* + `health.js`; ampliar la suite Node (happy + unhappy). El requirement «Manifest PWA de Vectra» deja de prohibir el SW.
- `vortex-api`: `GET /sw.js` MUST sustituir `{{VERSION}}` en el cuerpo (hoy son bytes crudos). Sin rutas nuevas; `Cache-Control: no-cache` y MIME de `.js` siguen.

## Impact

- **Repo:** `vectra-web/sw.js` (nuevo); `vectra-web/js/app.js`; `vectra-web/js/health.js`; `vectra-web/js/router.js`; `vectra-web/css/base.css` (estilos del banner con tokens existentes); `scripts/fixtures/vectra-web/sw.test.mjs` (nuevo); `scripts/fixtures/vectra-web/health.test.mjs`; `scripts/fixtures/vectra-web/manifest.test.mjs` (invertir «no registra SW»); `scripts/check-vectra-web.sh`; `vortex-core/crates/api/src/static_files.rs` (`get_sw_js`) y `vortex-core/crates/api/tests/static_files.rs`
- **APIs / OpenAPI:** sin paths nuevos; reutiliza `GET /sw.js` y `/static/<versión>/…` de T-00-19
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-14`; en `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-14 (criterios SW + aviso PA-04 de instalable/offline; actualización espera T-01-15)
- **Alcance:** cabe en ~2 h (`sw.js` + registro + banner + replace Gateway + tests Node/Rust); no se divide
