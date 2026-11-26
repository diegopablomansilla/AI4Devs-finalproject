# Design

## Context

En `develop`: T-02-35/T-02-38 dejan `data-tab="relaciones"` con `aria-disabled="true"` en alta y edición. T-02-38 habilita Stock solo en edición (`products.js` `sheetTab` ∈ `{datos, stock}`; `renderEditProductSheet` pinta Stock o Datos). T-04-05: `GET /api/v1/products/{id}/relations` devuelve `{ items: Relation[] }` plano (admin: inactivos incluidos); `PATCH`/`DELETE /api/v1/relations/{type}/{id}`. T-04-08 es el panel operador (`recommendations.js`); T-04-07 no cachea. Tests Node + `dom-stub.mjs` en `product-sheet.test.mjs`. Ver `proposal.md` para el porqué; spec para agrupación, PATCH/DELETE y unhappy path.

Constraints: MVP del curso; ~2 h; sin alta (T-04-10); sin OpenAPI ni crates; `openspec new change` exige kebab-case (`t-04-09` → rename `T-04-09`).

## Goals / Non-Goals

**Goals:**

- Habilitar la pestaña en edición, agrupar el GET plano, PATCH/DELETE en línea, abort + `showError`
- Actualizar el listado de la pestaña de inmediato tras 200/204 (HU-04-03 «de inmediato»)

**Non-Goals (diseño):**

- Sincronizar en vivo un `vx-side-panel` operador abierto (vistas distintas; sin caché basta reabrir)
- Módulo JS nuevo: el panel vive en `product-sheet.js` / `products.js` como Stock
- Confirmación de borrado, `direction=incoming`, precache SW

## Decisions

### 1. Mismo patrón que Stock: `sheetTab === "relaciones"`

Extender `sheetTab` a `"datos" | "stock" | "relaciones"`. En edición, quitar `aria-disabled` de Relaciones. `renderEditProductSheet` pinta `renderRelationsPanel(items, fieldErrors)` cuando el tab es `relaciones`. El click handler acepta `tab === "relaciones"` y dispara el GET.

**Por qué:** T-02-38 ya resolvió tabs, hash, alta deshabilitada y tests con `dom-stub`. Un tercer módulo duplicaría el ciclo de vida de la ficha.

**Alternativa:** `vectra-web/js/admin/relations.js`. Se descartó: ~2 h y el estado (producto, tab, abort) ya está en `products.js`. **Alternativa:** sección `#/admin/relaciones`. Se descartó: el ticket es la ficha (HU-04-03); el ítem de menú de T-01-11 sigue vacío.

### 2. Agrupar en cliente; ordenar por `priority`

El contrato es una lista plana. La UI particiona por `type` (`similar` / `substitute` / `complement`) y ordena cada grupo por `priority` ascendente (empate: índice en `items`). Títulos iguales al panel operador.

**Por qué:** HU-04-03 pide agrupado por tipo; el GET no agrupa. Ordenar por prioridad hace visible el mismo orden que verá el operador.

**Alternativa:** tres GET con `?type=`. Se descartó: más round-trips y abort. **Alternativa:** no reordenar. Se descartó: un `items` mezclado no garantiza orden por tipo.

### 3. Guardar por fila, no blur automático

Cada fila: `vx-field` `priority` y `note`, `data-action="save-relation"` y `data-action="delete-relation"`, más `data-relation-id` / `data-relation-type`. PATCH solo al guardar. Prioridad se parsea a entero; `< 1` o vacío → no fetch. Tras 200, reemplazar `items` por el GET de nuevo (un fetch) o fusionar `items` del 200 (el PATCH bidireccional puede devolver dos aristas; para salientes basta re-GET).

**Por qué:** blur en `vx-field` (shadow) es frágil en tests sin jsdom; un botón cumple DoD táctil y es explícito.

**Alternativa:** PATCH on blur. Se descartó: acoplamiento al custom element. **Alternativa:** un solo formulario para todas las filas. Se descartó: el ticket pide edición en línea por relación.

### 4. «De inmediato» = lista de la pestaña, no el panel operador

Tras PATCH/DELETE exitoso, el DOM de la pestaña se actualiza. MUST NOT llamar `/recommendations` desde admin. El Listo cuando humano incluye: editar/borrar en la ficha, abrir el producto en operador, ver el panel de T-04-08 al día (sin caché).

**Por qué:** HU nombra el panel de recomendaciones, pero este ticket es la ficha; T-04-08 no comparte estado. Cablear las dos vistas excede ~2 h y T-04-08.

**Alternativa:** tras PATCH, `CustomEvent` hacia el panel. Se descartó: el operador no está montado en `#/admin/productos`.

### 5. Fake `request` ramifica por `/relations`

Extender el fake de `product-sheet.test.mjs`: `GET .../products/{id}/relations` → `{ items }`; `PATCH/DELETE .../relations/{type}/{id}` → 200 `{ items }` / 204. Fixture HU-04-03: tres tipos, uno bidireccional (`bidirectionalGroup` ULID), uno con `target.active: false`. `sheet_edit_stock_tab_is_operable` deja de exigir Relaciones disabled.

Abort: activar Relaciones (promesa pendiente) y luego Datos; el GET no pinta. 404/503: `showError`, ficha abierta.

## Risks / Trade-offs

- **[PATCH bidireccional devuelve 2 ítems]** → re-GET saliente tras 200/204 evita pintar la arista inversa en este producto. Aceptado (un round-trip extra).
- **[`sheet_edit_stock_tab_is_operable` hoy aserta Relaciones disabled]** → actualizar ese test en el mismo apply; si no, la suite queda roja al habilitar el tab.
- **[Seed T-04-11 aún no existe]** → tests con fake; verificación a mano con `POST /relations`.
- **[macOS kebab-case del CLI]** → scaffold `t-04-09` y rename.

## Migration Plan

N/A (SPA local). Rollback = revertir el change. Ningún consumidor nuevo fuera de `#/admin/productos`.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
