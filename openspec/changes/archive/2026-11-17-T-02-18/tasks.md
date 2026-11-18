# Tasks

## 1. Tests (red)

- [x] 1.1 Agregar helper de seed (categoría existente) y test `product_repo_creates_perno_with_stock_50` en `vortex-core/crates/storage/tests/producto.rs` — `cargo test -p storage --test producto product_repo_creates_perno_with_stock_50` no compila o falla (escenario «Alta con stock inicial 50»; **Listo cuando**)
- [x] 1.2 Agregar tests happy `product_repo_create_zero_stock_has_no_movimiento` y `product_repo_get_returns_persisted` — `cargo test -p storage --test producto product_repo_create_zero_stock product_repo_get_returns` falla (escenarios «Alta con stock inicial 0», «get de producto existente»)
- [x] 1.3 Agregar tests unhappy `product_repo_rejects_duplicate_sku`, `product_repo_rejects_negative_stock_inicial` y `product_repo_get_unknown_id_is_not_found` — `cargo test -p storage --test producto product_repo_rejects product_repo_get_unknown` falla (escenarios «SKU duplicado», «stock_inicial negativo», «get de id inexistente»)

## 2. Implementación (green) — `ProductoRepo`

- [x] 2.1 Crear `vortex-core/crates/storage/src/producto.rs` con `ProductoRepo { db: Arc<Db> }` que implementa `get` (DTO `serde`, strip de prefijos, `Producto::try_new`, `not_found` si no hay fila) y reexportar desde `lib.rs` — `cargo test -p storage --test producto product_repo_get_returns_persisted product_repo_get_unknown_id_is_not_found` sale 0
- [x] 2.2 Implementar `create`: rechazar `stock_inicial < 0` y sku/nombre vacíos (`invalid_input`); `Tx` con `CREATE producto` (sin escribir `stock`; `version` 1), movimiento `inicial` + `UPDATE stock` solo si `stock_inicial > 0`, `OutboxEvent` `producto`/`upsert` (`motivo` `"stock"` o ausente); UNIQUE → `duplicate_sku` — `cargo test -p storage --test producto product_repo_creates_perno_with_stock_50 product_repo_create_zero_stock_has_no_movimiento product_repo_rejects_duplicate_sku product_repo_rejects_negative_stock_inicial` sale 0
- [x] 2.3 Confirmar non-goals: sin `update`/`set_activo`, sin `StockLedger`, sin HTTP, sin crate `catalog`, sin cambios en `openapi.yaml` ni `DomainError` ni `NewProducto` — `git diff --name-only` sin `docs/openapi.yaml` ni `vortex-core/crates/api` ni `vortex-core/crates/catalog` ni `vortex-core/crates/domain`

## 3. Cierre (regresión)

- [x] 3.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 3.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 3.3 `openspec validate T-02-18` sale 0