# Spec Delta

Ticket padre: T-09-11. Change: T-09-11. HU: HU-09-03 (panel de recomendaciones y alta de producto; búsqueda ya en T-09-10).

## ADDED Requirements

### Requirement: e2e H1 — panel de recomendaciones con tres bloques

La suite MUST incluir un spec Playwright cuyo nombre estable identifique el panel de recomendaciones (`recs_open_panel_shows_three_blocks` o equivalente). MUST reutilizar la fixture de T-09-09 (`vortex serve` + dataset `seed/` + Meilisearch) y el wait de índice de T-09-10. MUST abrir la vista operador, buscar un producto semilla con grafo (`TOR-AUTO-10X1`, `PER-M8X40-ZN` o `PER-ACERO-01` según el escenario) y **abrir** el resultado (clic en la `vx-product-card` o Enter sobre el hit activo). MUST NOT marcar el spec skipped.

Tras abrir, el `vx-side-panel` de `data-region="side-panel"` MUST quedar abierto. MUST mostrar los títulos «Similares», «Sustitutos» y «Se usa junto con» en ese orden. Las tarjetas del panel MUST ser `vx-product-card` `variant="compact"` (MUST NOT `full`). MUST NOT haber control «Agregar» ni carrito (T-08-06).

Sobre el grafo de `seed/relaciones.jsonl` (T-04-11), el spec MUST cubrir al menos:

1. `TOR-AUTO-10X1` → bloque Similares con `sku="TOR-4-16-AC-001"`
2. `PER-M8X40-ZN` → bloque «Se usa junto con» con `sku="ARA-PERNO-01"`
3. `PER-ACERO-01` → bloque Sustitutos con `sku="PER-ACERO-02"` (stock decimal equivalente a 12)

MUST NOT exigir un único producto con los tres tipos de arista (el seed no lo tiene). MUST NOT crear relaciones desde la UI (T-04-10). MUST NOT alterar `seed/`.

#### Scenario: abrir un resultado muestra los tres bloques (happy, HU-04-05, Listo cuando)

- **WHEN** la fixture dejó `vortex serve` escuchando con el dataset de desarrollo
- **AND** el operador busca `"tornilo"` y abre la tarjeta `TOR-AUTO-10X1`
- **THEN** el `vx-side-panel` de `data-region="side-panel"` está abierto
- **AND** se ven los títulos «Similares», «Sustitutos» y «Se usa junto con»
- **AND** hay un `vx-product-card` `compact` con `sku="TOR-4-16-AC-001"`
- **AND** no hay control «Agregar»
- **AND** el spec no queda skipped

#### Scenario: complementario del perno M8 (happy)

- **WHEN** el operador busca `PER-M8X40-ZN` y abre esa tarjeta
- **THEN** el panel abierto muestra `vx-product-card` `compact` con `sku="ARA-PERNO-01"`

#### Scenario: sustituto del perno de acero con stock (happy)

- **WHEN** el operador busca `PER-ACERO-01` y abre esa tarjeta (stock semilla 40, sin ajuste)
- **THEN** el panel abierto muestra `vx-product-card` `compact` con `sku="PER-ACERO-02"`
- **AND** el stock visible de esa tarjeta es un decimal equivalente a 12

#### Scenario: clic en un recomendado abre ese producto (happy)

- **WHEN** el panel de `TOR-AUTO-10X1` está abierto con la tarjeta `TOR-4-16-AC-001`
- **AND** el operador la elige (clic o Enter)
- **THEN** el panel permanece abierto
- **AND** se siguen viendo los tres títulos (ahora del producto elegido)

#### Scenario: al cerrar el foco vuelve al Omnibox (happy)

- **WHEN** el panel de recomendaciones está abierto
- **AND** el operador lo cierra (Escape o control de cierre)
- **THEN** el input del Omnibox recupera el foco

#### Scenario: el panel no abre si no hay hit (unhappy)

- **WHEN** corre el spec de abrir panel y no aparece la tarjeta del SKU pedido a tiempo
- **THEN** el spec termina en fallo (timeout o aserción), no skipped

### Requirement: e2e H1 — sustitutos destacados sin stock

