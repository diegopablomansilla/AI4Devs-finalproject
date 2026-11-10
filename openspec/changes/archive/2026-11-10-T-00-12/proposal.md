# Proposal

## Ticket padre

- **Ticket:** [T-00-12](../../../docs/7-tickets.md) — *Middlewares* transversales del Gateway
- **Change:** `T-00-12`
- **HU:** [HU-00-09](../../../docs/6-historias.md#hu-00-09--logs-estructurados)
- **Depende de:** T-00-11 (cerrado en `develop`)
- **Hace:** con `tower-http`: límite de cuerpo por defecto de 256 KB (`413 payload_too_large`), *timeout* por defecto de 5 s (`504 timeout`), compresión gzip/brotli excepto `text/event-stream`, `Cache-Control: no-store` en `/api/*` y *span* de `tracing` por petición (método, ruta, estado, duración)
- **No hace:** límites por ruta (cada endpoint los fija según [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway))
- **Listo cuando:** tests cubren cuerpo excedido, *timeout* y cabeceras

## Why

El Gateway ya traduce errores a `problem+json` (T-00-11) pero `build_router` no aplica los límites ni el comportamiento transversal de [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway). Sin tope de cuerpo, timeout, compresión, `no-store` ni span HTTP, las rutas de negocio (y `/metrics` en T-00-14) nacerían desprotegidas y HU-00-09 no vería duración por petición. Hay que instalar los defaults ahora, antes de endpoints.

## What Changes

- Layers de `tower-http` en `api::build_router`: `RequestBodyLimitLayer` 256 KiB → `413` / `payload_too_large`; `TimeoutLayer` 5 s → `504` / `timeout`; `CompressionLayer` gzip/brotli nunca sobre `text/event-stream`; `Cache-Control: no-store` solo en paths `/api` y `/api/*`; `TraceLayer` con método, ruta, estado y duración
- `ApiError` gana el código de Gateway `payload_too_large` (título en español); el `504 timeout` de middleware reutiliza el `code` / `title` ya definidos en T-00-11
- Allowlist de `api`: `[dependencies]` suma `tower-http` y `tracing`; `[dev-dependencies]` suma `tracing-subscriber` (captura del span). El fixture unhappy deja de citar `tower-http` como crate prohibida (sigue `clap`)
- TDD: happy = cuerpo ≤ 256 KiB, petición que termina antes de 5 s, gzip/br en JSON, `no-store` en `/api/*`, span con método/ruta/estado/duración; unhappy = cuerpo excedido, timeout, SSE sin comprimir, path fuera de `/api` sin `no-store`

## Non-goals

Este change **no** cubre:

- Límites o timeouts **por ruta** (search 2 s, imágenes 10 MB, speech 2 MB, etc.) — **No hace** del ticket; cada endpoint los fija según [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway) en su propio ticket
- `GET /metrics` ni histograma Prometheus — **T-00-14** (depende de este change)
- `GET /health`, estáticos `/static` y `/media`, TLS, sidecars — T-00-13, T-00-19, T-00-16
- Cancelación al cerrar el cliente, semáforos de inferencia, CORS, `X-Request-Id`
- `415 unsupported_media_type` (sigue fuera; los rechazos JSON siguen en `400 invalid_input`)
- Instalar el subscriber global de logging en `vortex serve` — T-00-06 ya existe; el arranque ordenado lo cablea en T-00-20
- Cambios de esquema en `docs/openapi.yaml` (`PayloadTooLarge`, `Timeout` y `payload_too_large` ya existen)

## Capabilities

### New Capabilities

- (ninguna — el comportamiento vive en el Gateway HTTP ya especificado)

### Modified Capabilities

- `vortex-api`: `build_router` aplica los middlewares transversales de [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway) (defaults 256 KiB / 5 s, compresión, `no-store` en `/api/*`, span por petición); `413 payload_too_large` y `504 timeout` de middleware salen como `application/problem+json`
- `cargo-workspace`: `api` MAY declarar `tower-http` y `tracing` en `[dependencies]` (`workspace = true`; `tower-http` MAY activar features `limit`, `timeout`, `compression-gzip`, `compression-br`, `trace`) además de la allowlist de T-00-11; `[dev-dependencies]` MAY incluir `tracing-subscriber` además de `tokio`

## Impact

- **Repo:** `vortex-core/crates/api/` (`Cargo.toml`, `src/lib.rs`, `error.rs`, layers, tests de middleware), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** sin paths nuevos; `413` / `504` se alinean a `PayloadTooLarge` y `Timeout` ya definidos en `docs/openapi.yaml`
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `scripts/check-workspace.sh`; `openspec validate T-00-12` (o `t-00-12` si el CLI exige minúsculas)
- **Ticket:** cierra T-00-12 (Listo cuando: tests de cuerpo excedido, timeout y cabeceras)
- **BREAKING** (observabilidad HTTP): peticiones a `/api/*` (incluido 404) pasan a llevar `Cache-Control: no-store`; cuerpos con `Content-Length` > 256 KiB pasan de 404 (ruta inexistente) a `413 payload_too_large`; un handler que tarde más de 5 s pasa a `504 timeout`
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `api` declara `tower-http` / `tracing` / `tracing-subscriber`; sigue fallando si declara crates fuera de la nueva allowlist
