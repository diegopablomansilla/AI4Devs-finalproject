# Proposal

## Ticket padre

- **Ticket:** [T-09-02](../../../docs/7-tickets.md) — *Harness* de tests de integración de `storage`
- **Change:** `T-09-02`
- **HU:** [HU-09-01](../../../docs/6-historias.md#hu-09-01--tests-de-integración-contra-motores-reales)
- **Depende de:** T-00-09 (en `develop`)
- **Hace:** `storage::testing::test_db()` (motor en memoria con todas las migraciones) y variante RocksDB en directorio temporal; convención de tests en `crates/storage/tests/`.
- **No hace:** tests de tablas concretas (cada migración trae los suyos).
- **Listo cuando:** un test de ejemplo usa el *harness* y pasa en CI.

## Why

T-00-08 y T-00-09 dejaron `Db::open` (RocksDB) y `Db::migrate()` con tests propios que abren y migran a mano. HU-09-01 y los tickets de catálogo necesitan una forma estándar de obtener una base SurrealDB con esquema al día: rápida en memoria (`kv-mem`, PM-03) y opcionalmente en disco. Sin ese harness, cada crate repetiría bootstrap y rompería la convención PC-01.

## What Changes

- Módulo `storage::testing` con `test_db()` (`Mem` + `migrate()`) y `test_db_rocks()` (RocksDB temporal + `migrate()` + limpieza al soltar)
- Feature `kv-mem` en `surrealdb` del crate `storage` (junto a `kv-rocksdb`; producción sigue usando solo `Db::open`)
- `vortex-core/crates/storage/tests/harness.rs`: test de ejemplo happy + unhappy que usa el harness
- **BREAKING** (harness interno): `check-workspace.sh` exige `kv-mem` y `kv-rocksdb` en la línea `surrealdb` de `storage`
- TDD: red = `harness.rs` sin módulo `testing`; green = helpers + migraciones aplicadas

## Non-goals

- Tests de tablas de negocio (`categoria`, `outbox`, …) — cada ticket de migración
- Meilisearch / Ollama / Whisper en CI — T-09-05, T-09-13, T-09-18
- Cablear harness en `vortex serve` — T-00-20
- Crate `tempfile` (no está en workspace dependencies)
- Refactor masivo de `tests/open.rs` / `tests/migrate.rs`
- Cambios a `docs/openapi.yaml`

## Capabilities

### New Capabilities

- (ninguna carpeta nueva bajo `specs/`)

### Modified Capabilities

- `vortex-storage`: módulo `testing`, harness mem/RocksDB migrado, convención `crates/storage/tests/`
- `cargo-workspace`: `storage` MAY activar `kv-mem` además de `kv-rocksdb` en `surrealdb`

## Impact

- **Repo:** `vortex-core/crates/storage/` (`src/testing.rs`, `lib.rs`, `Cargo.toml`, `tests/harness.rs`), `scripts/check-workspace.sh`, spec deltas
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `openspec validate T-09-02`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace` incluyendo `harness.rs`
