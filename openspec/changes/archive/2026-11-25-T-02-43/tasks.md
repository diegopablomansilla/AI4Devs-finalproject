# Tasks

## 1. Tests (red) — orquestador de productos en `catalog`

- [x] 1.1 Agregar `seed_import_products_creates_producto_and_stock_inicial` y `seed_import_products_zero_stock_has_no_inicial_movement` en `vortex-core/crates/catalog/src/seed.rs` (fixture `pernos`/`fixser`/`medida` + `PER-M8X40-ZN`; repos in-memory compartidos entre `import_masters` y `ProductService`; asertar `stock` `"50"` vs `"0"` y resumen `importados`/`rechazados`) — `cargo test -p catalog seed_import_products_creates_producto_and_stock_inicial seed_import_products_zero_stock_has_no_inicial_movement` falla (API inexistente; escenarios happy de primera carga y stock cero)
- [x] 1.2 Agregar `seed_import_products_second_run_does_not_duplicate`, `seed_import_dev_dataset_imports_all_skus` y `seed_import_products_does_not_persist_relaciones_imagenes` — `cargo test -p catalog seed_import_products_second_run_does_not_duplicate seed_import_dev_dataset_imports_all_skus seed_import_products_does_not_persist_relaciones_imagenes` falla (idempotencia, Listo cuando dataset ~200 SKUs, No hace relaciones/imágenes)
- [x] 1.3 Agregar `seed_import_products_rejects_non_numeric_precio_only`, `seed_import_products_rejects_malformed_line_only`, `seed_import_products_rejects_unknown_categoria_only` y `seed_import_products_rejects_missing_productos_file` — `cargo test -p catalog seed_import_products_rejects_` falla (unhappy: issue `archivo`/`linea`/`campo`; el SKU válido queda; el inválido no; maestros no se revierten)

## 2. Implementación (green) — `import_products`

- [x] 2.1 Ampliar `ProductoRow` con campos de `NewProducto`; parsear `productos.jsonl`; devolver `SeedProductSummary` sin llamar a `create` si hay issue de parseo/SKU duplicado en archivo o si falta el JSONL — pasan 1.3; `cargo test -p catalog seed_import_products_rejects_` verde
- [x] 2.2 Completar `import_products`: mapas slug/clave → id vía `list_all`; paginar `ProductGateway::list` (activo true y false) para skip-if-exists; un `create` por SKU nuevo; no tocar relaciones/imágenes; dataset `seed/` del repo — pasan 1.1 y 1.2; `cargo test -p catalog seed_import_products_ seed_import_dev_dataset_imports_all_skus` verde (Listo cuando in-memory)
- [x] 2.3 Conservar verdes de maestros y `validate_dir` — `cargo test -p catalog seed_import_masters_ seed_valid_minimal_directory_has_no_issues seed_repo_seed_dir_has_no_issues` verde

## 3. Tests (red) — CLI `vortex seed` con productos

- [x] 3.1 Extender `write_master_seed` en `vortex-core/bin/vortex/tests/seed.rs` con `productos.jsonl` mínimo (`PER-M8X40-ZN`, `stock_inicial` `"50"`) y agregar `vortex_seed_imports_masters_and_products_exit_0` y `vortex_seed_twice_does_not_duplicate_products` (reabrir `Db`; `ProductGateway::list` + `MovimientoStockRepo::list_by_producto`; reexportar desde `catalog` `FiltroProducto`/`TipoMovimiento` si hace falta, sin añadir `domain` a `vortex`) — `cargo test -p vortex --test seed vortex_seed_imports_masters_and_products_exit_0 vortex_seed_twice_does_not_duplicate_products` falla (aún no persiste productos ni `inicial`; escenarios happy CLI + idempotencia)
- [x] 3.2 Agregar `vortex_seed_dev_dataset_exit_0` (DIR = `seed/` del repo, 180–220 productos, exit 0) y `vortex_seed_invalid_precio_rejected_alone` (copia del dataset + `precio` no numérico: exit ≠ 0, issue `productos.jsonl`/`precio`, SKUs válidos persistidos, el inválido no) — `cargo test -p vortex --test seed vortex_seed_dev_dataset_exit_0 vortex_seed_invalid_precio_rejected_alone` falla (Listo cuando CLI happy + unhappy)

## 4. Implementación (green) — despacho CLI y README

- [x] 4.1 Tras `import_masters` Ok, construir `ProductService`/`ProductoRepo` y llamar `import_products`; imprimir issues y resumen `importados`/`rechazados`; exit 0 iff `rechazados == 0` — pasan 3.1 y 3.2; `cargo test -p vortex --test seed --test cli` verde; lock/config unhappy de T-02-42 siguen ≠ 0
- [x] 4.2 Actualizar `seed/README.md`: importación MVP = maestros **y** productos con movimiento `inicial`; imágenes/relaciones/sinónimos/notas siguen pendientes — escenario README de `seed-dataset`; el README no afirma «solo maestros»

## 5. Verificación de cierre

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 5.2 `openspec validate T-02-43` verde