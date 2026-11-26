# Tasks

## 1. Tests (red) — domain

- [x] 1.1 Agregar tests `ficha_producto_exposes_marca_ruta_and_resolved_attributes` y `update_producto_has_no_stock_or_activo_fields` en `vortex-core/crates/domain/src/producto.rs` — `cargo test -p domain ficha_producto_exposes update_producto_has_no_stock` no compila o falla (escenarios «FichaProducto expone marca, ruta y atributos resueltos», «UpdateProducto no incluye stock ni activo»)
- [x] 1.2 Reemplazar `product_repo_trait_exposes_create_get` por `product_repo_trait_exposes_create_get_update` y `product_repo_trait_get_returns_ficha` (dummy con `update`; `get`/`update` → `FichaProducto`) — `cargo test -p domain product_repo_trait_exposes_create_get_update product_repo_trait_get_returns_ficha` no compila o falla (escenarios «El puerto declara create y get», «El puerto declara update», «domain no persiste productos»)

## 2. Implementación (green) — domain

- [x] 2.1 Añadir `AtributoFicha`, `FichaProducto` y `UpdateProducto` (sin `stock` ni `activo`); reexportar desde `lib.rs` — `cargo test -p domain ficha_producto_exposes update_producto_has_no_stock` sale 0
- [x] 2.2 Extender `ProductRepo` con `update` y `get` → `FichaProducto`; actualizar dummy — `cargo test -p domain product_repo_trait_exposes_create_get_update product_repo_trait_get_returns_ficha` sale 0; `./scripts/check-domain-deps.sh` sale 0

## 3. Tests (red) — storage

- [x] 3.1 Extender `vortex-core/crates/storage/tests/producto.rs` con seed de marca, categoría hija (`ruta`) y `AtributoDef`; tests happy `product_repo_get_hydrates_marca_ruta_and_attributes`, `product_repo_update_increments_version`, `product_repo_update_does_not_write_stock` y `product_repo_update_replaces_atributos_and_codigos` — `cargo test -p storage --test producto product_repo_get_hydrates product_repo_update_increments product_repo_update_does_not_write product_repo_update_replaces` no compila o falla (escenarios «get hidrata…», «Edición incrementa version», «Edición no escribe stock», «Edición reemplaza atributos y códigos»)
- [x] 3.2 Agregar tests unhappy `product_repo_concurrent_updates_same_version_conflict` (**Listo cuando**), `product_repo_update_unknown_id_is_not_found` y `product_repo_update_rejects_duplicate_sku`; ajustar `product_repo_get_returns_persisted` al retorno `FichaProducto` — `cargo test -p storage --test producto product_repo_concurrent_updates product_repo_update_unknown product_repo_update_rejects_duplicate product_repo_get_returns_persisted` no compila o falla (escenarios «Dos ediciones con la misma version», «update de id inexistente», «update con SKU duplicado», «get de producto existente»)

## 4. Implementación (green) — storage

- [x] 4.1 Hidratar `ProductoRepo::get` a `FichaProducto` (`MarcaRepo` / `CategoriaRepo` / defs ya cargadas → `AtributoFicha`); MUST NOT leer `imagen_producto` — `cargo test -p storage --test producto product_repo_get_hydrates_marca_ruta_and_attributes product_repo_get_returns_persisted product_repo_get_unknown_id_is_not_found` sale 0
- [x] 4.2 Implementar `update`: `not_found` si no hay fila; `Tx` con `UPDATE … SET …, version += 1 WHERE version = $v`, reemplazo entero de `atributos`/`codigos`, sin escribir `stock` ni `activo`, `THROW 'version_conflict'` si 0 filas, outbox `producto`/`upsert` (`motivo` ausente); mapper en `tx.rs` si hace falta — `cargo test -p storage --test producto product_repo_update_increments_version product_repo_update_does_not_write_stock product_repo_update_replaces_atributos_and_codigos product_repo_concurrent_updates_same_version_conflict product_repo_update_unknown_id_is_not_found product_repo_update_rejects_duplicate_sku` sale 0 (incluye **Listo cuando**)
- [x] 4.3 Confirmar non-goals: sin `set_activo` ni listado, sin HTTP, sin crate `catalog`, sin `openapi.yaml`, sin imágenes, sin tocar `error.rs` — `rg -n "fn set_activo|fn list" vortex-core/crates/storage/src/producto.rs vortex-core/crates/domain/src/producto.rs` sin matches de producto; `git diff --name-only` sin `docs/openapi.yaml` ni `vortex-core/crates/api` ni `vortex-core/crates/catalog` ni `vortex-core/crates/domain/src/error.rs`

## 5. Cierre (regresión)

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 5.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 5.3 `openspec validate T-02-19` sale 0