# Proposal

## Ticket padre

- **Ticket:** [T-02-04](../../../docs/7-tickets.md) — Repositorio de categorías: mover, renombrar, ciclos
- **Change:** `T-02-04`
- **HU:** [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías)
- **Depende de:** T-02-03 (en `develop`)
- **Hace:** `update` que, al cambiar nombre o padre, recalcula `ancestros` y `ruta` de toda la rama en una transacción con un evento por categoría afectada; mover debajo de un descendiente → `category_cycle`.
- **No hace:** reindexar productos (T-03-08).
- **Listo cuando:** tests cubren renombrar una rama de 3 niveles y el intento de ciclo.

## Why

HU-02-01 exige renombrar una rama y evitar ciclos al mover. T-02-03 dejó `create`/`get`/`list_all`; sin `update` no hay persistencia de mover ni renombrar antes de T-02-05.

## What Changes

- `UpdateCategoria` y `CategoryRepo::update` en `domain`
- Adaptador `update` en `storage` con recálculo de subárbol, `reject_cycle`, outbox por categoría afectada
- Tests de integración (rama 3 niveles + unhappy `category_cycle`, `not_found`, `duplicate_slug`, profundidad)

## Non-goals

- Worker Meilisearch / fan-out — **T-03-08**
- Casos de uso `catalog`, HTTP, UI — **T-02-05** / **T-02-06** / **T-02-07**
- Baja/reactivar `activo` en este ticket
- Migración `0002` nueva, cambios `openapi.yaml`

## Capabilities

### Modified Capabilities

- `domain-categoria`: puerto `update`
- `vortex-storage`: implementación `update` y tests

## Impact

- `vortex-core/crates/domain/src/categoria.rs`, `vortex-core/crates/storage/src/categoria.rs`, `vortex-core/crates/storage/tests/categoria.rs`
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
