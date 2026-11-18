# Tasks

## 1. Contrato OpenAPI

- [x] 1.1 Agregar `'503': { $ref: '#/components/responses/Unavailable' }` a `POST /api/v1/products` y a `GET`/`PATCH /api/v1/products/{productId}` en `docs/openapi.yaml` **sin** registrar listado ni `DELETE` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en esos paths (escenario «sin servicio inyectado»)

## 2. Tests domain/storage/catalog (red) — terminal

- [x] 2.1 Agregar test `new_producto_accepts_optional_terminal` en `vortex-core/crates/domain/src/producto.rs` — `cargo test -p domain new_producto_accepts_optional_terminal` no compila o falla (escenario «NewProducto incluye terminal opcional»)
- [x] 2.2 Agregar test `storage_create_persists_terminal_on_initial_movement` en `vortex-core/crates/storage/tests/producto.rs` — `cargo test -p storage storage_create_persists_terminal_on_initial_movement` no compila o falla (escenario «Alta con terminal en el movimiento inicial»)
- [x] 2.3 Agregar test `catalog_create_forwards_terminal_to_repo` en `vortex-core/crates/catalog/src/producto.rs` — `cargo test -p catalog catalog_create_forwards_terminal_to_repo` no compila o falla (escenario «Alta reenvía terminal al puerto»)

## 3. Plomería terminal (green)

- [x] 3.1 Añadir `terminal: Option<String>` a `NewProducto` y actualizar call sites (`terminal: None` salvo tests 2.x) — `cargo test -p domain new_producto_accepts_optional_terminal` y `cargo test -p catalog catalog_create_forwards_terminal_to_repo` salen 0
- [x] 3.2 En `storage/src/producto.rs`, persistir `terminal` en el `CREATE movimiento_stock` de `stock_inicial > 0` — `cargo test -p storage storage_create_persists_terminal_on_initial_movement` sale 0; el alta sin terminal sigue con `terminal` ausente

## 4. Tests HTTP (red)

- [x] 4.1 Agregar `vortex-core/crates/api/tests/products.rs` con fake de `ProductGateway` y tests happy `post_product_perno_m8x40_returns_201_with_location`, `post_product_sends_terminal_on_initial_stock`, `get_product_returns_200`, `patch_product_price_increments_version`, `patch_product_partial_keeps_other_fields` — `cargo test -p api --test products post_product_perno get_product_returns patch_product_price patch_product_partial post_product_sends_terminal` no compila o falla (HU-02-04 / HU-02-05)
- [x] 4.2 Agregar tests unhappy `post_product_duplicate_sku_returns_409`, `post_product_cuarenta_on_number_attribute_returns_400`, `patch_product_stale_version_returns_409`, `get_product_unknown_id_returns_404`, `patch_product_unknown_id_returns_404`, `product_id_invalid_ulid_returns_400`, `post_product_missing_name_returns_400`, `post_product_price_as_json_number_returns_400`, `patch_product_active_returns_400`, `post_product_terminal_too_long_returns_400`, `post_product_without_service_returns_503`, `get_products_list_is_not_summary_page`, `product_problem_field_is_category_id_not_categoria` — siguen en rojo; los JSON llaman `assert_conforms` (Listo cuando / 5.3)
- [x] 4.3 Agregar `vortex_serve_get_unknown_product_returns_404` en `vortex-core/bin/vortex/tests/serve.rs` — falla o no compila (escenario «vortex serve GET producto desconocido es 404 no 503»)

## 5. Gateway (green)

- [x] 5.1 `AppState` con `Option<Arc<dyn ProductGateway>>` y `with_products`; DTO inglés ↔ dominio (`Product` / Create / Update; enums 5.3; `price`/`stock` string; `images: []`); remap `categoria`→`categoryId`, `marca`→`brandId`; `tipo` de ficha → `productType` en el handler — los tests 4.1–4.2 que no dependen de rutas registradas avanzan
- [x] 5.2 Handlers `POST /api/v1/products` y `GET`/`PATCH /api/v1/products/{productId}` (`Location`, `X-Vectra-Terminal` ≤ 64, ULID inválido → 400, sin servicio → 503, `active` → 400, duplicado → 409, `version_conflict` → 409, cuarenta → 400) **sin** `GET` de listado ni `DELETE` — `cargo test -p api --test products` verde (todos los escenarios de `vortex-api` Endpoints / DTO)

## 6. Composition root (green)

- [x] 6.1 `serve.rs` inyecta `ProductService` con `ProductoRepo` + `CategoriaRepo` + `MarcaRepo` + `AtributoDefRepo` vía `with_products` — `cargo test -p vortex -- vortex_serve_get_unknown_product_returns_404` pasa

## 7. Cierre (regresión)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 Confirmar non-goals: `rg` sin `DELETE /api/v1/products` ni handler de listado `GET /api/v1/products` en `crates/api/src`; `git diff docs/openapi.yaml` sin paths de listado ni `DELETE`
- [x] 7.4 `openspec validate T-02-21` sale 0