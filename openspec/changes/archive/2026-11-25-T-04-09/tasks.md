# Tasks

## 1. Tests (red) — pestaña Relaciones

- [x] 1.1 Extender el fake de `request` en `scripts/fixtures/vectra-web/product-sheet.test.mjs` (GET `.../products/{id}/relations` → `{ items }`; PATCH/DELETE `.../relations/{type}/{id}`) y agregar tests happy `sheet_edit_relations_tab_is_operable`, `sheet_relations_lists_grouped_by_type`, `sheet_relations_empty_items_still_show_headings`, `sheet_relations_shows_inactive_target`, `sheet_relations_shows_bidirectional_indicator` — `node --test scripts/fixtures/vectra-web/product-sheet.test.mjs` falla en rojo (HU-04-03 Listado)
- [x] 1.2 Agregar tests happy `sheet_relations_patch_priority_updates_row`, `sheet_relations_patch_note_updates_row`, `sheet_relations_delete_removes_row` — fallan en rojo (HU-04-03 Edición y eliminación)
- [x] 1.3 Agregar tests unhappy `sheet_relations_priority_below_one_does_not_patch`, `sheet_relations_invalid_input_sets_field_error`, `sheet_relations_not_found_shows_not_found`, `sheet_relations_unavailable_shows_unavailable`, `sheet_relations_network_shows_network`, `sheet_relations_abort_does_not_paint_stale`, `sheet_create_relations_tab_stays_disabled`, `sheet_relations_has_no_create_controls`, `sheet_relations_does_not_get_incoming` — los nueve fallan en rojo
- [x] 1.4 Actualizar `sheet_edit_stock_tab_is_operable` para que deje de exigir `aria-disabled` en Relaciones — el test deja de afirmar el non-goal de T-02-38 sobre esa pestaña

## 2. Implementación (green) — ficha + products.js

- [x] 2.1 En `product-sheet.js` / `products.js`: pestaña Relaciones operable en edición (`sheetTab === "relaciones"`); GET con `AbortController`; panel agrupado Similares / Sustitutos / Se usa junto con, filas con SKU/nombre, prioridad, nota e indicador bidireccional; alta sigue disabled — pasan 1.1; `sheet_create_relations_tab_stays_disabled` y `sheet_relations_does_not_get_incoming` en verde
- [x] 2.2 Guardar por fila (`PATCH` `RelationUpdate`, `priority` entero JSON) y eliminar (`DELETE`); tras 200/204 re-GET o actualizar el panel de inmediato; sin POST ni buscador de alta — pasan 1.2 y `sheet_relations_has_no_create_controls`
- [x] 2.3 Unhappy: prioridad &lt; 1 no PATCH; `invalid_input` en el `vx-field` de la fila; 404/503/`network` → `showError`; abort no pinta ni toast — pasan 1.3; MAY `base.css` solo con tokens `--vx-*` existentes

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (suite previa + Relaciones happy/unhappy); sin `POST /api/v1/relations`; sin cambios a `docs/openapi.yaml` ni crates Rust; `sheet_edit_stock_tab_is_operable` verde
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 3.3 `openspec validate T-04-09` verde