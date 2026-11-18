# Proposal

## Ticket padre

- **Ticket:** [T-02-18](../../../docs/7-tickets.md) — Repositorio de producto: alta con stock inicial
- **Change:** `T-02-18`
- **HU:** [HU-02-04](../../../docs/6-historias.md#hu-02-04--alta-de-producto)
- **Depende de:** T-02-15, T-02-16, T-02-17 (en `develop`: tabla `producto` + `uq_producto_sku`; tabla `movimiento_stock`; entidad `Producto` / `NewProducto` / puerto `ProductRepo`; entidad `MovimientoStock` y puerto `StockLedger` sin adaptador)
- **Hace:** `ProductRepo::create` en una transacción: `CREATE producto`, y si `initialStock > 0`, movimiento `inicial` + `producto.stock` + evento de outbox; índice único → `duplicate_sku`.
- **No hace:** edición (T-02-19).
- **Listo cuando:** test crea un producto con stock 50 y verifica `version = 1`, el movimiento y el evento.

## Why

HU-02-04 exige dar de alta un producto con ficha y stock inicial 50, dejarlo activo con `version` 1 y registrar un movimiento `inicial`. El dominio y el esquema ya existen; sin el adaptador SurrealDB de `ProductRepo::create` (transacción atómica ficha + ledger + outbox, UNIQUE → `duplicate_sku`) no hay persistencia hexagonal para T-02-19 ni para el HTTP de T-02-21.

## What Changes

- Adaptador `ProductoRepo` en el crate `storage` que implementa `domain::ProductRepo` (`create`, `get` básico) contra las tablas ya migradas `producto` y `movimiento_stock`
- `create` en una `Tx`: `CREATE producto` (`version = 1`, `activo = true`, `stock` inicial 0); si `stock_inicial > 0`, `CREATE movimiento_stock` tipo `inicial` + `UPDATE producto.stock`; siempre un `OutboxEvent` `producto`/`upsert`
- Violación de `uq_producto_sku` → `DomainError::Conflict` `duplicate_sku` (mapper de `Tx` ya existente); sin segunda fila, movimiento ni evento
- `get` hidrata el `Producto` persistido (IDs, no ficha resuelta); id ausente → `not_found`
- Tests de integración en `crates/storage/tests/` (happy Listo cuando: stock 50 + `version` 1 + movimiento + outbox; unhappy: SKU duplicado y `stock_inicial` negativo)
- TDD: red = tests que no compilan / fallan; green = `ProductoRepo` + `Tx`

## Non-goals

- `get` hidratado (marca, ruta de categoría, atributos resueltos) y `update` con `version` — **T-02-19** (No hace del ticket: edición)
- Casos de uso en `catalog` (existencia/activo de categoría y marca; validación de atributos contra defs cargadas) — **T-02-20**
- HTTP `POST`/`GET`/`PATCH /api/v1/products`, `X-Vectra-Terminal`, mapeo inglés ↔ español — **T-02-21**
- Baja/reactivación — **T-02-22**
- `StockLedger.append` transaccional para ingresos/ajustes — **T-02-24**
- Listado admin paginado — **T-02-27** / **T-00-21**
- Imágenes de producto — **T-02-28** (v2)
- Worker Meilisearch / aparición en búsqueda — **T-03-07**
- Ampliar `DomainError`, `NewProducto` (`terminal`) o el puerto `StockLedger` — ya cubiertos por **T-00-03** / **T-02-16** / **T-02-17**
- HTTP, `openapi.yaml`, `tracing`, UI, crate `catalog`

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: adaptador `ProductRepo` (`create` transaccional con movimiento `inicial` y outbox; UNIQUE `sku` → `duplicate_sku`; `get` básico)

## Impact

- **Repo:** `vortex-core/crates/storage/` (`src/producto.rs` nuevo, reexport en `lib.rs`, tests en `tests/producto.rs`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p storage --test producto`; `openspec validate T-02-18`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml` ni de `domain` (el puerto `ProductRepo` ya declara `create`/`get` y `stock_inicial`)
- **Ticket:** cierra T-02-18 (Listo cuando: alta con stock 50, `version = 1`, movimiento `inicial` y evento de outbox)
- **Ámbito:** un adaptador copiando `MarcaRepo` / `AtributoDefRepo` más el ledger inicial; cabe en ~2 h. No se divide el ticket.
