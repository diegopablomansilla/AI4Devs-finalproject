# Tasks

## 1. Tests (red) — harness, botón de alta y ficha Datos

- [x] 1.1 Crear `scripts/fixtures/vectra-web/product-sheet.test.mjs` con `sheet_new_product_opens_datos_tab` y `sheet_stock_and_relations_tabs_are_disabled` — `node --test scripts/fixtures/vectra-web/product-sheet.test.mjs` falla en rojo (aún no existe `vectra-web/js/admin/product-sheet.js`)
- [x] 1.2 Agregar `products_empty_list_offers_create` en `scripts/fixtures/vectra-web/products.test.mjs` — el test falla en rojo (el listado vacío de T-02-34 no ofrece «Nuevo producto»)
- [x] 1.3 En `scripts/check-vectra-web.sh` exigir `vectra-web/js/admin/product-sheet.js` (`sheet_missing_module_fails`) y registrar `product-sheet.test.mjs` en `node --test` — `./scripts/check-vectra-web.sh` falla en rojo porque falta el módulo

## 2. Implementación (green) — módulo, pestañas y acción Nuevo producto

- [x] 2.1 Crear `vectra-web/js/admin/product-sheet.js` exportando el HTML de ficha: `role="tablist"` Datos (seleccionada) / Stock y Relaciones (`aria-disabled`); `<form data-form="create-product">` con `vx-field` `sku`, `name`, `productType`, `description`, `price`, `location.aisle`, `location.shelf`, `location.position`, `initialStock` y `<select>`+`vx-field` para `categoryId`, `brandId`, `unitOfMeasure` — pasan 1.1
- [x] 2.2 En `vectra-web/js/admin/products.js` añadir `data-action="create"` «Nuevo producto» (también con lista vacía) y `mode: "create"` que pinta la ficha; «Cancelar» vuelve al listado sin POST — pasa 1.2; `products_empty_list_does_not_post` y `products_no_inline_edit` siguen verdes (ficha cerrada)
- [x] 2.3 En `vectra-web/css/base.css` añadir `.vx-product-sheet*` / `.vx-product-tabs` con *tokens* `--vx-*` existentes (sin *tokens* nuevos, sin pestaña Imágenes) — `node --test scripts/fixtures/vectra-web/tokens.test.mjs` sigue verde; `./scripts/check-vectra-web.sh` deja de fallar por el archivo ausente

## 3. Tests (red) — POST ProductCreate y Listo cuando

- [x] 3.1 Agregar `sheet_create_perno_posts_product_create` y `sheet_create_returns_to_list_with_new_sku` — los dos fallan en rojo (aún no hay `POST /api/v1/products` ni recarga del listado)
- [x] 3.2 Agregar `sheet_cancel_does_not_post` y `sheet_empty_required_does_not_post` — fallan en rojo si el submit vacío o cancelar llegan a POST

## 4. Implementación (green) — POST y vuelta al listado

- [x] 4.1 Implementar `readProductCreate` + submit: `POST /api/v1/products` con `sku`/`name`/`productType`/`categoryId`/`price` (decimales string), `initialStock` opcional, sin `attributes`/`codes`; si falta un obligatorio, no llamar a `request` — pasa `sheet_create_perno_posts_product_create` y `sheet_empty_required_does_not_post`
- [x] 4.2 Tras `201`, `mode: "list"` y recargar la primera página con los filtros previos; cancelar no POST — pasan `sheet_create_returns_to_list_with_new_sku` y `sheet_cancel_does_not_post`

## 5. Tests (red) — unhappy path y No hace

- [x] 5.1 Agregar `sheet_duplicate_sku_shows_duplicate_sku`, `sheet_invalid_input_sets_sku_field_error`, `sheet_invalid_input_sets_category_id_field_error` y `sheet_network_shows_network` — fallan en rojo (`showError` aún no se llama / la ficha se cierra)
- [x] 5.2 Agregar `sheet_no_attributes_or_codes` — falla en rojo si hay controles `attributes`/`codes` o el POST los envía no vacíos

## 6. Implementación (green) — errores por `code`/campo y No hace

- [x] 6.1 En el `catch` del POST, `showError(err)` (toast por `code`; `vx-field[name]` recibe `errors[].field`) y la ficha permanece abierta — pasan 5.1
- [x] 6.2 El HTML de la ficha no incluye atributos ni códigos; el cuerpo no manda esas claves (o las manda `[]`) — pasa 5.2

## 7. Verificación de cierre

- [x] 7.1 `./scripts/check-vectra-web.sh` verde (suite previa + `product-sheet.test.mjs` happy/unhappy + `products_empty_list_offers_create`); comentario del script actualizado
- [x] 7.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 7.3 `openspec validate T-02-35` verde