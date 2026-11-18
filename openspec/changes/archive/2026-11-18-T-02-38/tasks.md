# Tasks

## 1. Tests (red) — panel Stock y parseo

- [x] 1.1 En `product-sheet.test.mjs` agregar `sheet_stock_empty_quantity_does_not_post` y un caso de parseo que importe `readStockMovement` (cantidad vacía → `null`; ajuste sin motivo → `{ error: "reason" }`) — `node --test scripts/fixtures/vectra-web/product-sheet.test.mjs` falla (función inexistente)
- [x] 1.2 Agregar `sheet_create_stock_tab_stays_disabled` (sigue el escenario T-02-35) y un test de `renderStockPanel` que pinte `data-form="stock-movement"`, stock RO y `role="status"` — falla si `renderStockPanel` no existe

## 2. Implementación (green) — helpers de Stock

- [x] 2.1 En `product-sheet.js`: exportar `renderStockPanel` y `readStockMovement` (tipo Ingreso/Ajuste, `quantity` string, motivo obligatorio en ajuste); CSS `.vx-product-stock-panel` / `.vx-product-stock-result` con tokens `--vx-*` — pasan 1.1 y 1.2 (`sheet_create_stock_tab_stays_disabled` sigue verde en alta)

## 3. Tests (red) — pestaña operable e ingreso happy

- [x] 3.1 Agregar `sheet_edit_stock_tab_is_operable` y `sheet_stock_shows_form_without_http` — fallan (Stock sigue `aria-disabled` en edición; no hay cambio de pestaña)
- [x] 3.2 Agregar `sheet_stock_inbound_posts_and_shows_resulting_stock` (HU-02-07 Ingreso: stock `"10"` + `"20"` → `"30"`) — falla (sin submit `stock-movement`)

## 4. Implementación (green) — tabs + POST inbound

- [x] 4.1 En `products.js`: `sheetTab`, click `[data-tab]` en edición, `renderEditProductSheet` con Stock sin `aria-disabled` y panel según tab — pasan 3.1
- [x] 4.2 Submit `data-form="stock-movement"` → `POST /api/v1/products/{id}/stock-movements`; 201 actualiza `editSnapshot.stock` y muestra `resultingStock`; MUST NOT PATCH ni GET historial — pasa 3.2

## 5. Tests (red) — ajuste, unhappy e inmutabilidad

- [x] 5.1 Agregar `sheet_stock_adjustment_to_zero_with_reason` y `sheet_stock_adjustment_without_reason_shows_invalid_input` (HU-02-07) — el segundo falla si aún se hace POST sin motivo
- [x] 5.2 Agregar `sheet_stock_negative_keeps_stock_and_shows_negative_stock`, `sheet_stock_invalid_input_sets_field_error`, `sheet_stock_not_found_shows_not_found`, `sheet_stock_network_shows_network` — fallan
- [x] 5.3 Agregar `sheet_stock_has_no_edit_or_delete_movement` y `sheet_stock_does_not_get_or_export_history` (Inmutabilidad / No hace) — fallan si hay GET historial o acciones de mutate

## 6. Implementación (green) — ajuste y errores

- [x] 6.1 Guardia local de ajuste sin motivo (`invalid_input` en `reason`, sin POST) y POST de ajuste con `reason` — pasan 5.1
- [x] 6.2 Catch: `negative_stock` sin mutar stock; `invalid_input` pinta `quantity`/`reason`; `not_found` / `network` con `showError`; panel sin edit/delete/export ni GET historial — pasan 5.2 y 5.3

## 7. Verificación de cierre

- [x] 7.1 `./scripts/check-vectra-web.sh` verde (incluye los tests de la tabla del requirement «Tests de pestaña Stock»)
- [x] 7.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 7.3 `openspec validate T-02-38` verde