# Proposal

## Ticket padre

- **Ticket:** [T-00-14](../../../docs/7-tickets.md) — `GET /metrics` en formato Prometheus
- **Change:** `T-00-14`
- **HU:** [HU-00-05](../../../docs/6-historias.md#hu-00-05--salud-y-métricas)
- **Depende de:** T-00-12 (cerrado en `develop`; T-00-13 ya expone `GET /health`)
- **Hace:** `metrics-exporter-prometheus`; histograma `http_request_duration_seconds{route,method,status}` con la ruta de `MatchedPath`; convención de nombres para las métricas de los tickets siguientes (`outbox_lag`, `outbox_failed_total`, `backup_failures_total`, `<operacion>_duration_seconds`)
- **No hace:** métricas de dominio (las agrega cada ticket)
- **Listo cuando:** `/metrics` devuelve texto Prometheus válido con el histograma de una petición previa

## Why

El Gateway ya escucha HTTP, aplica middlewares y sirve `GET /health`, pero `build_router` sigue prohibiendo `/metrics` y no hay recorder Prometheus. HU-00-05 y el contrato (`GET /metrics` en `docs/openapi.yaml`, `text/plain`) necesitan el scrape ahora, antes de que T-03-09, T-04-07 o T-00-29 registren series de dominio. Sin el histograma HTTP y la convención de nombres, cada ticket posterior inventaría labels y nombres incompatibles.

## What Changes

- Recorder Prometheus (`metrics` + `metrics-exporter-prometheus`, pins ya en el workspace) instalado una vez en el crate `api`; `GET /metrics` renderiza el texto de exposición
- Histograma `http_request_duration_seconds` con labels `route` (`MatchedPath`), `method` y `status`; rutas sin match usan el label acotado `unmatched` (no el URI crudo)
- Convención de nombres reservados para tickets siguientes, **sin emitir** esas series aquí: `outbox_lag`, `outbox_failed_total`, `backup_failures_total`, `<operacion>_duration_seconds` (p. ej. `search_duration_seconds`, `recommendations_duration_seconds`)
- Allowlist de `api`: `[dependencies]` suma `metrics` y `metrics-exporter-prometheus` (`workspace = true`). El binario `vortex` sigue **sin** declarar `metrics` (sale por `api`)
- **BREAKING** (router): `GET /metrics` deja de ser 404 `not_found` y pasa a `text/plain` Prometheus
- TDD: happy = scrape válido con el histograma de una petición previa (`GET /health`); unhappy = método distinto de GET, cardinalidad acotada en 404, `/metrics` sin `no-store` ni `problem+json`, fixture `api` con crate fuera de allowlist

## Non-goals

Este change **no** cubre:

- Métricas de dominio (`outbox_lag`, `outbox_failed_total`, `sync_propagation_seconds`, `search_duration_seconds`, `recommendations_duration_seconds`, `speech_duration_seconds`, `backup_failures_total`) — **No hace** del ticket; las agregan T-03-09, T-03-13, T-04-07, T-06-04, T-00-29
- Autenticación o restricción de `/metrics` a localhost — el contrato MVP no tiene auth (`security: []`)
- Grafana, Prometheus server, alertas, dashboards
- Sidecar metrics de Meilisearch/Ollama, métricas de proceso (`process_*`) o de Tokio runtime
- Cambiar `docs/openapi.yaml` (el path `/metrics` ya está; `content: text/plain`)
- Validador OpenAPI de respuestas — **T-09-04**
- UI de estado — **T-01-12**; arranque ordenado — **T-00-20**; HTTPS — **T-00-16**

## Capabilities

### New Capabilities

- `prometheus-metrics`: recorder Prometheus, histograma HTTP `http_request_duration_seconds{route,method,status}` con `MatchedPath`, label acotado para rutas sin match, y convención de nombres reservados para tickets posteriores (sin emitir esas series)

### Modified Capabilities

- `vortex-api`: `build_router` registra `GET /metrics`; 200 `text/plain` con exposición Prometheus; el scrape de una petición previa incluye el histograma HTTP; `POST /metrics` no es un scrape exitoso; `GET /metrics` no lleva `Cache-Control: no-store` ni `problem+json`
- `cargo-workspace`: `api` MAY declarar `metrics` y `metrics-exporter-prometheus` (`workspace = true`); la suite MUST incluir los tests de `/metrics` y del histograma

## Impact

- **Repo:** `vortex-core/crates/api/` (`Cargo.toml`, ruta `/metrics`, layer de histograma, constantes de nombres, tests), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`; `vortex serve` reutiliza `build_router` (test de binario)
- **APIs / OpenAPI:** se implementa `GET /metrics` ya definido; sin cambios de schema
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `scripts/check-workspace.sh`; `openspec validate T-00-14` (o `t-00-14` si el CLI exige minúsculas)
- **Ticket:** cierra T-00-14 (Listo cuando: `/metrics` devuelve texto Prometheus válido con el histograma de una petición previa)
- **BREAKING** (router): `GET /metrics` deja de ser 404 `not_found`
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `api` declara `metrics` / `metrics-exporter-prometheus`; sigue fallando si `api` declara crates fuera de la nueva allowlist o si `vortex` declara `metrics`
