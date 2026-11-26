# Tasks

## 1. Tests (red) — recommendations + search

- [x] 1.1 Crear `scripts/fixtures/vectra-web/recommendations.test.mjs` y extender el fake de `request` (path `/recommendations` → `Recommendations`); en `search.test.mjs` los hits nuevos MUST incluir `id` ULID — el archivo importa `mountSearch` / `recommendations.js` y falla al resolver el módulo ausente, o los tests nuevos no existen aún
- [x] 1.2 Agregar tests happy `search_enter_opens_recommendations_panel`, `search_click_opens_recommendations_panel`, `recs_three_blocks_in_api_order`, `recs_compact_cards_not_full`, `recs_empty_arrays_still_show_headings`, `recs_inactive_sku_absent_from_payload_is_absent` — `node --test scripts/fixtures/vectra-web/recommendations.test.mjs scripts/fixtures/vectra-web/search.test.mjs` falla en rojo (escenarios HU-04-05 «Tres bloques» / «Productos inactivos»)
- [x] 1.3 Agregar tests happy `recs_highlights_substitutes_when_flag_true`, `recs_does_not_highlight_substitutes_when_flag_false`, `recs_opening_recommended_product_fetches_that_id`, `recs_close_returns_focus_to_omnibox` — fallan en rojo (HU-04-06 y «elegir un producto del panel» / foco Omnibox)
- [x] 1.4 Agregar tests unhappy `recs_missing_product_id_does_not_fetch`, `recs_404_calls_showError_and_does_not_paint`, `recs_503_calls_showError`, `recs_network_calls_showError`, `recs_abort_does_not_paint_stale`, `recs_abort_does_not_call_showError`, `recs_admin_view_does_not_fetch`, `recs_no_add_to_cart_control` — los ocho fallan en rojo

## 2. Implementación (green) — recommendations.js + search.js

- [x] 2.1 Crear `vectra-web/js/recommendations.js` (`mountRecommendations`): `GET /api/v1/products/{id}/recommendations` con `AbortController`, pintar tres headings + `vx-product-card` `compact` en orden de API, `data-product-id`, listbox T-01-10, `show` del `vx-side-panel` — pasan 1.2; arrays vacíos muestran títulos; MUST NOT llamar `/relations` ni reordenar
- [x] 2.2 Desteque de Sustitutos con `data-highlight-substitutes` (y etiqueta visible) solo si `highlightSubstitutes === true`; Enter/clic en compacta abre ese `id`; al `vx-closed` el Omnibox recupera el foco (T-01-08, sin tocar el overlay) — pasan 1.3; MAY `base.css` solo con tokens `--vx-*` existentes
- [x] 2.3 Unhappy: sin `data-product-id` no hay GET; 404/503/`network` → `showError` y no pintar; abort no pinta ni toast; admin no fetch; sin control «Agregar» — pasan 1.4
- [x] 2.4 En `vectra-web/js/search.js`: `data-product-id` en cada tarjeta; `onOpen` y clic delegan a `openRecommendations`; harness `scripts/check-vectra-web.sh` exige `recommendations.js` y corre `recommendations.test.mjs` — pasan `search_enter_opens_recommendations_panel` y `search_click_opens_recommendations_panel`; `recs_missing_module_fails` (falta el archivo → exit ≠ 0)

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (suite previa + panel happy/unhappy); `vx-side-panel.js` sigue sin lógica de recomendaciones; sin carrito; sin cambios a `docs/openapi.yaml` ni crates Rust
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 3.3 `openspec validate T-04-08` verde