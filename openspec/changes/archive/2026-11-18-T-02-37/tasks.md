# Tasks

## 1. Tests (red) — diff unitario y render edición

- [x] 1.1 En `product-sheet.test.mjs` agregar `diff_changed_product_fields_lists_price` importando `diffChangedProductFields` — `node --test scripts/fixtures/vectra-web/product-sheet.test.mjs` falla (función inexistente)
- [x] 1.2 Agregar `sheet_edit_stock_is_read_only` vía `renderEditProductSheet` — falla (función inexistente)

## 2. Implementación (green) — helpers de ficha edit

- [x] 2.1 En `product-sheet.js`: `diffChangedProductFields`, `renderEditProductSheet`, `readProductUpdate`; stock RO; CSS `.vx-product-stock-readonly` / `.vx-product-field-changed` — pasan 1.1 y 1.2

## 3. Tests (red) — mount edit y PATCH happy

- [x] 3.1 Agregar `sheet_edit_price_patches_with_version` y `products_list_offers_edit` — fallan (sin Editar ni submit edit)
- [x] 3.2 Agregar `sheet_edit_opens_hydrated_datos_tab` — falla

## 4. Implementación (green) — products.js edit mode

- [x] 4.1 `mode: "edit"`, botón Editar, GET ficha+defs, paint `renderEditProductSheet`, submit PATCH, éxito → listado — pasan 3.x

## 5. Tests (red) — conflict y unhappy

- [x] 5.1 `sheet_edit_version_conflict_reloads_and_lists_changed_fields`, `sheet_edit_does_not_merge_local_unsaved_fields` — fallan
- [x] 5.2 `sheet_edit_invalid_input_sets_field_error`, `sheet_edit_not_found_shows_not_found`, `sheet_edit_network_shows_network`, `sheet_edit_empty_required_does_not_patch` — fallan

## 6. Implementación (green) — version_conflict + errores

- [x] 6.1 Manejo `version_conflict`: GET, diff, status, changedFields, sin merge — pasan 5.1
- [x] 6.2 invalid_input remap, not_found al abrir, network, guardia obligatorios — pasan 5.2

## 7. Verificación de cierre

- [x] 7.1 `./scripts/check-vectra-web.sh` verde
- [x] 7.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes
- [x] 7.3 `openspec validate T-02-37` verde