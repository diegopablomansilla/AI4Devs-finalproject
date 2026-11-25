# Proposal

## Ticket padre

- **Ticket:** [T-04-01](../../../docs/7-tickets.md) — Migración de relaciones (`sustituible_por`)
- **Change:** `T-04-01`
- **HU:** [HU-04-01](../../../docs/6-historias.md#hu-04-01--crear-relación-entre-productos)
- **Depende de:** T-02-14 (en `develop`: tabla `producto` ya migrada)
- **Hace:** migración SurrealDB con tablas `similar_a`, `sustituible_por` y `complementa` según [4.6.1](../../../docs/4-modelo.md#461-tablas-de-relación) (PM-05; nombre ya documentado).
- **No hace:** lógica.
- **Listo cuando:** tests: `ENFORCED` rechaza un destino inexistente y el índice único `(in, out)` rechaza duplicados.

## Why

HU-04-01 necesita aristas de grafo entre productos para curar similares, sustitutos y complementarios. `producto` ya existe; sin las tablas `TYPE RELATION` de 4.6.1 no se puede persistir una arista, ni T-04-03 implementar `RELATE`, ni T-04-06 consultar recomendaciones. El nombre `sustituible_por` (PM-05) ya está en el modelo; el contrato HTTP sigue usando `substitute` (T-04-05).

## What Changes

- Migración embebida `0007_relaciones.surql`: tablas `SCHEMAFULL` `TYPE RELATION IN producto OUT producto ENFORCED` `similar_a`, `sustituible_por` y `complementa` según 4.6.1 (campos `prioridad`, `nota`, `grupo_bidireccional`, `creado_en`, `actualizado_en`, índices únicos `(in, out)`, `CHANGEFEED 3d`), con `DEFINE … IF NOT EXISTS` (4.10)
- Registro de la migración en `EMBEDDED` para que `Db::migrate()` / `storage::testing::test_db()` la apliquen
- Tests de integración en `crates/storage/tests/` (happy: esquema + `RELATE` entre dos productos existentes en las tres tablas; unhappy: `ENFORCED` con destino inexistente y UNIQUE `(in, out)` — Listo cuando —, `prioridad < 1`)
- Ajuste de tests existentes (T-00-09 / T-09-02) que cuentan filas de `migracion` (`len() == 7`)
- TDD: red = tests de esquema sin `0007`; green = `.surql` + embed

## Non-goals

- Entidad Relación, tipos `similar`/`substitute`/`complement`, bidireccionalidad por defecto, `prioridad ≥ 1` y `nota` ≤ 500 en dominio, `self_relation`, puerto `RelationRepo` — **T-04-02** (No hace del ticket: lógica)
- `RELATE` simple o bidireccional en transacción, mapeo UNIQUE → `duplicate_relation` — **T-04-03**
- Editar, eliminar y listar relaciones — **T-04-04**
- Endpoints HTTP de relaciones — **T-04-05**
- Consulta de recomendaciones (4.6.2) y endpoint — **T-04-06**, **T-04-07**
- UI de recomendaciones, ficha y carga rápida — **T-04-08**, **T-04-09**, **T-04-10** (carga rápida es v2)
- `vortex seed` de relaciones — **T-04-11**
- Rechazo de `in = out` a nivel de esquema (4.6.1 lo deja en dominio / T-04-02)
- HTTP, `openapi.yaml`, `tracing`, UI
- Ampliar `DomainError` o `storage::Tx` — ya cubiertos por **T-00-03** / **T-03-02**

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: migración `0007` con tablas `similar_a`, `sustituible_por` y `complementa`; tests de `ENFORCED` (destino inexistente) e índice único `(in, out)` (duplicados)

## Impact

- **Repo:** `vortex-core/crates/storage/migrations/0007_relaciones.surql`, `src/migrate.rs` (`EMBEDDED`), `src/lib.rs` (re-export de `MIGRATION_0007_SQL`), `tests/` (nuevo `relaciones.rs` + ajuste de `migrate.rs` / `harness.rs`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p storage --test relaciones`; `openspec validate T-04-01`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml`
- **Ticket:** cierra T-04-01 (Listo cuando: `ENFORCED` rechaza destino inexistente; UNIQUE `(in, out)` rechaza duplicados)
- **Ámbito:** migración de esquema + tests; cabe en ~2 h (mismo molde que T-02-14 / T-02-15). No se divide el ticket.
