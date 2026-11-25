# Tasks

## 1. Tests (red)

- [x] 1.1 Extender `scripts/fixtures/vectra-web/dom-stub.mjs`: `createElement` instancia custom elements registrados, `appendChild` invoca `connectedCallback`, `querySelector("vx-product-card")` resuelve por `localName`. Verificar: un `createElement("vx-product-card")` tras importar el componente deja de ser un `FakeNode` genérico (el test 1.2 falla en paint, no en selector).
- [x] 1.2 Ampliar `scripts/fixtures/vectra-web/search.test.mjs` (happy): `search_renders_full_product_card`, `search_renders_name_and_sku_as_text` ahora exige `vx-product-card`, `search_zero_stock_marks_out_of_stock`, `search_missing_image_url_shows_placeholder`, `search_image_url_sets_card_src`, `search_highlight_name_paints_mark` dentro del slot. Verificar: `node --test scripts/fixtures/vectra-web/search.test.mjs` rojo en esos casos.
- [x] 1.3 Ampliar `search.test.mjs` (unhappy): `search_html_in_plain_fields_is_text`, `search_malformed_hit_does_not_throw`, `search_highlight_description_does_not_paint_on_card`; retirar `search_highlight_description_paints_mark`. Verificar: esos tests existen y fallan (o el de description viejo deja de ser requerido).
- [x] 1.4 Importar `vx-product-card` + `setSharedStyleLoader` en el harness de search (como el Omnibox). Verificar: los tests nuevos corren sin `unknown element vx-product-card`.

## 2. Implementación (green)

- [x] 2.1 Reescribir `paintHits` en `vectra-web/js/search.js`: un `vx-product-card` `full` por hit, mapeo `SearchHit` → atributos, slot `name` con `appendSanitizedHtml`, sin `highlight.description`, skip de hits no objeto. Verificar: `search_renders_full_product_card`, `search_highlight_name_paints_mark`, `search_highlight_null_falls_back_to_name` y `search_malformed_hit_does_not_throw` verdes.
- [x] 2.2 Estado sin stock e imagen: `stock` cero → `data-out-of-stock` / «Sin stock»; `imageUrl` vacío → placeholder; URL presente → `image-url`. Verificar: `search_zero_stock_marks_out_of_stock`, `search_missing_image_url_shows_placeholder`, `search_image_url_sets_card_src` verdes.
- [x] 2.3 Unhappy de markup: highlight peligroso sigue sin `img`; `brand`/`sku`/`price`/`location` con HTML son texto. Verificar: `search_highlight_img_onerror_no_img_element` y `search_html_in_plain_fields_is_text` verdes.
- [x] 2.4 Conservar listbox + `bindKeyboardList` + abort/vacío/error. Verificar: `search_arrow_down_from_omnibox_selects_first_hit`, `search_cancels_previous_and_paints_last_only`, `search_empty_query_clears_without_fetch`, `search_empty_hits_shows_empty_list` y `search_problem_json_calls_showError` siguen verdes.
- [x] 2.5 Ajuste mínimo de slot/`innerHTML` en `vx-product-card` o en el stub solo si 2.1 no logra un `mark` observable. Verificar: `search_highlight_name_paints_mark` verde sin asignar el payload crudo a `innerHTML`.
- [x] 2.6 MAY `base.css` con *tokens* `--vx-*` existentes para el listado (gap). Verificar: no hay tokens nuevos; `./scripts/check-vectra-web.sh` no falla por CSS.

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (happy + unhappy de T-03-17 y regresión T-03-15/T-03-16).
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5).
- [x] 3.3 `openspec validate T-03-17` sin errores. Listo cuando humano (fuera de CI): escenarios HU-03-05 a mano (tarjeta completa + sin stock).