# Tasks

## 1. Tests (red) — happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/categoria.rs` con `migrate_applies_0002_categoria` (escenario «Migración aplica esquema categoria») — `cargo test -p storage --test categoria migrate_applies_0002` falla o no compila
- [x] 1.2 Tests `migrate_rerun_keeps_single_0002_record` y `categoria_allows_same_slug_under_different_padres` (escenarios «Re-arranque no duplica el registro de migración» y «Mismo slug bajo padres distintos») — `cargo test -p storage --test categoria` sigue en rojo

## 2. Tests (red) — unhappy

- [x] 2.1 Test `categoria_rejects_duplicate_padre_slug` (escenario «Índice único (padre, slug) rechaza duplicado»; **Listo cuando**) — `cargo test -p storage --test categoria categoria_rejects_duplicate_padre_slug` falla
- [x] 2.2 Test `categoria_rejects_empty_nombre` (escenario «nombre vacío rechazado») — `cargo test -p storage --test categoria categoria_rejects_empty_nombre` falla

## 3. Implementación (green)

- [x] 3.1 Crear `migrations/0002_categoria.surql` (tabla 4.4.1, `DEFINE … IF NOT EXISTS`, `CHANGEFEED 3d`, índices `uq_categoria_slug` e `idx_categoria_ancestros`) y registrar `MIGRATION_0002_SQL` en `EMBEDDED` — `cargo test -p storage --test categoria` en verde
- [x] 3.2 Ajustar `tests/migrate.rs` y `tests/harness.rs` para que `list_migrations` exija `version` 0 y 1 y acepte `version` 2 (checksums de `0000`/`0001` intactos) — `cargo test -p storage` en verde

## 4. Cierre

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `openspec validate T-02-01` sin errores
