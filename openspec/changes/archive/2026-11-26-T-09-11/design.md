# Design

## Context

En `develop`: T-09-10 dejó `e2e/tests/search.spec.js` + `e2e/search-ready.js` (poll de `"tornilo"` → `TOR-AUTO-10X1`) sobre el harness de T-09-09 (`workers: 1`, un stack por suite). T-04-08 pinta tres bloques en `vx-side-panel`; destaque = `data-highlight-substitutes` + texto «Sustitutos — Destacados» cuando `highlightSubstitutes === true`. T-04-11 sembró tres aristas: `TOR-AUTO-10X1`→similar `TOR-4-16-AC-001`, `PER-M8X40-ZN`→complementa `ARA-PERNO-01`, `PER-ACERO-01`→sustituible_por `PER-ACERO-02` (stock 12). `seed/productos.jsonl` no tiene `stock_inicial` `"0"`. El alta vive en `#/admin/productos` (`data-action="create"`, `vx-field` + `POST /api/v1/products`). El Index Sync Worker indexa el alta **después** del 201. Ver `proposal.md` para motivación.

Constraints: no cambiar el job `e2e` ni `check-ci.sh`; no tocar `vectra-web/` salvo bug que bloquee el Listo cuando; no alterar `seed/`; no mockear HTTP.

## Goals / Non-Goals

**Goals:**

- Dos specs Playwright de negocio sobre la fixture existente
- Highlight de sustitutos sin mutar `seed/`
- Esperar el índice del SKU nuevo sin marcar skipped
- Cubrir unhappy observables en UI (panel vacío, ficha incompleta, SKU duplicado)
- Dejar smoke, search y YAML de CI intactos

**Non-Goals (diseño):**

- Stub de `/recommendations` o `/products` (estos e2e hablan con el stack real)
- Firefox / WebKit / `fullyParallel`
- Grep nuevo en `check-ci.sh` para nombres de tests
- Unificar recomendaciones y alta en un solo archivo (T-09-10 ya separó search)
- Automatizar el formulario de stock de T-02-38

## Decisions

### 1. Specs nuevos; search y smoke intocados

**Elección:** `e2e/tests/recommendations.spec.js` y `e2e/tests/product-create.spec.js`. `search.spec.js` sigue **sin** Enter/clic que abran el panel. `smoke.spec.js` sigue sin tipear.

**Por qué:** T-09-10 lo dejó explícito; mezclar mutaciones de stock/alta con aserciones de búsqueda alarga el archivo y el orden serial importa (decisión 3).

**Alternativa:** un `h1.spec.js` único. Se descartó: el ajuste a 0 de `PER-ACERO-01` ensuciaría lecturas de stock si se intercalan tests de búsqueda de ese SKU (hoy search no lo usa, pero el archivo crece).

### 2. Abrir el panel por clic en la tarjeta; Enter como camino extra barato

**Elección:** helper `openProductFromSearch(page, sku, query)` hace fill de la query, espera `vx-product-card[sku=…]` y `click()`. Un test MAY usar ArrowDown + Enter (mismo `onOpen` de T-04-08). Aserciones sobre `vx-side-panel[open]` (o atributo `open`) y headings `.vx-recs-heading` / texto.

**Por qué:** clic no depende del `aria-activedescendant` ya cubierto en T-09-10. El panel usa shadow abierto; Playwright perfora.

**Alternativa:** solo teclado. Se deja clic como camino principal (menos flake) y un test de «recomendado» también por clic.

### 3. Stock 0 vía `POST .../stock-movements`, no UI ni seed; orden serial

**Elección:** helper `zeroProductStock(baseURL, sku)`: `GET /api/v1/search?q={sku}` (o hits ya conocidos) → `id`; `POST /api/v1/products/{id}/stock-movements` con `{ type: "adjustment", quantity: "-40", reason: "e2e T-09-11" }`; reintenta hasta que un GET de recomendaciones o de ficha reporte stock 0. `recs_substitutes_not_highlighted_when_in_stock` MUST declararse **antes** que `recs_highlights_substitutes_when_stock_zero` en el archivo. `fullyParallel: false` y `workers: 1` ya lo garantiza el config.

**Por qué:** HU-04-06 exige el highlight; el seed no tiene stock 0 (T-09-10 no lo agregó). Mutar `seed/` está en Non-goals. La pestaña Stock es T-02-38. El POST ya está en OpenAPI y lo usa la UI; como *setup* del e2e es el análogo al wait de índice.

