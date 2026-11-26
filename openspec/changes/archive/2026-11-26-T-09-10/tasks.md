# Tasks

## 1. Tests (red) — spec de búsqueda H1

- [x] 1.1 Crear `e2e/search-ready.js` que exporta una espera de índice (`GET /api/v1/search?q=tornilo` hasta `hits[].sku === "TOR-AUTO-10X1"`) y, en este paso, **falla de inmediato** (timeout 0 o throw) en un test mínimo que la llama sale ≠ 0 y **no** skipped (escenario «el índice aún no listo no se saltea»)
- [x] 1.2 Crear `e2e/tests/search.spec.js` con los nombres estables `search_typo_tornilo_finds_tornillo_autoperforante`, `search_card_shows_stock_40`, `search_arrow_down_selects_first_card`, `search_escape_from_listbox_restores_omnibox`, `search_unknown_query_shows_no_cards`, `search_sku_typo_does_not_match_tornillo`, `search_empty_query_clears_cards`, `search_no_facet_filter_chips` — `npx playwright test search.spec.js` en `e2e/` (con `VORTEX_BIN`/`MEILI_BIN`) sale ≠ 0 porque 1.1 aún no espera el índice o las aserciones de UI faltan (escenarios happy en rojo; Listo cuando HU-03-03)
- [x] 1.3 Verificar que `e2e/tests/smoke.spec.js` no tipea consultas y que no existe `vectra-web/package.json` — `grep -E "fill|type|pressSequentially" e2e/tests/smoke.spec.js` no matchea input de búsqueda; `test ! -f vectra-web/package.json` (regresión T-09-09 / non-goal npm)

## 2. Green — helper de índice y happy paths

- [x] 2.1 Implementar el poll en `e2e/search-ready.js` (timeout ~60 s, fallo no skip) y usarlo en `test.beforeAll` o al inicio de cada test de `search.spec.js` — `GET {baseURL}/api/v1/search?q=tornilo` termina con el SKU `TOR-AUTO-10X1` antes de tipear (escenario «tornilo encuentra…» desbloqueado a nivel API)
- [x] 2.2 Completar `search_typo_tornilo_finds_tornillo_autoperforante`: `#/`, fill `"tornilo"` en `vx-omnibox input`, esperar `vx-product-card[sku="TOR-AUTO-10X1"]` con texto «Tornillo autoperforante», asertar que `vx-side-panel` no está abierto — `npx playwright test search.spec.js -g search_typo_tornilo_finds_tornillo_autoperforante` en `e2e/` sale 0 (escenario Listo cuando HU-03-03)
- [x] 2.3 Completar `search_card_shows_stock_40`: misma tarjeta `full`, atributo `stock` decimal equivalente a 40 y stock visible — `npx playwright test search.spec.js -g search_card_shows_stock_40` sale 0 (escenario «la tarjeta del tornillo muestra stock 40»)
- [x] 2.4 Completar `search_arrow_down_selects_first_card` (ArrowDown desde el Omnibox → `aria-activedescendant` + `aria-selected` en la primera tarjeta, sin Enter) y `search_escape_from_listbox_restores_omnibox` (foco al listbox, Escape, Omnibox enfocado, tarjetas siguen) — `npx playwright test search.spec.js -g "search_arrow_down_selects_first_card|search_escape_from_listbox_restores_omnibox"` sale 0 (escenarios T-01-10)

## 3. Green — unhappy paths

- [x] 3.1 Completar `search_unknown_query_shows_no_cards` (`"zzzzz sinproducto"` → cero `vx-product-card`) y `search_sku_typo_does_not_match_tornillo` (`"TOR-AUTO-10XI"` → no hay tarjeta `TOR-AUTO-10X1`) — `npx playwright test search.spec.js -g "search_unknown_query_shows_no_cards|search_sku_typo_does_not_match_tornillo"` sale 0 (escenarios «consulta sin hits», «typo en SKU»)
- [x] 3.2 Completar `search_empty_query_clears_cards` (hits previos, Omnibox vacío → sin tarjetas) y `search_no_facet_filter_chips` (con resultados, no hay `vx-chip` de filtros de categoría/marca/atributo/«con stock») — `npx playwright test search.spec.js -g "search_empty_query_clears_cards|search_no_facet_filter_chips"` sale 0 (escenarios «vaciar el Omnibox», «no hay filtros facetados»)

## 4. Cierre (regresión, § 5 y non-goals)

- [x] 4.1 `npx playwright test` en `e2e/` sale 0 (search + smoke + axe); el smoke sigue sin tipear; `git diff` vacío en `.github/workflows/ci.yml`, `scripts/check-ci.sh`, `docs/openapi.yaml` y `seed/` (Listo cuando en la suite local; non-goals T-09-11 / T-03-19 / OpenAPI)
- [x] 4.2 `./scripts/check-ci.sh`, `./scripts/fixtures/ci/run-script-tests.sh` y `./scripts/check-vectra-web.sh` salen 0 (regresión T-09-09 / vectra-web)
- [x] 4.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.4 `openspec validate T-09-10` (o `t-09-10` si el CLI exige minúsculas) sale 0