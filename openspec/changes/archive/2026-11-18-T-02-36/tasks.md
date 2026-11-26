# Tasks

## 1. Tests (red) — controles por tipo y harness de lookup

- [x] 1.1 En `scripts/fixtures/vectra-web/product-sheet.test.mjs` agregar `sheet_attribute_controls_match_type_and_unit` (defs `largo` number+mm, `medida` text, `zincado` boolean, `grado` enum 2/5/8) — `node --test scripts/fixtures/vectra-web/product-sheet.test.mjs` falla en rojo (no hay `GET /api/v1/attribute-definitions` ni controles por tipo)
- [x] 1.2 Eliminar `sheet_no_attributes_or_codes` y agregar `sheet_no_barcode_code` — el test de ausencia de atributos ya no aplica (superseded); `sheet_no_barcode_code` falla si hay `barcode` o pasa en vacío hasta que existan códigos

## 2. Implementación (green) — lookup y editor

- [x] 2.1 En `products.js`, al pasar a `mode: "create"`, pedir `GET /api/v1/attribute-definitions` sin `includeInactive`; guardar `items`; si falla, `showError` y `attributeDefs = []` — pasa el GET del escenario «editor muestra control según tipo»; los tests previos de ficha reciben `{ items: [] }` como respuesta extra y siguen verdes
- [x] 2.2 En `product-sheet.js`, `renderCreateProductSheet` pinta atributos activos en orden API (`vx-field` text/number, `<select>` boolean/enum, unidad en etiqueta, `name="attributes[${i}].value"`) y códigos fabricante/OEM; filtrar `active !== false`; CSS `.vx-product-attrs` / `.vx-product-codes` con tokens `--vx-*` existentes — pasa `sheet_attribute_controls_match_type_and_unit`; `node --test scripts/fixtures/vectra-web/tokens.test.mjs` sigue verde

## 3. Tests (red) — POST attributes y codes

- [x] 3.1 Agregar `sheet_create_perno_posts_attributes_and_codes` (medida `M8`, largo `40` como JSON number, fabricante `HX-0840Z`) — falla en rojo (`readProductCreate` aún no manda `attributes`/`codes`)
- [x] 3.2 Agregar `sheet_empty_attributes_omitted_from_post` — falla en rojo si el POST envía `attributes` no vacío con defs sin valor; ajustar `sheet_create_perno_posts_product_create` para no exigir `attributes === undefined` cuando el lookup está vacío (omitido o `[]`)

## 4. Implementación (green) — parseo y POST

- [x] 4.1 Extender `readProductCreate`: omitir vacíos; number parseable → JSON number, si no string; boolean → `true`/`false`; codes `manufacturer`/`oem` — pasan 3.1 y 3.2; el POST no incluye `barcode`

## 5. Tests (red) — unhappy path y Listo cuando

- [x] 5.1 Agregar `sheet_cuarenta_on_largo_sets_attribute_field_error` — falla en rojo (el `vx-field` de largo no recibe `errors[].field` `attributes[n].value`)
- [x] 5.2 Agregar `sheet_inactive_attribute_defs_are_hidden` y `sheet_attribute_defs_network_shows_network` — fallan en rojo si se pintan inactivas o si el GET de defs no llama a `showError`

## 6. Implementación (green) — error sobre el campo y filtros

- [x] 6.1 Tras POST `400` `invalid_input`, remapear `attributes[n].value` al `definitionId` enviado y pintar `fieldErrors` sobre el `vx-field` de ese atributo; ficha abierta; toast `invalid_input` — pasa `sheet_cuarenta_on_largo_sets_attribute_field_error` (Listo cuando)
- [x] 6.2 No pintar defs `active === false`; `catch` del GET de definiciones llama `showError` y no lanza — pasan 5.2 y `sheet_no_barcode_code`

## 7. Verificación de cierre

- [x] 7.1 `./scripts/check-vectra-web.sh` verde (suite previa + tests nuevos happy/unhappy; sin `sheet_no_attributes_or_codes`)
- [x] 7.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 7.3 `openspec validate T-02-36` verde