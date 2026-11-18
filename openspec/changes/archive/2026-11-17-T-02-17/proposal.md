# Proposal

## Ticket padre

- **Ticket:** [T-02-17](../../../docs/7-tickets.md) — Dominio Movimiento de stock y `StockLedger`
- **Change:** `T-02-17`
- **HU:** [HU-02-07](../../../docs/6-historias.md#hu-02-07--ingresos-y-ajustes-de-stock)
- **Depende de:** T-00-04 (en `develop`: `Id<T>`, `Decimal` con signo, `Timestamp`; T-00-03 ya expone `invalid_input` y `negative_stock`; T-02-15 dejó la tabla `movimiento_stock`; T-02-16 dejó `Producto` / `Id<Producto>`)
- **Hace:** entidad `MovimientoStock`, reglas de signo por tipo ([4.5.1](../../../docs/4-modelo.md#451-movimiento_stock)), `motivo` obligatorio en `ajuste`; puerto `StockLedger` que solo expone `append` y consultas.
- **No hace:** persistencia (T-02-24).
- **Listo cuando:** tests unitarios de cada regla.

## Why

HU-02-07 exige registrar ingresos y ajustes sobre un ledger inmutable, rechazar un `ajuste` sin `motivo` (`invalid_input`) y no permitir editar ni borrar un movimiento. La tabla `movimiento_stock` ya existe (T-02-15) y los tipos `Decimal`/`Id` ya cubren cantidades con signo (T-00-04); sin entidad, reglas de signo por `tipo` y el puerto `StockLedger`, T-02-18 y T-02-24 no tienen contrato hexagonal que persistir.

## What Changes

- Entidad `MovimientoStock` en el crate `domain` (campos de [4.5.1](../../../docs/4-modelo.md#451-movimiento_stock) con nombres en español: `id`, `producto`, `tipo`, `cantidad`, `stock_resultante`, `motivo`, `venta`, `terminal`, `creado_en`) con `Id<MovimientoStock>`
- Enum `TipoMovimiento` (`inicial` / `ingreso` / `ajuste` / `venta`); el wire API `initial`/`inbound`/`adjustment`/`sale` se traduce en T-02-25
- Validación pura de reglas de signo: `inicial` e `ingreso` → `cantidad > 0`; `venta` → `cantidad < 0`; `ajuste` → `cantidad ≠ 0` (positivo o negativo); `cantidad == 0` siempre inválido
- `motivo` obligatorio (no vacío tras recortar) cuando `tipo = ajuste`; opcional en el resto
- `venta` obligatorio cuando `tipo = venta`; ausente en los demás tipos
- `stock_resultante ≥ 0`; violación → `negative_stock`
- Puerto `StockLedger` (`append`, `list_by_producto`) en `domain`; sin `update`/`delete`, sin adaptador ni SurrealDB
- Tests unitarios en `crates/domain` (happy de cada tipo válido + unhappy Listo cuando: signo invertido, `ajuste` sin motivo, cantidad 0)
- TDD: red = tests que no compilan / fallan; green = entidad, enum, validación y trait

## Non-goals

- Persistencia de `StockLedger.append` (movimiento + `producto.stock` + outbox, `THROW "stock_negativo"`, concurrencia) — **T-02-24** (No hace del ticket)
- Alta de producto con movimiento `inicial` — **T-02-18**
- HTTP `POST /api/v1/products/{productId}/stock-movements` (`inbound`/`adjustment`, `X-Vectra-Terminal`) — **T-02-25**
- Historial paginado `GET .../stock-movements` — **T-02-26** (v2) / cursor — **T-00-21**
- UI pestaña Stock — **T-02-38**
- Ventas POS y movimiento `venta` persistido — **T-08-03** / **T-08-01**
- Conciliación `vortex verify` — **T-02-45** (v2)
- Ampliar `DomainError`, `Id<T>` o `Decimal` — ya cubiertos por **T-00-03** y **T-00-04** (`negative_stock` ya existe; no se toca `error.rs`)
- Cambiar la migración `0006_movimiento_stock` — **T-02-15** (ya en `vortex-storage`)
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas crates en `domain` — la allowlist de T-00-04 se mantiene

## Capabilities

### New Capabilities

- `domain-movimiento-stock`: entidad `MovimientoStock`, `TipoMovimiento`, reglas de signo y `motivo`/`venta` por tipo, y puerto `StockLedger` (`append`, `list_by_producto`) en el crate `domain`

### Modified Capabilities

- (ninguna; se reutilizan `domain-types`, `domain-error` y `domain-producto` sin cambiar requisitos)

## Impact

- **Repo:** `vortex-core/crates/domain/` (`src/lib.rs`, módulo `movimiento_stock`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `openspec validate T-02-17`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`; `check-domain-deps.sh` y `check-workspace.sh` siguen en verde (sin deps nuevas)
- **Sin** cambios de API HTTP ni de `openapi.yaml` ni de `storage`
- **Ticket:** cierra T-02-17 (Listo cuando: tests unitarios de cada regla de signo y de `motivo` obligatorio en `ajuste`)
- **Ámbito:** dominio puro como T-02-16; cabe en ~2 h. No se divide el ticket.
