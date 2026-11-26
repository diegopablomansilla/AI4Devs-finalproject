# Tasks

## 1. Tests (red) — listado y harness

- [x] 1.1 Crear `scripts/fixtures/vectra-web/products.test.mjs` con `products_list_requests_active_true`, `products_lookups_request_categories_and_brands` y `products_empty_list_does_not_post` — `node --test scripts/fixtures/vectra-web/products.test.mjs` falla en rojo (aún no existe `vectra-web/js/admin/products.js`)
- [x] 1.2 Agregar en `scripts/check-vectra-web.sh` la comprobación `products_missing_module_fails` (el archivo `vectra-web/js/admin/products.js` MUST existir) y registrar `products.test.mjs` en `node --test` — `./scripts/check-vectra-web.sh` falla en rojo porque falta el módulo

## 2. Implementación (green) — módulo, tabla y GET

- [x] 2.1 Crear `vectra-web/js/admin/products.js` exportando `mountProducts(el, { request, showError })`: `GET /api/v1/products?active=true`, lookups de categorías y marcas, HTML de tabla con columnas de `ProductSummary`, listado vacío sin `POST` — pasan los tests de 1.1; `./scripts/check-vectra-web.sh` deja de fallar por el archivo ausente
- [x] 2.2 En `vectra-web/css/base.css` añadir `.vx-product-*` con *tokens* `--vx-*` existentes (sin *tokens* nuevos, sin estilos de imagen) — `node --test scripts/fixtures/vectra-web/tokens.test.mjs` sigue verde

## 3. Tests (red) — filtros y paginación (HU-02-10)

- [x] 3.1 Agregar `products_filter_category_sends_category_id` y `products_filter_inactive_shows_reactivate` — los dos fallan en rojo (los `<select>` aún no envían query ni cambian acciones)
- [x] 3.2 Agregar `products_load_more_appends_with_cursor` — falla en rojo («Cargar más» aún no concatena)

## 4. Implementación (green) — filtros y «Cargar más»

- [x] 4.1 Implementar filtros `category` / `brand` / `active`: al `change` recargar la primera página (sin `cursor`) con esos query params; inactivo ofrece reactivar — pasan 3.1
- [x] 4.2 Implementar «Cargar más»: visible solo si `nextCursor` no es `null`; GET con `cursor` y append de filas — pasa 3.2

## 5. Tests (red) — baja y reactivación (HU-02-06)

- [x] 5.1 Agregar `products_deactivate_deletes` y `products_reactivate_gets_then_patches_active_true` — los dos fallan en rojo (`DELETE` / `GET`+`PATCH` aún no existen)

## 6. Implementación (green) — DELETE y PATCH `{ version, active: true }`

- [x] 6.1 Implementar baja (`DELETE /api/v1/products/{productId}`) y recarga de la primera página con los filtros actuales — pasa `products_deactivate_deletes`
- [x] 6.2 Implementar reactivar: `GET` ficha + `PATCH { version, active: true }` (sin sku/name/price/stock) y recarga — pasa `products_reactivate_gets_then_patches_active_true`

## 7. Tests (red) — unhappy path y No hace

- [x] 7.1 Agregar `products_network_shows_network`, `products_not_found_shows_not_found`, `products_invalid_input_shows_invalid_input` y `products_version_conflict_shows_version_conflict` — fallan en rojo (`showError` aún no se llama)
- [x] 7.2 Agregar `products_no_inline_edit` — falla en rojo si el HTML tiene `contenteditable`, inputs en celdas o `<form>` de alta/edición

## 8. Implementación (green) — errores por `code` y No hace

- [x] 8.1 En `mountProducts`, `catch` de `request` llama `showError(err)` (mensaje por `code`, tabla previa intacta en mutaciones); el HTML no tiene edición en línea ni `POST` de alta — pasan 7.1–7.2

## 9. Cableado del router (green) y regresión admin

- [x] 9.1 `startRouter` acepta `mountProducts` opcional y, si `section === "productos"` y hay `request`, llama `mountProducts` sobre `[data-region="admin-content"]`; `app.js` inyecta `api.request`, `errors.showError` y `mountProducts` — un test de `products.test.mjs` o `router.test.mjs` con fake `request` observa el GET de productos
- [x] 9.2 Mover `router_admin_content_region_has_no_business_ui` y `router_admin_does_not_call_api` a `#/admin/stock` (ya no `#/admin/productos`) — ambos siguen verdes; el menú de ocho secciones no cambia

## 10. Verificación de cierre

- [x] 10.1 `./scripts/check-vectra-web.sh` verde (suite previa + `products.test.mjs` happy/unhappy); comentario del script actualizado
- [x] 10.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 10.3 `openspec validate T-02-34` verde