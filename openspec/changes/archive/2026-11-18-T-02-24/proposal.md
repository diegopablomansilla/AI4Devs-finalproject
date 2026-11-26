# Proposal

## Ticket padre

- **Ticket:** [T-02-24](../../../docs/7-tickets.md) — `StockLedger.append` transaccional
- **Change:** `T-02-24`
- **HU:** [HU-02-07](../../../docs/6-historias.md#hu-02-07--ingresos-y-ajustes-de-stock) (escenarios Ingreso, Stock negativo, Inmutabilidad)
- **Depende de:** T-02-17 (entidad `MovimientoStock` y puerto `StockLedger` sin adaptador); T-02-15 (tabla `movimiento_stock` READONLY); T-03-02 (`Tx` dato + outbox; `THROW "stock_negativo"` → `negative_stock`)
- **Hace:** implementación de [4.5.2](../../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción): movimiento + `producto.stock` + evento (`motivo: "stock"`) en una transacción; `THROW "stock_negativo"` → `negative_stock`; no incrementa `version`.
- **No hace:** ventas (T-08-03).
- **Listo cuando:** tests cubren stock 5 con ajuste −8 (error y stock intacto) y dos ingresos concurrentes (ambos aplicados).

## Why

HU-02-07 exige registrar ingresos y ajustes sobre un ledger inmutable, rechazar un ajuste que deje stock negativo (`negative_stock`, stock intacto) y no editar ni borrar un movimiento. El puerto `StockLedger` y la tabla ya existen, pero `append` no persiste: sin la transacción 4.5.2, T-02-25 no puede exponer `POST .../stock-movements` y dos ingresos concurrentes podrían perder actualizaciones si el stock se calcula fuera del motor.

## What Changes

- Adaptador SurrealDB de `StockLedger` en `storage` (`append` + `list_by_producto`)
- `append` ejecuta en una `Tx` (T-03-02) el script 4.5.2: `LET $nuevo = $producto.stock + $cantidad`; si `$nuevo < 0` → `THROW "stock_negativo"`; `CREATE movimiento_stock`; `UPDATE producto SET stock = $nuevo` **sin** `version += 1`; `OutboxEvent` `producto`/`upsert` con `motivo: "stock"`
- `stock_resultante` lo calcula el motor (no un read-modify-write en Rust)
- Validación de dominio **antes** de abrir `Tx` (signo por tipo, `motivo` en `ajuste`, `venta` solo en tipo `venta`)
- `list_by_producto` lee el ledger (sin cursor; T-02-26 pagina en HTTP)
- Tests TDD de integración: ingreso + terminal, ajuste −8 sobre stock 5, dos ingresos concurrentes, producto inexistente, `ajuste` sin motivo

## Non-goals

- Confirmar venta atómica (`contador:venta`, tabla `venta`, líneas múltiples) — **T-08-03** (No hace del ticket)
- Endpoint `POST /api/v1/products/{productId}/stock-movements` ni mapeo `inbound`/`adjustment` — **T-02-25**
- Historial paginado `GET .../stock-movements` — **T-02-26** (v2)
- UI de movimientos — **T-02-38**
- `vortex verify` / conciliación `Σ cantidad = producto.stock` — **T-02-45** (v2)
- Reescribir `ProductRepo::create` para reusar `StockLedger` en el movimiento `inicial` (sigue inline, T-02-18)
- Casos de uso en `catalog`, HTTP, `openapi.yaml`, Meilisearch, UI
- Ampliar `DomainError` (`negative_stock` y el mapper de `Tx` ya existen)
- Paginación por cursor (T-00-21) o `update`/`delete` del ledger (inmutabilidad HU-02-07)
- Codegen OpenAPI, e2e Playwright, sidecars

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `vortex-storage`: adaptador `StockLedger` (`append` transaccional 4.5.2 + `list_by_producto`); `ProductRepo` deja de vetar `append` (el movimiento `inicial` de `create` sigue inline)

## Impact

- `vortex-core/crates/storage/src/movimiento_stock.rs` (nuevo) + `lib.rs` (`mod` / `pub use`)
- `vortex-core/crates/storage/tests/stock_ledger.rs` (nuevo; sembrar producto vía `ProductoRepo`)
- `vortex-core/crates/domain`: sin cambio de puerto ni de `DomainError`
- `vortex-core/crates/catalog`, `api`, `docs/openapi.yaml`: sin cambio (T-02-25)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-24 (Listo cuando: ajuste −8 sobre stock 5; dos ingresos concurrentes)
- Ámbito: un adaptador + tests de integración; cabe en ~2 h. No se divide el ticket.
