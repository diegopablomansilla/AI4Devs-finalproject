# Proposal

## Ticket padre

- **Ticket:** [T-04-03](../../../docs/7-tickets.md) — Repositorio: crear relación simple y bidireccional
- **Change:** `T-04-03`
- **HU:** [HU-04-01](../../../docs/6-historias.md#hu-04-01--crear-relación-entre-productos), [HU-04-02](../../../docs/6-historias.md#hu-04-02--relaciones-bidireccionales)
- **Depende de:** T-04-01, T-04-02 (en `develop`: tablas `similar_a` / `sustituible_por` / `complementa`; entidad `Relacion`, `NewRelacion` y puerto `RelationRepo` sin adaptador)
- **Hace:** `RELATE` simple, o dos aristas con el mismo `grupo_bidireccional` en una transacción; índice único → `duplicate_relation`.
- **No hace:** edición.
- **Listo cuando:** tests: "A similar B" crea A→B y B→A; repetir da `duplicate_relation`.

## Why

HU-04-01 y HU-04-02 necesitan persistir aristas del grafo (simple o el par bidireccional en un solo paso) y rechazar duplicados con `duplicate_relation`. El dominio y el esquema ya existen; sin el adaptador SurrealDB de `RelationRepo::create` no hay persistencia hexagonal para T-04-04 ni para el HTTP de T-04-05.

## What Changes

- Adaptador `RelacionRepo` en el crate `storage` que implementa `domain::RelationRepo::create` contra las tablas ya migradas `similar_a`, `sustituible_por` y `complementa`
- `create` unidireccional: un `RELATE` origen→destino en una `Tx`
- `create` bidireccional: dos `RELATE` (A→B y B→A) en la misma transacción, mismo `grupo_bidireccional` (ULID); default del tipo (`similar_a` sí; `sustituible_por` / `complementa` no) o flag explícito
- Violación del índice único `(in, out)` → `DomainError::Conflict` `duplicate_relation` (extender el mapper de `Tx` para índices `uq_similar_a` / `uq_sustituible_por` / `uq_complementa`; el token `relation` del mapper actual no cubre esos nombres)
- `update` / `delete` / `list_for_product` quedan `Unavailable` (T-04-04)
- Tests de integración en `crates/storage/tests/` (happy Listo cuando: "A similar B" crea A→B y B→A; unhappy: repetir da `duplicate_relation`)
- TDD: red = tests que no compilan / fallan; green = `RelacionRepo` + `Tx`

## Non-goals

- `update` / `delete` que aplican al grupo bidireccional y `list_for_product` agrupado — **T-04-04** (No hace del ticket: edición)
- Endpoints HTTP de relaciones, casos de uso en `recommend` — **T-04-05**
- Consulta de recomendaciones (4.6.2) y endpoint — **T-04-06**, **T-04-07**
- UI de recomendaciones, ficha y carga rápida — **T-04-08**, **T-04-09**, **T-04-10** (carga rápida es v2)
- `vortex seed` de relaciones — **T-04-11**
- Ampliar `DomainError`, `Relacion`, `NewRelacion` o el puerto `RelationRepo` — ya cubiertos por **T-00-03** / **T-04-02**
- Evento de outbox al `RELATE` — el grafo no está en `OutboxEntity` ni en [4.8.1](../../../docs/4-modelo.md#481-outbox); T-04-06 lee SurrealDB
- HTTP, `openapi.yaml`, `tracing`, UI, crate `catalog` / `recommend`

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: adaptador `RelationRepo::create` (`RELATE` simple o par bidireccional en `Tx`; UNIQUE → `duplicate_relation`; mapper de índices `uq_*` de relación)

## Impact

- **Repo:** `vortex-core/crates/storage/` (`src/relacion.rs` nuevo, reexport en `lib.rs`, ajuste de `tx.rs` para UNIQUE de relaciones, tests en `tests/relacion_repo.rs`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p storage --test relacion_repo`; `openspec validate T-04-03`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml` ni de `domain` (el puerto `RelationRepo` ya declara `create`)
- **Ticket:** cierra T-04-03 (Listo cuando: "A similar B" crea A→B y B→A; repetir da `duplicate_relation`)
- **Ámbito:** un adaptador copiando `MarcaRepo` / `ProductoRepo` más el par bidireccional; cabe en ~2 h. No se divide el ticket.
