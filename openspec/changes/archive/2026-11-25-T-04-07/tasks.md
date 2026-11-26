# Tasks

## 1. Contrato OpenAPI y 5.3

- [x] 1.1 Agregar `'400': { $ref: '#/components/responses/BadRequest' }` y `'503': { $ref: '#/components/responses/Unavailable' }` a `GET /api/v1/products/{productId}/recommendations` en `docs/openapi.yaml` — el path declara 200/400/404/503 (escenario «sin servicio es 503» / «productId inválido es 400»)
- [x] 1.2 Completar [docs/5-contract-api.md](../../../docs/5-contract-api.md) § 5.3 con `highlightSubstitutes`→`destacar_sustitutos`, `similar`/`substitutes`/`complements`→`similares`/`sustitutos`/`complementarios`, `relationId`→`relacion_id`, y `location` del panel como string formateado — las filas existen y no contradicen `location.aisle` de la ficha (escenario «GET no filtra claves de dominio»)

## 2. Tests (red) — happy path HTTP

- [x] 2.1 Agregar `vortex-core/crates/api/tests/support/fake_recommendations.rs` y tests happy `get_recommendations_three_blocks_returns_200`, `get_recommendations_highlights_substitutes_when_stock_zero`, `get_recommendations_excludes_inactive_target`, `get_recommendations_empty_when_no_edges_returns_200`, `get_recommendations_image_url_is_null` y `get_recommendations_json_is_english` en `vortex-core/crates/api/tests/recommendations.rs` — `cargo test -p api --test recommendations get_recommendations_three_blocks get_recommendations_highlights get_recommendations_excludes get_recommendations_empty get_recommendations_image_url get_recommendations_json` no compila o falla (escenarios happy de «Endpoint GET /api/v1/products/{productId}/recommendations» y «DTO inglés»; `assert_conforms` 200)

## 3. Implementación (green) — ruta, DTO y AppState

- [x] 3.1 `AppState::with_recommendations` (`Option<Arc<dyn RecommendationGateway>>`), módulo `api/src/recommendations.rs`, ruta `GET /api/v1/products/{product_id}/recommendations` y mapper `Recomendaciones` → `Recommendations` (5.3: `highlightSubstitutes`, bloques inglés, `imageUrl` null, `stock`/`price` string) — `cargo test -p api --test recommendations get_recommendations_three_blocks get_recommendations_highlights get_recommendations_excludes get_recommendations_empty get_recommendations_image_url get_recommendations_json` sale 0 (escenarios happy + «GET recommendations está registrado»)

## 4. Tests (red) — unhappy path

- [x] 4.1 Agregar tests unhappy `get_recommendations_invalid_ulid_returns_400`, `get_recommendations_unknown_product_returns_404`, `get_recommendations_without_service_returns_503`, `get_recommendations_unavailable_returns_503` y `get_recommendations_has_no_max_age_cache` — `cargo test -p api --test recommendations get_recommendations_invalid_ulid get_recommendations_unknown_product get_recommendations_without_service get_recommendations_unavailable get_recommendations_has_no_max_age` no compila o falla; JSON con `assert_conforms` 400/404/503 (escenarios unhappy de «Endpoint GET …/recommendations»)

## 5. Implementación (green) — validación y errores

- [x] 5.1 ULID inválido → `400 invalid_input` field `productId`; `not_found` → 404; ausencia de servicio y `Unavailable` → 503; MUST NOT `Cache-Control: max-age` — `cargo test -p api --test recommendations get_recommendations_invalid_ulid get_recommendations_unknown_product get_recommendations_without_service get_recommendations_unavailable get_recommendations_has_no_max_age` sale 0

## 6. Tests (red) — métrica

- [x] 6.1 Agregar test `get_recommendations_records_duration_histogram` — `cargo test -p api --test recommendations get_recommendations_records_duration_histogram` no compila o falla (escenario «una consulta queda en el histograma»)

## 7. Implementación (green) — histograma

- [x] 7.1 `describe_histogram` + `histogram!(recommendations_duration_seconds)` al completar `GET .../recommendations` (éxito o error HTTP, sin labels) — `cargo test -p api --test recommendations get_recommendations_records_duration_histogram` sale 0; `cargo test -p api --test metrics` sigue en verde (escenario «sin consultas no exige la serie»)

## 8. Composition root

- [x] 8.1 `vortex serve` inyecta `RecommendationService` (`RelacionRepo`) vía `AppState::with_recommendations`; test `vortex_serve_get_recommendations_is_not_404` en `vortex-core/bin/vortex/tests/serve.rs` — `cargo test -p vortex vortex_serve_get_recommendations_is_not_404` sale 0 (404 de producto inexistente aceptable; nunca fallback de ruta no registrada)

## 9. Cierre (regresión)

- [x] 9.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 9.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 9.3 Confirmar non-goals: `rg` en el handler sin caché/`max-age`/hidratación de `imageUrl`; `recommend` sin rutas Axum; sin UI
- [x] 9.4 `openspec validate T-04-07` sale 0