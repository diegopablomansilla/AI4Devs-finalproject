# Tasks

## 1. Tests (red) — puerto `OutboxRepo`

- [x] 1.1 Agregar `outbox_repo_dummy_lists_after_cursor` en `vortex-core/crates/domain/` — `cargo test -p domain outbox_repo_dummy_lists_after_cursor` no compila o falla (escenario «Lote después del cursor»)
- [x] 1.2 Agregar `outbox_repo_dummy_advance_not_found` — `cargo test -p domain outbox_repo_dummy_advance_not_found` no compila o falla (escenario «Cursor desconocido»)

## 2. Implementación (green) — puerto `OutboxRepo`

- [x] 2.1 Exponer `OutboxRow` y `OutboxRepo` (`list_after`, `get_cursor`, `advance_cursor`) en `domain`; dummy de tests — `cargo test -p domain outbox_repo_dummy` sale 0; `./scripts/check-domain-deps.sh` sale 0

## 3. Tests (red) — adaptador `OutboxRepo`

- [x] 3.1 Agregar `storage_outbox_repo_lists_after_cursor` y `storage_outbox_repo_advances_meilisearch_cursor` en `vortex-core/crates/storage/tests/` — `cargo test -p storage storage_outbox_repo_lists storage_outbox_repo_advances` no compila o falla (escenarios de lectura y avance)
- [x] 3.2 Agregar `storage_outbox_repo_limits_batch_to_200` y `storage_outbox_repo_advance_unknown_event_is_not_found` — `cargo test -p storage storage_outbox_repo_limits storage_outbox_repo_advance_unknown` no compila o falla (lote 200 + unhappy `not_found`)

## 4. Implementación (green) — adaptador `OutboxRepo`

- [x] 4.1 Implementar el adaptador SurrealDB de `OutboxRepo` (LIMIT 200, cursor `sync_cursor:meilisearch`, `registro` sin prefijo) — `cargo test -p storage storage_outbox_repo` sale 0

## 5. Tests (red) — upsert 4.11.1

- [x] 5.1 Agregar `#[ignore = "meilisearch"]` `meilisearch_adapter_upserts_full_search_document` en `vortex-core/crates/search/tests/` — `cargo test -p search -- --include-ignored meilisearch_adapter_upserts_full_search_document` no compila o falla (escenario «Upsert envía el documento 4.11.1»)

## 6. Implementación (green) — cuerpo completo

- [x] 6.1 `MeiliTextSearch::upsert_documents` serializa `SearchDocument` completo (primary key `id`); conservar hits solo con `id` — `cargo test -p search -- --include-ignored meilisearch_adapter_upserts` sale 0 si el sidecar está arriba; `cargo test -p search meilisearch_adapter_unavailable_when_unreachable` sale 0

## 7. Allowlist de `sync`

- [x] 7.1 `sync_deps_allowed` en `scripts/check-workspace.sh` + fixtures happy/unhappy; `Cargo.toml` de `sync` con `domain`, `storage`, `tokio` y `search` en dev — `./scripts/check-workspace.sh` sale 0; el fixture que declara `meilisearch-sdk` en `sync` falla

## 8. Tests (red) — worker unidad

- [x] 8.1 Agregar `worker_deletes_inactive_product`, `worker_deletes_missing_product`, `worker_skips_categoria_without_fanout`, `worker_advances_cursor_on_imagen_producto_without_mutating_index`, `worker_empty_outbox_does_not_call_search` en `vortex-core/crates/sync/` — `cargo test -p sync worker_deletes worker_skips worker_advances worker_empty` no compila o falla
- [x] 8.2 Agregar `worker_does_not_advance_cursor_when_search_unavailable` y `worker_does_not_advance_cursor_on_invalid_input` — `cargo test -p sync worker_does_not_advance_cursor` no compila o falla (unhappy)

## 9. Implementación (green) — worker

- [x] 9.1 Worker genérico: lote 200, `producto` upsert/delete, no-op + avance para otras entidades, cursor tras *task*, `drain_once` + loop notify/1 s — `cargo test -p sync worker_` sale 0 (unidad; sin sidecar)

## 10. Tests (red) — Listo cuando

- [x] 10.1 Agregar `#[ignore = "meilisearch"]` `worker_indexes_new_product_within_one_second` y `worker_reprocessing_same_event_is_idempotent` en `vortex-core/crates/sync/tests/` — `cargo test -p sync -- --include-ignored meilisearch` no compila o falla (escenarios Listo cuando)

## 11. Implementación (green) — Listo cuando

- [x] 11.1 Completar el drain contra `test_db` + `MeiliTextSearch` (índice efímero ≠ `productos`, teardown) — `cargo test -p sync -- --include-ignored meilisearch` sale 0 con sidecar

## 12. Cierre (regresión y non-goals)

- [x] 12.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 12.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 12.3 Confirmar non-goals: sin fan-out, sin `outbox_fallo`/métricas, sin wiring en `bin/vortex`, sin cambios en `.github/workflows/ci.yml` ni `docs/openapi.yaml` — `git diff --name-only` sin esos paths de código
- [x] 12.4 `openspec validate T-03-07` sin errores