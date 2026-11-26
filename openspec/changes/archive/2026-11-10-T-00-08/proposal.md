# Proposal

## Ticket padre

- **Ticket:** [T-00-08](../../../docs/7-tickets.md) — Apertura de SurrealDB embebido
- **Change:** `T-00-08`
- **HU:** [HU-00-02](../../../docs/6-historias.md#hu-00-02--surrealdb-embebido-con-migraciones-versionadas)
- **Depende de:** T-00-03, T-00-05 (ambos en `develop`; T-00-06 y T-00-07 también)
- **Hace:** `storage::Db::open(path)` con RocksDB en `<datos>/db`, `NS vectra`, `DB comercio`; versión exacta de `surrealdb` fijada; errores de apertura tipados
- **No hace:** migraciones (T-00-09)
- **Listo cuando:** tests abren una base en un directorio temporal y verifican que un directorio corrupto o sin permisos devuelve error con el motivo

## Why

Vortex Core ya tiene workspace, taxonomía de errores, config TOML y CLI, pero el crate `storage` sigue vacío: no hay fuente de verdad. HU-00-02 y [3.4.12](../../../docs/3-componentes.md#3412-storage-surrealdb-embebido) exigen SurrealDB embebido con RocksDB (PM-03, PC-01) y corte de arranque si la base no abre. T-00-09 necesita esa conexión para aplicar migraciones. Hay que abrir el motor ahora, con errores tipados y tests de disco, antes de cualquier `.surql`.

## What Changes

- Exponer `storage::Db::open(path)` asíncrono: motor embebido RocksDB (`kv-rocksdb`) en el directorio `path` (convención de instalación: `<paths.datos>/db`)
- Tras abrir, seleccionar `NS vectra` y `DB comercio` ([4.3](../../../docs/4-modelo.md#43-convenciones)); un directorio vacío las crea
- Usar `surrealdb = "=2.7.0"` ya pinneado en `[workspace.dependencies]`; el crate `storage` activa la feature `kv-rocksdb`
- Errores de apertura tipados (`OpenError`) cuyo mensaje incluye el **motivo** (permiso, corrupción u otro fallo de RocksDB/IO); alineados conceptualmente a `DomainError::Unavailable` (`code` `unavailable`) sin ampliar la taxonomía de T-00-03
- **BREAKING** (harness interno): `check-workspace.sh` deja de exigir que `storage` no tenga `[dependencies]`. `storage` MAY declarar `surrealdb` (`workspace = true`, feature `kv-rocksdb`) y `thiserror`; MAY declarar `tokio` en `[dev-dependencies]`. Los demás libs (salvo `domain`) siguen sin deps
- TDD: happy = abrir en directorio temporal y comprobar `NS`/`DB`; unhappy = directorio corrupto o sin permisos → error con motivo

## Non-goals

Este change **no** cubre:

- Migraciones `.surql`, tabla `migracion`, checksum SHA-256 — **No hace** / **T-00-09**
- `storage::testing::test_db()` ni motor `kv-mem` — **T-09-02** (PM-03: `kv-mem` solo en tests de ese harness)
- Cablear `Db::open` en `vortex serve`, `AppState` o apagado ordenado — **T-00-10** / **T-00-20**
- Canal Unix socket cuando la base ya está abierta (lock exclusivo RocksDB) — **T-00-22** (H-01, v2)
- Tablas de negocio, outbox, repositorios — T-03-01, T-02-01+, T-03-02
- Traducir `OpenError` a `problem+json` / HTTP — **T-00-11**
- `/health` componente `storage` — **T-00-13**
- Cambiar `DomainError` (sin campo de motivo en `Unavailable`) ni el árbol de `domain` — T-00-03 / `domain-deps`
- Cargar `Config` ni usar `paths.datos` desde `storage` (el caller pasa `path`; T-00-20 concatena `<datos>/db`)
- Recarga en caliente, backups, `vortex seed` / `reindex` / `verify`
- Relajar `[dependencies]` en libs distintos de `storage` y `domain`
- Cambios a `docs/openapi.yaml`, `bin/vortex` o CI YAML

## Capabilities

### New Capabilities

- `vortex-storage`: apertura de SurrealDB embebido (RocksDB) en el crate `storage`, selección `NS vectra` / `DB comercio`, errores de apertura tipados con motivo

### Modified Capabilities

- `cargo-workspace`: el crate `storage` MAY declarar `[dependencies]` (`surrealdb` con `workspace = true` y feature `kv-rocksdb`; `thiserror` con `workspace = true`) y `[dev-dependencies]` (`tokio` con `workspace = true`, features de test). Los demás libs distintos de `domain` MUST NOT declarar deps. `cargo test --workspace` MUST ejecutar también los tests de apertura de `storage`. El resto de libs MAY seguir sin lógica de producto.

## Impact

- **Repo:** `vortex-core/crates/storage/` (`Cargo.toml`, `src/lib.rs`, tests de integración), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** tests del package `storage` (RocksDB en temp + unhappy de permiso/corrupción); primera compilación nativa de `rocksdb-sys` en el job `test` de T-09-01; [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core`; `check-workspace.sh` y `check-domain-deps.sh` en verde (`domain` sigue sin `surrealdb`)
- **Ticket:** cierra T-00-08 (Listo cuando: tests abren en temp y un directorio corrupto o sin permisos devuelve error con el motivo)
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `storage` declara `surrealdb`/`thiserror`; sigue fallando si `storage` declara otra crate o si `api` (u otro lib) declara deps
