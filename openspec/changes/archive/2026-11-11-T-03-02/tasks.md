# Tasks

## 1. Tests (red) — OutboxEvent happy

- [x] 1.1 Agregar tests `outbox_event_valid_producto_upsert` y `outbox_event_motivo_none` en `vortex-core/crates/domain/` (escenarios «Construcción válida» y «motivo ausente») — `cargo test -p domain outbox_event_valid` falla o no compila

## 2. Tests (red) — OutboxEvent unhappy

- [x] 2.1 Agregar tests `outbox_event_rejects_table_prefix` y `outbox_event_rejects_unknown_entidad_or_operacion` (escenarios «Prefijo de tabla» y «entidad u operación fuera de lista») — `cargo test -p domain outbox_event_rejects` falla o no compila

## 3. Implementación (green) — OutboxEvent

- [x] 3.1 Exponer `OutboxEvent` / `OutboxEntity` / `OutboxOp` en `domain` (`try_new`, ULID sin prefijo, enums 4.8.1) y exportarlos en `lib.rs` — `cargo test -p domain outbox_event` en verde

## 4. Tests (red) — Tx happy

- [x] 4.1 Agregar `vortex-core/crates/storage/tests/tx.rs` con `tx_commit_writes_data_and_outbox` y `tx_commit_writes_multiple_outbox_events` (tabla auxiliar + `OutboxEvent`; escenarios «Commit escribe dato y evento» y «Varios eventos») — `cargo test -p storage --test tx tx_commit_writes` falla o no compila
- [x] 4.2 Test `tx_commit_notifies_waiter` (escenario «Aviso solo tras commit») — `cargo test -p storage --test tx tx_commit_notifies_waiter` falla o no compila

## 5. Tests (red) — Tx unhappy

- [x] 5.1 Tests `tx_throw_rolls_back_data_and_outbox` y `tx_failed_does_not_notify` (**Listo cuando**; escenarios «THROW no deja dato ni evento» y «Fallo no avisa») — `cargo test -p storage --test tx tx_throw_rolls_back` y `tx_failed_does_not_notify` fallan o no compilan
- [x] 5.2 Tests `tx_throw_stock_negativo_is_negative_stock`, `tx_unique_sku_is_duplicate_sku` y `tx_unclassified_error_is_unavailable` (escenarios de mapeo UNIQUE/THROW/`Unavailable`) — `cargo test -p storage --test tx tx_throw_stock_negativo` (y los otros dos) fallan o no compilan

## 6. Implementación (green) — allowlist, Tx, mapper, notifier

- [x] 6.1 Ampliar `storage_deps_allowed` en `scripts/check-workspace.sh` (y fixture happy de `scripts/fixtures/workspace/` si aplica) para `domain` path + `tokio` feature `sync` en `[dependencies]` — `./scripts/check-workspace.sh` sale 0 con el `Cargo.toml` real; `axum` en `storage` sigue fallando
- [x] 6.2 Declarar `domain` y `tokio` (`sync`) en `vortex-core/crates/storage/Cargo.toml`; implementar `Tx` (BEGIN/COMMIT + `CREATE outbox` por evento), mapper UNIQUE/THROW → `DomainError` y `OutboxNotifier` post-commit — `cargo test -p storage --test tx` en verde

## 7. Cierre

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `openspec validate T-03-02` sin errores