El spec de recomendaciones MUST cubrir HU-04-06 sobre el stack real. `seed/` MUST NOT ganarle un SKU con `stock_inicial` `"0"`. Para el camino con stock 0, el spec MAY registrar un movimiento `adjustment` de cantidad `"-40"` (stock semilla de `PER-ACERO-01`) vía `POST /api/v1/products/{productId}/stock-movements` contra el `baseURL` de la fixture, con `reason` no vacío; MUST NOT usar la pestaña Stock de la ficha como escenario propio (T-02-38). Ese ajuste MUST ejecutarse **después** del escenario «producto con stock no destaca». MUST NOT marcar skipped.

Con `highlightSubstitutes` verdadero, el bloque Sustitutos MUST quedar destacado de forma no solo-color: atributo `data-highlight-substitutes` en el encabezado y texto que incluye «Destacados». MUST mostrar `PER-ACERO-02` y MUST NOT mostrar un sustituto con stock 0 (el seed solo tiene uno, con stock 12).

Con stock > 0, el encabezado «Sustitutos» MUST NOT tener `data-highlight-substitutes` ni el texto «Destacados».

#### Scenario: producto sin stock destaca sustitutos (happy, HU-04-06, Listo cuando)

- **WHEN** el stock de `PER-ACERO-01` en la fixture quedó en 0
- **AND** el operador busca `PER-ACERO-01` y abre esa tarjeta
- **THEN** el encabezado de Sustitutos tiene `data-highlight-substitutes`
- **AND** el texto del encabezado incluye «Destacados»
- **AND** hay `vx-product-card` `compact` con `sku="PER-ACERO-02"`
- **AND** no hay tarjeta compacta con stock decimal 0 en el panel

#### Scenario: producto con stock no destaca sustitutos (happy, HU-04-06)

- **WHEN** `PER-ACERO-01` aún tiene el stock semilla (> 0)
- **AND** el operador abre esa tarjeta
- **THEN** el bloque «Sustitutos» se muestra
- **AND** el encabezado no tiene `data-highlight-substitutes`
- **AND** el texto del encabezado no incluye «Destacados»

#### Scenario: el ajuste a stock 0 no se saltea (unhappy)

- **WHEN** corre el spec de sustitutos destacados y el stock de `PER-ACERO-01` no queda en 0 a tiempo (API de movimiento o proyección)
- **THEN** el spec termina en fallo (no skipped)

### Requirement: e2e H1 — panel sin aristas (unhappy)

La suite MUST abrir el panel de un SKU semilla **sin** filas en `seed/relaciones.jsonl` (p. ej. `TOR-4-16-ZI-002`). MUST mostrar los tres títulos. MUST NOT pintar `vx-product-card` compactas **dentro del panel** (las tarjetas de resultados del Omnibox no cuentan). MUST NOT mockear 404/503 (T-09-12).

#### Scenario: producto sin relaciones muestra títulos vacíos (unhappy)

- **WHEN** el operador busca `TOR-4-16-ZI-002` y abre esa tarjeta
- **THEN** el panel está abierto
- **AND** se ven «Similares», «Sustitutos» y «Se usa junto con»
- **AND** el panel no contiene `vx-product-card`

### Requirement: e2e H1 — alta de producto visible en la búsqueda

La suite MUST incluir un spec Playwright cuyo nombre estable identifique el alta (`product_create_appears_in_search` o equivalente). MUST reutilizar la misma fixture. MUST abrir `#/admin/productos`, pulsar «Nuevo producto» y completar la pestaña Datos con un SKU **que no exista** en `seed/` (p. ej. `E2E-ALTA-001`), nombre distinguible (p. ej. «Producto e2e alta visible»), tipo, categoría (primera opción activa del select) y precio. MUST pulsar Guardar. MUST NOT rellenar atributos ni códigos más allá de lo obligatorio para que el POST 201 (T-02-36 no es el Hace). MUST NOT marcar skipped.

Tras el alta, el listado admin MUST mostrar una fila cuyo SKU es el creado. MUST volver a `#/`, esperar a que el índice sea consultable para ese SKU (`GET /api/v1/search` hasta el hit; `GET /health` 200 no alcanza) y escribir el SKU o el nombre en el Omnibox. MUST aparecer un `vx-product-card` con ese `sku`.

#### Scenario: el producto creado aparece en la búsqueda (happy, HU-02-04, Listo cuando)

- **WHEN** el administrador crea `E2E-ALTA-001` desde `#/admin/productos`
- **THEN** el listado muestra una celda con ese SKU
- **AND** tras esperar el índice, el operador busca `E2E-ALTA-001` en el Omnibox
- **AND** aparece un `vx-product-card` con `sku="E2E-ALTA-001"`
- **AND** el spec no queda skipped

