# Design

## Context

En `develop`, T-03-13 expone `GET /api/v1/search` (timeout 2 s, Drop al abortar el cliente). T-01-06 reenvía `AbortSignal` en `request`. T-01-09: `vx-omnibox` emite `vx-query` `{ mode, text }` y **no** llama a `/search`. T-01-10: `bindKeyboardList` + región `data-region="results"` vacía. `startRouter` ya inyecta `request` / `showError` para mounts de admin; la vista operador no monta búsqueda. Tests Node usan `dom-stub.mjs` (sin jsdom). Ver `proposal.md` para el porqué; spec para abort, lista mínima y unhappy path.

Constraints: MVP del curso; ~2 h; sin debounce; sin facetas ni tarjetas completas; `openspec new change` exige kebab-case (`t-03-15` → rename `T-03-15`).

## Goals / Non-Goals

**Goals:**

- Cliente search-as-you-type testeable en Node: abort + pintar solo la última respuesta
- Omnibox sigue sin conocer HTTP
- Lista mínima (nombre + SKU) lista para T-03-16 / T-03-17

**Non-Goals (diseño):**

- Sanitizar `<mark>` (T-03-16)
- `vx-product-card` compact/full en resultados (T-03-17)
- Combobox / `aria-expanded` en el input (T-03-21)
- Medir p95 < 100 ms (HU-03-03 Latencia; T-09-08 / T-03-14 cubren motor)

## Decisions

### 1. `mountSearch` en `search.js`, no en `vx-omnibox`

```js
export function mountSearch(root, { request, showError })
```

`root` es el *outlet* o el `main` operador. Escucha `vx-query` (bubbles + composed). Devuelve `unmount()` que aborta, `unbind()` del listbox y quita el listener. `startRouter` lo llama al pintar operador (espejo de `mountProducts`); `app.js` importa y pasa `mountSearch`.

**Por qué:** T-01-09 dejó el evento como único contrato. Meter `fetch` en el custom element rompería tests de omnibox y el No hace de modos.

**Alternativa:** escuchar `input` nativo en el shadow. Se descartó: duplica la lógica de modo (`?` → IA) que ya resuelve `_effectiveMode`. **Alternativa:** `document` global en `app.js`. Se descartó: no se limpia al ir a admin.

### 2. Un `AbortController` por consulta; sin debounce

En cada `vx-query` texto no vacío: `previous.abort()`, `controller = new AbortController()`, `request("GET", path, { signal: controller.signal })`. Path: `/api/v1/search?q=` + `encodeURIComponent(q.slice(0, 200))`.

AbortError (`err.name === "AbortError"` o `signal.aborted`): return. Cualquier otro error: `showError(err)` (T-01-07 ya ignora AbortError; el módulo no debe llamarlo igual).

**Por qué:** el ticket y [5.7](../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación) piden cancelar en cada pulsación. Debounce ocultaría las cancelaciones del Listo cuando.

**Alternativa:** debounce 50–100 ms. Se descartó: el Hace dice «en cada input». **Alternativa:** generation counter sin abort. Se descartó: dejaría trabajo en Axum; T-03-13 ya prueba el Drop.

### 3. Lista de texto + `bindKeyboardList`; no tarjetas

Tras 200, vaciar la región, crear un contenedor `role="listbox"`, un nodo por hit con `textContent = name + " · " + sku` (o dos nodos de texto). `unbind()` previo si existía. `bindKeyboardList({ list, getItems, controllers: [omniboxInput], restoreFocus: omnibox, onOpen: () => {} })`.

Pintar con `textContent` hace que `highlight` con `<img onerror>` no se ejecute (cubre el unhappy de HTML hasta T-03-16).

**Por qué:** No hace = tarjeta completa. T-01-10 existe para este listbox. Enter/onOpen vacío: no hay ficha operador (T-04-08).

**Alternativa:** `vx-product-card variant="compact"` con name/sku. Se descartó: T-03-17 redefine la tarjeta; compact ya muestra campos de catálogo. **Alternativa:** `innerHTML` de `highlight.name`. Se descartó: T-03-16.

### 4. Consulta vacía y modos no-texto

Texto recortado `""`: abort + `results.innerHTML = ""` (o borrar hijos) + unbind; sin `request`. Modo ≠ `text`: abort; no fetch; no vaciar la lista (el operador puede volver a Texto y seguir viendo el último resultado). Al `unmount` (hashchange a admin): abort + unbind.

**Por qué:** Escape (T-01-08) emite `vx-query` con texto vacío; dejar hits viejos contradice «solo la última consulta». Modo IA no debe disparar search ni borrar lo que el operador acaba de ver si solo pulsó IA por error.

**Alternativa:** vaciar también al cambiar de modo. Se descartó: extra y no está en el ticket.

### 5. Tests Node: `request` falso que captura `signal`

`search.test.mjs` instala `dom-stub.mjs`, pinta un operador mínimo (`vx-omnibox` + `data-region="results"`) o llama `mountSearch` + `startRouter`. El fake de `request` guarda `{ path, signal }` y resuelve un `SearchResponse` mínimo (`query`, `hits`, `estimatedTotalHits`, `offset`, `limit`, `processingTimeMs`, `degraded`). Para abort: primera llamada con Promise pendiente; segunda aborta el `signal` de la primera y resuelve la última.

Harness: `[ -f vectra-web/js/search.js ]` y `node --test … search.test.mjs`. Extender el stub **solo** si falta `textContent` / `encodeURIComponent` (ya hay `CustomEvent`).

El Listo cuando humano (pestaña de red) no se automatiza aquí (T-09-10).

## Risks / Trade-offs

- **[Respuesta abortada que igual llega]** → no pintar si `signal.aborted` antes de mutar el DOM; test `search_aborted_response_does_not_paint`.
- **[Escape limpia Omnibox y dispara vx-query]** → rama consulta vacía; `omnibox_escape_clears_query` sigue verde.
- **[startRouter innerHTML destruye nodos]** → `unmount` en cada `paint`; abort al salir.
- **[macOS: kebab-case del CLI]** → scaffold `t-03-15` y rename (igual que T-03-14).
- **[Lista pobre vs HU-03-05]** → aceptado; T-03-16/17 la sustituyen. Nombre+SKU alcanza para verificar search-as-you-type.

## Migration Plan

N/A (SPA local). Rollback = revertir el change. Ningún consumidor de `mountSearch` fuera del router.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
