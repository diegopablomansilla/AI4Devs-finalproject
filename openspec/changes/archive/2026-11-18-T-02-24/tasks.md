# Tasks

## 1. Tests storage (red)

- [x] 1.1 Agregar tests happy `stock_ledger_append_ingreso_guarda_terminal_caja_1`, `stock_ledger_append_ajuste_a_cero_con_motivo`, `stock_ledger_append_does_not_increment_version`, `stock_ledger_concurrent_ingresos_both_apply` y `stock_ledger_list_by_producto_newest_first` en `vortex-core/crates/storage/tests/stock_ledger.rs` — `cargo test -p storage --test stock_ledger stock_ledger_append_ingreso_guarda_terminal stock_ledger_append_ajuste_a_cero stock_ledger_append_does_not_increment_version stock_ledger_concurrent_ingresos_both_apply stock_ledger_list_by_producto_newest_first` no compila o falla (escenarios Ingreso, ajuste a cero, version intacta, Listo cuando concurrente, list newest-first)
- [x] 1.2 Agregar tests unhappy `stock_ledger_append_ajuste_minus_8_on_stock_5_is_negative_stock`, `stock_ledger_append_ajuste_sin_motivo_is_invalid_input`, `stock_ledger_append_unknown_product_is_not_found`, `stock_ledger_append_ingreso_cantidad_negativa_is_invalid_input` y `stock_ledger_adapter_has_no_update_or_delete` — `cargo test -p storage --test stock_ledger stock_ledger_append_ajuste_minus_8 stock_ledger_append_ajuste_sin_motivo stock_ledger_append_unknown_product stock_ledger_append_ingreso_cantidad_negativa stock_ledger_adapter_has_no_update_or_delete` no compila o falla (Listo cuando HU-02-07, `invalid_input`, `not_found`, inmutabilidad)

## 2. Storage StockLedger (green)

- [x] 2.1 Implementar `MovimientoStockRepo` (`append`: validación de dominio, `THROW "stock_negativo"`, movimiento + `producto.stock` + outbox `motivo: "stock"` en una `Tx` **sin** `version += 1`; `list_by_producto` por `creado_en` DESC) y reexportar — `cargo test -p storage --test stock_ledger` sale 0
- [x] 2.2 Confirmar que `ProductRepo::create` sigue escribiendo el movimiento `inicial` inline (no llama a `StockLedger`) — `cargo test -p storage --test producto product_repo_creates_perno_with_stock_50` sale 0 (escenario «alta inicial intacta» / No hace: extraer `inicial`)

## 3. Cierre (regresión)

- [x] 3.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 3.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 3.3 Confirmar non-goals: `git diff --name-only` sin `vortex-core/crates/catalog`, `vortex-core/crates/api`, `vortex-core/bin/vortex` ni `docs/openapi.yaml`; `rg` sin `CREATE venta` ni `contador:venta` en el adaptador
- [x] 3.4 `openspec validate T-02-24` sale 0