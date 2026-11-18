# Tasks

## 1. Contrato OpenAPI

- [x] 1.1 Agregar `'503': { $ref: '#/components/responses/Unavailable' }` a `GET /api/v1/products` en `docs/openapi.yaml` **sin** agregar `q` ni tocar `ProductSummary` / `ProductSummaryPage` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en ese path (escenario «sin servicio inyectado»)

## 2. Tests domain (red)

- [x] 2.1 Agregar tests `domain_filtro_producto_has_categoria_marca_activo` y `domain_resumen_producto_exposes_marca_nombre_and_ruta` en `vortex-core/crates/domain` — `cargo test -p domain domain_filtro_producto domain_resumen_producto` no compila o falla (escenarios FiltroProducto / ResumenProducto)
- [x] 2.2 Agregar test `domain_product_repo_declares_list` que no compile hasta que `ProductRepo` declare `list` — `cargo test -p domain domain_product_repo_declares_list` no compila o falla (escenario «El puerto declara list»)

## 3. Domain (green)

- [x] 3.1 Exponer `FiltroProducto` y `ResumenProducto` en `domain::producto`; agregar `ProductRepo::list(filtro, after, take) -> Vec<ResumenProducto>`; actualizar dummies para que compilen (`Ok(vec![])` si no ejercitan listado) — `cargo test -p domain domain_filtro_producto domain_resumen_producto domain_product_repo_declares_list` sale 0 (capabilities `domain-producto`)

## 4. Tests catalog (red)

- [x] 4.1 Agregar tests happy `catalog_list_branch_includes_descendants` y `catalog_list_inactive_only` en `vortex-core/crates/catalog` (repos en memoria) — `cargo test -p catalog catalog_list_branch_includes_descendants catalog_list_inactive_only` no compila o falla (HU-02-10 / Listo cuando)
- [x] 4.2 Agregar tests `catalog_list_filters_by_brand`, `catalog_list_orders_by_id_and_after` y `catalog_list_unknown_category_is_empty` — `cargo test -p catalog catalog_list_filters_by_brand catalog_list_orders_by_id_and_after catalog_list_unknown_category_is_empty` no compila o falla

## 5. Catalog list (green)

- [x] 5.1 Implementar `ProductService::list` + `ProductGateway::list` que delegan en `ProductRepo::list`; fake en memoria con filtro de rama vía `Categoria.ancestros`; sin HTTP ni SurrealDB — `cargo test -p catalog catalog_list` sale 0 (capability `catalog-producto`)

## 6. Tests storage (red)

- [x] 6.1 Agregar `storage_list_branch_includes_descendants` y `storage_list_inactive_only` en `vortex-core/crates/storage/tests/producto.rs` — `cargo test -p storage --test producto storage_list_branch storage_list_inactive` no compila o falla (HU-02-10 / Listo cuando)
- [x] 6.2 Agregar tests `storage_list_filters_by_brand`, `storage_list_after_excludes_prior_ids`, `storage_list_unknown_category_is_empty` y `storage_list_does_not_write_outbox` — siguen en rojo (escenarios marca / after / vacía / no escribe)

## 7. Storage list (green)

- [x] 7.1 Implementar `ProductoRepo::list` con `categoria = $c OR categoria.ancestros CONTAINS $c`, filtro `activo`/`marca`, `id > after`, `ORDER BY id`, `LIMIT take`, hidratando `marca_nombre` y `categoria_ruta` sin atributos — `cargo test -p storage --test producto storage_list` sale 0 (capability `vortex-storage`)

## 8. Tests HTTP (red)

- [x] 8.1 Reemplazar `get_products_list_is_not_summary_page` por tests happy `get_products_branch_filter_includes_descendants` y `get_products_active_false_returns_inactive_only` en `vortex-core/crates/api/tests/products.rs` (fake de `ProductGateway`) — `cargo test -p api --test products get_products_branch_filter get_products_active_false` no compila o falla (HU-02-10 / Listo cuando; `assert_conforms` 200)
- [x] 8.2 Agregar tests happy `get_products_default_excludes_inactive`, `get_products_filters_by_brand`, `get_products_paginates_by_cursor`, `get_products_empty_page` y `get_products_summary_has_english_dto` — siguen en rojo (`assert_conforms` 200)
- [x] 8.3 Agregar tests unhappy `get_products_invalid_cursor_returns_400`, `get_products_limit_zero_returns_400`, `get_products_limit_101_returns_400`, `get_products_invalid_category_id_returns_400`, `get_products_invalid_brand_id_returns_400`, `get_products_q_does_not_text_search` y `get_products_without_service_returns_503` — siguen en rojo; JSON con `assert_conforms` 400/503 (unhappy / No hace)
- [x] 8.4 Agregar `vortex_serve_get_products_empty_returns_200` en `vortex-core/bin/vortex/tests/serve.rs` — falla o no compila (escenario «servicio inyectado: 200 vacío no 503»)

## 9. Gateway (green)

- [x] 9.1 DTO `ProductSummary` / `ProductSummaryPage` (5.3; `brand` string; `imageUrl` `null`); query `categoryId`/`brandId`/`active`/`cursor`/`limit`; remap ULID inválido → `categoryId`/`brandId`; helper T-00-21 para cursor/`limit`/`paginate` — los tests 8.x que no dependen de la ruta GET avanzan (escenarios DTO / unhappy parseo)
- [x] 9.2 Registrar `GET /api/v1/products` junto al `POST` existente; sin servicio → 503; `q` ignorado; **sin** `GET /search` — `cargo test -p api --test products get_products` verde (todos los escenarios HTTP de `vortex-api`)

## 10. Composition root (green)

- [x] 10.1 Confirmar que `serve.rs` ya inyecta `ProductService` vía `with_products` (mismo gateway que la ficha); no hace falta un segundo servicio — `cargo test -p vortex -- vortex_serve_get_products_empty_returns_200` pasa

## 11. Cierre (regresión)

- [x] 11.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 11.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 11.3 Confirmar non-goals: `rg` sin handler `GET /search` ni campo `q` en el query de listado; `git diff docs/openapi.yaml` sin cambios a `ProductSummary` ni a paths de búsqueda; `imageUrl` siempre `null` en el mapper
- [x] 11.4 `openspec validate T-02-27` sale 0