# Spec Delta

Ticket padre: T-09-10. Change: T-09-10. HU: HU-09-03 (flujo H1 de búsqueda; sin facetas, recableo § 7.5.1).

## ADDED Requirements

### Requirement: e2e H1 — búsqueda con error tipográfico

La suite MUST incluir un spec Playwright cuyo nombre estable identifique la búsqueda con typo (`search_typo_tornilo_finds_tornillo_autoperforante` o equivalente). MUST reutilizar la fixture de T-09-09 (`vortex serve` + dataset `seed/` + Meilisearch). MUST abrir la vista operador y escribir `"tornilo"` en el campo de búsqueda del Omnibox (modo texto). MUST esperar a que el índice de productos esté consultable: `GET /health` 200 **no** alcanza si el outbox de Meilisearch aún no indexó el seed; el spec MUST reintentar hasta ver el hit o fallar por timeout. MUST NOT marcar el spec skipped.

Tras la espera, `data-region="results"` MUST contener un `vx-product-card` del producto semilla cuyo `nombre` es `"Tornillo autoperforante 10 x 1"` (SKU `TOR-AUTO-10X1`). MUST NOT exigir coincidencia exacta del string en el atributo `name` si el nombre va resaltado con `<mark>`: basta con que la tarjeta sea identificable por `sku="TOR-AUTO-10X1"` y el texto visible incluya «Tornillo autoperforante». MUST NOT abrir el panel de recomendaciones ni navegar a administración (T-09-11). MUST NOT enviar `facets`, `categoryId`, `brandId`, `inStock` ni `attr` desde la UI (recableo: sin T-03-19).

#### Scenario: tornilo encuentra Tornillo autoperforante (happy, HU-03-03, Listo cuando)

- **WHEN** la fixture dejó `vortex serve` escuchando con el dataset de desarrollo
- **AND** el navegador abre la vista del operador
- **AND** el operador escribe `"tornilo"` en el Omnibox
- **THEN** aparece un `vx-product-card` con `sku="TOR-AUTO-10X1"`
- **AND** el texto visible de esa tarjeta incluye «Tornillo autoperforante»
- **AND** no se abre `vx-side-panel` de recomendaciones
- **AND** el spec no queda skipped

#### Scenario: el índice aún no listo no se saltea (unhappy)

- **WHEN** corre el spec de typo y el índice no devuelve el tornillo a tiempo
- **THEN** el spec termina en fallo (timeout o aserción), no skipped

### Requirement: e2e H1 — tarjetas de resultado con stock

El spec de búsqueda (el de typo u otro en el mismo archivo) MUST asertar que el `vx-product-card` de `TOR-AUTO-10X1` es variante `full` y expone stock observable: atributo `stock` con valor decimal equivalente a `"40"` (`"40"`, `"40.0"` o `"40.00"`) **y** el valor visible en la tarjeta. MUST NOT exigir el estado «Sin stock» (`data-out-of-stock`): el dataset de desarrollo no incluye `stock_inicial` `"0"` y este ticket MUST NOT alterar `seed/`. MUST NOT exigir foto principal (`imageUrl` suele ser `null` en el MVP).

#### Scenario: la tarjeta del tornillo muestra stock 40 (happy, HU-03-05)

- **WHEN** la búsqueda `"tornilo"` ya pintó el `vx-product-card` `TOR-AUTO-10X1`
- **THEN** esa tarjeta tiene `variant="full"` o el default full
- **AND** el atributo `stock` es un decimal equivalente a 40
- **AND** el stock es visible en la tarjeta (no solo un dato interno)

#### Scenario: stock ausente o no visible hace fallar el spec (unhappy)

- **WHEN** la tarjeta de `TOR-AUTO-10X1` no expone `stock` equivalente a 40 o el stock no es visible
- **AND** corre el spec de tarjetas con stock
- **THEN** el spec termina en fallo (no skipped)

### Requirement: e2e H1 — navegación por teclado de resultados

Tras resultados visibles, con el foco en el input del Omnibox, el spec MUST enviar `ArrowDown`. MUST asertar que el `role="listbox"` de `data-region="results"` tiene `aria-activedescendant` igual al `id` de la primera `vx-product-card` (opción) y que esa tarjeta tiene `aria-selected="true"`. MUST NOT pulsar Enter ni clic en la tarjeta (eso abre recomendaciones, T-09-11). Un segundo escenario MUST enfocar el listbox y pulsar Escape: el foco MUST volver al Omnibox y las tarjetas MUST permanecer (no se vacía la consulta).

#### Scenario: ArrowDown desde el Omnibox activa la primera tarjeta (happy, T-01-10)

- **WHEN** hay al menos un `vx-product-card` en resultados
- **AND** el input del Omnibox tiene el foco
- **AND** el operador pulsa ArrowDown
- **THEN** el listbox tiene `aria-activedescendant` igual al `id` de la primera tarjeta
- **AND** esa tarjeta tiene `aria-selected="true"`
- **AND** no se abre el panel de recomendaciones

