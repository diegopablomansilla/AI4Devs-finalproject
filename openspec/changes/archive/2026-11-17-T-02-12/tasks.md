# Tasks

## 1. Contrato OpenAPI

- [x] 1.1 Agregar `'503': { $ref: '#/components/responses/Unavailable' }` a `GET`/`POST /api/v1/attribute-definitions` y `PATCH`/`DELETE /api/v1/attribute-definitions/{attributeId}` en `docs/openapi.yaml` **sin** agregar `409` en `DELETE` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en esos paths (escenarios «sin servicio inyectado» / «DELETE no declara 409»)

## 2. Tests `catalog` (red)

- [x] 2.1 Agregar `vortex-core/crates/catalog/src/atributo_def.rs` (o módulo de tests) con happy `catalog_create_grado_enum_sets_options`, `catalog_create_medida_texto`, `catalog_update_etiqueta`, `catalog_update_clave_unused_succeeds`, `catalog_deactivate_sets_inactive`, `catalog_reactivate_sets_active` — `cargo test -p catalog -- catalog_create_grado catalog_create_medida catalog_update_etiqueta catalog_update_clave catalog_deactivate catalog_reactivate` no compila o falla (HU-02-03)
- [x] 2.2 Agregar tests unhappy `catalog_create_duplicate_clave_is_conflict`, `catalog_create_enum_without_options_is_invalid_input`, `catalog_update_unknown_id_is_not_found`, `catalog_deactivate_unknown_id_is_not_found` — siguen en rojo (escenarios unhappy de `catalog-atributo-def`, HU-02-03 enum sin opciones)

## 3. Casos de uso (green)

- [x] 3.1 Fake en memoria (`AttributeDefRepo`: `HashMap`, `clave` única → `conflict`, validación de `NewAtributoDef`/`UpdateAtributoDef`) suficiente para los tests 2.x — los tests 2.1–2.2 avanzan sin abrir SurrealDB
- [x] 3.2 `AttributeDefService` (crear, get, list_all, editar con merge, baja sin occupancy, reactivar) + trait object-safe `AttributeDefGateway` y reexport en `catalog::lib` — `cargo test -p catalog` verde (escenarios de `catalog-atributo-def`, incl. «el servicio es inyectable»)

## 4. Tests HTTP (red)

- [x] 4.1 Agregar `vortex-core/crates/api/tests/attribute_definitions.rs` con fake de `AttributeDefGateway` y tests happy `post_attribute_def_grado_returns_201`, `get_attribute_defs_omits_inactive_by_default`, `get_attribute_defs_include_inactive_lists_inactive`, `get_attribute_defs_ordered_by_order`, `patch_attribute_def_updates_label`, `patch_attribute_def_changes_key_without_in_use`, `patch_attribute_def_reactivates`, `delete_attribute_def_returns_204` — `cargo test -p api --test attribute_definitions` no compila o falla (HU-02-03 / Listo cuando)
- [x] 4.2 Agregar tests unhappy `post_attribute_def_duplicate_key_returns_409`, `post_attribute_def_enum_without_options_returns_400`, `patch_attribute_def_unknown_id_returns_404`, `delete_attribute_def_unknown_id_returns_404`, `post_attribute_def_missing_key_returns_400`, `patch_attribute_def_empty_body_returns_400`, `attribute_id_invalid_ulid_returns_400`, `get_attribute_defs_without_service_returns_503`, `attribute_def_problem_field_is_key_not_clave` — siguen en rojo; los JSON (salvo 204) llaman `assert_conforms` (5.3 / Listo cuando)
- [x] 4.3 Agregar `vortex_serve_get_attribute_definitions_returns_items` en tests del binario `vortex` — falla o no compila (escenario «vortex serve GET /api/v1/attribute-definitions lista items»)

## 5. Gateway (green)

- [x] 5.1 `AppState` con `Option<Arc<dyn AttributeDefGateway>>` y `with_attribute_defs`; DTO inglés ↔ dominio (`AttributeDefinition` / Create / Update; `text`↔`texto` …); remap de `errors[].field` (`clave`→`key`, `etiqueta`→`label`, `tipo`→`type`, `opciones`→`options`; path `attributeId`) — los tests 4.1–4.2 que no dependen de rutas registradas avanzan
- [x] 5.2 Handlers `GET`/`POST /api/v1/attribute-definitions` y `PATCH`/`DELETE /api/v1/attribute-definitions/{attributeId}` (`includeInactive`, orden por `order`/`key`, ULID inválido → 400, sin servicio → 503, PATCH merge + `active`, DELETE 204, duplicado → 409 `conflict`, enum sin opciones → 400) — `cargo test -p api --test attribute_definitions` verde (todos los escenarios de `vortex-api` Endpoints / DTO)

## 6. Composition root (green)

- [x] 6.1 `serve.rs` inyecta `AttributeDefService<AtributoDefRepo>` con `Arc<Db>` vía `with_attribute_defs` — `cargo test -p vortex -- vortex_serve_get_attribute_definitions_returns_items` pasa

## 7. Cierre

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`; `openspec validate T-02-12` — [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5