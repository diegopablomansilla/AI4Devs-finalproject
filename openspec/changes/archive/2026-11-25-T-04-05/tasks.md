# Tasks

## 1. Contrato OpenAPI y allowlist

- [x] 1.1 Agregar `'503': { $ref: '#/components/responses/Unavailable' }` a `POST /api/v1/relations`, `PATCH`/`DELETE /api/v1/relations/{relationType}/{relationId}` y `GET /api/v1/products/{productId}/relations` en `docs/openapi.yaml` **sin** registrar `/recommendations` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en esos paths (escenario «sin servicio es 503»)
- [x] 1.2 Permitir `recommend` (`domain` path + `tokio` en dev), `api` path `recommend` y `vortex` path `recommend`: `Cargo.toml` de esos crates, `scripts/check-workspace.sh` y casos happy/unhappy en `scripts/fixtures/workspace/run-script-tests.sh` — `./scripts/check-workspace.sh` y el fixture de allowlist salen 0 (escenarios «recommend declara path domain», «api declara path recommend», «vortex declara path recommend»)

## 2. Tests domain/storage (red) — incoming

- [x] 2.1 Extender `relation_repo_trait_exposes_crud` (o agregar `relation_repo_trait_exposes_list_incoming`) en `vortex-core/crates/domain/src/relacion.rs` para exigir `list_incoming_for_product` — `cargo test -p domain relation_repo_trait` no compila o falla (escenario «El puerto declara list_incoming_for_product»)
- [x] 2.2 Agregar tests `relation_repo_list_incoming_summarizes_origin` y `relation_repo_list_incoming_empty` en `vortex-core/crates/storage/tests/relacion_repo.rs` — `cargo test -p storage --test relacion_repo relation_repo_list_incoming` no compila o falla (escenarios «Listado incoming con resumen del origen», «Incoming vacío»)

## 3. Puerto e incoming (green)

- [x] 3.1 Añadir `list_incoming_for_product` al trait `RelationRepo` y al dummy de test en `domain` — `cargo test -p domain relation_repo_trait` sale 0
- [x] 3.2 Implementar `list_incoming_for_product` en `storage/src/relacion.rs` (`out` = producto, `DestinoResumen` del `in`) **sin** cambiar `list_for_product` ni el esquema `0007` — `cargo test -p storage --test relacion_repo` sale 0

## 4. Tests recommend (red)

- [x] 4.1 Agregar tests happy `recommend_create_complement_perno_arandela`, `recommend_create_similar_default_bidirectional`, `recommend_update_priority_from_b_updates_both`, `recommend_list_includes_inactive_target` en `vortex-core/crates/recommend` (repos en memoria) — `cargo test -p recommend recommend_create_complement recommend_create_similar recommend_update_priority recommend_list_includes` no compila o falla (HU-04-01 / HU-04-02 / HU-04-03)
- [x] 4.2 Agregar tests unhappy `recommend_create_self_relation`, `recommend_create_duplicate_relation`, `recommend_list_unknown_product_is_not_found` — siguen en rojo (escenarios unhappy de «Casos de uso de ABM de relaciones»)

## 5. RelationService (green)

- [x] 5.1 Implementar `RelationService` + `RelationGateway` object-safe (`create` / `update` / `delete` / `list`; GET 404 vía `ProductRepo`; POST/PATCH re-lista el grupo bidireccional) **sin** HTTP ni SurrealDB — `cargo test -p recommend` verde (todos los escenarios de `vortex-recommend`)

## 6. Tests HTTP (red)

- [x] 6.1 Agregar `vortex-core/crates/api/tests/relations.rs` con fake de `RelationGateway` y tests happy `post_relation_complement_perno_arandela_returns_201`, `post_relation_similar_omits_bidirectional_creates_pair`, `patch_relation_priority_from_b_updates_both`, `delete_relation_from_b_removes_both`, `get_product_relations_lists_three_types`, `get_product_relations_incoming` — `cargo test -p api --test relations post_relation_complement post_relation_similar patch_relation_priority delete_relation_from_b get_product_relations_lists get_product_relations_incoming` no compila o falla (HU-04-01 / HU-04-02 / HU-04-03; `assert_conforms` 201/200/204)
- [x] 6.2 Agregar tests unhappy `post_relation_self_returns_400`, `post_relation_duplicate_returns_409`, `post_relation_missing_product_returns_404`, `get_product_relations_unknown_product_returns_404`, `patch_relation_unknown_id_returns_404`, `relation_id_invalid_ulid_returns_400`, `relation_type_unknown_returns_400`, `post_relation_priority_zero_returns_400`, `post_relation_without_service_returns_503`, `get_recommendations_is_not_registered`, `relation_problem_field_is_target_product_id` — siguen en rojo; JSON con `assert_conforms` (Listo cuando / 5.3)
- [x] 6.3 Agregar `vortex_serve_get_unknown_product_relations_returns_404` en `vortex-core/bin/vortex/tests/serve.rs` — falla o no compila (escenario composition root 404 ≠ 503)

## 7. Gateway HTTP (green)

- [x] 7.1 `AppState` con `Option<Arc<dyn RelationGateway>>` y `with_relations`; DTO inglés ↔ dominio (`Relation` / Create / Update; enums 5.3; `target` mínimo); remap `origen`→`sourceProductId`, `destino`→`targetProductId`, `prioridad`→`priority`, `nota`→`note` — los tests 6.1–6.2 que no dependen de rutas registradas avanzan
- [x] 7.2 Handlers `POST /api/v1/relations`, `PATCH`/`DELETE /api/v1/relations/{relationType}/{relationId}`, `GET /api/v1/products/{productId}/relations` (ULID inválido → 400, sin servicio → 503, `self_relation` → 400, duplicado → 409) **sin** `GET .../recommendations` — `cargo test -p api --test relations` verde (escenarios de `vortex-api` Endpoints / DTO)

## 8. Composition root (green)

- [x] 8.1 `serve.rs` inyecta `RelationService` con `RelacionRepo` + `ProductoRepo` vía `with_relations`; `bin/vortex/Cargo.toml` declara path `recommend` — `cargo test -p vortex vortex_serve_get_unknown_product_relations_returns_404` pasa (404, nunca 503)

## 9. Cierre (regresión)

- [x] 9.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 9.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 9.3 Confirmar non-goals: `rg` sin handler de `GET /api/v1/products/{id}/recommendations` en `crates/api/src`; `git diff docs/openapi.yaml` sin ese path nuevo de implementación; `catalog` sin `RelationGateway`
- [x] 9.4 `openspec validate T-04-05` sale 0
