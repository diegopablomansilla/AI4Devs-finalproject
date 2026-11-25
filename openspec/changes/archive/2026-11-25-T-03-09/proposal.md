# Proposal

## Ticket padre

- **Ticket:** [T-03-09](../../../docs/7-tickets.md) — Worker: reintentos, fallos, métricas y salud
- **Change / rama:** `T-03-09` (CLI OpenSpec: `t-03-09` → carpeta `T-03-09`)
- **HU:** [HU-03-01](../../../docs/6-historias.md#hu-03-01--sincronización-del-índice-de-búsqueda) (escenarios «Meilisearch caído» y «Error permanente»)
- **Depende de:** T-03-07 (worker + cursor), T-00-14 (nombres Prometheus reservados)
- **Hace:** `Unavailable`/`Timeout` → reintento con *backoff* exponencial (tope 30 s) sin avanzar el cursor; `InvalidInput` → fila `fallido` en `outbox_fallo` y el cursor avanza; métricas `outbox_lag{consumidor}`, `outbox_failed_total` y `sync_propagation_seconds`; `sync` en `/health` con `outboxLag` y `failedEvents`
- **No hace:** reproceso manual de fallidos
- **Listo cuando:** tests de "Meilisearch caído" (no se pierden eventos) y "error permanente" de HU-03-01

## Why

T-03-07/T-03-08 ya drenan el outbox y hacen *fan-out*, pero **cualquier** `DomainError` (incluido `invalid_input`) deja el cursor quieto y `run_while` ignora el error y vuelve a pollar a 1 s. Un documento rechazado por Meilisearch atasca el consumidor para siempre; Meilisearch caído no tiene *backoff* acotado ni señal en `/health` ni `/metrics`. HU-03-01 y [4.8.2](../../../docs/4-modelo.md#482-sync_cursor-y-outbox_fallo) exigen reintentos transitorios sin pérdida, fallos permanentes visibles, y lag/fallidos en salud y Prometheus.

## What Changes

- Clasificar errores del worker: `unavailable`/`timeout` = transitorio (no avanza cursor; *backoff* exponencial tope 30 s); `invalid_input` = permanente (`outbox_fallo` `fallido`, cursor avanza, el lote sigue)
- Extender `OutboxRepo` para registrar fallos, contar lag del consumidor `meilisearch` y contar filas `fallido` (tabla ya migrada en T-03-01)
- Emitir `outbox_lag{consumidor}`, `outbox_failed_total` y `sync_propagation_seconds` (nombres de T-00-14)
- `GET /health`: `components.sync` incluye `outboxLag` y `failedEvents` (contrato OpenAPI ya existente)
- Tests TDD happy + unhappy; Listo cuando = Meilisearch caído no pierde eventos (y se aplican al recuperar) + error permanente registra fallido, avanza cursor y aparece en métricas

## Non-goals

- Reproceso manual de `outbox_fallo` → **No hace** del ticket (no hay ticket de reproceso en el MVP)
- Purga del outbox / de `outbox_fallo` → **T-03-10** (v2)
- Wiring del worker en `bin/vortex` / arranque ordenado → **T-00-27**
- `vortex reindex` → **T-03-11**
- Consumidor `vectores` (cursor, fallos, lag) → **T-05-08** (v2)
- Worker de vectores, sinónimos, facetas, `GET /search` → **T-05-08**, **T-03-27**, **T-03-18**, **T-03-12+**
- Escribir filas `reintentando` en `outbox_fallo` (el cursor quieto basta para transitorios)
- Cambiar `docs/openapi.yaml` (los campos `outboxLag` / `failedEvents` ya están); CI YAML

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `vortex-sync`: *backoff* exponencial (tope 30 s) en transitorios; `invalid_input` → `outbox_fallo` + avance; telemetría de lag/fallidos/propagación; deja de atascar el cursor en error permanente
- `outbox-event`: `OutboxRepo` escribe `outbox_fallo`, expone lag/conteo de fallidos y `creado_en` para el histograma de propagación
- `vortex-storage`: adaptador Surreal de fallos, lag y conteo (deja de ser “MUST NOT escribir `outbox_fallo`”)
- `health-report`: `components.sync` lleva `outboxLag` y `failedEvents` (enteros ≥ 0)
- `vortex-api`: `GET /health` serializa esos campos; `GET /metrics` puede exponer las series de sync cuando hay telemetría
- `prometheus-metrics`: se emiten las series reservadas `outbox_lag`, `outbox_failed_total` y `sync_propagation_seconds`

## Impact

- **Repo:** `vortex-core/crates/sync/` (worker); `vortex-core/crates/domain/` (`OutboxRepo`, snapshot de salud, puerto de telemetría); `vortex-core/crates/storage/` (adaptador); `vortex-core/crates/api/` (`/health`, implementación Prometheus); `vortex-core/crates/supervisor/` (snapshot con extras)
- **APIs / OpenAPI:** sin paths nuevos; `GET /health` pasa a **emitir** `components.sync.outboxLag` y `failedEvents` ya definidos en `docs/openapi.yaml`
- **CI / verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo fmt --all -- --check`; `cargo clippy --workspace -- -D warnings`; `cargo test --workspace`; `openspec validate T-03-09`
- **Ticket:** cierra T-03-09 (Listo cuando: HU-03-01 Meilisearch caído + error permanente); desbloquea T-03-10 (v2) y T-00-27 (wiring)
- **Ámbito:** cabe en ~2 h si el “Meilisearch caído” es unidad con `TextSearch` falso (falla → recupera) y el sidecar ignore no se vuelve a exigir. No se divide el ticket
