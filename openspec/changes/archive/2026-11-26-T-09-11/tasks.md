# Tasks

## 1. Tests (red) — specs de recomendaciones y alta

- [x] 1.1 Extender `e2e/search-ready.js` (o helper nuevo) con `waitForSkuInIndex(baseURL, sku)` que con `timeoutMs <= 0` **falla de inmediato** (throw, no skip) — un test mínimo que la llama con timeout 0 sale ≠ 0 (escenario «el índice del alta no se saltea»)
- [x] 1.2 Crear `e2e/tests/recommendations.spec.js` con los nombres estables `recs_open_panel_shows_three_blocks`, `recs_perno_m8_shows_complement`, `recs_perno_acero_shows_substitute`, `recs_opening_recommended_product_keeps_panel`, `recs_close_returns_focus_to_omnibox`, `recs_substitutes_not_highlighted_when_in_stock`, `recs_highlights_substitutes_when_stock_zero`, `recs_product_without_relations_shows_empty_blocks` (el de highlight **después** del de no-destacado) — `npx playwright test recommendations.spec.js` en `e2e/` (con `VORTEX_BIN`/`MEILI_BIN`) sale ≠ 0 porque faltan aserciones de panel o el ajuste a 0 (escenarios HU-04-05 / HU-04-06 en rojo)
- [x] 1.3 Crear `e2e/tests/product-create.spec.js` con `product_create_appears_in_search`, `product_create_incomplete_stays_on_sheet`, `product_create_duplicate_sku_does_not_create` — `npx playwright test product-create.spec.js` en `e2e/` sale ≠ 0 (escenario Listo cuando HU-02-04 en rojo) — *implementado en `e2e/tests/zz-product-create.spec.js` (orden alfabético tras search/recs)*
- [x] 1.4 Verificar que `e2e/tests/search.spec.js` y `smoke.spec.js` no abren el panel ni navegan a admin, y que no existe `vectra-web/package.json` — `grep -E "Enter|#/admin" e2e/tests/search.spec.js e2e/tests/smoke.spec.js` no introduce el flujo de este ticket; `test ! -f vectra-web/package.json` (regresión T-09-10 / T-09-09)

## 2. Green — panel de recomendaciones y sustitutos destacados

- [x] 2.1 Helper `openProductFromSearch(page, query, sku)`: fill Omnibox, click en `vx-product-card[sku]` — `recs_open_panel_shows_three_blocks` puede llegar a la tarjeta (sigue rojo si el panel no aserta)
- [x] 2.2 Completar `recs_open_panel_shows_three_blocks`: panel `open`, tres títulos, compact `TOR-4-16-AC-001`, sin «Agregar» — `npx playwright test recommendations.spec.js -g recs_open_panel_shows_three_blocks` sale 0 (escenario Listo cuando HU-04-05)
- [x] 2.3 Completar `recs_perno_m8_shows_complement` (`ARA-PERNO-01`) y `recs_perno_acero_shows_substitute` (`PER-ACERO-02` stock ~12) — `-g "recs_perno_m8_shows_complement|recs_perno_acero_shows_substitute"` sale 0
- [x] 2.4 Completar `recs_opening_recommended_product_keeps_panel` (clic en `TOR-4-16-AC-001` → panel sigue abierto con tres títulos) y `recs_close_returns_focus_to_omnibox` (Escape → input Omnibox focused) — `-g "recs_opening_recommended_product_keeps_panel|recs_close_returns_focus_to_omnibox"` sale 0
- [x] 2.5 Completar `recs_substitutes_not_highlighted_when_in_stock` (encabezado Sustitutos **sin** `data-highlight-substitutes` ni «Destacados») **antes** del ajuste — `-g recs_substitutes_not_highlighted_when_in_stock` sale 0 (HU-04-06 con stock)
- [x] 2.6 Helper `zeroProductStock`: leer stock actual de `PER-ACERO-01`, `POST .../stock-movements` adjustment negativo hasta 0 (falla, no skip) y completar `recs_highlights_substitutes_when_stock_zero` (`data-highlight-substitutes`, «Destacados», `PER-ACERO-02`, sin stock 0 en el panel) — `-g recs_highlights_substitutes_when_stock_zero` sale 0 (escenario Listo cuando HU-04-06)
- [x] 2.7 Completar `recs_product_without_relations_shows_empty_blocks` (`TOR-4-16-ZI-002`: tres títulos, cero `vx-product-card` en el panel) — `-g recs_product_without_relations_shows_empty_blocks` sale 0 (unhappy panel vacío)

## 3. Green — alta de producto y unhappy

- [x] 3.1 Helper `fillVxField(page, name, value)` (fill + `change`/`blur`) y navegación `#/admin/productos` → `data-action="create"` — el formulario `data-form="create-product"` es visible (sigue rojo el POST)
- [x] 3.2 Completar `product_create_appears_in_search`: SKU `E2E-ALTA-001`, nombre «Producto e2e alta visible», tipo, categoría (opción 1 del select), precio, Guardar; fila admin; `waitForSkuInIndex`; Omnibox muestra `vx-product-card[sku="E2E-ALTA-001"]` — `npx playwright test product-create.spec.js -g product_create_appears_in_search` sale 0 (escenario Listo cuando HU-02-04)
- [x] 3.3 Completar `product_create_incomplete_stays_on_sheet` (Guardar vacío → sigue la ficha, no hay fila `E2E-ALTA-001`) y `product_create_duplicate_sku_does_not_create` (`TOR-AUTO-10X1` → error observable; `"tornilo"` muestra **una** tarjeta) — `-g "product_create_incomplete_stays_on_sheet|product_create_duplicate_sku_does_not_create"` sale 0 (unhappy alta)

## 4. Cierre (regresión, § 5 y non-goals)

- [x] 4.1 `npx playwright test` en `e2e/` sale 0 (smoke + axe + search + recs + alta); `git diff` vacío en `.github/workflows/ci.yml`, `scripts/check-ci.sh`, `docs/openapi.yaml`, `seed/` y (salvo bug de Listo cuando) `vectra-web/` (Listo cuando local; non-goals T-03-19 / T-08-06 / OpenAPI) — *`vectra-web/js/search.js`: `limit=50` en GET search (bug: default API 20 excluía `TOR-AUTO-10X1` con q=`tornilo` tras suite)*
- [x] 4.2 `./scripts/check-ci.sh`, `./scripts/fixtures/ci/run-script-tests.sh` y `./scripts/check-vectra-web.sh` salen 0 (regresión T-09-09 / vectra-web)
- [x] 4.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.4 `openspec validate T-09-11` (o `t-09-11` si el CLI exige minúsculas) sale 0