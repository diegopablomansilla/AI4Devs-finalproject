# Tasks

## 1. Contrato OpenAPI

- [x] 1.1 Agregar `'503': { $ref: '#/components/responses/Unavailable' }` a `POST /api/v1/products/{productId}/stock-movements` en `docs/openapi.yaml` **sin** implementar ni alterar `GET` — `npx @redocly/cli lint docs/openapi.yaml` (o el job equivalente) no reporta error en ese path (escenario «sin servicio inyectado»)

## 2. Tests catalog (red)

- [x] 2.1 Agregar tests happy `catalog_register_inbound_sets_resulting_stock_and_terminal` y `catalog_register_adjustment_to_zero_with_reason` en `vortex-core/crates/catalog` (ledger en memoria) — `cargo test -p catalog catalog_register_inbound_sets_resulting_stock catalog_register_adjustment_to_zero` no compila o falla (escenarios Ingreso / ajuste a cero, HU-02-07)
- [x] 2.2 Agregar tests unhappy `catalog_register_ajuste_sin_motivo_is_invalid_input`, `catalog_register_ajuste_minus_8_on_stock_5_is_negative_stock`, `catalog_register_inicial_is_invalid_input`, `catalog_register_venta_is_invalid_input`, `catalog_register_unknown_product_is_not_found` y `catalog_register_ingreso_cantidad_negativa_is_invalid_input` — `cargo test -p catalog catalog_register_ajuste_sin_motivo catalog_register_ajuste_minus_8 catalog_register_inicial catalog_register_venta catalog_register_unknown_product catalog_register_ingreso_cantidad_negativa` no compila o falla (HU-02-07 / rechazo `inicial`/`venta`)

## 3. Catalog StockService (green)

- [x] 3.1 Implementar `StockService` + fachada object-safe `StockGateway::register` en `vortex-core/crates/catalog/src/stock.rs` (reexport en `lib.rs`): solo `ingreso`/`ajuste`; reenvía `terminal`; rechaza `inicial`/`venta` con `invalid_input` field `tipo`; propaga `negative_stock` / `not_found` del puerto; sin `list`/`update`/`delete` — `cargo test -p catalog catalog_register` sale 0 (capability `catalog-stock`)

## 4. Tests HTTP (red)

- [x] 4.1 Agregar `vortex-core/crates/api/tests/stock_movements.rs` con fake de `StockGateway` y tests happy `post_stock_inbound_20_on_10_returns_201_with_terminal` y `post_stock_adjustment_to_zero_returns_201` — `cargo test -p api --test stock_movements post_stock_inbound_20 post_stock_adjustment_to_zero` no compila o falla (HU-02-07 Ingreso / ajuste a cero; `assert_conforms` 201)
- [x] 4.2 Agregar tests unhappy `post_stock_adjustment_without_reason_returns_400`, `post_stock_adjustment_minus_8_on_5_returns_400_negative_stock`, `post_stock_unknown_product_returns_404`, `post_stock_invalid_product_id_returns_400`, `post_stock_inbound_negative_quantity_returns_400`, `post_stock_type_initial_returns_400`, `post_stock_type_sale_returns_400`, `post_stock_quantity_as_json_number_returns_400`, `post_stock_terminal_too_long_returns_400`, `post_stock_without_service_returns_503`, `get_stock_movements_is_not_page`, `patch_stock_movement_is_not_registered`, `delete_stock_movement_is_not_registered` y `stock_problem_field_is_reason_not_motivo` — siguen en rojo; JSON con `assert_conforms` (Listo cuando HU-02-07 / 5.3 / inmutabilidad / T-02-26)
- [x] 4.3 Agregar `vortex_serve_post_stock_unknown_product_returns_404` en `vortex-core/bin/vortex/tests/serve.rs` — falla o no compila (escenario «servicio inyectado: 404 no 503»)

## 5. Gateway (green)

- [x] 5.1 `AppState` con `Option<Arc<dyn StockGateway>>` y `with_stock`; remap `cantidad`→`quantity`, `motivo`→`reason`; DTO `StockMovement` / `StockMovementCreate` (enums `inbound`/`adjustment`, decimales string) — los tests 4.x que no dependen de la ruta registrada avanzan (escenarios DTO 5.3)
- [x] 5.2 Handler `POST /api/v1/products/{productId}/stock-movements` (`X-Vectra-Terminal` ≤ 64, ULID inválido → 400, sin servicio → 503, `adjustment` sin `reason` → 400, `negative_stock` → 400, `initial`/`sale` → 400) **sin** `GET`/`PATCH`/`DELETE` — `cargo test -p api --test stock_movements` verde (todos los escenarios HTTP de `vortex-api`)

## 6. Composition root (green)

- [x] 6.1 `serve.rs` inyecta `StockService` con `MovimientoStockRepo` vía `with_stock` — `cargo test -p vortex -- vortex_serve_post_stock_unknown_product_returns_404` pasa

## 7. Cierre (regresión)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 Confirmar non-goals: `rg` sin handler `GET` de historial ni `PATCH`/`DELETE` de movimiento en `crates/api/src`; `git diff docs/openapi.yaml` sin cambios al `GET` ni a `StockMovementPage`
- [x] 7.4 `openspec validate T-02-25` sale 0
