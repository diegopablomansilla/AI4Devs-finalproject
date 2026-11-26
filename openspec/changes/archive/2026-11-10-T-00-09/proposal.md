# Proposal

## Ticket padre

- **Ticket:** [T-00-09](../../../docs/7-tickets.md) — Motor de migraciones con *checksum*
- **Change:** `T-00-09`
- **HU:** [HU-00-02](../../../docs/6-historias.md#hu-00-02--surrealdb-embebido-con-migraciones-versionadas)
- **Depende de:** T-00-08 (en `develop`)
- **Hace:** archivos `crates/storage/migrations/NNNN_nombre.surql` embebidos; migración `0000` tabla `migracion` ([4.10](../../../docs/4-modelo.md#410-operación-del-esquema)); aplicación en orden, registro SHA-256; corte si cambió el checksum de una migración ya aplicada
- **No hace:** tablas de negocio
- **Listo cuando:** tests cubren primera instalación, re-arranque sin cambios y migración alterada (error que nombra la versión)

## Why

T-00-08 abre RocksDB y fija `NS vectra` / `DB comercio` sin esquema versionado. HU-00-02 y PC-01 exigen migraciones al arrancar con registro en `migracion` y detección de drift por checksum. T-00-20, T-09-02 y las migraciones de catálogo/outbox dependen de este motor.

## What Changes

- Directorio `vortex-core/crates/storage/migrations/` con `0000_migracion.surql` embebido vía `include_str!`
- `Db::migrate()` aplica migraciones embebidas en orden; `Db::open` sigue sin migrar (T-00-08)
- Tabla `migracion` según 4.10; fila por migración aplicada (`version`, `nombre`, `checksum` SHA-256 hex del archivo, `aplicada_en`)
- `MigrateError` tipado; mismatch de checksum MUST incluir el número de versión en `Display`
- Pin `sha2` en workspace y allowlist `storage` en `check-workspace.sh`
- TDD: happy = primera instalación + re-arranque; unhappy = checksum alterado

## Non-goals

- `storage::testing::test_db()`, `kv-mem` — **T-09-02**
- Cablear `vortex serve` / composición — **T-00-20**
- Migraciones `categoria`, `outbox`, etc. — **T-02-01**, **T-03-01**
- HTTP, `problem+json`, `tracing` en fallo de migración — **T-00-11** / logs en **T-00-20**
- Ampliar `DomainError` — **T-00-03**
- Cambios a `openapi.yaml`

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: motor de migraciones embebidas, `0000_migracion`, checksum SHA-256
- `cargo-workspace`: pin `sha2`; allowlist `storage` incluye `sha2`

## Impact

- **Repo:** `vortex-core/crates/storage/` (migrations, `migrate`, tests), `vortex-core/Cargo.toml`, `Cargo.lock`, `scripts/check-workspace.sh`, fixtures de workspace
- **Verificación:** § 5 en `vortex-core/`; `check-workspace.sh`; `openspec validate T-00-09`
