# Tasks

## 1. Tests (red) — dominio

- [x] 1.1 Agregar tipos `ProductoRecomendado` y `Recomendaciones` (pueden no compilar aún) y tests `producto_recomendado_covers_panel_item` y `recomendaciones_groups_three_blocks` en `vortex-core/crates/domain/src/relacion.rs` — `cargo test -p domain producto_recomendado_covers recomendaciones_groups_three_blocks` no compila o falla (escenarios «ProductoRecomendado cubre el ítem del panel» y «Recomendaciones agrupa los tres bloques»)
- [x] 1.2 Extender el dummy de `RelationRepo` y agregar `relation_repo_trait_exposes_recommendations_for_product` — `cargo test -p domain relation_repo_trait_exposes_recommendations_for_product` falla o no compila (escenario «El puerto declara recommendations_for_product»)

## 2. Implementación (green) — dominio

- [x] 2.1 Implementar `ProductoRecomendado` / `Recomendaciones`, reexportarlos en `domain::lib`, añadir `recommendations_for_product` al trait y al dummy (éxito vacío o `not_found`) — `cargo test -p domain relacion` sale 0 (incluye 1.1 y 1.2)
- [x] 2.2 Promover el formateo de `ubicacion` a API pública de `domain` (misma regla que `build_search_document`: `P3 · E-B · 12`) y dejar que `search_document` lo reutilice — `cargo test -p domain build_search_document_formats_ubicacion` sale 0
- [x] 2.3 Ajustar `RelacionRepo` en `storage` y el `InMemoryRelationRepo` de `recommend` con un stub de `recommendations_for_product` (p. ej. `not_found`) para que el workspace compile — `cargo test -p recommend recommend_list_unknown_product_is_not_found` y `cargo test -p storage --test relacion_repo relation_repo_list_for_product_empty_when_no_edges` salen 0

## 3. Tests (red) — storage

- [x] 3.1 Agregar tests happy `relation_repo_recommendations_three_blocks_ordered` y `relation_repo_recommendations_excludes_inactive_target` en `vortex-core/crates/storage/tests/relacion_repo.rs` — `cargo test -p storage --test relacion_repo relation_repo_recommendations_three_blocks relation_repo_recommendations_excludes_inactive` falla (escenarios HU-04-05 «Tres bloques ordenados por prioridad» y «Productos inactivos no aparecen»; **Listo cuando**)
- [x] 3.2 Agregar tests happy `relation_repo_recommendations_highlights_substitutes_when_stock_zero` y `relation_repo_recommendations_does_not_highlight_when_in_stock` — `cargo test -p storage --test relacion_repo relation_repo_recommendations_highlights relation_repo_recommendations_does_not_highlight` falla (escenarios HU-04-06)
- [x] 3.3 Agregar tests `relation_repo_recommendations_includes_formatted_location`, `relation_repo_recommendations_empty_when_no_edges` (happy) y `relation_repo_recommendations_unknown_product_is_not_found` (unhappy) — `cargo test -p storage --test relacion_repo relation_repo_recommendations_includes_formatted relation_repo_recommendations_empty relation_repo_recommendations_unknown` falla (ubicación, vectores vacíos, `not_found`)

## 4. Implementación (green) — storage

- [x] 4.1 Implementar `recommendations_for_product` con **una** consulta 4.6.2 (`destacar_sustitutos`, tres bloques, `out.activo`, `out.stock > 0` en sustitutos, id de arista, stock/precio/ubicación); producto ausente → `not_found`; **sin** cambiar `list_for_product` ni el esquema `0007` — `cargo test -p storage --test relacion_repo relation_repo_recommendations` sale 0 (todos los escenarios de `vortex-storage`)
- [x] 4.2 Confirmar que `list_for_product` sigue incluyendo destino inactivo y stock 0 — `cargo test -p storage --test relacion_repo relation_repo_list_for_product_groups_by_type_with_destino_resumen` sale 0

## 5. Tests (red) — recommend

- [x] 5.1 Agregar tests happy `recommend_get_three_blocks_ordered`, `recommend_get_excludes_inactive_target`, `recommend_get_highlights_substitutes_when_stock_zero` y `recommend_get_does_not_highlight_when_in_stock` en `vortex-core/crates/recommend` (fake del puerto) — `cargo test -p recommend recommend_get_three_blocks recommend_get_excludes recommend_get_highlights recommend_get_does_not_highlight` no compila o falla (HU-04-05 / HU-04-06)
- [x] 5.2 Agregar tests `recommend_get_unknown_product_is_not_found` (unhappy) y `recommendation_gateway_is_object_safe` — `cargo test -p recommend recommend_get_unknown_product recommendation_gateway_is_object_safe` falla o no compila (escenarios `not_found` y fachada object-safe)

## 6. Implementación (green) — recommend

- [x] 6.1 Implementar `RecommendationService` + `RecommendationGateway` object-safe (`get` delega en `recommendations_for_product`, propaga `not_found`) **sin** HTTP ni SurrealDB; el fake en memoria aplica los filtros 4.6.2 — `cargo test -p recommend recommend_get recommendation_gateway` sale 0 (escenarios de `vortex-recommend` consulta)
- [x] 6.2 Confirmar non-goals HTTP/DC-05: `rg` sin handler de `GET /api/v1/products/{id}/recommendations` en `crates/api/src`; `catalog` sin `RecommendationGateway`; tests de ABM de `recommend` siguen verdes — `cargo test -p recommend` y `cargo test -p api --test relations get_recommendations_is_not_registered` salen 0

## 7. Cierre (regresión)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 `openspec validate T-04-06` sale 0