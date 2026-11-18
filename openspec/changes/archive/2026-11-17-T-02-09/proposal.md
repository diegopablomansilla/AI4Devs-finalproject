# Proposal

## Ticket padre

- **Ticket:** [T-02-09](../../../docs/7-tickets.md) — Marca: casos de uso y endpoints
- **Change:** `T-02-09`
- **HU:** [HU-02-02](../../../docs/6-historias.md#hu-02-02--marcas)
- **Depende de:** T-02-08 (en `develop`: entidad `Marca`, puerto `BrandRepo`, adaptador `MarcaRepo` + outbox); T-09-04 (`assert_conforms` contra `docs/openapi.yaml`, en `develop`)
- **Hace:** casos de uso en `catalog` y `GET`/`POST /api/v1/brands`, `PATCH`/`DELETE /api/v1/brands/{brandId}`; la baja con productos asociados está permitida.
- **No hace:** UI.
- **Listo cuando:** respuestas validadas contra `openapi.yaml`.

## Why

HU-02-02 exige alta de "Fixser" y baja lógica que deja inactiva la marca sin romper referencias. T-02-08 ya persiste el maestro plano; T-02-06 ya dejó el patrón de Gateway (`AppState` + fachada object-safe + `assert_conforms`). Sin casos de uso en `catalog` ni rutas `/api/v1/brands` no hay contrato HTTP para T-02-10 ni para el escenario de baja con productos asociados.

## What Changes

- Casos de uso en el crate `catalog`: crear, editar (nombre y slug opcional), listar, dar de baja lógica y reactivar sobre `BrandRepo`
- Baja lógica **sin** occupancy: con productos asociados (cuando existan, T-02-14) la marca queda inactiva y las referencias se conservan (HU-02-02); no hay `409` por ocupación
- Fachada object-safe `BrandGateway` para inyectar `Arc<dyn …>` en `AppState`
- Repositorio en memoria para tests de `catalog` (happy + unhappy)
- Registrar `GET`/`POST /api/v1/brands` y `PATCH`/`DELETE /api/v1/brands/{brandId}` en el router de `api`
- DTO inglés (`Brand`, `BrandCreate`, `BrandUpdate`) ↔ `Marca` / `NewMarca` / `UpdateMarca` según [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos); `errors[].field` en inglés (`name`, `brandId`, …)
- Tests de integración de `api` (happy + unhappy) con fake inyectado; cada JSON de éxito o `problem+json` pasa por `assert_conforms` (T-09-04)
- Alinear `docs/openapi.yaml`: declarar `503` en las operaciones de `/api/v1/brands` (mismo patrón que categorías); **no** agregar `409` en `DELETE` (la baja con productos está permitida)
- Composition root: `vortex serve` cablea `MarcaRepo` y lo inyecta en `AppState`

## Non-goals

- UI de listado, alta, edición y baja/reactivación — **T-02-10**
- Logos de marca — **T-02-10** (No hace de HU-02-02 / T-02-10)
- Tabla `producto` y occupancy real — **T-02-14** / **T-02-18** (este ticket **permite** la baja; no hay puerto de ocupación)
- Worker *fan-out* de marca a Meilisearch — **T-03-08**
- Seed de marcas — **T-02-42**
- `GET /api/v1/brands/{brandId}` (no está en OpenAPI)
- `DELETE` físico; ampliar `DomainError`; cambiar `BrandRepo` / migración `marca`
- Codegen desde OpenAPI (PAPI-04)
- Sidecars, Meilisearch, e2e Playwright, UI vanilla

## Capabilities

### New Capabilities

- `catalog-marca`: casos de uso de marcas en `catalog` (crear, editar, listar, baja, reactivar) con fake en memoria; baja sin occupancy; fachada object-safe para el Gateway

### Modified Capabilities

- `vortex-api`: registrar `/api/v1/brands`; DTO 5.3; `AppState` con servicio de marcas; tests de integración contra `openapi.yaml`; `vortex serve` inyecta el servicio; `503` en el contrato OpenAPI de marcas

## Impact

- `vortex-core/crates/catalog/` (`marca.rs` + reexport; fake en tests)
- `vortex-core/crates/api/` (handlers, DTO, `AppState`, tests de integración)
- `vortex-core/bin/vortex/src/serve.rs` (composition root)
- `docs/openapi.yaml` (`503` en operaciones de brands)
- Sin cambios de allowlist (`api` y `vortex` ya MAY depender de `catalog`; `MarcaRepo` ya posee `Arc<Db>`)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-09 (Listo cuando: respuestas validadas contra `openapi.yaml`)
- Alcance: cabe en ~2 h (el andamiaje de T-02-05/T-02-06 ya existe; marcas son un maestro plano sin árbol ni H-10); no se divide
