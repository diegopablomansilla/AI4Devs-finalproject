# Tasks

## 1. Tests (red) — dominio

- [x] 1.1 Agregar tests unitarios `marca_constructs_fixser`, `marca_rejects_blank_nombre` y `marca_rejects_nombre_too_long` en `vortex-core/crates/domain` — `cargo test -p domain -- marca_` no compila o falla (escenarios «Construcción válida Fixser», «nombre vacío», «nombre demasiado largo»)
- [x] 1.2 Agregar test `brand_repo_trait_exposes_create_get_list_all_update_set_activo` (dummy que implementa el trait) — `cargo test -p domain -- brand_repo_trait_exposes` falla (escenario «El puerto declara create, get, list_all, update y set_activo»)

## 2. Implementación (green) — dominio

- [x] 2.1 Crear `vortex-core/crates/domain/src/marca.rs` con `Marca`/`MarcaDraft`/`try_new` (campos 4.4.2, `Id<Marca>`, `Slug` reutilizado, nombre 1..=100) y reexportar desde `lib.rs` — `cargo test -p domain -- marca_constructs_fixser marca_rejects_blank_nombre marca_rejects_nombre_too_long` sale 0
- [x] 2.2 Declarar `NewMarca`, `UpdateMarca` y el trait `BrandRepo` (`create`, `get`, `list_all`, `update`, `set_activo`; `impl Future` sin `async-trait` ni `tokio`) más dummy de test — `cargo test -p domain -- brand_repo_trait_exposes` sale 0
- [x] 2.3 Confirmar que `domain` no persiste marcas ni gana crates — `rg -n "surrealdb|CREATE marca" vortex-core/crates/domain` sin matches de producto; `./scripts/check-domain-deps.sh` sale 0 (escenario «domain no persiste marcas»)

## 3. Tests (red) — migración

- [x] 3.1 Agregar `vortex-core/crates/storage/tests/marca.rs` con `migrate_applies_0003_marca` y `migrate_rerun_keeps_single_0003_record` — `cargo test -p storage --test marca migrate_` falla o no compila (escenarios «Migración aplica esquema marca», «Re-arranque no duplica el registro de migración»)
- [x] 3.2 Agregar tests unhappy `marca_rejects_duplicate_slug` y `marca_rejects_empty_nombre` — `cargo test -p storage --test marca marca_rejects` falla (escenarios «Índice único slug rechaza duplicado», «nombre vacío rechazado»)

## 4. Implementación (green) — migración

- [x] 4.1 Crear `migrations/0003_marca.surql` (tabla 4.4.2, `DEFINE … IF NOT EXISTS`, `CHANGEFEED 3d`, índice `uq_marca_slug`) y registrar `MIGRATION_0003_SQL` en `EMBEDDED` — `cargo test -p storage --test marca migrate_ marca_rejects` en verde
- [x] 4.2 Ajustar `tests/migrate.rs` y `tests/harness.rs` para que `list_migrations` exija `version` 0–3 (`len() == 4`, checksum de `0003`) — `cargo test -p storage --test migrate --test harness` en verde

## 5. Tests (red) — repositorio

- [x] 5.1 Agregar tests happy `brand_repo_creates_fixser`, `brand_repo_get_returns_persisted`, `brand_repo_list_all_includes_inactive` y `brand_repo_create_emits_outbox_upsert` — `cargo test -p storage --test marca brand_repo_creates brand_repo_get brand_repo_list_all brand_repo_create_emits` no compila o falla (escenarios de alta/get/list/outbox; **Listo cuando** alta HU-02-02)
- [x] 5.2 Agregar tests `brand_repo_update_renames_and_regenerates_slug` y `brand_repo_set_activo_persists_inactive` — `cargo test -p storage --test marca brand_repo_update_renames brand_repo_set_activo_persists` falla (escenarios «Edición regenera slug», «Baja lógica persiste inactiva»; **Listo cuando**)
- [x] 5.3 Agregar tests unhappy `brand_repo_rejects_duplicate_slug`, `brand_repo_update_duplicate_slug_is_duplicate_slug`, `brand_repo_get_unknown_id_is_not_found`, `brand_repo_update_unknown_id_is_not_found`, `brand_repo_set_activo_unknown_id_is_not_found` y `brand_repo_create_blank_nombre_is_invalid_input` — `cargo test -p storage --test marca brand_repo_rejects brand_repo_update_duplicate brand_repo_get_unknown brand_repo_update_unknown brand_repo_set_activo_unknown brand_repo_create_blank` falla

## 6. Implementación (green) — `MarcaRepo`

- [x] 6.1 Crear `vortex-core/crates/storage/src/marca.rs` con `MarcaRepo { db: Arc<Db> }` que implementa `get` y `list_all` (strip de `marca:`, `Marca::try_new`) y reexportar desde `lib.rs` — `cargo test -p storage --test marca brand_repo_get_returns_persisted brand_repo_list_all_includes_inactive brand_repo_get_unknown_id_is_not_found` sale 0
- [x] 6.2 Implementar `create`: *slug* desde `nombre` o explícito, `Tx` + `OutboxEvent` `marca`/`upsert`, UNIQUE → `duplicate_slug`, nombre vacío → `invalid_input` — `cargo test -p storage --test marca brand_repo_creates_fixser brand_repo_create_emits_outbox_upsert brand_repo_rejects_duplicate_slug brand_repo_create_blank_nombre_is_invalid_input` sale 0
- [x] 6.3 Implementar `update` (regenera *slug* si no viene explícito; colisión → `duplicate_slug`; id ausente → `not_found`) y `set_activo` (outbox `upsert`, no `DELETE`) — `cargo test -p storage --test marca brand_repo_update_renames_and_regenerates_slug brand_repo_set_activo_persists_inactive brand_repo_update_duplicate_slug_is_duplicate_slug brand_repo_update_unknown_id_is_not_found brand_repo_set_activo_unknown_id_is_not_found` sale 0
- [x] 6.4 Confirmar non-goals: sin HTTP, sin crate `catalog`, sin occupancy, sin cambios en `openapi.yaml` — `git diff --name-only` sin `docs/openapi.yaml` ni `vortex-core/crates/api` ni `vortex-core/crates/catalog`

## 7. Cierre (regresión)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 `openspec validate T-02-08` sale 0