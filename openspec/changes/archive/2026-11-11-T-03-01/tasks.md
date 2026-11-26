# Tasks

## 1. Tests (red) — happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/outbox.rs` con `migrate_applies_0001_outbox_and_seeds_cursors` (escenario «Migración aplica esquema y siembra cursores») — `cargo test -p storage --test outbox migrate_applies_0001` falla o no compila
- [x] 1.2 Tests `migrate_rerun_does_not_duplicate_cursors` y `outbox_accepts_valid_entidad_and_operacion` (escenarios «Re-arranque no duplica cursores» y «Evento válido se inserta») — `cargo test -p storage --test outbox` sigue en rojo

## 2. Tests (red) — unhappy

- [x] 2.1 Tests `outbox_rejects_invalid_entidad` y `outbox_rejects_invalid_operacion` (escenarios «entidad inválida» / «operacion inválida»; **Listo cuando**) — `cargo test -p storage --test outbox outbox_rejects` falla
- [x] 2.2 Test `outbox_fallo_rejects_invalid_consumidor` (escenario «consumidor inválido en outbox_fallo») — `cargo test -p storage --test outbox outbox_fallo_rejects_invalid_consumidor` falla

## 3. Implementación (green)

- [x] 3.1 Crear `migrations/0001_outbox.surql` (tablas 4.8, `DEFINE … IF NOT EXISTS`, seed de `sync_cursor:meilisearch` y `sync_cursor:vectores`) y registrar `MIGRATION_0001_SQL` en `EMBEDDED` — `cargo test -p storage --test outbox` en verde
- [x] 3.2 Ajustar `tests/migrate.rs` y `tests/harness.rs` para que `list_migrations` exija `version` 0 y acepte `version` 1 (checksum de `0000` intacto) — `cargo test -p storage` en verde

## 4. Cierre

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `openspec validate T-03-01` sin errores
