# Tasks

## 1. Contrato OpenAPI

- [x] 1.1 Agregar `'503': { $ref: '#/components/responses/Unavailable' }` a `DELETE /api/v1/products/{productId}` en `docs/openapi.yaml` **sin** agregar listado `GET /api/v1/products` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en ese path (escenario «DELETE sin servicio inyectado»)

## 2. Tests domain (red)

- [x] 2.1 Agregar test `product_repo_trait_exposes_set_activo` en `vortex-core/crates/domain/src/producto.rs` — `cargo test -p domain product_repo_trait_exposes_set_activo` no compila o falla (escenario «El puerto declara set_activo»)

## 3. Puerto domain (green)

- [x] 3.1 Añadir `set_activo(id, activo) -> FichaProducto` a `ProductRepo` y un dummy de test — `cargo test -p domain product_repo_trait_exposes_set_activo` sale 0; `UpdateProducto` sigue sin `activo`

## 4. Tests storage (red)

- [x] 4.1 Agregar tests `storage_set_activo_false_keeps_ledger_intact`, `storage_set_activo_true_reactivates`, `storage_set_activo_false_is_idempotent`, `storage_set_activo_unknown_id_is_not_found` en `vortex-core/crates/storage/tests/producto.rs` — `cargo test -p storage storage_set_activo` no compila o falla (HU-02-06 / Listo cuando: ledger intacto)

## 5. Storage set_activo (green)

- [x] 5.1 Implementar `ProductoRepo::set_activo` (`UPDATE activo` + outbox `producto`/`upsert`, sin tocar `movimiento_stock`/`stock`/`version`) — `cargo test -p storage storage_set_activo` sale 0; `cargo test -p storage` sigue verde en create/get/update

## 6. Tests catalog (red)

- [x] 6.1 Agregar tests `catalog_deactivate_sets_inactive`, `catalog_reactivate_sets_active`, `catalog_deactivate_idempotent_when_already_inactive`, `catalog_get_inactive_product_returns_ficha`, `catalog_deactivate_unknown_id_is_not_found`, `catalog_reactivate_unknown_id_is_not_found` en `vortex-core/crates/catalog/src/producto.rs` — `cargo test -p catalog catalog_deactivate catalog_reactivate catalog_get_inactive` no compila o falla (HU-02-06)

## 7. Catalog baja/reactivación (green)

- [x] 7.1 Implementar `deactivate`/`reactivate` en `ProductService`, extender `ProductGateway` y el fake en memoria con `set_activo` — `cargo test -p catalog catalog_deactivate catalog_reactivate catalog_get_inactive` sale 0; `PatchProducto` sigue sin `activo`

## 8. Tests HTTP (red)

- [x] 8.1 En `vortex-core/crates/api/tests/products.rs`, reemplazar `patch_product_active_returns_400` y agregar happy `delete_product_returns_204`, `patch_product_active_true_reactivates`, `patch_product_active_false_deactivates`, `get_inactive_product_returns_200` — `cargo test -p api --test products delete_product_returns patch_product_active get_inactive_product` no compila o falla (HU-02-06 / Listo cuando)
- [x] 8.2 Agregar unhappy `delete_product_unknown_id_returns_404`, `delete_product_invalid_ulid_returns_400`, `delete_product_without_service_returns_503`, `patch_product_active_unknown_id_returns_404` — siguen en rojo; los JSON llaman `assert_conforms` salvo 204
- [x] 8.3 Agregar `vortex_serve_delete_unknown_product_returns_404` en `vortex-core/bin/vortex/tests/serve.rs` — falla o no compila (404 ≠ 503)

## 9. Gateway HTTP (green)

- [x] 9.1 Extender fake `ProductGateway` (`deactivate`/`reactivate`); quitar `ApiError::patch_product_active`; `PATCH { active }` llama al gateway; `{ version }` solo sigue 400 — los tests 8.1 que no dependen de `DELETE` registrado avanzan
- [x] 9.2 Registrar `DELETE /api/v1/products/{productId}` (204, ULID inválido → 400, sin servicio → 503) — `cargo test -p api --test products` verde (todos los escenarios de baja/reactivación)

## 10. Composition root (green)

- [x] 10.1 Confirmar que `serve.rs` ya inyecta `ProductGateway` (sin cambio de cableado salvo que el trait rompa compilación) — `cargo test -p vortex -- vortex_serve_delete_unknown_product_returns_404 vortex_serve_get_unknown_product_returns_404` pasa

## 11. Cierre (regresión)

- [x] 11.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 11.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 11.3 Confirmar non-goals: `rg` sin llamada a Meilisearch ni handler de listado `GET /api/v1/products` en `crates/api/src`; `set_activo` no ejecuta `DELETE` de `movimiento_stock`
- [x] 11.4 `openspec validate T-02-22` sale 0