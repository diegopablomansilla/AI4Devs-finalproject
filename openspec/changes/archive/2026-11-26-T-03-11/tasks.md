# Tasks

## 1. Tests (red) — orquestador en `sync` (fakes)

- [x] 1.1 Agregar `reindex_unavailable_does_not_advance_cursor` y `reindex_invalid_document_aborts_before_swap` en `vortex-core/crates/sync/` (fakes de `TextSearch` / `OutboxRepo` / `ProductRepo`; el fake de search falla `unavailable` en `create_index` o `invalid_input` en `upsert_documents`) — `cargo test -p sync reindex_unavailable_does_not_advance_cursor reindex_invalid_document_aborts_before_swap` no compila o falla (escenarios unhappy de `vortex-reindex`: sin `swap_indexes`, cursor intacto)
- [x] 1.2 Agregar `reindex_sync_exposes_total_rebuild` (fake: un producto activo, search y outbox en memoria) — `cargo test -p sync reindex_sync_exposes_total_rebuild` no compila o falla (escenario happy de `vortex-sync`: no arranca el worker)

## 2. Implementación (green) — `reindex_meilisearch` con fakes

- [x] 2.1 Exponer `sync::reindex_meilisearch` (snapshot de outbox por `list_after_meilisearch`, crear/limpiar uid temporal, `apply_settings` 4.11.2, listar activos, `build_search_document`, upsert, `swap_indexes`, borrar uid viejo, `advance_meilisearch_cursor` si hay evento) — `cargo test -p sync reindex_unavailable_does_not_advance_cursor reindex_invalid_document_aborts_before_swap reindex_sync_exposes_total_rebuild` sale 0
- [x] 2.2 Reexportar la función en `sync/src/lib.rs` sin agregar `search` a `[dependencies]` de `sync` — `cargo test -p sync reindex_` verde; `./scripts/check-workspace.sh` no rechaza el crate `sync`

## 3. Tests (red) — Meilisearch real (Listo cuando)

- [x] 3.1 Agregar `reindex_rebuilds_productos_from_masters_and_swaps`, `reindex_omits_inactive_products`, `reindex_second_run_idempotent` y `reindex_empty_catalog_succeeds` `#[ignore = "meilisearch"]` en `vortex-core/crates/sync/tests/meili_reindex.rs` (Storage + `MeiliTextSearch`; uids de prueba o `productos`/`productos_nuevo` aislados por prefijo efímero **solo si** el orquestador acepta uids inyectados; el contrato de producción sigue siendo `productos` / `productos_nuevo`) — `cargo test -p sync --test meili_reindex -- --include-ignored` no compila o falla (escenarios happy de rebuild / inactivos / idempotencia / vacío)
- [x] 3.2 Agregar `reindex_search_loop_never_empty_or_error` `#[ignore = "meilisearch"]`: sembrar el uid vivo con un documento, buscar en bucle ese `q` mientras corre `reindex_meilisearch` — `cargo test -p sync --test meili_reindex -- --include-ignored reindex_search_loop_never_empty_or_error` no compila o falla (Listo cuando: ninguna búsqueda con error ni `hits` vacío)

## 4. Implementación (green) — orquestador contra el motor

- [x] 4.1 Completar create-if-missing del uid vivo, limpieza de `productos_nuevo` huérfano y espera de tasks vía el adaptador existente — `cargo test -p sync --test meili_reindex -- --include-ignored` sale 0 (si Meilisearch local o CI T-09-05)
- [x] 4.2 Conservar verdes del worker: `cargo test -p sync meili_product_worker` (sin ignored) no rompe T-03-07/T-03-08/T-03-09

## 5. Tests (red) — CLI `vortex reindex`

- [x] 5.1 Quitar `reindex` del loop `vortex_top_level_stubs_print_unimplemented`; agregar `vortex_reindex_without_config_is_error` y `vortex_reindex_missing_config_file_is_not_unimplemented` en `vortex-core/bin/vortex/tests/cli.rs`; cambiar `config_cli.rs` (`vortex_reindex_with_*_still_stub` → el stub de config inexistente usa `backup`) — `cargo test -p vortex --test cli --test config_cli vortex_reindex_without_config_is_error vortex_reindex_missing_config_file_is_not_unimplemented` falla (stderr aún tiene `no implementado`; escenarios unhappy de `vortex-cli` / `vortex-reindex`)
- [x] 5.2 Agregar `vortex_reindex_locked_db_is_error` y `vortex_reindex_meilisearch_down_is_error` en `vortex-core/bin/vortex/tests/reindex.rs` (TOML + `datos/` temp como `tests/seed.rs`; lock = `Db::open` vivo; URL con puerto cerrado) — `cargo test -p vortex --test reindex` no compila o falla (unhappy CLI: lock y Meili down, sin `no implementado`)
- [x] 5.3 Agregar `vortex_reindex_exit_0` `#[ignore = "meilisearch"]` (producto activo en Storage, Meili en la URL del TOML) — `cargo test -p vortex --test reindex -- --include-ignored vortex_reindex_exit_0` no compila o falla (happy CLI)

## 6. Implementación (green) — despacho CLI

- [x] 6.1 Despachar `Commands::Reindex` en `bin/vortex`: `resolve_config_path`, `Config`, `Db::open` + `migrate`, `MeiliTextSearch`, abortar si Meili no está `up`, `reindex_meilisearch` con uids `productos` / `productos_nuevo`, exit 0/1; span `tracing` de reindex — `cargo test -p vortex --test cli --test config_cli --test reindex` verde (unhappy + stubs `backup`/`cert`/… siguen con `no implementado`)
- [x] 6.2 Con Meilisearch local: `cargo test -p vortex --test reindex -- --include-ignored vortex_reindex_exit_0` sale 0

## 7. Verificación de cierre

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `openspec validate T-03-11` verde
- [x] 7.3 Confirmar non-goals: sin vectores, sin Unix socket, sin `openapi.yaml`, sin supervisor Meilisearch — `git diff --name-only` sin esos paths de código