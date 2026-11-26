# Design

## Context

En `develop`: T-03-17/T-03-23 pintan `vx-product-card` `full` y dejan `onOpen: () => {}`; la spec prohíbe abrir el panel. T-01-04: `vx-side-panel.show(opener)` / Escape / `vx-closed`. T-01-08: el Omnibox recupera el foco al `vx-closed`. T-04-07: `GET /api/v1/products/{productId}/recommendations` (`Recommendations` + `highlightSubstitutes`). Las tarjetas de resultado **no** guardan `id`. Tests Node + `dom-stub.mjs`. Ver `proposal.md` para el porqué; spec para bloques, destaque y unhappy path.

Constraints: MVP del curso; ~2 h; sin carrito; sin OpenAPI ni crates; `openspec new change` exige kebab-case (`t-04-08` → rename `T-04-08`).

## Goals / Non-Goals

**Goals:**

- Cablear apertura de resultado → panel con tres bloques compactos
- Abort + `showError` alineados con `search.js`
- Desteque de Sustitutos solo con el flag del DTO (sin re-filtrar stock)

**Non-Goals (diseño):**

- Historial / botón Atrás entre productos del panel (el ticket no lo pide)
- Estado de carga esqueleto
- Precache de `recommendations.js` / `search.js` en el SW (T-03-15 tampoco)

## Decisions

### 1. Módulo `recommendations.js`, `search.js` solo dispara

`vectra-web/js/recommendations.js` exporta algo equivalente a `mountRecommendations(root, { request, showError })` que resuelve el `vx-side-panel` de `data-region="side-panel"` y `openRecommendations(productId, opener)`. `search.js` pone `data-product-id` en cada tarjeta, pasa `onOpen` a `bindKeyboardList`, y un listener de `click` en las tarjetas.

**Por qué:** el panel tiene abort, pintado y un segundo listbox; mezclarlo en `paintResults` hincha el módulo de búsqueda. El overlay sigue sin negocio (spec de T-01-04).

**Alternativa:** todo en `search.js`. Se descartó: tests del panel quedarían acoplados al flujo `vx-query`. **Alternativa:** Web Component `vx-recs-panel`. Se descartó: el ticket pide reutilizar `vx-side-panel`.

### 2. Enter y clic; `keyboard-list` no cambia

`onOpen` cubre Enter. El clic se cablea en `search.js` / panel (el helper de T-01-10 no tiene pointer). No se extiende `keyboard-list.js`.

**Por qué:** HU «cuando lo selecciono» cubre teclado (DoD) y mouse; tocar el helper arriesga T-01-10 / sugerencias.

**Alternativa:** solo Enter. Se descartó: el operador toca la tarjeta. **Alternativa:** `onOpen` también en `pointerup` dentro de `keyboard-list`. Se descartó: fuera de este ticket.

### 3. Un listbox en el panel; títulos no son opciones

Tres headings fijos + un `role="listbox"` de todas las compactas (Similares, luego Sustitutos, luego complementos). `getItems` solo tarjetas. Mismo patrón que «Código exacto».

**Por qué:** un único T-01-10, ArrowDown recorre el grafo. Tres listboxes romperían el tab order con la trampa de foco.

**Alternativa:** tres listboxes. Se descartó. **Alternativa:** ocultar headings vacíos. Se descartó: HU pide ver los tres bloques.

### 4. Desteque = atributo en el bloque, no un token nuevo

Si `highlightSubstitutes`: el contenedor de Sustitutos lleva `data-highlight-substitutes` y un texto visible («Destacados» o equivalente). CSS con `--vx-color-accent` / `--vx-color-error` ya existentes. No filtrar `stock` en el cliente.

**Por qué:** HU-04-06 es UI; el filtro ya está en T-04-06. Contraste no-solo-color (DoD).

**Alternativa:** re-chequear `stock` del origen con otro GET. Se descartó: el DTO ya trae el flag.

### 5. Fake `request` ramifica por path

Extender el fake: `/recommendations` → `Recommendations`. Fixture HU-04-05: tres bloques con prioridades distintas. Fixture HU-04-06: `highlightSubstitutes: true` + un sustituto `stock: "12"`. El Listo cuando humano (relaciones vía `POST /relations` + `vortex serve`) no se automatiza (T-09-11). `fullHit()` de `search.test.mjs` MUST pasar a incluir `id` en los tests nuevos.

Abort: abrir producto A (promesa pendiente) y luego B; A no pinta. 404/503: `showError`, panel no `open` si era la primera apertura.

## Risks / Trade-offs

- **[Clic en shadow DOM de `vx-product-card`]** → el listener va en el host (custom element), no en `article` interno. Tests usan `card.click()` / `dispatchEvent`.
- **[Escape: listbox de resultados vs panel]** → con el panel `open` la trampa de foco de T-01-04 se queda el Escape. Aceptado.
- **[Navegación profunda sin Atrás]** → elegir un recomendado reemplaza el contenido. Aceptado (ticket).
- **[Seed T-04-11 aún no existe]** → tests con fake; verificación a mano con `POST /relations`.
- **[macOS kebab-case del CLI]** → scaffold `t-04-08` y rename.

## Migration Plan

N/A (SPA local). Rollback = revertir el change. Ningún consumidor nuevo fuera de la vista operador.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
