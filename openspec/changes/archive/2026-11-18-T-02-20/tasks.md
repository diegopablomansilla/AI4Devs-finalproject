# Tasks

## 1. Tests (red) — crear y obtener

- [x] 1.1 Agregar tests happy `catalog_create_perno_m8x40_sets_version_1` y `catalog_get_returns_ficha_with_marca_ruta_and_attributes` en `vortex-core/crates/catalog/src/producto.rs` — `cargo test -p catalog catalog_create_perno_m8x40_sets_version_1 catalog_get_returns_ficha_with_marca_ruta_and_attributes` no compila o falla (escenarios «Alta completa perno M8 x 40», «Obtener ficha existente»)
- [x] 1.2 Agregar tests unhappy `catalog_create_duplicate_sku_is_duplicate_sku`, `catalog_create_cuarenta_on_number_attribute_is_invalid_input`, `catalog_create_inactive_categoria_is_invalid_input`, `catalog_create_inactive_marca_is_invalid_input`, `catalog_create_unknown_categoria_is_invalid_input` y `catalog_get_unknown_id_is_not_found` — `cargo test -p catalog catalog_create_duplicate_sku catalog_create_cuarenta catalog_create_inactive_categoria catalog_create_inactive_marca catalog_create_unknown_categoria catalog_get_unknown_id` no compila o falla (escenarios HU-02-04 / refs inactivas / `not_found`)

## 2. Implementación (green) — crear y obtener

- [x] 2.1 Fakes en memoria: `InMemoryProductRepo` (SKU único → `duplicate_sku`; `create` con `version` 1, `activo` true, `stock = stock_inicial`; `get` hidrata `FichaProducto`) y seed de categoría/marca/`AtributoDef` (reusar `marca::testing` / `atributo_def::testing`; exponer `pub mod testing` en `categoria` si hace falta) — suficiente para los tests 1.x
- [x] 2.2 `ProductService` con `create` (cargar defs, exigir categoría/marca activas, `ProductRepo::create` + `get` → `FichaProducto`) y `get` — `cargo test -p catalog catalog_create_perno_m8x40_sets_version_1 catalog_get_returns_ficha_with_marca_ruta_and_attributes catalog_create_duplicate_sku_is_duplicate_sku catalog_create_cuarenta_on_number_attribute_is_invalid_input catalog_create_inactive_categoria_is_invalid_input catalog_create_inactive_marca_is_invalid_input catalog_create_unknown_categoria_is_invalid_input catalog_get_unknown_id_is_not_found` sale 0

## 3. Tests (red) — editar y fachada

- [x] 3.1 Agregar tests happy `catalog_update_price_increments_version` y `catalog_update_partial_price_keeps_other_fields` — `cargo test -p catalog catalog_update_price_increments_version catalog_update_partial_price_keeps_other_fields` no compila o falla (escenarios «Edición exitosa de precio», «Parche parcial conserva el resto de la ficha»)
- [x] 3.2 Agregar tests unhappy `catalog_update_stale_version_is_version_conflict`, `catalog_update_unknown_id_is_not_found` y `catalog_update_inactive_categoria_is_invalid_input` — `cargo test -p catalog catalog_update_stale_version_is_version_conflict catalog_update_unknown_id_is_not_found catalog_update_inactive_categoria_is_invalid_input` no compila o falla (escenarios HU-02-05 / Listo cuando / categoría inactiva al editar)
- [x] 3.3 Agregar test `product_gateway_is_object_safe` — `cargo test -p catalog product_gateway_is_object_safe` no compila o falla (escenario «el servicio es inyectable en AppState»)

## 4. Implementación (green) — editar y fachada

- [x] 4.1 `PatchProducto` + `ProductService::update` (get → merge → validar refs/defs → `ProductRepo::update`; fake con `version_conflict`; MUST NOT escribir `stock` ni `activo`) — `cargo test -p catalog catalog_update_price_increments_version catalog_update_partial_price_keeps_other_fields catalog_update_stale_version_is_version_conflict catalog_update_unknown_id_is_not_found catalog_update_inactive_categoria_is_invalid_input` sale 0
- [x] 4.2 `ProductGateway` object-safe (`create` / `get` / `update`) implementado por `ProductService`; reexport en `lib.rs`; MUST NOT exponer baja ni listado — `cargo test -p catalog product_gateway_is_object_safe` sale 0; `rg -n "fn deactivate|fn reactivate|fn list" vortex-core/crates/catalog/src/producto.rs` sin matches

## 5. Cierre (regresión)

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 5.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 5.3 Confirmar non-goals: `git diff --name-only` sin `docs/openapi.yaml`, sin `vortex-core/crates/api`, sin `vortex-core/crates/storage`, sin `vortex-core/crates/domain`
- [x] 5.4 `openspec validate T-02-20` sale 0