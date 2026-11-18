# Proposal

## Ticket padre

- **Ticket:** [T-02-15](../../../docs/7-tickets.md) — Migración `movimiento_stock`
- **Change:** `T-02-15`
- **HU:** [HU-02-07](../../../docs/6-historias.md#hu-02-07--ingresos-y-ajustes-de-stock)
- **Depende de:** T-02-14 (en `develop`: tabla `producto` ya migrada)
- **Hace:** tabla de [4.5.1](../../../docs/4-modelo.md#451-movimiento_stock) con campos `READONLY`. El campo `venta` se define como `option<record>` genérico hasta que exista la tabla `venta` (T-08-01 lo ajusta).
- **No hace:** lógica.
- **Listo cuando:** test verifica que un `UPDATE` sobre un movimiento falla.

## Why

HU-02-07 necesita un ledger inmutable como fuente de verdad del stock: ingresos, ajustes y (más adelante) ventas no editan `producto.stock` a mano. `producto` ya existe; sin la tabla `movimiento_stock` de 4.5.1 no se puede persistir un movimiento, ni T-02-18 cargar stock inicial, ni T-02-24 implementar `StockLedger.append`.

## What Changes

- Migración embebida `0006_movimiento_stock.surql`: tabla `SCHEMAFULL` `movimiento_stock` según 4.5.1 (campos `READONLY`, `ASSERT` de `tipo` / `cantidad` / `stock_resultante`, índice `idx_mov_producto`, `CHANGEFEED 3d`), con `DEFINE … IF NOT EXISTS` (4.10)
- Campo `venta` como `option<record>` **genérico** (sin `record<venta>`): la tabla `venta` no existe hasta T-08-01 (v2 / fuera del MVP del curso). Se documenta la desviación temporal en [4.5.1](../../../docs/4-modelo.md#451-movimiento_stock)
- Registro de la migración en `EMBEDDED` para que `Db::migrate()` / `storage::testing::test_db()` la apliquen
- Tests de integración en `crates/storage/tests/` (happy: esquema + alta mínima de `ingreso`; unhappy: `UPDATE` rechazado — Listo cuando —, `tipo` fuera del enum, `cantidad = 0`, `stock_resultante < 0`)
- Ajuste de tests existentes (T-00-09 / T-09-02) que cuentan filas de `migracion` (`len() == 6`)
- TDD: red = tests de esquema sin `0006`; green = `.surql` + embed + nota en 4.5.1

## Non-goals

- Entidad `MovimientoStock`, reglas de signo por tipo, `motivo` obligatorio en `ajuste`, puerto `StockLedger` — **T-02-17** (No hace del ticket: lógica)
- `StockLedger.append` transaccional (movimiento + proyección + outbox) — **T-02-24**
- `ProductRepo::create` con stock inicial — **T-02-18**
- HTTP `POST/GET .../stock-movements`, UI de movimientos — **T-02-25**, **T-02-38**
- Historial paginado — **T-02-26** (v2, HU-02-08)
- Conciliación `vortex verify` — **T-02-45** (v2)
- Tabla `venta` y ajuste de `movimiento_stock.venta` a `option<record<venta>>` — **T-08-01** (v2)
- Permisos SurrealDB que nieguen `DELETE` de filas (READONLY cubre `UPDATE`; el puerto no expone borrar)
- HTTP, `openapi.yaml`, `tracing`, UI
- Ampliar `DomainError` o `storage::Tx` — ya cubiertos por **T-00-03** / **T-03-02**

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: migración `0006` con tabla `movimiento_stock`, campos `READONLY` de 4.5.1 (con `venta` genérico) e índice `idx_mov_producto`; tests de `UPDATE` rechazado, enum de `tipo`, `cantidad != 0` y `stock_resultante >= 0`

## Impact

- **Repo:** `vortex-core/crates/storage/migrations/0006_movimiento_stock.surql`, `src/migrate.rs` (`EMBEDDED`), `src/lib.rs` (re-export de `MIGRATION_0006_SQL`), `tests/` (nuevo `movimiento_stock.rs` + ajuste de `migrate.rs` / `harness.rs`); `docs/4-modelo.md` § 4.5.1 (nota `option<record>` hasta T-08-01)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p storage --test movimiento_stock`; `openspec validate T-02-15`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml`
- **Ticket:** cierra T-02-15 (Listo cuando: un `UPDATE` sobre un movimiento falla)
- **Ámbito:** migración de esquema + tests + nota en 4.5.1; cabe en ~2 h (mismo molde que T-02-14). No se divide el ticket.
