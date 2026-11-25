# Tasks

## 1. Tests (red) — tipos happy

- [x] 1.1 Agregar tests `search_query_constructs_perno` y `search_query_allows_empty_q` en `vortex-core/crates/domain/` — `cargo test -p domain search_query_constructs_perno` no compila o falla (escenarios «SearchQuery válida» y «q vacía permitida»)
- [x] 1.2 Agregar tests `search_filters_constructs_categoria_marca_stock_attr` y `search_page_constructs_hit_without_facets` — `cargo test -p domain search_filters_constructs` no compila o falla (escenarios «SearchFilters con categoria, marca, stock y atributo» y «SearchPage con hit y facetas ausentes»)
- [x] 1.3 Agregar tests `index_uid_constructs_productos`, `search_document_constructs_ulid` e `index_settings_constructs_empty` — `cargo test -p domain index_uid_constructs_productos` no compila o falla (escenarios «índice productos», «documento con ULID» e «IndexSettings vacío»)

## 2. Tests (red) — tipos unhappy

- [x] 2.1 Agregar tests `search_query_rejects_q_too_long`, `search_query_rejects_offset_or_limit_out_of_range` y `search_query_rejects_english_or_unknown_orden` — `cargo test -p domain search_query_rejects` no compila o falla (escenarios «q demasiado larga», «offset o limit fuera de rango» y «orden desconocido»)
- [x] 2.2 Agregar tests `index_uid_rejects_blank`, `swap_indexes_rejects_same_uid` y `search_document_rejects_table_prefix` — `cargo test -p domain index_uid_rejects_blank` no compila o falla (escenarios «índice vacío», «swap con el mismo índice» e «id de documento con prefijo de tabla»)

## 3. Tests (red) — puerto y dummy

- [x] 3.1 Agregar test `text_search_trait_exposes_nine_ops` (dummy que implementa el *trait*) — `cargo test -p domain text_search_trait_exposes_nine_ops` no compila o falla (escenario «El puerto declara las nueve operaciones»)
- [x] 3.2 Agregar tests `text_search_dummy_search_returns_page` y `text_search_dummy_health_is_up` — `cargo test -p domain text_search_dummy_search_returns_page` no compila o falla (escenarios «Dummy search happy path» y «Dummy health up»)
- [x] 3.3 Agregar tests unhappy `text_search_dummy_search_unavailable`, `text_search_dummy_search_timeout` y `text_search_dummy_upsert_invalid_input` — `cargo test -p domain text_search_dummy_search_unavailable` no compila o falla (escenarios «Dummy search Unavailable», «Dummy search Timeout» y «Dummy upsert InvalidInput»)

## 4. Implementación (green) — tipos

- [x] 4.1 Crear `vortex-core/crates/domain/src/text_search.rs` con `SearchQuery` / `SearchOrden` / `SearchFilters` (`try_new`, límites OpenAPI, nombres en español, `atributos` sin prefijo `attr_`) y reexportar desde `lib.rs` — `cargo test -p domain search_query_` y `search_filters_constructs` salen 0
- [x] 4.2 Implementar `SearchHit`, `SearchPage` (facetas opcionales ausentes) e `IndexSettings` vacío — `cargo test -p domain search_page_constructs_hit_without_facets index_settings_constructs_empty` sale 0
- [x] 4.3 Implementar `IndexUid` (no vacío) y `SearchDocument` (ULID 26 caracteres sin `:`) más rechazo de `swap` con el mismo uid — `cargo test -p domain index_uid_ search_document_` y `swap_indexes_rejects_same_uid` salen 0

## 5. Implementación (green) — puerto y dummy

- [x] 5.1 Declarar el *trait* `TextSearch` con las nueve operaciones (RPITIT `impl Future` + `Send`, `DomainError`, `health` → `ComponentHealth`; sin `async-trait` ni `meilisearch-sdk`) y un dummy de test — `cargo test -p domain text_search_trait_exposes_nine_ops` sale 0
- [x] 5.2 Dummy happy: `search` / `facet_search` → `SearchPage`; `health` → `ComponentStatus::Up`; `upsert`/`delete`/`apply_settings`/`create_index`/`delete_index`/`swap_indexes` → `Ok` — `cargo test -p domain text_search_dummy_search_returns_page text_search_dummy_health_is_up` sale 0
- [x] 5.3 Dummy unhappy: `search` → `unavailable` / `timeout`; `upsert_documents` → `invalid_input` — `cargo test -p domain text_search_dummy_search_unavailable text_search_dummy_search_timeout text_search_dummy_upsert_invalid_input` sale 0
- [x] 5.4 Confirmar que `domain` no declara crates nuevas y que `crates/search` no cambia — `./scripts/check-workspace.sh` sale 0; `git diff -- vortex-core/crates/search` vacío

## 6. Cierre

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 6.2 `./scripts/check-domain-deps.sh` sale 0 (escenario «check-domain-deps sigue verde»; Listo cuando: `domain` sin SDKs)
- [x] 6.3 `openspec validate T-03-03` sin errores