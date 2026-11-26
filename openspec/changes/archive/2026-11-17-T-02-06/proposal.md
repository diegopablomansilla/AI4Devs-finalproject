# Proposal

## Ticket padre

- **Ticket:** [T-02-06](../../../docs/7-tickets.md) — Endpoints `/categories`
- **Change:** `T-02-06`
- **HU:** [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías)
- **Depende de:** T-02-05 (casos de uso en `catalog`, en `develop`); T-09-04 (`assert_conforms` contra `docs/openapi.yaml`, en `develop`)
- **Hace:** `GET`/`POST /api/v1/categories` y `PATCH`/`DELETE /api/v1/categories/{categoryId}`; DTO inglés ↔ dominio según [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos).
- **No hace:** UI.
- **Listo cuando:** tests de integración de `api` validan cada respuesta, incluidos errores, contra `openapi.yaml`.

## Why

HU-02-01 necesita el árbol de categorías por HTTP. T-02-05 ya orquesta crear, editar, mover, baja (H-10) y reactivar en `catalog`, y T-09-04 ya valida cuerpos contra OpenAPI, pero el Gateway sigue sin rutas de negocio bajo `/api/v1`. Sin este change no hay contrato HTTP para T-02-07 ni para el escenario `category_cycle` / `duplicate_slug` de la historia.

## What Changes

- Registrar `GET`/`POST /api/v1/categories` y `PATCH`/`DELETE /api/v1/categories/{categoryId}` en el router de `api`
- DTO inglés (`Category`, `CategoryCreate`, `CategoryUpdate`) ↔ `Categoria` / `NewCategoria` según 5.3; `errors[].field` en inglés (`name`, `parentId`, …)
- `AppState` inyecta el servicio de categorías como `Arc<dyn …>`; el Gateway llama a `catalog::CategoryService`, no a `CategoryRepo`
- Tests de integración de `api` (happy + unhappy) con fake inyectado; cada JSON de éxito o `problem+json` pasa por `assert_conforms` (T-09-04)
- Alinear `DELETE` en `docs/openapi.yaml` con H-10: respuesta `409` (`Conflict`)
- Composition root: `vortex serve` cablea `CategoriaRepo` + occupancy stub (`false` hasta T-02-18) y lo inyecta en `AppState`
- Allowlist: `api` y `vortex` MAY declarar path `catalog`

## Non-goals

- UI de árbol, selector de padre, toasts por `code` — **T-02-07**
- Reindexar productos / worker *fan-out* — **T-03-08**
- Consulta real de productos activos (occupancy) — **T-02-14** / **T-02-18** (stub `false`; H-10 de productos no rige en HTTP hasta entonces)
- `GET /api/v1/categories/{categoryId}` (no está en OpenAPI)
- Marca, `atributo_def`, producto, seed — **T-02-09** / **T-02-12** / **T-02-21** / **T-02-42**
- `DELETE` físico; arrastrar y soltar
- Codegen desde OpenAPI (PAPI-04)
- Ampliar `DomainError` o casos de uso de `catalog` (slug en `UpdateCategoria` no existe; PATCH `slug` se rechaza)
- Sidecars, Meilisearch, e2e Playwright

## Capabilities

### New Capabilities

- Ninguna. Los endpoints de categorías son el primer recurso de negocio del Gateway (`vortex-api`).

### Modified Capabilities

- `vortex-api`: registrar `/api/v1/categories`; DTO 5.3; `AppState` con servicio de categorías; tests de integración contra `openapi.yaml`
- `catalog-categoria`: fachada object-safe para el Gateway y occupancy stub de producción (`has_active_products` = `false`)
- `cargo-workspace`: `api` MAY depender de `catalog`; `vortex` MAY depender de `catalog`; la suite workspace incluye los tests HTTP de categorías
- `vortex-storage`: `CategoriaRepo` posee `Arc<Db>` (`'static`) para inyectarse en `AppState`

## Impact

- `vortex-core/crates/api/` (handlers, DTO, `AppState`, tests de integración)
- `vortex-core/crates/catalog/` (trait object-safe + stub occupancy)
- `vortex-core/crates/storage/src/categoria.rs` (`Arc<Db>`)
- `vortex-core/bin/vortex/src/serve.rs` (composition root)
- `docs/openapi.yaml` (`DELETE` + `409`)
- `scripts/check-workspace.sh` y fixtures de allowlist
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-06 (Listo cuando: tests de `api` validan cada respuesta, incluidos errores, contra `openapi.yaml`)
- Alcance: cabe en ~2 h (handlers + DTO + fake + OpenAPI 409 + allowlist + wiring); no se divide
