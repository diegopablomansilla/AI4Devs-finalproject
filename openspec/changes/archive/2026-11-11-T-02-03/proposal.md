# Proposal

## Ticket padre

- **Ticket:** [T-02-03](../../../docs/7-tickets.md) — Repositorio de categorías: alta, lectura, listado
- **Change:** `T-02-03`
- **HU:** [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías)
- **Depende de:** T-02-01, T-02-02, T-03-02 (en `develop`: tabla `categoria`, entidad/`CategoryRepo`, `storage::Tx` + outbox)
- **Hace:** `create` (calcula `ancestros` y `ruta` desde el padre), `get` y `list_all`, con evento de outbox; índice único → `duplicate_slug`.
- **No hace:** mover ni renombrar (T-02-04).
- **Listo cuando:** tests de integración crean "Fijaciones > Pernos" y verifican `ruta = ["Fijaciones", "Pernos"]`.

## Why

HU-02-01 exige persistir el árbol de categorías con ruta materializada y rechazar *slug* duplicado bajo el mismo padre. T-02-02 dejó el puerto `CategoryRepo` sin adaptador; T-02-01 dejó la tabla y T-03-02 el helper atómico dato + outbox. Sin `create`/`get`/`list_all` en `storage`, los casos de uso (T-02-05) y el alta "Fijaciones > Pernos" no tienen persistencia hexagonal.

## What Changes

- Adaptador SurrealDB de `CategoryRepo` en el crate `storage` (`create`, `get`, `list_all`)
- `create` calcula `ancestros` y `ruta` desde el padre (raíz: `ancestros` vacío, `ruta = [nombre]`; hija: ancestros del padre + padre, ruta del padre + nombre)
- Alta en la misma `Tx` que un `OutboxEvent` `categoria` / `upsert`
- Violación de `uq_categoria_slug` → `DomainError::Conflict` con `code` `duplicate_slug` (mapper de T-03-02)
- Tests de integración contra `storage::testing::test_db()` (happy "Fijaciones > Pernos" + unhappy `duplicate_slug`, `not_found`, padre ausente)
- TDD: red = tests de integración que no compilan / fallan; green = adaptador mínimo

## Non-goals

- `update`, mover, renombrar, recálculo de rama, `category_cycle` al mover — **T-02-04** (No hace del ticket)
- Casos de uso en `catalog`, HTTP `/categories`, UI de árbol — **T-02-05**, **T-02-06**, **T-02-07**
- Repositorio en memoria para casos de uso — **T-02-05**
- Reindexar productos / worker *fan-out* — **T-03-08**
- Tablas / entidades `marca`, `atributo_def`, `producto` — **T-02-08**, **T-02-11**, **T-02-14**
- Seed de categorías — **T-02-42**
- Ampliar `DomainError`, el puerto `CategoryRepo` (`update`) o la migración `0002` — ya cubiertos por **T-00-03** / **T-02-02** / **T-02-01**
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas crates en `domain` (el adaptador vive en `storage`)

## Capabilities

### New Capabilities

- (ninguna; el contrato de árbol ya está en `domain-categoria` y el esquema en `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: el crate `storage` implementa `CategoryRepo` (`create` calcula `ancestros`/`ruta`, `get`, `list_all`) con evento de outbox; UNIQUE `(padre, slug)` → `duplicate_slug`; se levanta el MUST NOT de T-03-02 / T-02-01 sobre repositorios de catálogo

## Impact

- **Repo:** `vortex-core/crates/storage/` (adaptador `CategoryRepo`, tests de integración). Sin cambios de API HTTP ni de `openapi.yaml`.
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p storage --test categoria`; `openspec validate T-02-03`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`; `check-domain-deps.sh` sigue en verde (`domain` no gana SDKs)
- **Ticket:** cierra T-02-03 (Listo cuando: tests de integración crean "Fijaciones > Pernos" y `ruta = ["Fijaciones", "Pernos"]`)