#### Scenario: Escape desde el listbox restaura el Omnibox sin vaciar (happy)

- **WHEN** hay resultados y el listbox tiene el foco
- **AND** el operador pulsa Escape
- **THEN** el input del Omnibox recupera el foco
- **AND** las `vx-product-card` siguen en `data-region="results"`

#### Scenario: sin selección de teclado el spec de flecha falla (unhappy)

- **WHEN** hay resultados y ArrowDown no actualiza `aria-activedescendant` ni `aria-selected`
- **AND** corre el spec de teclado
- **THEN** el spec termina en fallo (no skipped)

### Requirement: e2e H1 — caminos no felices de búsqueda (sin facetas)

La suite MUST cubrir estos unhappy paths en el mismo spec de búsqueda (nombres estables). MUST NOT implementar UI de filtros para hacerlos pasar.

1. Consulta de texto sin coincidencias (con espacio, para no disparar búsqueda por código): p. ej. `"zzzzz sinproducto"`. MUST dejar `data-region="results"` sin `vx-product-card`. MAY mostrar un mensaje en español «Sin resultados». MUST NOT dejar tarjetas de una consulta anterior si el spec busca primero y luego esta consulta.
2. Typo en SKU: escribir `"TOR-AUTO-10XI"` (un carácter distinto de `TOR-AUTO-10X1`). MUST NOT pintar un `vx-product-card` con `sku="TOR-AUTO-10X1"` (typo desactivado en `sku` / `codigos`).
3. Vaciar el Omnibox después de una búsqueda con hits: MUST quitar todas las `vx-product-card` de la región.
4. Ausencia de filtros facetados (No hace / recableo): con resultados visibles, MUST NOT haber una barra de `vx-chip` de filtros por atributo, marca, categoría o «con stock» en la vista operador.

#### Scenario: consulta sin hits deja la lista vacía (unhappy)

- **WHEN** el operador escribe `"zzzzz sinproducto"` en el Omnibox (índice ya consultable)
- **THEN** `data-region="results"` no contiene `vx-product-card`
- **AND** el spec no queda skipped

#### Scenario: typo en SKU no encuentra el tornillo (unhappy)

- **WHEN** el operador escribe `"TOR-AUTO-10XI"` en el Omnibox
- **THEN** no hay `vx-product-card` con `sku="TOR-AUTO-10X1"`

#### Scenario: vaciar el Omnibox limpia las tarjetas (unhappy)

- **WHEN** una búsqueda previa pintó al menos una `vx-product-card`
- **AND** el operador deja el Omnibox vacío
- **THEN** `data-region="results"` no contiene `vx-product-card`

#### Scenario: no hay filtros facetados en la vista operador (unhappy / non-goal)

- **WHEN** hay resultados de búsqueda de texto en la vista operador
- **THEN** no hay barra de filtros con `vx-chip` de categoría, marca, atributo o «con stock»

### Requirement: Tests previstos de e2e búsqueda H1

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. El smoke de T-09-09 MUST seguir pasando y MUST NOT tipear consultas.

| Test | Ubicación | Camino |
|---|---|---|
| `search_typo_tornilo_finds_tornillo_autoperforante` | `e2e/tests/search.spec.js` | happy (Listo cuando, HU-03-03) |
| `search_card_shows_stock_40` | mismo spec | happy (HU-03-05) |
| `search_arrow_down_selects_first_card` | mismo spec | happy (T-01-10) |
| `search_escape_from_listbox_restores_omnibox` | mismo spec | happy |
| `search_unknown_query_shows_no_cards` | mismo spec | unhappy |
| `search_sku_typo_does_not_match_tornillo` | mismo spec | unhappy |
| `search_empty_query_clears_cards` | mismo spec | unhappy |
| `search_no_facet_filter_chips` | mismo spec | unhappy (No hace T-03-19) |
| `smoke_app_loads_omnibox_focused` | `e2e/tests/smoke.spec.js` | happy (regresión T-09-09) |

`npx playwright test` en `e2e/` MUST ejecutar el Listo cuando (typo + stock + teclado) y los unhappy de esta tabla. El job CI `e2e` existente MUST seguir invocando esa suite; este change MUST NOT exigir un job nuevo ni cambios a `scripts/check-ci.sh`. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../docs/desarrollo-git-openspec.md) § 5). `./scripts/check-vectra-web.sh` MUST seguir en verde.

#### Scenario: suite de búsqueda H1 cubre happy y unhappy (happy)

- **WHEN** se ejecuta `npx playwright test` en `e2e/`
- **THEN** pasan `search_typo_tornilo_finds_tornillo_autoperforante`, `search_card_shows_stock_40` y `search_arrow_down_selects_first_card`
- **AND** pasan los unhappy de consulta vacía de hits, typo de SKU, Omnibox vacío y ausencia de facetas
- **AND** el smoke de Omnibox de T-09-09 sigue pasando
