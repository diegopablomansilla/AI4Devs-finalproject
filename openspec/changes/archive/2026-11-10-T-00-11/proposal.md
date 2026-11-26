# Proposal

## Ticket padre

- **Ticket:** [T-00-11](../../../docs/7-tickets.md) — Errores de dominio a `problem+json`
- **Change:** `T-00-11`
- **HU:** [HU-01-05](../../../docs/6-historias.md#hu-01-05--estado-del-sistema-y-errores-visibles), [HU-09-02](../../../docs/6-historias.md#hu-09-02--validación-del-contrato-de-api)
- **Depende de:** T-00-03, T-00-10 (cerrados en `develop`)
- **Hace:** `ApiError` que traduce `DomainError` según la tabla de [5.4](../../../docs/5-contract-api.md#54-errores) (`type`, `title` en español, `status`, `code`, `detail`, `instance`, `errors[]`); rechazos de JSON de Axum → `400 invalid_input` con el campo; 404 y 500 genéricos sin filtrar detalles internos
- **No hace:** límites ni *timeouts* (T-00-12)
- **Listo cuando:** tests verifican estado HTTP, `Content-Type: application/problem+json` y `code` para cada variante

## Why

`crates/api` ya sirve HTTP (`build_router` + `vortex serve`) y `domain` ya expone `DomainError` con `code` estable, pero el Gateway responde 404 de Axum sin RFC 9457. HU-01-05 y HU-09-02 necesitan `application/problem+json` con `code` para que la SPA (T-01-06 / T-01-07) y los tests de contrato (T-09-04) no interpreten textos. Hay que traducir ahora, antes de middlewares (T-00-12) y de rutas de negocio.

## What Changes

- `api::ApiError` que mapea cada variante de `DomainError` a HTTP según [5.4](../../../docs/5-contract-api.md#54-errores): `type` (`https://vectra.local/problems/{kebab-code}`), `title` en español, `status`, `code`, `detail`, `instance`, `errors[]` en `InvalidInput`
- Códigos solo de Gateway en `api` (no en `domain`): `internal` para 500 genérico; `payload_too_large` / `unsupported_media_type` quedan fuera de este change
- Rechazos de JSON de Axum → `400` / `invalid_input` con `errors[{ field, message }]`
- Fallback de ruta inexistente → `404` / `not_found` en `problem+json`; error no previsto → `500` / `internal` sin filtrar detalles internos
- `503` de `Unavailable` incluye cabecera `Retry-After`
- Allowlist: `crates/api` MAY declarar path `domain` y `serde` / `serde_json` / `thiserror` (`workspace = true`) además de `axum`
- TDD: happy = cada `DomainError` y el 404 de ruta inexistente con `Content-Type` + `code`; unhappy = JSON inválido, 500 sin leak, `errors[]` ausente fuera de validación

## Non-goals

Este change **no** cubre:

- Límites de cuerpo, *timeouts*, compresión, `Cache-Control`, *span* de tracing — **T-00-12** — **No hace** del ticket (`413 payload_too_large`, `504` de middleware)
- `415 unsupported_media_type` como código distinto (los rechazos de JSON, incluido content-type ausente, van a `400 invalid_input` en este ticket)
- Mensajes de UI por `code` — **T-01-07**
- Cliente HTTP de la SPA — **T-01-06** (consume este contrato; no se implementa aquí)
- Validador de respuestas contra `openapi.yaml` — **T-09-04**
- `GET /health`, `GET /metrics`, estáticos, TLS, rutas de negocio — T-00-13, T-00-14, T-00-19, T-00-16, T-02-06+
- Cambiar la taxonomía de `DomainError` ni filtrar HTTP al crate `domain` — T-00-03 ya cerrado
- Cambios de esquema en `docs/openapi.yaml` (`Problem` / `ErrorCode` ya existen)

## Capabilities

### New Capabilities

- (ninguna — el mapeo vive en el Gateway HTTP ya especificado)

### Modified Capabilities

- `vortex-api`: el router traduce `DomainError` y rechazos JSON a RFC 9457; el 404 de ruta inexistente deja de ser el cuerpo por defecto de Axum y pasa a `application/problem+json` con `code: not_found`; existe `ApiError` / 500 `internal` sin detalles internos
- `cargo-workspace`: `api` MAY declarar en `[dependencies]` `axum`, path `domain`, `serde`, `serde_json` y `thiserror` (crates.io con `workspace = true`); `[dev-dependencies]` sigue siendo solo `tokio`. `cargo test --workspace` MUST ejecutar también los tests de `problem+json`

## Impact

- **Repo:** `vortex-core/crates/api/` (`Cargo.toml`, `src/lib.rs`, módulo de error, tests del router/`problem+json`), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** sin paths nuevos; las respuestas de error se alinean al esquema `Problem` ya definido en `docs/openapi.yaml` § components
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `scripts/check-workspace.sh`; `openspec validate T-00-11`
- **Ticket:** cierra T-00-11 (Listo cuando: HTTP + `Content-Type: application/problem+json` + `code` por variante)
- **BREAKING** (observabilidad HTTP): `GET` a ruta inexistente pasa de 404 de framework a `application/problem+json` con `code: not_found`. El status 404 no cambia.
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `api` declara `domain` / `serde` / `serde_json` / `thiserror`; sigue fallando si `api` declara crates fuera de esa allowlist
