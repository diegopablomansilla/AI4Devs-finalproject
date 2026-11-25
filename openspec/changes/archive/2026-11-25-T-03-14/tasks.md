# Tasks

## 1. Harness de índice efímero

- [x] 1.1 Crear `vortex-core/crates/search/tests/meilisearch_ranking.rs` con helpers `MEILISEARCH_URL`, uid `t0314-<ulid>`, `create_index` + `productos_index_settings([])` + `apply_settings` y `cleanup_index` (mismo molde que `meilisearch_adapter.rs`) — `cargo test -p search --test meilisearch_ranking` compila (cero tests o ignore; escenario «la suite por defecto no exige el sidecar»)

## 2. Tests (red → green) — happy path / Listo cuando

- [x] 2.1 Agregar `meilisearch_ranking_tornilo_finds_tornillo_autoperforante` (`#[ignore = "meilisearch"]`): upsert `nombre` `"Tornillo autoperforante 10 x 1"`, `q` `"tornilo"`, `SearchOrden::Relevancia` — `cargo test -p search -- --include-ignored meilisearch_ranking_tornilo_finds_tornillo_autoperforante` sale 0 (escenario «tornilo encuentra Tornillo autoperforante»)
- [x] 2.2 Agregar `meilisearch_ranking_perno_orders_steel_before_washer`: upsert `"Perno de Acero"` y `"Arandela para Perno"`, `q` `"Perno"` — `cargo test -p search -- --include-ignored meilisearch_ranking_perno_orders_steel_before_washer` sale 0 y el hit de acero aparece antes (escenario «Perno ordena Perno de Acero antes que Arandela para Perno»)
- [x] 2.3 Agregar `meilisearch_ranking_equal_relevance_orders_stock_40_before_3`: dos documentos con el mismo `nombre`/`tipo`, `sku` distintos que no están en `q`, `stock_disponible` 40 y 3 — `cargo test -p search -- --include-ignored meilisearch_ranking_equal_relevance_orders_stock_40_before_3` sale 0 (escenario «a igual relevancia gana el stock 40»)

## 3. Tests (red → green) — unhappy path

- [x] 3.1 Agregar `meilisearch_ranking_sku_typo_does_not_match`: documento `sku` `"SKUABCDEFGH"` y `nombre` sin ese token; `q` `"SKUABCDEFGI"` — `cargo test -p search -- --include-ignored meilisearch_ranking_sku_typo_does_not_match` sale 0 y `hits` no incluye el documento (escenario «typo en sku no encuentra el producto»)
- [x] 3.2 Agregar `meilisearch_ranking_unknown_query_returns_empty_hits`: índice con al menos un documento; `q` `"xyzzy-no-match-9f3"` — `cargo test -p search -- --include-ignored meilisearch_ranking_unknown_query_returns_empty_hits` sale 0, `hits` vacío, sin `DomainError` (escenario «consulta sin coincidencias devuelve página vacía»)

## 4. Cierre (regresión)

- [x] 4.1 En `vortex-core/`: `cargo test --workspace` **sin** `--include-ignored` sale 0 y los `meilisearch_ranking_*` figuran como ignorados (escenario «la suite por defecto no exige el sidecar»)
- [x] 4.2 Con sidecar (o en CI): `cargo test --workspace -- --include-ignored meilisearch` sale 0 e incluye los tres Listo cuando (escenario «los tres escenarios Listo cuando corren en CI»)
- [x] 4.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.4 Confirmar non-goals: sin cambios en `productos_index_settings`, handler `api`, `openapi.yaml`, CI yaml ni `seed/`; `rg` en el test sin `GET /api/v1/search` ni Surreal
- [x] 4.5 `openspec validate T-03-14` sale 0