#### Scenario: el índice del alta no se saltea (unhappy)

- **WHEN** corre el spec de alta y el índice no devuelve `E2E-ALTA-001` a tiempo
- **THEN** el spec termina en fallo (timeout o aserción), no skipped

### Requirement: e2e H1 — caminos no felices del alta

La suite MUST cubrir estos unhappy paths en el spec de alta (nombres estables). MUST NOT implementar UI nueva para hacerlos pasar.

1. Formulario incompleto: en «Nuevo producto», Guardar sin SKU/nombre/tipo/categoría/precio. MUST permanecer en la ficha (título «Nuevo producto» o `data-form="create-product"`). MUST NOT agregar una fila `E2E-ALTA-001` (ni otro SKU de este spec) al listado.
2. SKU duplicado: completar la ficha con `sku="TOR-AUTO-10X1"` (ya en el seed) y el resto válido. MUST NOT crear un segundo producto con ese SKU. MUST mostrar error observable (`vx-field[name="sku"]` con `error`, y/o toast). Tras volver a `#/`, la búsqueda `"tornilo"` MUST seguir mostrando **una** tarjeta `TOR-AUTO-10X1` (no dos).

#### Scenario: ficha incompleta no crea el producto (unhappy)

- **WHEN** el administrador abre «Nuevo producto» y pulsa Guardar sin completar los campos obligatorios
- **THEN** sigue visible el formulario de alta
- **AND** no hay fila de listado con SKU `E2E-ALTA-001`

#### Scenario: SKU duplicado no crea el producto (unhappy)

- **WHEN** el administrador intenta crear un producto con `sku="TOR-AUTO-10X1"`
- **THEN** no vuelve al listado como si el alta hubiera sido 201, o el error de SKU es visible
- **AND** en el operador, buscar `"tornilo"` muestra una sola `vx-product-card` con ese SKU

### Requirement: Tests previstos de e2e recomendaciones y alta H1

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. Los specs de T-09-09 y T-09-10 MUST seguir pasando.

| Test | Ubicación | Camino |
|---|---|---|
| `recs_open_panel_shows_three_blocks` | `e2e/tests/recommendations.spec.js` | happy (Listo cuando, HU-04-05) |
| `recs_perno_m8_shows_complement` | mismo spec | happy |
| `recs_perno_acero_shows_substitute` | mismo spec | happy |
| `recs_opening_recommended_product_keeps_panel` | mismo spec | happy |
| `recs_close_returns_focus_to_omnibox` | mismo spec | happy |
| `recs_substitutes_not_highlighted_when_in_stock` | mismo spec | happy (HU-04-06; **antes** del ajuste a 0) |
| `recs_highlights_substitutes_when_stock_zero` | mismo spec | happy (Listo cuando, HU-04-06) |
| `recs_product_without_relations_shows_empty_blocks` | mismo spec | unhappy |
| `product_create_appears_in_search` | `e2e/tests/product-create.spec.js` | happy (Listo cuando, HU-02-04) |
| `product_create_incomplete_stays_on_sheet` | mismo spec | unhappy |
| `product_create_duplicate_sku_does_not_create` | mismo spec | unhappy |
| `search_typo_tornilo_finds_tornillo_autoperforante` | `e2e/tests/search.spec.js` | happy (regresión T-09-10) |
| `smoke_app_loads_omnibox_focused` | `e2e/tests/smoke.spec.js` | happy (regresión T-09-09) |

`npx playwright test` en `e2e/` MUST ejecutar el Listo cuando (panel + highlight + alta en búsqueda) y los unhappy de esta tabla. El job CI `e2e` existente MUST seguir invocando esa suite; este change MUST NOT exigir un job nuevo ni cambios a `scripts/check-ci.sh`. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) § 5). `./scripts/check-vectra-web.sh` MUST seguir en verde.

#### Scenario: suite de recomendaciones y alta H1 cubre happy y unhappy (happy)

- **WHEN** se ejecuta `npx playwright test` en `e2e/`
- **THEN** pasan `recs_open_panel_shows_three_blocks`, `recs_highlights_substitutes_when_stock_zero` y `product_create_appears_in_search`
- **AND** pasan los unhappy de panel vacío, ficha incompleta y SKU duplicado
- **AND** el smoke de T-09-09 y la búsqueda de T-09-10 siguen pasando
