# Tasks

## 1. Tests (red) — apertura happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/open.rs` con `open_empty_dir_selects_ns_and_db` — `cargo test -p storage --test open -- open_empty_dir` no compila o falla (escenario «Primera apertura en directorio temporal» / Listo cuando)
- [x] 1.2 Agregar test `reopen_same_dir_keeps_ns_and_db` — `cargo test -p storage --test open -- reopen_same_dir` sigue en rojo (escenario «Reapertura del mismo directorio»)

## 2. Tests (red) — apertura unhappy

- [x] 2.1 Agregar test `open_rejects_corrupt_directory` — `cargo test -p storage --test open -- open_rejects_corrupt` falla: no hay `OpenError` con motivo (escenario «Directorio corrupto»)
- [x] 2.2 Agregar test `open_rejects_unwritable_directory` (`#[cfg(unix)]`) que además trata el error como `std::error::Error` y `Display` no vacío — `cargo test -p storage --test open -- open_rejects_unwritable` sigue en rojo (escenarios «Directorio sin permisos», «OpenError es std::error::Error»)

## 3. Tests (red) — allowlist de `check-workspace.sh`

- [x] 3.1 Fixture temporal (`CHECK_ROOT`) de `crates/storage/Cargo.toml` con `[dependencies] axum.workspace = true` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `storage` (escenario «storage declara una dependencia fuera de la allowlist»; hoy falla cualquier `[dependencies]` en `storage`)
- [x] 3.2 Fixture temporal de un lib que no es `domain` ni `storage` (p. ej. `api`) con `[dependencies]` — el script sale ≠ 0 y nombra ese crate (escenario «Un lib distinto de domain declara dependencias»)
- [x] 3.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (`storage` todavía sin `surrealdb` en `Cargo.toml`; escenario «Pins exactos» previo a este change)

## 4. Harness (green) — allowlist `storage`

- [x] 4.1 En `scripts/check-workspace.sh`: permitir en `storage` `{surrealdb, thiserror}` en `[dependencies]` (`workspace = true`; `surrealdb` MAY `default-features = false` y MUST feature `kv-rocksdb`) y `tokio` en `[dev-dependencies]`; rechazar `axum` y cualquier otra crate — una fixture temporal con esa allowlist sale 0 (escenario «storage declara la allowlist»)
- [x] 4.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy del repo; happy de `storage` con allowlist; unhappy de `storage` usa `axum`; unhappy de `api` se conserva — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 4.3 Re-correr 3.1 y 3.2: `axum` en `storage` y deps en `api` siguen ≠ 0 con el needle del crate
- [x] 4.4 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin `surrealdb` en el `Cargo.toml` real de `storage`)

## 5. Implementación (green) — `Db::open` + `OpenError`

- [x] 5.1 Declarar en `vortex-core/crates/storage/Cargo.toml` `surrealdb = { workspace = true, default-features = false, features = ["kv-rocksdb"] }`, `thiserror = { workspace = true }` y `tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }` en `[dev-dependencies]`; actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «storage declara la allowlist»)
- [x] 5.2 Implementar `Db::open`, `OpenError`, constante `DB_DIR_NAME = "db"` y helper de sesión NS/DB (RocksDB + `use_ns("vectra")` / `use_db("comercio")`; `DEFINE … IF NOT EXISTS` solo si hace falta) — `cargo test -p storage --test open` sale 0 (escenarios 1.1–2.2 / Listo cuando)
- [x] 5.3 Confirmar que no hay `migrations/`, tabla `migracion`, `kv-mem`, `Config` ni llamadas desde `bin/vortex` — `rg "migracion|kv-mem|from_path|Db::open" vortex-core/crates/storage vortex-core/bin/vortex/src` no muestra migraciones ni cableado CLI (non-goals T-00-09, T-09-02, T-00-10, T-00-20)

## 6. Cierre (regresión, § 5 y non-goals)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde»)
- [x] 6.2 `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`surrealdb` no está en `domain`)
- [x] 6.3 `git diff` vacío en `vortex-core/bin/vortex/`, `vortex-core/crates/domain/`, `docs/openapi.yaml` y `.github/workflows/ci.yml`; `storage` no declara `axum` ni `domain`; ningún test usa `kv-mem` (non-goals: T-00-09, T-09-02, T-00-10, T-00-11, T-00-03)
- [x] 6.4 `openspec validate T-00-08` (o `t-00-08` si el CLI exige minúsculas) sale 0
