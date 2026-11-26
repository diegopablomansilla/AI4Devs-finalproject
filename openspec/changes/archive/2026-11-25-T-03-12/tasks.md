# Tasks

## 1. Tests (red) — dominio: hit, resaltado y claves

- [x] 1.1 Agregar tests happy `search_hit_constructs_with_resaltado` y actualizar `search_page_constructs_hit_without_facets` al nuevo `SearchHit` en `vortex-core/crates/domain/` — `cargo test -p domain search_hit_constructs_with_resaltado search_page_constructs_hit_without_facets` no compila o falla (escenarios «SearchHit con resaltado», «SearchPage con hit y facetas ausentes»)
- [x] 1.2 Agregar tests unhappy `search_query_rejects_attr_prefix_clave` y `search_query_rejects_invalid_atributo_clave` — `cargo test -p domain search_query_rejects_attr_prefix_clave search_query_rejects_invalid_atributo_clave` no compila o falla (escenarios «clave de atributo con prefijo attr_», «clave de atributo inválida»)

## 2. Implementación (green) — dominio

- [x] 2.1 Extender `SearchHit` con campos 4.11.1 por defecto y `resaltado: Option<SearchHighlight>`; validar claves de `filtros.atributos` en `SearchQuery::try_new` (misma regla que `productos_index_settings`, sin prefijo `attr_`) — `cargo test -p domain search_hit_constructs_with_resaltado search_page_constructs_hit_without_facets search_query_rejects_attr_prefix_clave search_query_rejects_invalid_atributo_clave search_query_constructs_perno search_filters_constructs_categoria_marca_stock_attr` sale 0

## 3. Tests (red) — caso de uso con TextSearch falso

- [x] 3.1 Agregar tests happy `search_service_forwards_q_with_empty_filters`, `search_service_forwards_each_filter` y `search_service_allows_empty_q_with_con_stock` en `vortex-core/crates/search/` — `cargo test -p search search_service_forwards_q_with_empty_filters search_service_forwards_each_filter search_service_allows_empty_q_with_con_stock` no compila o falla (escenarios «Búsqueda con q y filtros vacíos», «Traduce cada filtro al SearchQuery» Listo cuando, «q vacía con filtro de stock»)
- [x] 3.2 Agregar tests unhappy `search_service_rejects_q_too_long`, `search_service_rejects_offset_or_limit_out_of_range`, `search_service_rejects_attr_prefix_clave`, `search_service_propagates_unavailable` y `search_service_propagates_timeout` — `cargo test -p search search_service_rejects_q_too_long search_service_rejects_offset_or_limit_out_of_range search_service_rejects_attr_prefix_clave search_service_propagates_unavailable search_service_propagates_timeout` no compila o falla (escenarios unhappy del caso de uso)

## 4. Implementación (green) — SearchService

- [x] 4.1 `SearchService<S: TextSearch>` + fake que registra el `SearchQuery` (sin Meilisearch): construye `SearchQuery`, llama `search` en el uid inyectado, propaga `invalid_input` / `unavailable` / `timeout` sin remapear — `cargo test -p search search_service_forwards_q_with_empty_filters search_service_forwards_each_filter search_service_allows_empty_q_with_con_stock search_service_rejects_q_too_long search_service_rejects_offset_or_limit_out_of_range search_service_rejects_attr_prefix_clave search_service_propagates_unavailable search_service_propagates_timeout` sale 0

## 5. Tests (red) — traducción Meilisearch y fachada

- [x] 5.1 Agregar tests happy `meili_filter_joins_categoria_marca_stock_and_attr`, `meili_filter_omitted_when_empty`, `meili_sort_maps_precio_and_stock`, `meili_sort_omitted_for_relevancia` y `meili_search_requests_nombre_descripcion_highlight` — `cargo test -p search meili_filter_joins_categoria_marca_stock_and_attr meili_filter_omitted_when_empty meili_sort_maps_precio_and_stock meili_sort_omitted_for_relevancia meili_search_requests_nombre_descripcion_highlight` no compila o falla (escenarios de traducción y `_formatted`)
- [x] 5.2 Agregar test `search_gateway_is_object_safe` — `cargo test -p search search_gateway_is_object_safe` no compila o falla (escenario «la fachada es inyectable»)

## 6. Implementación (green) — adaptador y Gateway

- [x] 6.1 Funciones puras `meili_filter` / `meili_sort` y `search_inner`: `with_filter` / `with_sort` cuando hay valor, `attributesToHighlight` `nombre`+`descripcion` con tags `<mark>`, mapear documento 4.11.1 + `_formatted` → `SearchHit`; MUST NOT PATCH settings ni enviar facetas — `cargo test -p search meili_filter_joins_categoria_marca_stock_and_attr meili_filter_omitted_when_empty meili_sort_maps_precio_and_stock meili_sort_omitted_for_relevancia meili_search_requests_nombre_descripcion_highlight` sale 0
- [x] 6.2 `SearchGateway` object-safe (`search`) implementado por `SearchService`; reexport en `lib.rs`; MUST NOT exponer facetas ni respaldo — `cargo test -p search search_gateway_is_object_safe` sale 0; `rg -n "facet|fallback|degraded" vortex-core/crates/search/src/query.rs` sin métodos de faceta/respaldo

## 7. Cierre (regresión)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 Confirmar non-goals: `git diff --name-only` sin `docs/openapi.yaml`, sin `vortex-core/crates/api`, sin handlers HTTP de `/search`
- [x] 7.4 `openspec validate T-03-12` sale 0