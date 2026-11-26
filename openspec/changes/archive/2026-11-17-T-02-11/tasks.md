# Tasks

## 1. Tests (red) — dominio

- [x] 1.1 Agregar tests unitarios `atributo_def_constructs_grado_enum`, `atributo_def_constructs_medida_texto`, `atributo_def_rejects_enum_without_options`, `atributo_def_rejects_invalid_clave` y `atributo_def_rejects_blank_etiqueta` en `vortex-core/crates/domain` — `cargo test -p domain -- atributo_def_` no compila o falla (escenarios «Construcción válida grado enum», «medida texto», «enum sin opciones», «clave inválida», «etiqueta vacía»; **Listo cuando** enum sin opciones)
- [x] 1.2 Agregar test `attribute_def_repo_trait_exposes_create_get_list_all_update_set_activo` (dummy que implementa el trait) — `cargo test -p domain -- attribute_def_repo_trait_exposes` falla (escenario «El puerto declara create, get, list_all, update y set_activo»)

## 2. Implementación (green) — dominio

- [x] 2.1 Crear `vortex-core/crates/domain/src/atributo_def.rs` con `TipoAtributo`, `AtributoDef`/`AtributoDefDraft`/`try_new` (campos 4.4.3, `Id<AtributoDef>`, clave `^[a-z][a-z0-9_]*$`, etiqueta 1..=60, enum exige opciones) y reexportar desde `lib.rs` — `cargo test -p domain -- atributo_def_constructs atributo_def_rejects` sale 0
- [x] 2.2 Declarar `NewAtributoDef`, `UpdateAtributoDef` y el trait `AttributeDefRepo` (`create`, `get`, `list_all`, `update`, `set_activo`; `impl Future` sin `async-trait` ni `tokio`) más dummy de test — `cargo test -p domain -- attribute_def_repo_trait_exposes` sale 0
- [x] 2.3 Confirmar que `domain` no persiste `atributo_def` ni gana crates — `rg -n "surrealdb|CREATE atributo_def" vortex-core/crates/domain` sin matches de producto; `./scripts/check-domain-deps.sh` sale 0 (escenario «domain no persiste atributo_def»)

## 3. Tests (red) — migración

- [x] 3.1 Agregar `vortex-core/crates/storage/tests/atributo_def.rs` con `migrate_applies_0004_atributo_def` y `migrate_rerun_keeps_single_0004_record` — `cargo test -p storage --test atributo_def migrate_` falla o no compila (escenarios «Migración aplica esquema atributo_def», «Re-arranque no duplica el registro de migración»)
- [x] 3.2 Agregar tests unhappy `atributo_def_rejects_duplicate_clave` y `atributo_def_rejects_invalid_tipo` — `cargo test -p storage --test atributo_def atributo_def_rejects` falla (escenarios «Índice único clave rechaza duplicado», «tipo fuera del enum rechazado»)

## 4. Implementación (green) — migración

- [x] 4.1 Crear `migrations/0004_atributo_def.surql` (tabla 4.4.3, `DEFINE … IF NOT EXISTS`, `CHANGEFEED 3d`, índice `uq_atributo_clave`, `ASSERT` de `tipo`) y registrar `MIGRATION_0004_SQL` en `EMBEDDED` — `cargo test -p storage --test atributo_def migrate_ atributo_def_rejects` en verde
- [x] 4.2 Ajustar `tests/migrate.rs` y `tests/harness.rs` para que `list_migrations` exija `version` 0–4 (`len() == 5`, checksum de `0004`) — `cargo test -p storage --test migrate --test harness` en verde

## 5. Tests (red) — repositorio

- [x] 5.1 Agregar tests happy `attribute_def_repo_creates_grado`, `attribute_def_repo_get_returns_persisted`, `attribute_def_repo_list_all_ordered_by_orden` y `attribute_def_repo_create_emits_outbox_upsert` — `cargo test -p storage --test atributo_def attribute_def_repo_creates attribute_def_repo_get attribute_def_repo_list_all attribute_def_repo_create_emits` no compila o falla (escenarios de alta/get/list/outbox; HU-02-03)
- [x] 5.2 Agregar tests `attribute_def_repo_update_changes_etiqueta` y `attribute_def_repo_set_activo_persists_inactive` — `cargo test -p storage --test atributo_def attribute_def_repo_update_changes attribute_def_repo_set_activo_persists` falla (escenarios «Edición cambia etiqueta», «Baja lógica persiste inactiva»)
- [x] 5.3 Agregar tests unhappy `attribute_def_repo_rejects_duplicate_clave`, `attribute_def_repo_update_duplicate_clave_is_conflict`, `attribute_def_repo_create_enum_without_options_is_invalid_input`, `attribute_def_repo_get_unknown_id_is_not_found`, `attribute_def_repo_update_unknown_id_is_not_found`, `attribute_def_repo_set_activo_unknown_id_is_not_found` y `attribute_def_repo_create_invalid_clave_is_invalid_input` — `cargo test -p storage --test atributo_def attribute_def_repo_rejects attribute_def_repo_update_duplicate attribute_def_repo_create_enum attribute_def_repo_get_unknown attribute_def_repo_update_unknown attribute_def_repo_set_activo_unknown attribute_def_repo_create_invalid` falla (**Listo cuando** clave duplicada → `conflict` / 409 y enum sin opciones)

## 6. Implementación (green) — `AtributoDefRepo`

- [x] 6.1 Crear `vortex-core/crates/storage/src/atributo_def.rs` con `AtributoDefRepo { db: Arc<Db> }` que implementa `get` y `list_all` (strip de `atributo_def:`, orden por `orden`/`clave`, `AtributoDef::try_new`) y reexportar desde `lib.rs` — `cargo test -p storage --test atributo_def attribute_def_repo_get_returns_persisted attribute_def_repo_list_all_ordered_by_orden attribute_def_repo_get_unknown_id_is_not_found` sale 0
- [x] 6.2 Implementar `create`: `Tx` + `OutboxEvent` `atributo_def`/`upsert`, UNIQUE → `conflict`, enum sin opciones / clave inválida → `invalid_input` — `cargo test -p storage --test atributo_def attribute_def_repo_creates_grado attribute_def_repo_create_emits_outbox_upsert attribute_def_repo_rejects_duplicate_clave attribute_def_repo_create_enum_without_options_is_invalid_input attribute_def_repo_create_invalid_clave_is_invalid_input` sale 0
- [x] 6.3 Implementar `update` (colisión de `clave` → `conflict`; id ausente → `not_found`; sin `attribute_in_use`) y `set_activo` (outbox `upsert`, no `DELETE`) — `cargo test -p storage --test atributo_def attribute_def_repo_update_changes_etiqueta attribute_def_repo_set_activo_persists_inactive attribute_def_repo_update_duplicate_clave_is_conflict attribute_def_repo_update_unknown_id_is_not_found attribute_def_repo_set_activo_unknown_id_is_not_found` sale 0
- [x] 6.4 Confirmar non-goals: sin HTTP, sin crate `catalog`, sin `attribute_in_use`, sin cambios en `openapi.yaml` ni `DomainError` — `git diff --name-only` sin `docs/openapi.yaml` ni `vortex-core/crates/api` ni `vortex-core/crates/catalog` ni `vortex-core/crates/domain/src/error.rs`

## 7. Cierre (regresión)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 `openspec validate T-02-11` sale 0
