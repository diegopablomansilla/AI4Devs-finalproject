# Tasks

## 1. Contrato OpenAPI

- [x] 1.1 Agregar `'503': { $ref: '#/components/responses/Unavailable' }` a `GET`/`POST /api/v1/brands` y `PATCH`/`DELETE /api/v1/brands/{brandId}` en `docs/openapi.yaml` **sin** agregar `409` en `DELETE` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en esos paths (escenarios «sin servicio inyectado» / «DELETE no declara 409»)

## 2. Tests `catalog` (red)

- [x] 2.1 Agregar `vortex-core/crates/catalog/src/marca.rs` (o módulo de tests) con happy `catalog_create_fixser_sets_slug`, `catalog_rename_updates_nombre`, `catalog_update_slug`, `catalog_deactivate_sets_inactive`, `catalog_deactivate_with_associated_products_succeeds`, `catalog_reactivate_sets_active` — `cargo test -p catalog -- catalog_create_fixser catalog_rename catalog_update_slug catalog_deactivate catalog_reactivate` no compila o falla (HU-02-02 / Listo cuando)
- [x] 2.2 Agregar tests unhappy `catalog_create_duplicate_slug_is_duplicate_slug`, `catalog_create_blank_nombre_is_invalid_input`, `catalog_rename_unknown_id_is_not_found`, `catalog_deactivate_unknown_id_is_not_found` — siguen en rojo (escenarios unhappy de `catalog-marca`)

## 3. Casos de uso (green)

- [x] 3.1 Fake en memoria (`BrandRepo`: `HashMap`, slug único → `duplicate_slug`, `Slug::from_nombre`) suficiente para los tests 2.x — los tests 2.1–2.2 avanzan sin abrir SurrealDB
- [x] 3.2 `BrandService` (crear, get, list_all, editar nombre/slug, baja sin occupancy, reactivar) + trait object-safe `BrandGateway` y reexport en `catalog::lib` — `cargo test -p catalog` verde (escenarios de `catalog-marca`, incl. «el servicio es inyectable»)

## 4. Tests HTTP (red)

- [x] 4.1 Agregar `vortex-core/crates/api/tests/brands.rs` con fake de `BrandGateway` y tests happy `post_brand_fixser_returns_201`, `get_brands_omits_inactive_by_default`, `get_brands_include_inactive_lists_inactive`, `get_brands_q_filters_prefix`, `get_brands_ordered_by_name`, `patch_brand_renames`, `patch_brand_updates_slug`, `patch_brand_reactivates`, `delete_brand_returns_204`, `delete_brand_with_associated_products_returns_204` — `cargo test -p api --test brands` no compila o falla (HU-02-02 / Listo cuando)
- [x] 4.2 Agregar tests unhappy `post_brand_duplicate_slug_returns_409`, `patch_brand_unknown_id_returns_404`, `delete_brand_unknown_id_returns_404`, `post_brand_missing_name_returns_400`, `patch_brand_empty_body_returns_400`, `brand_id_invalid_ulid_returns_400`, `get_brands_q_too_long_returns_400`, `get_brands_without_service_returns_503`, `brand_problem_field_is_name_not_nombre` — siguen en rojo; los JSON (salvo 204) llaman `assert_conforms` (5.3 / Listo cuando)
- [x] 4.3 Agregar `vortex_serve_get_brands_returns_items` en tests del binario `vortex` — falla o no compila (escenario «vortex serve GET /api/v1/brands lista items»)

## 5. Gateway (green)

- [x] 5.1 `AppState` con `Option<Arc<dyn BrandGateway>>` y `with_brands`; DTO inglés ↔ dominio (`Brand` / `BrandCreate` / `BrandUpdate`); remap de `errors[].field` (`nombre`→`name`; path `brandId`) — los tests 4.1–4.2 que no dependen de rutas registradas avanzan
- [x] 5.2 Handlers `GET`/`POST /api/v1/brands` y `PATCH`/`DELETE /api/v1/brands/{brandId}` (`includeInactive`, `q` prefijo case-insensitive, orden por `name`, ULID inválido → 400, `q` > 100 → 400, sin servicio → 503, PATCH slug permitido, DELETE 204 sin 409) — `cargo test -p api --test brands` verde (todos los escenarios de `vortex-api` Endpoints / DTO)

## 6. Composition root (green)

- [x] 6.1 `serve.rs` inyecta `BrandService<MarcaRepo>` con `Arc<Db>` vía `with_brands` — `cargo test -p vortex -- vortex_serve_get_brands_returns_items` pasa

## 7. Cierre

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`; `openspec validate T-02-09` — [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5