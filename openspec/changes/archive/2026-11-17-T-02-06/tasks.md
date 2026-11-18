# Tasks

## 1. Allowlist y contrato OpenAPI

- [x] 1.1 Extender `scripts/check-workspace.sh` (y fixtures) para que `api` MAY declarar path `catalog` y `vortex` MAY declarar path `catalog` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0 (escenarios «api declara path catalog» / «vortex declara path catalog» / «api declara una dependencia fuera de la allowlist»)
- [x] 1.2 Agregar `'409': { $ref: '#/components/responses/Conflict' }` a `DELETE /api/v1/categories/{categoryId}` en `docs/openapi.yaml` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en ese path

## 2. Fachada `catalog` y `CategoriaRepo` `'static` (red → green)

- [x] 2.1 Exponer trait object-safe + occupancy de producción (`has_active_products` = `false`) y un test `catalog_production_occupancy_is_false` — `cargo test -p catalog -- catalog_production_occupancy_is_false` pasa (escenarios «el servicio es inyectable» / «occupancy no consulta productos»)
- [x] 2.2 Cambiar `CategoriaRepo` a poseer `Arc<Db>` — `cargo test -p storage --test categoria` sigue en verde (escenario «el adaptador sobrevive al stack de serve»)

## 3. Tests HTTP (red)

- [x] 3.1 Agregar `vortex-core/crates/api/tests/categories.rs` con fake de la fachada y tests happy `post_category_child_returns_201_with_path`, `get_categories_omits_inactive_by_default`, `get_categories_include_inactive_lists_inactive`, `patch_category_renames`, `patch_category_moves`, `patch_category_reactivates`, `delete_category_returns_204` — `cargo test -p api --test categories` no compila o falla (HU-02-01 / Listo cuando)
- [x] 3.2 Agregar tests unhappy `post_category_duplicate_slug_returns_409`, `patch_category_cycle_returns_400`, `delete_category_with_active_child_returns_409`, `patch_category_unknown_id_returns_404`, `delete_category_unknown_id_returns_404`, `post_category_missing_name_returns_400`, `patch_category_slug_only_returns_400`, `category_id_invalid_ulid_returns_400`, `get_categories_without_service_returns_503`, `category_problem_field_is_parent_id_not_padre` — siguen en rojo; los JSON (salvo 204) llaman `assert_conforms` (H-10 / 5.3 / Listo cuando)
- [x] 3.3 Agregar `vortex_serve_get_categories_returns_items` en tests del binario `vortex` — falla o no compila (escenario «vortex serve GET /api/v1/categories lista items»)

## 4. Gateway (green)

- [x] 4.1 Dependencia path `catalog` en `api`; `AppState` con `Option<Arc<dyn …>>` y `with_categories`; DTO inglés ↔ dominio; remap de `errors[].field` (`padre`→`parentId`, `nombre`→`name`) — los tests 3.1–3.2 que no dependen de rutas registradas avanzan
- [x] 4.2 Handlers `GET`/`POST /api/v1/categories` y `PATCH`/`DELETE /api/v1/categories/{categoryId}` (`includeInactive`, mapeo PATCH, ULID inválido → 400, sin servicio → 503) — `cargo test -p api --test categories` verde (todos los escenarios de `vortex-api` Endpoints / DTO)

## 5. Composition root (green)

- [x] 5.1 `vortex` declara path `catalog`; `serve.rs` inyecta `CategoryService<CategoriaRepo, OccupancyStub>` con `Arc<Db>` — `cargo test -p vortex -- vortex_serve_get_categories_returns_items` pasa

## 6. Cierre

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`; `openspec validate T-02-06` — [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5