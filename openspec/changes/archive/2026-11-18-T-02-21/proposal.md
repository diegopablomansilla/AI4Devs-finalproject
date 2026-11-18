# Proposal

## Ticket padre

- **Ticket:** [T-02-21](../../../docs/7-tickets.md) — Endpoints `POST/GET/PATCH /products`
- **Change:** `T-02-21`
- **HU:** [HU-02-04](../../../docs/6-historias.md#hu-02-04--alta-de-producto), [HU-02-05](../../../docs/6-historias.md#hu-02-05--edición-con-bloqueo-optimista)
- **Depende de:** T-02-20 (en `develop`: `ProductService` / `ProductGateway` crear, obtener, editar), T-09-04 (`assert_conforms`)
- **Hace:** `POST /api/v1/products` (201 + `Location`, `X-Vectra-Terminal` para el movimiento inicial), `GET` y `PATCH /api/v1/products/{productId}`; mapeo de enums y decimales como string según [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos).
- **No hace:** listado (T-02-27).
- **Listo cuando:** respuestas y errores (`duplicate_sku`, `invalid_input`, `version_conflict`) validados contra `openapi.yaml`.

## Why

HU-02-04 exige el alta HTTP de "Perno hexagonal M8 x 40 zincado" con 201, movimiento `initial` y rechazo `duplicate_sku` / `invalid_input`. HU-02-05 exige `PATCH` con `version` y `409 version_conflict`. T-02-20 dejó `ProductGateway` sin HTTP: el Gateway aún no registra `/products`, no traduce inglés ↔ español ni valida contra `openapi.yaml`. Sin estos endpoints, T-02-22 / T-02-25 / T-02-35 no tienen ficha que consumir.

## What Changes

- `POST /api/v1/products` → 201 + cabecera `Location: /api/v1/products/{id}` y cuerpo `Product`; cabecera opcional `X-Vectra-Terminal` (máx. 64) se persiste en el movimiento `inicial` si hay `initialStock` > 0
- `GET /api/v1/products/{productId}` → 200 `Product` o 404 `not_found`
- `PATCH /api/v1/products/{productId}` → 200 `Product` con `version` incrementada; `version` desactualizada → 409 `version_conflict`
- DTO inglés (`sku`, `name`, `productType`, `brandId`, `categoryId`, `unitOfMeasure`, `price` string, `codes[].type`, …) ↔ dominio español ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos))
- `errors[].field` en inglés (`categoryId`, `brandId`, `productId`, `attributes[n].value`, …)
- `NewProducto.terminal: Option<String>` y escritura en `movimiento_stock.terminal` al alta (T-02-18 lo dejó para este ticket)
- Tests de `api` con `assert_conforms` (T-09-04); inyección de `ProductGateway` en `AppState` y `vortex serve`

## Non-goals

- Listado admin paginado `GET /api/v1/products` — **T-02-27** / **T-00-21**
- Baja y reactivación (`DELETE`, `PATCH { active: true }`) — **T-02-22**
- `POST .../stock-movements` (ingresos/ajustes) — **T-02-25** (el movimiento `inicial` ya lo crea T-02-18)
- Historial de movimientos — **T-02-26**
- Imágenes (`images` no vacío, `POST .../images`) — **T-02-28** / **T-02-30** (v2)
- Worker Meilisearch / aparición en búsqueda en &lt; 1 s — **T-03-07**
- UI de alta / conflicto de versión — **T-02-35** / **T-02-37**
- Paginación por cursor — **T-00-21**
- `StockLedger.append` transaccional — **T-02-24**
- `vortex seed` de productos — **T-02-43**
- Cambiar reglas de `ProductService` (validación de defs / categoría / marca activas) — ya cerradas en T-02-20

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `vortex-api`: registrar `POST /api/v1/products` y `GET`/`PATCH /api/v1/products/{productId}`; DTO 5.3; `Location` y `X-Vectra-Terminal`; inyectar `ProductGateway`; tests `assert_conforms`
- `domain-producto`: `NewProducto` gana `terminal: Option<String>` para el movimiento inicial
- `vortex-storage`: el `CREATE movimiento_stock` de `ProductRepo::create` persiste `terminal` cuando viene en `NewProducto`
- `catalog-producto`: el caso de uso de crear reenvía `terminal` al puerto (sin validar HTTP ni escribir el ledger)

## Impact

- `vortex-core/crates/api/` (`src/products.rs`, `lib.rs` router/`AppState`, `error.rs` remap de campos, `tests/products.rs`, `tests/support/fake_catalog.rs`)
- `vortex-core/bin/vortex/src/serve.rs` (inyectar `ProductService` + `ProductoRepo`)
- `vortex-core/crates/domain/src/producto.rs` (`NewProducto.terminal`)
- `vortex-core/crates/storage/src/producto.rs` (SQL del movimiento `inicial`)
- `docs/openapi.yaml`: declarar `503` en las tres operaciones (mismo patrón que marcas/categorías); MUST NOT añadir `GET` de listado ni `DELETE`
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-21 (Listo cuando: respuestas y errores validados contra `openapi.yaml`)
- Ámbito: HTTP + mapeo 5.3 + plomería de `terminal` que T-02-18 diferió; cabe en ~2 h. No se divide el ticket.
