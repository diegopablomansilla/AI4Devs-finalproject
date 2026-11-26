# Tasks

## 1. Tests (red) — convención de nombres e histograma

- [x] 1.1 Agregar en `vortex-core/crates/api` el test `reserved_metric_names_match_convention`: las constantes incluyen `outbox_lag`, `outbox_failed_total`, `backup_failures_total` y `operation_duration_seconds("search")` es `search_duration_seconds` — `cargo test -p api -- reserved_metric_names` no compila o falla (escenario «constantes coinciden con la convención»)
- [x] 1.2 Agregar `vortex-core/crates/api/tests/metrics.rs` con `http_histogram_records_health_route`: `GET /health` luego scrape → `http_request_duration_seconds` con `route="/health"` y `method="GET"` y series `_bucket`/`_sum`/`_count` — `cargo test -p api --test metrics -- http_histogram_records_health` no compila o falla (escenario «petición a /health queda en el histograma» / Listo cuando)
- [x] 1.3 Agregar `unmatched_route_uses_unmatched_label`: `GET /no-such-route-xyz` luego scrape → `route="unmatched"` y `status="404"`; el cuerpo no incluye `route="/no-such-route-xyz"` — `cargo test -p api --test metrics -- unmatched_route` no compila o falla (escenario «404 no usa el URI crudo»)
- [x] 1.4 Agregar `scrape_does_not_emit_domain_series`: `GET /health` + `GET /metrics` → el cuerpo no incluye series `outbox_lag`, `outbox_failed_total` ni `backup_failures_total` — `cargo test -p api --test metrics -- scrape_does_not_emit` no compila o falla (escenario «el scrape no emite series de dominio»)

## 2. Tests (red) — GET /metrics

- [x] 2.1 Agregar `get_metrics_returns_prometheus_text`: `GET /metrics` → HTTP 200, `Content-Type` empieza por `text/plain`, cuerpo no vacío y sin `code` de problema — `cargo test -p api --test metrics -- get_metrics_returns` no compila o falla (escenario «GET /metrics es 200 text/plain»)
- [x] 2.2 Agregar `get_metrics_includes_prior_request_histogram`: `GET /health` luego `GET /metrics` → 200, cuerpo con `http_request_duration_seconds`, `route="/health"` y `method="GET"` — `cargo test -p api --test metrics -- get_metrics_includes_prior` no compila o falla (escenario «GET /metrics incluye el histograma de una petición previa» / Listo cuando)
- [x] 2.3 Agregar `get_metrics_is_not_problem_json` y `get_metrics_omits_cache_control_no_store`: GET no es `application/problem+json`; no lleva `Cache-Control: no-store` — `cargo test -p api --test metrics -- not_problem omits_cache` no compila o falla (escenarios «GET /metrics no es problem+json», «GET /metrics no lleva no-store»)
- [x] 2.4 Agregar `post_metrics_is_not_prometheus_200`: `POST /metrics` no es 200 con cuerpo de exposición Prometheus — `cargo test -p api --test metrics -- post_metrics` no compila o falla (escenario «POST /metrics no es scrape 200»)

## 3. Tests (red) — vortex serve

- [x] 3.1 Agregar `vortex_serve_get_metrics_returns_prometheus` en `bin/vortex/tests/serve.rs`: `vortex serve` + `GET /health` + `GET /metrics` → 200, `text/plain`, cuerpo con `http_request_duration_seconds` — `cargo test -p vortex -- vortex_serve_get_metrics` no compila o falla (escenario «vortex serve GET /metrics responde Prometheus»)

## 4. Tests (red) — allowlist

- [x] 4.1 Fixture temporal de `crates/api/Cargo.toml` con `clap` en `[dependencies]` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `api` (escenario «api declara una dependencia fuera de la allowlist»; **no** usar `metrics`, pasará a estar permitido)
- [x] 4.2 Fixture temporal de `crates/api/Cargo.toml` con `metrics` y `metrics-exporter-prometheus` (`workspace = true`) junto al resto de la allowlist vigente — hoy el script sale ≠ 0 (escenario «api declara la allowlist prometheus» aún en rojo)
- [x] 4.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (api todavía sin `metrics`; escenario previo a este change)

## 5. Harness (green) — allowlist api

- [x] 5.1 En `scripts/check-workspace.sh`: permitir en `api` `metrics` y `metrics-exporter-prometheus` en `[dependencies]` (`workspace = true`); `clap` en api sigue prohibido — el fixture 4.2 sale 0 y el 4.1 sigue ≠ 0 nombrando `api` (escenarios «api declara la allowlist prometheus», «api declara una dependencia fuera de la allowlist»)
- [x] 5.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy `api con allowlist prometheus`; unhappy `api con clap` y `vortex` con `metrics` siguen — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 5.3 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin las deps nuevas en el `Cargo.toml` real de `api`)

## 6. Implementación (green) — recorder, histograma y nombres

- [x] 6.1 Declarar en `vortex-core/crates/api/Cargo.toml` `metrics` y `metrics-exporter-prometheus` (`workspace = true`); instalar el recorder una vez (`OnceLock` + API 0.18) — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «api declara la allowlist prometheus»)
- [x] 6.2 Publicar constantes `OUTBOX_LAG`, `OUTBOX_FAILED_TOTAL`, `BACKUP_FAILURES_TOTAL` y `operation_duration_seconds` — `cargo test -p api -- reserved_metric_names` sale 0 (escenario 1.1)
- [x] 6.3 Layer HTTP: observa `http_request_duration_seconds{route,method,status}` con `MatchedPath`; `unmatched` si falta; no observa `GET /metrics` — `cargo test -p api --test metrics -- http_histogram unmatched scrape_does_not_emit` sale 0 (escenarios 1.2–1.4)
- [x] 6.4 Confirmar que `domain` no declara `metrics` ni HTTP — `./scripts/check-domain-deps.sh` sale 0

## 7. Implementación (green) — GET /metrics y vortex serve

- [x] 7.1 `router_shell` / `build_router` registra `GET /metrics` que renderiza el handle (`200`, `text/plain`); `POST /metrics` no es scrape 200 — `cargo test -p api --test metrics` sale 0 (escenarios 2.1–2.4 / Listo cuando)
- [x] 7.2 `vortex serve` reutiliza `build_router` (sin declarar `metrics` en el binario) — `cargo test -p vortex -- vortex_serve_get_metrics` sale 0 y los tests previos de `serve` siguen en verde (escenario 3.1)
- [x] 7.3 Confirmar que no hay series de dominio emitidas, ni rutas de negocio, ni estáticos, ni cambios en `docs/openapi.yaml` — `rg` / `git diff` (non-goals T-03-09, T-00-19, T-00-29)

## 8. Cierre (regresión y § 5)

- [x] 8.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde» / tablas de tests de `prometheus-metrics` y `vortex-api`)
- [x] 8.2 `./scripts/check-workspace.sh`, `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`api` declara `metrics` / `metrics-exporter-prometheus`; `vortex` y `domain` no declaran `metrics`)
- [x] 8.3 `openspec validate T-00-14` (o `t-00-14` si el CLI exige minúsculas) sale 0
