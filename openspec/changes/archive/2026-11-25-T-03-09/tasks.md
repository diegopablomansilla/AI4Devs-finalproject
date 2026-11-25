# Tasks

## 1. Tests (red) — OutboxRepo fallos y lag

- [x] 1.1 Agregar `outbox_repo_dummy_records_fallido`, `outbox_repo_dummy_lag_counts_pending` y `outbox_repo_dummy_record_unknown_event_is_not_found` en `vortex-core/crates/domain/` — `cargo test -p domain outbox_repo_dummy` no compila o falla (escenarios registro de fallido, lag meilisearch, evento inexistente)
- [x] 1.2 Agregar `storage_outbox_repo_records_fallido`, `storage_outbox_repo_lag_counts_pending`, `storage_outbox_repo_failed_count` y `storage_outbox_repo_record_unknown_event_is_not_found` en `vortex-core/crates/storage/tests/` — `cargo test -p storage storage_outbox_repo` no compila o falla (escenarios persistir fallido, lag, conteo, not_found)

## 2. Implementación (green) — puerto y adaptador

- [x] 2.1 Extender `OutboxRepo` / `OutboxRow` (`creado_en`, record fallido, lag, failed_count) y el dummy — `cargo test -p domain outbox_repo_dummy` sale 0
- [x] 2.2 Implementar las operaciones en `OutboxStore` (SELECT `creado_en`, upsert `outbox_fallo`, count lag/fallidos; sin migración nueva) — `cargo test -p storage storage_outbox_repo` sale 0

## 3. Tests (red) — worker transitorio, permanente y telemetría

- [x] 3.1 Agregar `worker_retries_after_unavailable_and_does_not_lose_events`, `worker_timeout_does_not_advance_cursor`, `worker_does_not_write_fallo_on_unavailable` y `worker_backoff_caps_at_thirty_seconds` en `vortex-core/crates/sync/` — `cargo test -p sync worker_retries_after_unavailable worker_timeout_does_not_advance worker_does_not_write_fallo worker_backoff_caps` no compila o falla (Listo cuando HU-03-01 Meilisearch caído + timeout + tope 30 s)
- [x] 3.2 Reemplazar `worker_does_not_advance_cursor_on_invalid_input` y `worker_fanout_invalid_registro_does_not_advance` por `worker_records_fallido_and_advances_on_invalid_input`, `worker_fanout_invalid_registro_records_fallido_and_advances` y `worker_invalid_input_continues_remaining_batch` — `cargo test -p sync worker_records_fallido worker_fanout_invalid_registro_records worker_invalid_input_continues` no compila o falla (Listo cuando HU-03-01 error permanente)
- [x] 3.3 Agregar `worker_empty_batch_does_not_backoff`, `worker_publishes_outbox_lag_after_drain` y `worker_observes_propagation_on_success` — `cargo test -p sync worker_empty_batch_does_not_backoff worker_publishes_outbox_lag worker_observes_propagation` no compila o falla (telemetría + poll 1 s)

## 4. Implementación (green) — clasificación, backoff y telemetría

- [x] 4.1 Clasificar `unavailable`/`timeout` vs `invalid_input` en `drain_once`; inyectar `SyncTelemetry`; `transient_backoff`; `run_while` con tope 30 s y notifier ignorado en backoff — `cargo test -p sync worker_retries_after_unavailable worker_timeout worker_does_not_write_fallo worker_backoff_caps worker_records_fallido worker_fanout_invalid_registro_records worker_invalid_input_continues worker_empty_batch worker_publishes_outbox_lag worker_observes_propagation worker_does_not_advance_cursor worker_fanout_does_not_advance worker_deletes worker_empty_outbox worker_sinonimo` sale 0 (unidad; sin sidecar)

## 5. Tests (red) — `/health` y `/metrics`

- [x] 5.1 Agregar `registry_new_sync_stats_are_zero`, `registry_set_sync_stats_are_reflected` y `aggregate_ignores_outbox_lag` — `cargo test -p supervisor registry_new_sync_stats registry_set_sync_stats` y `cargo test -p domain aggregate_ignores_outbox_lag` no compilan o fallan (extras de sync en snapshot)
- [x] 5.2 Agregar `get_health_sync_extras_default_zero` y `get_health_sync_extras_reflect_snapshot` en `vortex-core/crates/api/tests/` — `cargo test -p api get_health_sync_extras` no compila o falla
- [x] 5.3 Reemplazar `scrape_does_not_emit_domain_series` por `scrape_includes_outbox_lag_after_telemetry`, `scrape_includes_outbox_failed_total_after_increment` y `scrape_does_not_emit_vectores_lag_or_backup_failures` (y `scrape_does_not_emit_backup_failures_total` si hace falta) — `cargo test -p api scrape_includes_outbox scrape_does_not_emit` no compila o falla

## 6. Implementación (green) — snapshot, JSON y Prometheus

- [x] 6.1 Extender `ComponentHealth`/`HealthSnapshot`/`HealthRegistry` con `outboxLag`/`failedEvents` (default 0; lag no degrada) — `cargo test -p domain aggregate_ignores_outbox_lag` y `cargo test -p supervisor registry_new_sync_stats registry_set_sync_stats` salen 0
- [x] 6.2 Serializar extras en `GET /health`; impl Prometheus de `SyncTelemetry` en `api` — `cargo test -p api get_health_sync_extras scrape_includes_outbox scrape_does_not_emit reserved_metric_names` sale 0

## 7. Cierre (regresión y non-goals)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 7.3 Confirmar non-goals: sin reproceso manual, sin purga, sin wiring en `bin/vortex`, sin `metrics` en `sync`, sin cambios en `.github/workflows/ci.yml` ni `docs/openapi.yaml` — `git diff --name-only` sin esos paths de código
- [x] 7.4 `openspec validate T-03-09` sin errores