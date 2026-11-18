# Tasks

## 1. Tests (red) — entidad y reglas de signo

- [x] 1.1 Agregar test `movimiento_ingreso_guarda_terminal_caja_1` en `vortex-core/crates/domain` — `cargo test -p domain -- movimiento_ingreso_guarda_terminal_caja_1` no compila o falla (escenario «Construcción válida ingreso con terminal»; HU-02-07)
- [x] 1.2 Agregar tests `movimiento_inicial_cantidad_positiva`, `movimiento_ajuste_negativo_con_motivo` y `movimiento_venta_cantidad_negativa` — `cargo test -p domain -- movimiento_inicial_cantidad_positiva movimiento_ajuste_negativo_con_motivo movimiento_venta_cantidad_negativa` no compila o falla (escenarios happy de cada `tipo`)
- [x] 1.3 Agregar tests unhappy `movimiento_rejects_ingreso_cantidad_negativa`, `movimiento_rejects_inicial_cantidad_negativa` y `movimiento_rejects_venta_cantidad_positiva` — `cargo test -p domain -- movimiento_rejects_ingreso_cantidad_negativa movimiento_rejects_inicial_cantidad_negativa movimiento_rejects_venta_cantidad_positiva` falla (escenarios Listo cuando de signo)
- [x] 1.4 Agregar tests unhappy `movimiento_rejects_cantidad_cero` y `tipo_movimiento_rejects_english_or_unknown` — `cargo test -p domain -- movimiento_rejects_cantidad_cero tipo_movimiento_rejects_english_or_unknown` falla (escenarios «cantidad cero», «TipoMovimiento en inglés del contrato»)

## 2. Tests (red) — motivo, venta y puerto

- [x] 2.1 Agregar test unhappy `movimiento_rejects_ajuste_sin_motivo` — `cargo test -p domain -- movimiento_rejects_ajuste_sin_motivo` falla (escenario «ajuste sin motivo»; **Listo cuando** / HU-02-07)
- [x] 2.2 Agregar test unhappy `movimiento_rejects_stock_resultante_negativo` — `cargo test -p domain -- movimiento_rejects_stock_resultante_negativo` falla (escenario «stock_resultante negativo»; `code` `negative_stock`)
- [x] 2.3 Agregar tests unhappy `movimiento_rejects_venta_sin_referencia` y `movimiento_rejects_ingreso_con_venta` — `cargo test -p domain -- movimiento_rejects_venta_sin_referencia movimiento_rejects_ingreso_con_venta` falla (escenarios de `venta` por tipo)
- [x] 2.4 Agregar tests `stock_ledger_trait_exposes_append_and_list` (dummy que implementa el trait) y `stock_ledger_trait_has_no_update_or_delete` — `cargo test -p domain -- stock_ledger_trait_exposes_append_and_list stock_ledger_trait_has_no_update_or_delete` falla (escenarios «El puerto declara append y list_by_producto», «El puerto no expone update ni delete»)

## 3. Implementación (green) — módulo `movimiento_stock`

- [x] 3.1 Crear `vortex-core/crates/domain/src/movimiento_stock.rs` con `TipoMovimiento` (wire español 4.5.1; `"inbound"` rechazado), marcador `Venta` y reexportar desde `lib.rs` — `cargo test -p domain -- tipo_movimiento_rejects_english_or_unknown` sale 0 (escenario «TipoMovimiento en inglés del contrato»)
- [x] 3.2 Implementar `MovimientoStockDraft` y `MovimientoStock::try_new` (campos 4.5.1, `Id<MovimientoStock>`, reglas de signo, `cantidad ≠ 0`) — `cargo test -p domain -- movimiento_ingreso_guarda_terminal_caja_1 movimiento_inicial_cantidad_positiva movimiento_ajuste_negativo_con_motivo movimiento_venta_cantidad_negativa movimiento_rejects_ingreso_cantidad_negativa movimiento_rejects_inicial_cantidad_negativa movimiento_rejects_venta_cantidad_positiva movimiento_rejects_cantidad_cero` sale 0
- [x] 3.3 Completar `motivo` obligatorio en `ajuste`, `venta` solo en tipo `venta`, y `stock_resultante < 0` → `negative_stock` — `cargo test -p domain -- movimiento_rejects_ajuste_sin_motivo movimiento_rejects_stock_resultante_negativo movimiento_rejects_venta_sin_referencia movimiento_rejects_ingreso_con_venta` sale 0
- [x] 3.4 Declarar `NewMovimientoStock` (sin `stock_resultante`) y el trait `StockLedger` (`append`, `list_by_producto`; `impl Future` sin `async-trait` ni `tokio`) más dummy de test — `cargo test -p domain -- stock_ledger_trait_exposes_append_and_list stock_ledger_trait_has_no_update_or_delete` sale 0
- [x] 3.5 Confirmar que no hay persistencia ni `update`/`delete` en el puerto, y que `domain` no declara crates nuevas — `rg -n "surrealdb|CREATE movimiento_stock|fn update|fn delete" vortex-core/crates/domain/src/movimiento_stock.rs` sin matches de persistencia/mutación; `./scripts/check-workspace.sh` sale 0

## 4. Cierre (regresión y non-goals)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `./scripts/check-domain-deps.sh` sale 0 (escenario «domain no persiste el ledger»: sin `surrealdb` ni `storage` en el árbol)
- [x] 4.3 `git diff` vacío en `vortex-core/crates/storage`, `vortex-core/crates/api`, `vortex-core/crates/catalog`, `docs/openapi.yaml` (non-goals: T-02-24, HTTP)
- [x] 4.4 `openspec validate T-02-17` sale 0