# Design

## Context

`develop` ya tiene `GET /health` (T-00-13, esquema `Health`), `api.js` (`request` + `ApiError`) y `vx-omnibox` con selector Texto/Voz/IA y atributo `vision` (T-01-09). El *router* pinta encabezados `vx-operator-header` / `vx-admin-header` sin indicador. `request()` trata todo no-2xx sin `code` como `ApiError` `internal`; `/health` en `starting`/`down` es **503 con el mismo JSON Health**, no `problem+json`. El stub Node no es jsdom: tests de *poller* inyectan `fetch` / `setInterval` / `dispatchEvent`. Ver proposal.md para el porqué; spec para mapeo de capacidades y Listo cuando.

## Goals / Non-Goals

**Goals:**

- *Poller* testeable en Node: `capabilitiesFromHealth` puro + `startHealth` con deps inyectadas
- Listo cuando automatizado: snapshot `llm: down` → IA `disabled`, Texto operable, indicador degradado
- Reaplicar UI tras `hashchange` (el *outlet* se reemplaza)

**Non-Goals:**

- Extender `request()` para 503 Health (el *poller* lee `/health` con `fetch` inyectado)
- Extraer un Web Component `vx-health-indicator`
- Fake timers reales de 10 s en CI (se aserta el argumento `10000` de `setInterval`)

## Decisions

### 1. `fetch` inyectado en `health.js`, no `request()`

`createApi().request` rechaza 503 sin `code` → `internal` y se pierde el snapshot `starting`/`down`. Alternativa descartada: un flag `acceptStatuses` en `api.js` — el ticket no pide cambiar el cliente y T-01-06 ya cerró su contrato.

`health.js` exporta:

```js
export const HEALTH_POLL_MS = 10000;

export function capabilitiesFromHealth(body) → { status, capabilities }
export function startHealth({ fetch, window, document, intervalMs, setInterval, clearInterval })
```

`fetchHealth(fetchFn)` hace `GET /health`. Si HTTP es 200 o 503 y el JSON tiene `components`, se mapea. Red, abort, parse error o JSON incompleto → snapshot `unknown` (spec). No se importa `showError`.

`app.js` llama `startHealth({ fetch: globalThis.fetch, window, document })` **después** de `startRouter`, para que el listener `hashchange` del *poller* corra después del *paint* del *router*.

Alternativa descartada: arrancar el *poller* desde `startRouter` — rompería `router_admin_does_not_call_api` (spy de `fetch` al pintar admin).

### 2. Mapeo puro `capabilitiesFromHealth`

| Campo | Regla |
|---|---|
| `ai` | `body.components?.llm?.status === "up"` |
| `voice` | `body.components?.speech?.status === "up"` |
| `vision` | `body.components?.vision?.status === "up"` (si `vision` falta → `false`) |
| `textSearch` | siempre `true` en este ticket |
| `status` | `body.status` si es `ok\|degraded\|starting\|down`; si no, `"unknown"` |

`disabled` de `llm`/`speech` no es `up` → modos off (ticket: «si no está `up`»). `meilisearch: down` no apaga texto ni pinta "Búsqueda en modo reducido".

Evento `vx-health` en `document` con `{ status, capabilities }` para T-06-10 / T-07-03 sin acoplar al DOM.

### 3. Atributos `ai-unavailable` / `voice-unavailable` en el host

Ausentes = T-01-09 intacto (`?hola` → `ai`). Presentes = botones nativos `disabled` + `aria-disabled`, prefijo `?` no gana, fallback a `text`. El *poller* setea/quita atributos en `outlet.querySelector("vx-omnibox")` (si no hay Omnibox, solo actualiza `[data-health]`).

`vision` se setea o se quita como hoy (T-01-09): en MVP el JSON no trae `vision` → se quita.

No `vx-chip`/`vx-button` anidados (mismo motivo que T-01-09: stub).

### 4. Placeholder `[data-health]` en ambos encabezados

`viewHtml` agrega, p. ej. `<p data-health aria-live="polite"></p>` en `vx-operator-header` y `vx-admin-header`. El *poller* escribe texto en español:

| `status` | Texto |
|---|---|
| `ok` | Sistema operativo |
| `degraded` | Sistema degradado |
| `starting` | Arrancando |
| `down` | Sistema no disponible |
| `unknown` | Estado desconocido |

`base.css`: clase mínima (muted + `--vx-touch-min` no aplica; no es control). Sin *tokens* nuevos. `not-found` no lleva indicador.

Tras `hashchange`, `applyHealthUi(document, lastSnapshot)` rellena el nodo nuevo. Spy de `fetch` en tests de *router* sin `startHealth` sigue en 0 llamadas.

### 5. Tests: `health.test.mjs` + omnibox; intervalo mockeado

`scripts/fixtures/vectra-web/health.test.mjs`: fake `fetch` que devuelve `Response` JSON; fake `window`/`document` con `addEventListener` y un `[data-health]` + `vx-omnibox` mínimo (o se importa el componente real como en `omnibox.test.mjs`). `setInterval` inyectado: el test `health_polls_every_10s` aserta `(fn, 10000)` y ejecuta `fn` a mano para el refetch. `focus` / `visibilitychange` se despachan sobre el fake.

`omnibox.test.mjs`: tres tests de atributos sin *poller*. `check-vectra-web.sh` agrega `health.js` al árbol y `health.test.mjs` a `node --test`.

## Risks / Trade-offs

- **[503 vs `request()`]** → decisión 1; un futuro `getHealth` en `api.js` podría unificar, fuera de este ticket.
- **[carrera hashchange]** → registrar el *poller* después del *router* en `app.js`; si un test arranca solo el *router*, no hay fetch.
- **[`?` vs `ai-unavailable`]** → la spec nueva es más específica; sin el atributo T-01-09 no cambia.
- **[macOS: `openspec new change T-01-12`]** → scaffold `t-01-12` y rename a `T-01-12` (igual que T-01-11).

## Migration Plan

N/A (SPA local). Rollback = revertir el change. No hay datos que migrar.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