**Alternativa:** crear un producto con `initialStock: "0"` y una relación. Se descartó: alta de relaciones es T-04-10 (v2). **Alternativa:** UI de ajuste. Se descartó: duplica T-02-38 y añade flake de pestañas.

### 4. `vx-field`: fill + `change` (no solo `input`)

**Elección:** `page.locator('vx-field[name="sku"]').locator("input").fill(value)` y disparar `change` (Tab, `blur`, o `dispatchEvent`) para que el host copie `value` al atributo. `readProductCreate` lee `getAttribute("value")`, no el value vivo del input. Selects nativos (`categoryId`) con `selectOption` por índice 1 (saltar «Elegir categoría»).

**Por qué:** `vx-field` solo sincroniza en el listener `change`. Un `fill()` que no dispare `change` deja el POST con body `null` y el test de alta «pasa» el unhappy de incompleto por accidente.

**Alternativa:** `page.evaluate` seteando el atributo. Se descartó: no ejercita el teclado/operador.

### 5. Wait de índice del SKU nuevo, no solo `/health`

**Elección:** generalizar o extender `search-ready.js` con `waitForSkuInIndex(baseURL, sku)` (`GET /api/v1/search?q=sku` hasta `hits[].sku === sku`, timeout ~90 s, throw no skip). El alta lo llama **después** del 201 visible en el listado y **antes** de tipear en el Omnibox.

**Por qué:** el outbox de Meili no es síncrono con el POST. T-09-10 ya fijó el patrón.

**Alternativa:** `outboxLag === 0`. Se descarta como único criterio (igual que T-09-10).

### 6. Unhappy de alta observables, sin mock

**Elección:** incompleto = Guardar con ficha vacía → sigue `data-form="create-product"` / título «Nuevo producto» (el handler retorna si `readProductCreate` es `null`; puede no pintar `error` por campo). Duplicado = SKU `TOR-AUTO-10X1` → `applyFieldErrors` + toast; asertar `vx-field[name="sku"][error]` o toast y **una** tarjeta en búsqueda.

**Por qué:** no hay 404/503 de recomendaciones sin matar sidecars (T-09-12). El panel vacío (`TOR-4-16-ZI-002`) cubre el unhappy de grafo sin mock.

**Alternativa:** interceptar `page.route`. Se descartó: T-09-10 no mockea; el contrato es el stack real.

### 7. Sin cambios de CI YAML

**Elección:** el job `e2e` ya corre `npx playwright test` sobre `e2e/tests/`. Timeout 30 min alcanza dos specs extra en serie.

**Por qué:** Listo cuando = «pasa en CI», no «nuevo job».

## Risks / Trade-offs

- **[Orden de tests y stock 0]** → highlight **después** de «no destaca»; no re-afirmar stock 40 de `PER-ACERO-01` en otros specs. Search usa `TOR-AUTO-10X1`.
- **[quantity "-40" si el stock ya no es 40]** → el helper MUST leer stock actual (hit o GET producto) y ajustar `-stock` si no es 40; fallar si el POST no deja 0.
- **[`vx-field` sin `change`]** → helper `fillVxField` que hace fill + blur; si el alta incompleta «pasa» el happy, es este bug.
- **[Select de categoría vacío si GET categorías falla]** → el spec falla (no skip); la fixture ya tiene seed de categorías.
- **[Panel `open` como atributo vs propiedad]** → asertar `vx-side-panel[open]` o `.toHaveJSProperty("open", true)` según lo que pinte T-01-04.
- **[Alcance ~2 h]** → dos archivos + helpers. No se divide.

## Migration Plan

1. Red: agregar los dos specs con los nombres de la tabla; `npx playwright test recommendations.spec.js product-create.spec.js` falla.
2. Green: helpers (abrir panel, zero stock, fill `vx-field`, wait SKU) + aserciones.
3. Regresión: `npx playwright test` (smoke + axe + search + recs + alta); § 5 en `vortex-core/`; `./scripts/check-vectra-web.sh`; `./scripts/check-ci.sh`.
4. Rollback: borrar los dos specs y helpers nuevos; el job `e2e` vuelve a T-09-10.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El ajuste a 0 es API de fixture; el formulario de stock queda fuera.
