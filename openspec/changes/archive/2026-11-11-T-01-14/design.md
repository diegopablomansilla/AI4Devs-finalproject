# Design

## Context

`develop` ya sirve `GET /sw.js` como bytes crudos con `no-cache` (T-00-19) y sustituye `{{VERSION}}` en `index.html` y el *manifest* (T-01-13). `vectra-web/` no tiene `sw.js`; `manifest.test.mjs` aserta que **no** hay registro. `health.js` ante fallo de red publica `status: "unknown"` y **no** muestra aviso PA-04. El stub Node no es jsdom ni hay Cache Storage nativo. Ver proposal.md para el porqué; spec para precache, estrategias y Listo cuando.

## Goals / Non-Goals

**Goals:**

- `sw.js` clásico testeable en Node (parse + VM con `caches`/`fetch` fake)
- Precache versionado resoluble en runtime vía `{{VERSION}}` en el Gateway
- Banner PA-04 persistente, no toast, sin tokens nuevos
- Listo cuando automatizado: navegación offline → shell cacheado + banner; `/api/*` y `/health` sin fallback (sin Playwright)

**Non-Goals:**

- Service Worker como ES Module (`type: "module"`) — sin bundler y con soporte desigual
- `skipWaiting` / `clients.claim` / borrar cachés viejas (T-01-15)
- Fake Cache Storage del navegador; el VM alcanza para las estrategias

## Decisions

### 1. Sustituir `{{VERSION}}` en `get_sw_js` (igual que el *manifest*)

`sw.js` vive en `/sw.js` (`no-cache`). El precache MUST usar `/static/{{VERSION}}/…`. `get_sw_js` lee el archivo como texto y hace `replace("{{VERSION}}", version)`. MIME `text/javascript` y `no-cache` no cambian.

Alternativas descartadas:

- **Descubrir la versión en `install` vía `/health` o parseando `index.html`** — el SW quedaría inútil si `/health` está caído justo al instalar, y complica el test.
- **Preachear solo `/` y cache-as-you-go** — no garantiza el *app shell* completo (módulos ES) en la primera apertura offline.
- **Ruta `/static/<versión>/sw.js`** — perdería `no-cache`; el browser no revalidaría el worker.

Si el fixture no trae placeholder, el replace es no-op: `sw_js_sends_no_cache` sigue verde.

### 2. `sw.js` clásico, no ES Module

Script clásico en la raíz del origen (`/sw.js`) con `self.addEventListener("install"|"fetch")`. Sin `importScripts` extra (evitaría otra ruta Gateway). El test Node evalúa el archivo en `vm` con `self`, `caches` y `fetch` inyectados.

Alternativa descartada: extraer `sw-handlers.js` importable — duplicaría el script o exigiría `importScripts` con versión.

### 3. Precache explícito del *app shell*

Nombre de caché `vectra-shell-{{VERSION}}`. Lista fija: `/`, `/index.html`, `/manifest.webmanifest`, CSS, módulos JS que la SPA carga (incluidos `overlay.js` y componentes) e íconos PNG. MUST NOT incluir `/api/`, `/health`, `/media/` ni `dev/components.html`. `install` hace `cache.addAll` de esa lista.

`fetch`:

| Request | Estrategia |
|---|---|
| `mode === "navigate"` | red primero; si falla, `caches.match` de `/` o `/index.html` |
| pathname `/health` o prefijo `/api/` | solo `fetch`; MUST NOT `caches.put` |
| pathname prefijo `/media/` | solo `fetch`; MUST NOT `caches.put` |
| resto GET | solo red (no cache-as-you-go) |

Los estáticos versionados ya están en la caché de `install`; no hace falta cache-first aparte.

MUST NOT llamar `skipWaiting` ni `clients.claim`.

### 4. Registro en `app.js`

Tras arrancar *router* y *poller*: si `navigator.serviceWorker` existe, `register("/sw.js")` (promesa ignorada / catch vacío). Si no existe, no tira. MUST NOT registrar desde `index.html` inline.

Se **invierte** `manifest_does_not_register_service_worker` → `app_registers_service_worker`.

### 5. Banner PA-04 en *router* + `health.js`

Operador y admin incluyen `[data-offline-banner]` en el encabezado (`hidden` por defecto, `aria-live="assertive"`, texto exacto «Sin conexión con el Nodo Maestro»). Tras cada snapshot (y `hashchange`), `health.js` lo muestra solo si el `GET /health` falló por red; lo oculta si hay respuesta parseable (200 o 503). `[data-health]` sigue en `unknown` / «Estado desconocido». `errors.js` no cambia. Clases con `--vx-*` existentes (p. ej. `--vx-color-error`); MUST NOT agregar tokens.

Alternativas descartadas:

- **Solo toast** — efímero; el Listo cuando pide aviso visible con el shell.
- **Reusar `[data-health]`** — mezclaría «Estado desconocido» con el copy PA-04 y rompería tests T-01-12 del indicador.
- **`window.offline`** — no distingue «sin LAN» de «Nodo Maestro caído»; `/health` es la señal correcta y el SW no la cachea.

### 6. Tests Node/VM + Rust, sin Playwright

`scripts/fixtures/vectra-web/sw.test.mjs`: parse del precache (`{{VERSION}}`, sin `/api/` ni `/media/`) y VM (`install` + `fetch` navigate/API/health). Unhappy in-memory (precache con `/api/`). `health.test.mjs` cubre el banner. Harness exige `sw.js`. Crate `api`: `sw_js_replaces_version` sobre fixture temporal.

## Risks / Trade-offs

- **[Sustitución en el Gateway]** → un `replace` extra; T-00-19 decía plantilla solo en HTML. El delta `vortex-api` lo documenta (igual que T-01-13 con el *manifest*).
- **[Listo cuando sin navegador]** → las estrategias y el banner cubren el criterio; e2e real es T-09-10.
- **[Primer load no controlado por el SW]** → esperado sin `clients.claim`; la app *instalada* abre en un load posterior.
- **[macOS: `openspec new change T-01-14`]** → scaffold `t-01-14` y rename a `T-01-14` (igual que T-01-13).

## Migration Plan

N/A (SPA local + un handler existente). Rollback = revertir el change. No hay datos que migrar. Tras el primer deploy, los clientes sin SW siguen igual hasta recargar.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
