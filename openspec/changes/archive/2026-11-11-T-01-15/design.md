# Design

## Context

`develop` ya tiene `sw.js` clásico con precache `vectra-shell-{{VERSION}}`, `install` + `fetch` (red-primero en navigate; `/api/*`, `/health`, `/media/*` solo red) y registro en `app.js`. T-01-14 **prohíbe** `skipWaiting` / `clients.claim` y no hay `activate`. `sw.test.mjs` aserta `swOmitsUpdateHacks`. `GET /sw.js` es `no-cache` con `{{VERSION}}` sustituido. La SPA es *hash-only*; `display: standalone` no tiene chrome de recarga. El stub Node no es jsdom; hay Cache Storage fake parcial (`open` / `put` / `match`, sin `keys` / `delete`). Ver proposal.md para el porqué; spec para aviso, Recargar y Listo cuando.

## Goals / Non-Goals

**Goals:**

- `activate` testeable en VM: borra `vectra-shell-*` viejas, conserva `CACHE`
- Aviso persistente + Recargar (`location.reload` solamente) sin forzar el worker
- Detectar *waiting* en SPA (`update()` al foco) sin Playwright

**Non-Goals:**

- `skipWaiting` / `clients.claim` / `reload` automático al detectar *waiting`
- Service Worker como ES Module
- Fake Cache Storage del navegador; extender el mock de T-01-14 alcanza

## Decisions

### 1. No `skipWaiting` ni `clients.claim` (el No hace *es* la decisión)

T-01-14 aplazó esos hacks a T-01-15, pero el **No hace** del ticket es «actualizaciones forzadas sin recargar», que es exactamente `skipWaiting` + `clients.claim` (el worker nuevo toma clientes abiertos). El worker en *waiting* se activa al recargar el único cliente (MVP: una terminal). Recargar = `location.reload()`. MUST NOT `postMessage('SKIP_WAITING')`.

Alternativas descartadas:

- **`skipWaiting` en `install`** — activa sin recargar; viola el No hace.
- **`skipWaiting` vía postMessage y luego reload** — innecesario con un solo cliente; introduce el mismo API que el No hace prohíbe.
- **`clients.claim`** — la página abierta seguiría ejecutando JS viejo hasta reload, pero el fetch ya pasaría por el SW nuevo a mitad de sesión.

`swOmitsUpdateHacks` se **conserva** (y cubre también `sw-update.js`).

### 2. `activate` borra solo `vectra-shell-*` distintas de `CACHE`

```text
caches.keys() → filter name.startsWith("vectra-shell-") && name !== CACHE → caches.delete
```

No borrar otras cachés (futuro). El Listo cuando («recargar aplica los estáticos nuevos») en tests: tras `activate` de `vectra-shell-{{VERSION}}`, la caché `vectra-shell-old` desaparece y la actual permanece. En runtime, `GET /` es red-primero (T-01-14), así que un reload online ya pide `index.html` nuevo; el `activate` evita que el *fallback* offline sirva el shell viejo.

### 3. Módulo `js/sw-update.js`, no lógica inline en `app.js`

Mismo patrón que `health.js`: exportar `startSwUpdateWatcher` / `applyUpdateBanner` para tests Node. `app.js` lo arranca si hay `#app`. Hay que sumar el archivo al precache y al harness (una línea cada uno).

Detección:

- `registration.waiting` **y** `navigator.serviceWorker.controller` → aviso
- `updatefound` → `statechange` a `installed` con `controller` → aviso
- Sin `controller` (primera visita) → no mostrar «Nueva versión disponible»
- `registration.update()` en `focus` / `visibilitychange` a `visible` (la SPA no recarga documento)

Alternativa descartada: intervalo 10 s acoplado al *poller* `/health`. El foco cubre «volver a la tablet»; el Listo cuando asume un reload explícito.

### 4. Banner persistente + Recargar, no toast

Igual que PA-04: `[data-update-banner]` en el encabezado de operador y admin (`hidden` por defecto, `aria-live="polite"`, texto **Nueva versión disponible**). Control `[data-update-reload]` tipo `button` con `var(--vx-touch-min)`. Clases `--vx-*` existentes. Tras `hashchange`, reaplicar visibilidad (el *outlet* se reemplaza).

`polite` (no `assertive`): no interrumpe el aviso offline. Distinto atributo para no mezclar señales.

Alternativas descartadas:

- **Solo toast** — efímero; el operador en mostrador lo pierde.
- **Reusar `[data-offline-banner]`** — copy y semántica distintas.
- **Sin Recargar** — en *standalone* no hay F5.

### 5. Tests Node/VM, sin Playwright ni Rust nuevo

Extender `createMemoryCacheMock` con `keys()` y `delete(name)`. `loadSwHandlers` ya captura `activate`. Fake de `ServiceWorkerContainer` (controller, registration.waiting, updatefound, `update()` que rechaza). Unhappy in-memory: fuente con `skipWaiting` falla el oráculo. Sin tests nuevos en el crate `api` (`sw_js_replaces_version` ya cubre el bump de versión en el cuerpo).

## Risks / Trade-offs

- **[Varias pestañas]** → el worker *waiting* espera a que todas recarguen. MVP = una terminal; aceptable.
- **[Listo cuando sin navegador]** → `activate` + Recargar = `reload` cubren el criterio; e2e real es T-09-10.
- **[Reload online ya pide HTML nuevo]** → red-primero de T-01-14; este ticket asegura el aviso, el purge y el *standalone*.
- **[macOS: `openspec new change T-01-15`]** → scaffold `t-01-15` y rename a `T-01-15` (igual que T-01-14).

## Migration Plan

N/A (SPA local). Rollback = revertir el change. Clientes con SW de T-01-14 siguen hasta el próximo reload, que instala el worker con `activate`. No hay datos que migrar.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
