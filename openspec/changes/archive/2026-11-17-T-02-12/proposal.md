# Proposal

## Ticket padre

- **Ticket:** [T-02-12](../../../docs/7-tickets.md) — `atributo_def`: casos de uso y endpoints
- **Change:** `T-02-12`
- **HU:** [HU-02-03](../../../docs/6-historias.md#hu-02-03--definición-de-atributos-técnicos)
- **Depende de:** T-02-11 (en `develop`: entidad `AtributoDef`, puerto `AttributeDefRepo`, adaptador `AtributoDefRepo` + outbox; UNIQUE `clave` → `conflict`); T-09-04 (`assert_conforms` contra `docs/openapi.yaml`, en `develop`)
- **Hace:** casos de uso y `GET`/`POST /api/v1/attribute-definitions`, `PATCH`/`DELETE …/{attributeId}`.
- **No hace:** UI.
- **Listo cuando:** respuestas validadas contra `openapi.yaml`.

## Why

HU-02-03 exige crear el atributo "grado" (`enum` 2/5/8, facetable) y rechazar un `enum` sin opciones con `invalid_input`. T-02-11 ya persiste el maestro; T-02-09/T-02-06 dejaron el patrón de Gateway (`AppState` + fachada object-safe + `assert_conforms`). Sin casos de uso en `catalog` ni rutas `/api/v1/attribute-definitions` no hay contrato HTTP para T-02-13 ni para el escenario de enum sin opciones.

## What Changes

- Casos de uso en el crate `catalog`: crear, editar (clave, etiqueta, tipo, unidad, opciones, facetable, buscable, orden), listar, dar de baja lógica y reactivar sobre `AttributeDefRepo`
- **Sin** `attribute_in_use`: no hay tabla `producto` (T-02-14) ni regla de ocupación (T-02-23); cambiar `key`/`type` de una definición existente MUST suceder; MUST NOT devolver `409 attribute_in_use`
- Fachada object-safe `AttributeDefGateway` para inyectar `Arc<dyn …>` en `AppState`
- Repositorio en memoria para tests de `catalog` (happy + unhappy)
- Registrar `GET`/`POST /api/v1/attribute-definitions` y `PATCH`/`DELETE /api/v1/attribute-definitions/{attributeId}` en el router de `api`
- DTO inglés (`AttributeDefinition`, `AttributeDefinitionCreate`, `AttributeDefinitionUpdate`) ↔ `AtributoDef` / `NewAtributoDef` / `UpdateAtributoDef` según [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos); enum de tipo `text`/`number`/`boolean`/`enum` ↔ `texto`/`numero`/`booleano`/`enum`; `errors[].field` en inglés (`key`, `label`, `type`, `options`, `attributeId`, …)
- Tests de integración de `api` (happy + unhappy) con fake inyectado; cada JSON de éxito o `problem+json` pasa por `assert_conforms` (T-09-04)
- Alinear `docs/openapi.yaml`: declarar `503` en las operaciones de `/api/v1/attribute-definitions` (mismo patrón que marcas); **no** implementar ni exigir `attribute_in_use` (el `409` de PATCH queda en el YAML para T-02-23)
- Composition root: `vortex serve` cablea `AtributoDefRepo` y lo inyecta en `AppState`

## Non-goals

- UI de listado, alta, edición y baja — **T-02-13**
- Regla `attribute_in_use` (cambio de `key`/`type` con productos) — **T-02-23**
- Validación de valores de producto contra `AtributoDef` — **T-02-16**
- Tabla `producto` — **T-02-14** / **T-02-18**
- Worker *fan-out* de atributo a Meilisearch / `filterableAttributes` — **T-03-08**
- Seed de atributos — **T-02-42**
- `GET /api/v1/attribute-definitions/{attributeId}` (no está en OpenAPI)
- `DELETE` físico; ampliar `DomainError`; cambiar `AttributeDefRepo` / migración `0004`
- Codegen desde OpenAPI (PAPI-04)
- Sidecars, Meilisearch, e2e Playwright, UI vanilla

## Capabilities

### New Capabilities

- `catalog-atributo-def`: casos de uso de definiciones de atributo en `catalog` (crear, editar, listar, baja, reactivar) con fake en memoria; sin `attribute_in_use`; fachada object-safe para el Gateway

### Modified Capabilities

- `vortex-api`: registrar `/api/v1/attribute-definitions`; DTO 5.3 (incl. mapeo de `tipo`); `AppState` con servicio de atributos; tests de integración contra `openapi.yaml`; `vortex serve` inyecta el servicio; `503` en el contrato OpenAPI de attribute-definitions

## Impact

- `vortex-core/crates/catalog/` (`atributo_def.rs` + reexport; fake en tests)
- `vortex-core/crates/api/` (handlers, DTO, `AppState`, tests de integración)
- `vortex-core/bin/vortex/src/serve.rs` (composition root)
- `docs/openapi.yaml` (`503` en operaciones de attribute-definitions)
- Sin cambios de allowlist (`api` y `vortex` ya MAY depender de `catalog`; `AtributoDefRepo` ya posee `Arc<Db>`)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-12 (Listo cuando: respuestas validadas contra `openapi.yaml`)
- Alcance: cabe en ~2 h (el andamiaje de T-02-09 ya existe; maestro plano como marcas, más mapeo de `tipo` y orden por `order`); no se divide
