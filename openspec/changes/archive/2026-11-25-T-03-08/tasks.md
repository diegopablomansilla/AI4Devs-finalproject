# Tasks

## 1. Tests (red) — `productos_index_settings` en domain

- [x] 1.1 Mover/agregar `productos_index_settings_includes_4_11_2_h03_h04` y `productos_index_settings_omits_synonyms` en `vortex-core/crates/domain/` — `cargo test -p domain productos_index_settings_includes productos_index_settings_omits` no compila o falla (escenarios «Perfil productos con un atributo facetable» y omisión de `synonyms`)
- [x] 1.2 Agregar `productos_index_settings_rejects_invalid_clave` — `cargo test -p domain productos_index_settings_rejects_invalid_clave` no compila o falla (escenario «Clave facetable inválida»)

## 2. Implementación (green) — constructor en domain y reexport

- [x] 2.1 Exponer `productos_index_settings` en `domain`; reexportar `search::productos_index_settings`; quitar la copia en `search` — `cargo test -p domain productos_index_settings` sale 0; `cargo test -p search productos_index_settings meilisearch_apply_productos_settings` sale 0 (unidad + ignore si hay sidecar); `./scripts/check-domain-deps.sh` sale 0

## 3. Tests (red) — fan-out unidad

- [x] 3.1 Reemplazar `worker_skips_categoria_without_fanout` por `worker_fanout_categoria_reindexes_full_branch` y agregar `worker_fanout_marca_reindexes_products` en `vortex-core/crates/sync/` — `cargo test -p sync worker_fanout_categoria worker_fanout_marca` no compila o falla (escenarios «Fan-out de categoría reindexa toda la rama» y «Fan-out de marca»)
- [x] 3.2 Agregar `worker_fanout_atributo_def_reindexes_users_and_applies_settings`, `worker_fanout_empty_affected_advances_cursor` y `worker_sinonimo_still_skips_without_mutating_index` — `cargo test -p sync worker_fanout_atributo worker_fanout_empty worker_sinonimo` no compila o falla (escenarios atributo_def, vacío y non-goal `sinonimo`)
- [x] 3.3 Agregar `worker_fanout_does_not_advance_cursor_when_search_unavailable`, `worker_fanout_does_not_advance_cursor_when_apply_settings_unavailable` y `worker_fanout_invalid_registro_does_not_advance` — `cargo test -p sync worker_fanout_does_not_advance worker_fanout_invalid` no compila o falla (unhappy: motor, settings, registro)

## 4. Implementación (green) — fan-out en el worker

- [x] 4.1 Despachar `categoria` / `marca` / `atributo_def` a *fan-out* (`list` rama/marca; `apply_settings` + list+get para atributo; cursor tras todas las *tasks*) — `cargo test -p sync worker_fanout worker_sinonimo worker_deletes worker_empty worker_does_not_advance worker_advances` sale 0 (unidad; sin sidecar)

## 5. Tests (red) — Listo cuando HU-02-01

- [x] 5.1 Agregar `#[ignore = "meilisearch"]` `worker_fanout_categoria_rename_updates_lvl0_within_one_second` en `vortex-core/crates/sync/tests/` — `cargo test -p sync -- --include-ignored worker_fanout_categoria_rename` no compila o falla (escenario «Renombrar Fijaciones actualiza categoria.lvl0 en menos de 1 s»)

## 6. Implementación (green) — Listo cuando

- [x] 6.1 Completar el test contra `test_db` + `MeiliTextSearch` (árbol Fijaciones/Pernos, producto, rename a Tornillería, índice `t0308-<ulid>`, helper de lectura de `categoria.lvl0`, teardown) — `cargo test -p sync -- --include-ignored meilisearch` sale 0 con sidecar (incluye T-03-07 + T-03-08)

## 7. Cierre (regresión y non-goals)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 Confirmar non-goals: sin `sinonimo`→Meilisearch, sin `outbox_fallo`/métricas, sin wiring en `bin/vortex`, sin cambios en `.github/workflows/ci.yml` ni `docs/openapi.yaml` — `git diff --name-only` sin esos paths de código
- [x] 7.4 `openspec validate T-03-08` sin errores