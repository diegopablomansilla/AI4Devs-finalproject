# Tasks

## 1. Tests (red) — árbol, stub y tarjeta

- [x] 1.1 Extender `check-vectra-web.sh` para exigir `vx-product-card.js` y ambas variantes (`variant="full"` y `variant="compact"`) en `dev/components.html` (`components_tree_includes_product_card`, `components_html_catalog_loads_product_card_variants`; unhappy: falta `vx-product-card.js`) — `./scripts/check-vectra-web.sh` falla en rojo
- [x] 1.2 Extender `dom-stub.mjs` con `querySelector` de host para `[slot="name"]` y en el shadow para `[data-field]`, `img` y `[data-out-of-stock]` — un test mínimo de stub encuentra un nodo con `data-field="sku"`
- [x] 1.3 Agregar en `components.test.mjs` los tests `vx_product_card_full_renders_search_hit_fields`, `vx_product_card_compact_hides_location_and_image`, `vx_product_card_zero_stock_sets_out_of_stock`, `vx_product_card_name_slot_projects_highlight`, `vx_product_card_html_in_name_attr_is_text`, `vx_product_card_unknown_variant_falls_back_to_full`, `vx_product_card_missing_image_shows_placeholder`, `vx_product_card_zero_stock_without_marker_fails` — `node --test scripts/fixtures/vectra-web/components.test.mjs` falla en rojo

## 2. Implementación (green) — componente y muestra

- [x] 2.1 `vx-product-card.js` + registro en `index.js` (variantes `full`/`compact`, *slot* `name`, `data-out-of-stock`, placeholder de imagen, `textContent` del atributo `name`) — pasan los ocho tests de 1.3
- [x] 2.2 Actualizar `dev/components.html` con una tarjeta `full` (marca, ubicación, imagen o placeholder), una `compact` y al menos un `stock="0"`, datos de ejemplo — harness de catálogo verde; ambas variantes visibles a mano (Listo cuando)

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (happy + unhappy de este ticket)
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 3.3 `openspec validate T-01-05` verde
