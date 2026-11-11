# Proposal

## Ticket padre

- **Ticket:** [T-02-05](../../../docs/7-tickets.md) — Casos de uso de categorías
- **Change:** `T-02-05`
- **HU:** [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías)
- **Depende de:** T-02-04 (en `develop`: `CategoryRepo::update` con recálculo de rama y `category_cycle`)
- **Hace:** en `catalog`: crear, editar, mover, dar de baja y reactivar; la baja se rechaza con `409 conflict` si tiene subcategorías o productos activos (hallazgo H-10).
- **No hace:** HTTP.
- **Listo cuando:** tests con repositorio en memoria cubren cada caso y la baja rechazada.

## Why

HU-02-01 exige crear, editar, mover y dar de baja el árbol de categorías. El puerto y el adaptador Surreal ya persisten alta, lectura, listado y `update`; sin casos de uso en `catalog` no hay orquestación hexagonal (H-10 incluido) antes de los endpoints (T-02-06).

## What Changes

- Casos de uso en el crate `catalog`: crear, editar (renombrar), mover, dar de baja lógica y reactivar
- Puerto de aplicación `CategoryOccupancy` (`has_active_products`) inyectado en el servicio; la baja consulta subcategorías activas vía `list_all` y productos activos vía ese puerto
- Baja rechazada con `DomainError::Conflict` `code` `conflict` (H-10; HTTP `409` es T-02-06)
- `CategoryRepo::set_activo` en `domain` y adaptador en `storage` (el trait no compilaba el workspace si solo el fake lo implementa)
- Repositorio en memoria para tests de `catalog` (Listo cuando)
- Allowlist de `cargo-workspace`: `catalog` puede depender de `domain` y de `tokio` en tests

## Non-goals

- HTTP `/api/v1/categories`, DTO inglés, `openapi.yaml`, tests de `api` — **T-02-06**
- UI de árbol — **T-02-07**
- Reindexar productos / worker *fan-out* — **T-03-08**
- Tabla/entidad `producto` y consulta Surreal de productos activos — **T-02-14** / **T-02-18** (el fake de occupancy cubre H-10 en este ticket)
- Marca, `atributo_def`, seed — **T-02-08** / **T-02-11** / **T-02-42**
- Arrastrar y soltar, `DELETE` físico
- Ampliar `DomainError` con un `code` nuevo (`category_in_use` no existe en 5.4; se reutiliza `conflict`)
- Spans de `tracing` en `catalog` (el Gateway ya instrumenta en T-02-06)

## Capabilities

### New Capabilities

- `catalog-categoria`: casos de uso de categorías en `catalog` (crear, editar, mover, baja, reactivar) con fake en memoria y rechazo H-10

### Modified Capabilities

- `domain-categoria`: `CategoryRepo::set_activo`
- `vortex-storage`: adaptador `set_activo` (UPDATE `activo` + outbox `upsert`)
- `cargo-workspace`: `catalog` MAY declarar `domain` y `tokio` (dev)

## Impact

- `vortex-core/crates/catalog/` (primer código de producto del crate), `vortex-core/crates/domain/src/categoria.rs`, `vortex-core/crates/storage/src/categoria.rs`
- Sin cambios de API HTTP ni de `openapi.yaml`
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-05 (Listo cuando: tests en memoria cubren cada caso y la baja rechazada)
