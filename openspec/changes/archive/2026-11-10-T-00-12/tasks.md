# Tasks

## 1. Tests (red) — cuerpo y timeout

- [x] 1.1 Agregar `vortex-core/crates/api/tests/middleware.rs` con `oversized_body_returns_413_payload_too_large`: `POST` con `Content-Length` 262145 a una ruta del router de `build_router` → 413, `Content-Type: application/problem+json`, `code` `payload_too_large`, `title` "El cuerpo es demasiado grande", sin `errors` — `cargo test -p api --test middleware -- oversized_body` no compila o falla (escenario «cuerpo sobre el límite es 413 payload_too_large»)
- [x] 1.2 En el mismo archivo, `body_at_limit_is_not_413`: `POST` con `Content-Length` 262144 a una ruta inexistente → no 413 y el JSON no tiene `code` `payload_too_large` — el test no compila o falla (escenario «cuerpo en el límite no es 413»)
- [x] 1.3 Agregar `slow_handler_returns_504_timeout`: sonda de test `/_probe/slow` (no producción) que no responde en 5 s (reloj Tokio pausado) → 504, `application/problem+json`, `code` `timeout`, `title` "La operación superó el tiempo de espera", sin `errors` — `cargo test -p api --test middleware -- slow_handler` no compila o falla (escenario «handler lento es 504 timeout»)
- [x] 1.4 Agregar `fast_request_is_not_504`: `GET /no-such-route` → 404 / `not_found` y no 504 — el test no compila o falla (escenario «handler rápido no es 504»)

## 2. Tests (red) — cabeceras y span

- [x] 2.1 Agregar `api_path_sends_cache_control_no_store`: `GET /api/v1/no-such-route` → cabecera `Cache-Control: no-store` y 404 `not_found` — `cargo test -p api --test middleware -- api_path_sends` falla (escenario «/api/* envía no-store»)
- [x] 2.2 Agregar `non_api_path_omits_cache_control_no_store`: `GET /no-such-route` → no incluye `Cache-Control: no-store` y sigue 404 `not_found` — el test falla (escenario «path fuera de /api no lleva no-store»)
- [x] 2.3 Agregar `json_response_is_gzip_compressed`: sonda `/_probe/large` (JSON ≥ 1 KiB, no producción) con `Accept-Encoding: gzip` → `Content-Encoding: gzip` y Content-Type JSON — `cargo test -p api --test middleware -- gzip` no compila o falla (escenario «JSON acepta gzip»)
- [x] 2.4 Agregar `json_response_is_brotli_compressed`: la misma sonda con `Accept-Encoding: br` → `Content-Encoding: br` — el test no compila o falla (escenario «JSON acepta brotli»)
- [x] 2.5 Agregar `sse_response_is_not_compressed`: sonda `/_probe/sse` con `Accept-Encoding: gzip` → `Content-Type` `text/event-stream` y sin `Content-Encoding` gzip/br — el test no compila o falla (escenario «SSE no se comprime»)
- [x] 2.6 Agregar `http_request_span_includes_method_route_status_duration`: subscriber de test, `GET /no-such-route` → span con método `GET`, ruta `/no-such-route`, estado 404 y duración — `cargo test -p api -- http_request_span` no compila o falla (escenario «span incluye método, ruta, estado y duración»)

## 3. Tests (red) — allowlist

- [x] 3.1 Fixture temporal de `crates/api/Cargo.toml` con `clap` en `[dependencies]` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `api` (escenario «api declara una dependencia fuera de la allowlist»; **no** usar `tower-http`, pasará a estar permitido)
- [x] 3.2 Fixture temporal de `crates/api/Cargo.toml` con la allowlist de middlewares (`axum`, path `domain`, `serde`, `serde_json`, `thiserror`, `tower-http`, `tracing`; `[dev-dependencies]` `tokio` + `tracing-subscriber`) — hoy el script sale ≠ 0 (escenario «api declara la allowlist» aún en rojo)
- [x] 3.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (api todavía sin `tower-http`/`tracing`; escenario previo a este change)

## 4. Harness (green) — allowlist `api`

- [x] 4.1 En `scripts/check-workspace.sh`: permitir en `api` `tower-http` y `tracing` en `[dependencies]` y `tracing-subscriber` en `[dev-dependencies]`; `clap` sigue prohibido — el fixture 3.2 sale 0 y el 3.1 sigue ≠ 0 nombrando `api` (escenarios «api declara la allowlist», «api declara una dependencia fuera de la allowlist»)
- [x] 4.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy T-00-11 (problem+json) sigue MAY; happy con allowlist de middlewares; unhappy `api con clap` sin cambios — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 4.3 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin las deps nuevas en el `Cargo.toml` real de `api`)

## 5. Implementación (green) — límite, timeout y `ApiError`

- [x] 5.1 Declarar en `vortex-core/crates/api/Cargo.toml` `tower-http` (features `limit`, `timeout`, `compression-gzip`, `compression-br`, `trace`) y `tracing` (`workspace = true`); `tracing-subscriber` en `[dev-dependencies]`; actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «api declara la allowlist»)
- [x] 5.2 Implementar `ApiError::payload_too_large` y `ApiError::timeout` (Gateway-only, `instance`, sin `errors`, títulos de la spec) — un test unitario o los de middleware ven `code` / `title` correctos
- [x] 5.3 Instalar `RequestBodyLimitLayer` 256 KiB, bajar `DefaultBodyLimit` al mismo tope y mapear exceso a `ApiError::payload_too_large` — `cargo test -p api --test middleware -- oversized_body body_at_limit` sale 0 (escenarios 1.1–1.2 / Listo cuando cuerpo)
- [x] 5.4 Instalar `TimeoutLayer` 5 s mapeado a `ApiError::timeout` (no 408); exportar `DEFAULT_BODY_LIMIT` y `DEFAULT_REQUEST_TIMEOUT` — `cargo test -p api --test middleware -- slow_handler fast_request` sale 0 (escenarios 1.3–1.4 / Listo cuando timeout)

## 6. Implementación (green) — cabeceras, compresión y span

- [x] 6.1 Middleware `Cache-Control: no-store` solo si el path es `/api` o `/api/*` — `cargo test -p api --test middleware -- cache_control` sale 0 (escenarios 2.1–2.2 / Listo cuando cabeceras)
- [x] 6.2 `CompressionLayer` gzip/brotli con predicado que excluye `text/event-stream` — `cargo test -p api --test middleware -- gzip brotli sse_response` sale 0 (escenarios 2.3–2.5)
- [x] 6.3 `TraceLayer` con método, ruta, estado y duración — `cargo test -p api -- http_request_span` sale 0 (escenario 2.6)
- [x] 6.4 Confirmar que no hay rutas `/health`, `/metrics` ni `/api/v1` de negocio, que no hay límites por ruta distintos de 256 KiB / 5 s, y que `docs/openapi.yaml` no cambió — `rg` / `git diff` (non-goals T-00-13, T-00-14, T-02-30, 5.10 por endpoint)

## 7. Cierre (regresión y § 5)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde» / tabla de tests de `vortex-api`)
- [x] 7.2 `./scripts/check-workspace.sh`, `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`api` depende de `domain` y `tower-http`, no al revés)
- [x] 7.3 `openspec validate T-00-12` (o `t-00-12` si el CLI exige minúsculas) sale 0
