# Tasks

## 1. Allowlist `api` → `search`

- [x] 1.1 Permitir `search` (path a `crates/search`) en `[dependencies]` de `api`: `vortex-core/crates/api/Cargo.toml`, `scripts/check-workspace.sh` (`api_deps_allowed`) y caso happy `api declara path search` en `scripts/fixtures/workspace/run-script-tests.sh` — `./scripts/check-workspace.sh` y el fixture de allowlist salen 0 (escenario «api declara path search»)

## 2. Tests (red) — happy path HTTP

- [x] 2.1 Agregar tests happy `get_search_q_perno_returns_200`, `get_search_empty_q_with_in_stock_returns_200`, `get_search_maps_attr_deep_object`, `get_search_maps_category_brand_in_stock`, `get_search_maps_sort_price_asc`, `get_search_highlight_from_formatted` y `get_search_facets_true_returns_null_facets` en `vortex-core/crates/api/tests/search.rs` (fake de `SearchGateway`) — `cargo test -p api --test search get_search_q_perno get_search_empty_q get_search_maps_attr get_search_maps_category get_search_maps_sort get_search_highlight get_search_facets` no compila o falla (escenarios happy de «Endpoint GET /api/v1/search»; `assert_conforms` 200)
- [x] 2.2 Agregar test `get_search_is_registered` o reusar el GET del router: `GET /api/v1/search` no es 404 — el test de 2.1 ya lo cubre cuando el router no registra la ruta (escenario «GET /api/v1/search está registrado»)

## 3. Implementación (green) — ruta, DTO y query

- [x] 3.1 `AppState::with_search` (`Option<Arc<dyn SearchGateway>>`), módulo `api/src/search.rs`, ruta `GET /api/v1/search` y mapper `SearchPage` → `SearchResponse` (5.3: `name`/`price` string/`unitOfMeasure`/`highlight` desde `resaltado`; `degraded: false`; `facets: null`) — los tests de 2.1 siguen en rojo hasta parsear query
- [x] 3.2 Parsear query OpenAPI (`q`, `categoryId`, `brandId`, `inStock`, `attr[<clave>]` *deepObject* sin crate nueva, `sort`, `offset`, `limit`; `facets` ignorado) y llamar `SearchGateway::search` — `cargo test -p api --test search get_search_q_perno get_search_empty_q get_search_maps_attr get_search_maps_category get_search_maps_sort get_search_highlight get_search_facets` sale 0

## 4. Tests (red) — unhappy path

- [x] 4.1 Agregar tests unhappy `get_search_q_too_long_returns_400`, `get_search_offset_1001_returns_400`, `get_search_limit_zero_returns_400`, `get_search_limit_51_returns_400`, `get_search_invalid_category_id_returns_400`, `get_search_invalid_brand_id_returns_400`, `get_search_invalid_sort_returns_400`, `get_search_attr_prefix_clave_returns_400`, `get_search_unavailable_returns_503` y `get_search_without_service_returns_503` — `cargo test -p api --test search get_search_q_too_long get_search_offset_1001 get_search_limit_zero get_search_limit_51 get_search_invalid_category get_search_invalid_brand get_search_invalid_sort get_search_attr_prefix get_search_unavailable get_search_without_service` no compila o falla; JSON con `assert_conforms` 400/503 (escenarios unhappy de «Endpoint GET /api/v1/search»)

## 5. Implementación (green) — validación y errores

- [x] 5.1 Mapear ULID/sort inválidos y `SearchQuery` de dominio a `400 invalid_input` (`errors[].field` inglés); `Unavailable` y ausencia de servicio a `503`; MUST NOT devolver `200` con `degraded: true` — `cargo test -p api --test search get_search_q_too_long get_search_offset_1001 get_search_limit_zero get_search_limit_51 get_search_invalid_category get_search_invalid_brand get_search_invalid_sort get_search_attr_prefix get_search_unavailable get_search_without_service` sale 0

## 6. Tests (red) — timeout, cancelación y métrica

- [x] 6.1 Agregar tests `get_search_timeout_returns_504` y `get_search_client_abort_drops_work` — `cargo test -p api --test search get_search_timeout_returns_504 get_search_client_abort_drops_work` no compila o falla (`assert_conforms` 504; Listo cuando: cancelación; escenarios de «Timeout de 2 s y cancelación»)
- [x] 6.2 Agregar test `get_search_records_duration_histogram` — `cargo test -p api --test search get_search_records_duration_histogram` no compila o falla (escenario «una búsqueda queda en el histograma»)

## 7. Implementación (green) — timeout 2 s, Drop y histograma

- [x] 7.1 `tokio::time::timeout(2s)` alrededor de `SearchGateway::search`; agotado → `504 timeout`; fake colgado + `TcpStream` abortado dropea el future — `cargo test -p api --test search get_search_timeout_returns_504 get_search_client_abort_drops_work` sale 0
- [x] 7.2 `describe_histogram` + `histogram!(search_duration_seconds)` al completar `GET /search` — `cargo test -p api --test search get_search_records_duration_histogram` sale 0

## 8. OpenAPI y composition root

- [x] 8.1 Declarar `'503'` en `GET /api/v1/search` de `docs/openapi.yaml` — `assert_conforms` de 503 en `get_search_unavailable_returns_503` / `get_search_without_service_returns_503` no falla por schema ausente
- [x] 8.2 `vortex serve` inyecta `SearchService` (`MeiliTextSearch` + uid `productos`) vía `AppState::with_search`; test `vortex_serve_get_search_is_not_404` en `vortex-core/bin/vortex/tests/serve.rs` — `cargo test -p vortex vortex_serve_get_search_is_not_404` sale 0 (503 aceptable con Meilisearch en `:1`; nunca 404)

## 9. Cierre (regresión)

- [x] 9.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 9.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 9.3 Confirmar non-goals: `rg` en el handler sin `SearchFacets` / fallback Surreal / `degraded: true`; sin UI; sin `serde_qs`
- [x] 9.4 `openspec validate T-03-13` sale 0