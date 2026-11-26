# Tasks

## 1. Tests (red) — harness happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/harness.rs` con `test_db_applies_embedded_migrations` — no compila o falla sin `storage::testing`
- [x] 1.2 Tests `test_db_rocks_applies_embedded_migrations`, `test_db_instances_are_isolated` — en rojo

## 2. Tests (red) — harness unhappy

- [x] 2.1 Tests `test_db_has_no_business_tables`, `open_still_rejects_file_path` — en rojo

## 3. Tests (red) — allowlist kv-mem

- [x] 3.1 Ajustar `check-workspace.sh` / fixtures si el contrato exige ambas features en `surrealdb`

## 4. Implementación (green)

- [x] 4.1 `kv-mem` en `storage/Cargo.toml`; `src/testing.rs`, bootstrap compartido, `Db::query` / `record_count`
- [x] 4.2 `cargo test -p storage` en verde (open, migrate, harness)

## 5. Cierre

- [x] 5.1 § 5 en `vortex-core/`, `check-workspace.sh`, `openspec validate T-09-02`
