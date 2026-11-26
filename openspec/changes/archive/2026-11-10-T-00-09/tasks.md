# Tasks

## 1. Tests (red) — migrate happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/migrate.rs` con `migrate_first_install_records_0000` — falla o no compila
- [x] 1.2 Test `migrate_rerun_is_noop` — en rojo

## 2. Tests (red) — migrate unhappy

- [x] 2.1 Test `migrate_rejects_altered_checksum` — falla: sin `MigrateError` con versión

## 3. Tests (red) — allowlist sha2

- [x] 3.1 Actualizar fixture unhappy: `storage` con `sha2` MUST pasar tras allowlist; `axum` sigue fallando

## 4. Harness (green) — allowlist y pin sha2

- [x] 4.1 Pin `sha2` en `vortex-core/Cargo.toml`; extender `storage_deps_allowed` y spec delta
- [x] 4.2 Actualizar `run-script-tests.sh` si hace falta

## 5. Implementación (green)

- [x] 5.1 `migrations/0000_migracion.surql`, módulo migrate, `MigrateError`, `Db::migrate` / `apply_migrations`, checksum SHA-256
- [x] 5.2 `cargo test -p storage --test migrate` en verde

## 6. Cierre

- [x] 6.1 § 5 en `vortex-core/`, `check-workspace.sh`, `check-domain-deps.sh`
- [x] 6.2 `openspec validate T-00-09`
