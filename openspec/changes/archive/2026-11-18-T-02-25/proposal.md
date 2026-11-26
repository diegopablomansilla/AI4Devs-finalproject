# Proposal

## Ticket padre

- **Ticket:** [T-02-25](../../../docs/7-tickets.md) — Endpoint `POST .../stock-movements`
- **Change:** `T-02-25`
- **HU:** [HU-02-07](../../../docs/6-historias.md#hu-02-07--ingresos-y-ajustes-de-stock)
- **Depende de:** T-02-24 (`StockLedger.append` transaccional en `develop`), T-02-21 (HTTP de ficha + `X-Vectra-Terminal` + `assert_conforms`)
- **Hace:** `POST /api/v1/products/{productId}/stock-movements` solo para `inbound` y `adjustment`; guarda `X-Vectra-Terminal`.
- **No hace:** historial (T-02-26).
- **Listo cuando:** escenarios de HU-02-07 como tests de `api` validados contra `openapi.yaml`.

## Why

HU-02-07 exige registrar un ingreso (stock 10 + 20 → 30, terminal `"caja-1"`), rechazar un ajuste sin motivo (`invalid_input`), rechazar un ajuste que deje stock negativo (`negative_stock`, stock intacto) y no ofrecer edición ni borrado de un movimiento. T-02-24 ya persiste el ledger; el Gateway aún no expone `POST .../stock-movements`, no traduce `inbound`/`adjustment` ni reenvía `X-Vectra-Terminal`. Sin este endpoint, T-02-38 no tiene API que consumir.

## What Changes

- `POST /api/v1/products/{productId}/stock-movements` → 201 `StockMovement`; tipos permitidos solo `inbound` y `adjustment`
- Cabecera opcional `X-Vectra-Terminal` (máx. 64) persistida en `movimiento_stock.terminal`
- DTO inglés ↔ dominio español ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `type` `inbound`→`ingreso`, `adjustment`→`ajuste`; `quantity`/`reason`/`resultingStock`/`productId`
- Caso de uso en `catalog` que delega en `StockLedger::append` y rechaza `inicial` / `venta` (alta y POS son otros tickets)
- Tests de `api` con `assert_conforms` que cubren los escenarios de HU-02-07 (Ingreso, Ajuste sin motivo, Stock negativo, Inmutabilidad)
- `docs/openapi.yaml`: declarar `503` en `POST` (mismo patrón que productos); MUST NOT implementar `GET`

## Non-goals

- Historial paginado `GET .../stock-movements` — **T-02-26** (v2; No hace del ticket)
- UI de movimientos (pestaña Stock) — **T-02-38**
- Confirmar venta / tipo `sale` — **T-08-03**
- Movimiento `initial` (sigue en `ProductRepo::create`) — **T-02-18** / **T-02-21**
- Paginación por cursor — **T-00-21**
- `vortex verify` / conciliación del ledger — **T-02-45** (v2)
- Reindexación Meilisearch del stock — **T-03-07**
- Ampliar el puerto `StockLedger` (`update`/`delete`, cursor) o reescribir `append`
- Codegen OpenAPI, e2e Playwright, sidecars

## Capabilities

### New Capabilities

- `catalog-stock`: caso de uso de registrar ingreso/ajuste sobre `StockLedger` (fachada object-safe para el Gateway; sin HTTP ni SurrealDB)

### Modified Capabilities

- `vortex-api`: registrar `POST /api/v1/products/{productId}/stock-movements`; DTO 5.3; `X-Vectra-Terminal`; inyectar `StockGateway`; tests `assert_conforms` de HU-02-07; MUST NOT registrar `GET` de historial

## Impact

- `vortex-core/crates/catalog/` (`src/stock.rs` nuevo, `lib.rs`)
- `vortex-core/crates/api/` (`src/stock.rs` nuevo, `lib.rs` router/`AppState`, `error.rs` remap `cantidad`/`motivo`, `tests/stock_movements.rs`, fake en `tests/support/`)
- `vortex-core/bin/vortex/src/serve.rs` (inyectar `StockService` + `MovimientoStockRepo`)
- `docs/openapi.yaml`: `'503'` en `POST .../stock-movements`; MUST NOT tocar `GET` ni esquemas de página
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-25 (Listo cuando: escenarios HU-02-07 como tests de `api` contra `openapi.yaml`)
- Ámbito: HTTP + caso de uso delgado + wiring; cabe en ~2 h. No se divide el ticket.
