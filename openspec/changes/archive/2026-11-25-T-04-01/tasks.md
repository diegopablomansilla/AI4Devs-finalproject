# Tasks

## 1. Tests (red) — happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/relaciones.rs` con `migrate_applies_0007_relaciones` (escenario «Migración aplica esquema de relaciones») — `cargo test -p storage --test relaciones migrate_applies_0007` falla o no compila
- [x] 1.2 Tests `migrate_rerun_keeps_single_0007_record` y `relacion_relates_existing_products` (escenarios «Re-arranque no duplica el registro de migración» y «RELATE entre dos productos existentes») — `cargo test -p storage --test relaciones migrate_rerun relacion_relates_existing` sigue en rojo
- [x] 1.3 Test `relacion_allows_same_pair_on_distinct_tables` (escenario «mismo par en tablas distintas») — `cargo test -p storage --test relaciones relacion_allows_same_pair` falla

## 2. Tests (red) — unhappy

- [x] 2.1 Test `relacion_enforced_rejects_missing_target` (escenario «ENFORCED rechaza destino inexistente»; **Listo cuando**) — `cargo test -p storage --test relaciones relacion_enforced_rejects_missing_target` falla
- [x] 2.2 Test `relacion_unique_rejects_duplicate_edge` (escenario «índice único (in, out) rechaza duplicado»; **Listo cuando**) — `cargo test -p storage --test relaciones relacion_unique_rejects_duplicate_edge` falla
- [x] 2.3 Test `relacion_rejects_prioridad_below_one` (escenario «prioridad menor que 1 rechazada») — `cargo test -p storage --test relaciones relacion_rejects_prioridad_below_one` falla

## 3. Implementación (green)

- [x] 3.1 Crear `migrations/0007_relaciones.surql` (tablas 4.6.1 `similar_a`, `sustituible_por`, `complementa`; `TYPE RELATION IN producto OUT producto ENFORCED`; `DEFINE … IF NOT EXISTS`; `CHANGEFEED 3d`; índices `uq_similar_a`, `uq_sustituible_por`, `uq_complementa`) y registrar `MIGRATION_0007_SQL` en `EMBEDDED` — `cargo test -p storage --test relaciones` en verde
- [x] 3.2 Ajustar `tests/migrate.rs` y `tests/harness.rs` para que `list_migrations` exija `version` 0–7 (`len() == 8`, checksum de `0007`) — `cargo test -p storage --test migrate --test harness` en verde

## 4. Cierre (regresión)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 Confirmar non-goals: sin entidad Relación, sin `RelationRepo`, sin HTTP ni cambios en `openapi.yaml` — `git diff --name-only` sin `vortex-core/crates/domain`, `vortex-core/crates/recommend`, `vortex-core/crates/api` ni `docs/openapi.yaml`
- [x] 4.3 `openspec validate T-04-01` sale 0