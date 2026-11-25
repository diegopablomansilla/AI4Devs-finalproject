# Proposal

## Ticket padre

- **Ticket:** [T-04-02](../../../docs/7-tickets.md) — Dominio Relación y `RelationRepo`
- **Change:** `T-04-02`
- **HU:** [HU-04-01](../../../docs/6-historias.md#hu-04-01--crear-relación-entre-productos), [HU-04-02](../../../docs/6-historias.md#hu-04-02--relaciones-bidireccionales)
- **Depende de:** T-00-03 (en `develop`: `DomainError` ya expone `self_relation` y `duplicate_relation`; T-00-04 dejó `Id<T>` / `Timestamp`; T-02-16 dejó `Producto` / `Id<Producto>`; T-04-01 dejó las tablas `TYPE RELATION`)
- **Hace:** tipos `similar`/`substitute`/`complement`, bidireccionalidad por defecto por tipo (`similar` sí, los demás no), `prioridad ≥ 1`, `nota` ≤ 500, `self_relation`; puerto `RelationRepo` en `domain`, usado por `recommend` (DC-05).
- **No hace:** persistencia.
- **Listo cuando:** tests unitarios de cada regla.

## Why

HU-04-01 y HU-04-02 necesitan una arista de grafo con tipo, prioridad, nota y default de bidireccionalidad, más el rechazo de relacionar un producto consigo mismo (`self_relation`). Las tablas ya existen (T-04-01) y `DomainError` ya reserva los `code`; sin entidad, reglas puras y el puerto hexagonal, T-04-03 no tiene contrato que persistir y `recommend` no puede consumir el agregado (DC-05).

## What Changes

- Entidad `Relacion` en el crate `domain` (campos de [4.6.1](../../../docs/4-modelo.md#461-tablas-de-relación) con nombres en español: `id`, `origen`, `destino`, `tipo`, `prioridad`, `nota`, `grupo_bidireccional`, `creado_en`, `actualizado_en`) con `Id<Relacion>`
- Enum `TipoRelacion` (`similar_a` / `sustituible_por` / `complementa`); el wire API `similar`/`substitute`/`complement` se traduce en T-04-05 ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos))
- Función pura `default_bidireccional(tipo)`: `similar_a` → `true`; `sustituible_por` y `complementa` → `false` (HU-04-02)
- Validación pura: `origen ≠ destino` → `self_relation`; `prioridad ≥ 1`; `nota` opcional de longitud ≤ 500
- Puerto `RelationRepo` (`create`, `update`, `delete`, `list_for_product`) en `domain`; sin adaptador ni SurrealDB. El consumidor previsto es `recommend` (DC-05), no `catalog`
- Tests unitarios en `crates/domain` (happy de cada tipo + default bidireccional; unhappy Listo cuando: `self_relation`, `prioridad < 1`, `nota` > 500, tipo en inglés del contrato)
- TDD: red = tests que no compilan / fallan; green = entidad, enum, validación y trait

## Non-goals

- Persistencia de `RELATE` simple o bidireccional, índice UNIQUE → `duplicate_relation` — **T-04-03** (No hace del ticket: persistencia)
- Editar, eliminar y listar agrupado en el adaptador SurrealDB — **T-04-04**
- Casos de uso en `recommend` y HTTP `POST/PATCH/DELETE /api/v1/relations` — **T-04-05**
- Consulta de recomendaciones (4.6.2) y endpoint — **T-04-06**, **T-04-07**
- UI de recomendaciones, ficha y carga rápida — **T-04-08**, **T-04-09**, **T-04-10** (carga rápida es v2)
- `vortex seed` de relaciones — **T-04-11**
- Dependencia `domain` en el crate `recommend` y allowlist de `cargo-workspace` — **T-04-05** (`recommend` sigue placeholder vacío; DC-05 se documenta en el puerto)
- Ampliar `DomainError` — ya cubierto por **T-00-03** (`self_relation` y `duplicate_relation` existen; no se toca `error.rs`)
- Cambiar la migración `0007_relaciones` — **T-04-01** (ya en `vortex-storage`)
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas crates en `domain` — la allowlist de T-00-04 se mantiene

## Capabilities

### New Capabilities

- `domain-relacion`: entidad `Relacion`, `TipoRelacion`, default de bidireccionalidad por tipo, reglas `self_relation` / `prioridad` / `nota`, y puerto `RelationRepo` en el crate `domain`

### Modified Capabilities

- (ninguna; se reutilizan `domain-types`, `domain-error` y `domain-producto` sin cambiar requisitos)

## Impact

- **Repo:** `vortex-core/crates/domain/` (`src/lib.rs`, módulo `relacion`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `openspec validate T-04-02`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`; `check-domain-deps.sh` y `check-workspace.sh` siguen en verde (sin deps nuevas; `recommend` sigue vacío)
- **Sin** cambios de API HTTP ni de `openapi.yaml` ni de `storage` ni de `recommend`
- **Ticket:** cierra T-04-02 (Listo cuando: tests unitarios de cada regla de tipo, default bidireccional, `prioridad`, `nota` y `self_relation`)
- **Ámbito:** dominio puro como T-02-16 / T-02-17; cabe en ~2 h. No se divide el ticket.
