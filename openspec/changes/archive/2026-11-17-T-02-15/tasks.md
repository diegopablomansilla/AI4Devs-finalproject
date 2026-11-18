# Tasks

## 1. Tests (red) — happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/movimiento_stock.rs` con `migrate_applies_0006_movimiento_stock` (escenario «Migración aplica esquema movimiento_stock») — `cargo test -p storage --test movimiento_stock migrate_applies_0006` falla o no compila
- [x] 1.2 Tests `migrate_rerun_keeps_single_0006_record` y `movimiento_stock_creates_ingreso` (escenarios «Re-arranque no duplica el registro de migración» y «Alta mínima de movimiento ingreso») — `cargo test -p storage --test movimiento_stock migrate_rerun movimiento_stock_creates_ingreso` sigue en rojo

## 2. Tests (red) — unhappy

- [x] 2.1 Test `movimiento_stock_rejects_update` (escenario «UPDATE sobre un movimiento falla»; **Listo cuando**) — `cargo test -p storage --test movimiento_stock movimiento_stock_rejects_update` falla
- [x] 2.2 Tests `movimiento_stock_rejects_invalid_tipo`, `movimiento_stock_rejects_zero_cantidad` y `movimiento_stock_rejects_negative_stock_resultante` (escenarios «tipo fuera del enum», «cantidad cero» y «stock_resultante negativo») — `cargo test -p storage --test movimiento_stock movimiento_stock_rejects_invalid_tipo movimiento_stock_rejects_zero_cantidad movimiento_stock_rejects_negative_stock_resultante` falla

## 3. Implementación (green)

- [x] 3.1 Crear `migrations/0006_movimiento_stock.surql` (tabla 4.5.1, campos `READONLY`, `DEFINE … IF NOT EXISTS`, `CHANGEFEED 3d`, `venta` como `option<record>`, índice `idx_mov_producto` no UNIQUE) y registrar `MIGRATION_0006_SQL` en `EMBEDDED` — `cargo test -p storage --test movimiento_stock` en verde (si `READONLY` no rechaza `UPDATE`, aplicar el fallback de design.md decisión 3)
- [x] 3.2 Documentar en `docs/4-modelo.md` § 4.5.1 que hasta T-08-01 `venta` es `option<record>` genérico — la nota queda junto al snippet de 4.5.1
- [x] 3.3 Ajustar `tests/migrate.rs` y `tests/harness.rs` para que `list_migrations` exija `version` 0–6 (`len() == 7`, checksum de `0006`) — `cargo test -p storage --test migrate --test harness` en verde

## 4. Cierre (regresión)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 Confirmar non-goals: sin entidad `MovimientoStock`, sin `StockLedger`, sin HTTP ni cambios en `openapi.yaml` — `git diff --name-only` sin `vortex-core/crates/domain`, `vortex-core/crates/catalog`, `vortex-core/crates/api` ni `docs/openapi.yaml`
- [x] 4.3 `openspec validate T-02-15` sale 0