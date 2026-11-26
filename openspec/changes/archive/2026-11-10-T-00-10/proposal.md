# Proposal

## Ticket padre

- **Ticket:** [T-00-10](../../../docs/7-tickets.md) — Servidor Axum mínimo con apagado ordenado
- **Change:** `T-00-10`
- **HU:** [HU-00-06](../../../docs/6-historias.md#hu-00-06--servicio-del-sistema-y-arranque-ordenado)
- **Depende de:** T-00-07 (cerrado en `develop`)
- **Hace:** `api::build_router(state)`, `AppState` con servicios inyectados como `Arc<dyn …>`, `vortex serve` escuchando en HTTP, apagado ordenado con `SIGTERM` y `Ctrl+C`
- **No hace:** TLS (T-00-16) ni rutas de negocio
- **Listo cuando:** `vortex serve` arranca, responde 404 en una ruta inexistente y termina limpio con `SIGTERM`

## Why

`bin/vortex` ya expone `serve` como stub (`no implementado`, exit 1) y `crates/api` es un `lib.rs` vacío. HU-00-06 y el API Gateway ([3.4.1](../../../docs/3-componentes.md#341-api-gateway)) necesitan un proceso HTTP real en `localhost` (MVP del curso; HTTPS es v2). T-00-11, T-00-13 y T-00-19 se enganchan a este router: hay que materializar Axum, `AppState` y el apagado ordenado ahora, sin rutas de negocio ni TLS.

## What Changes

- `api::build_router(AppState) -> Router` y `AppState` clonable sobre `Arc` (patrón de inyección `Arc<dyn Trait>`; sin puertos de dominio todavía)
- `vortex serve` deja de ser stub: carga `Config` (`--config` gana sobre `VORTEX_CONFIG`), escucha HTTP en `server.bind`, responde 404 en rutas inexistentes
- Apagado ordenado: `SIGTERM` y `Ctrl+C` cierran el listener y el proceso termina con código 0
- Allowlist: `crates/api` MAY declarar `axum`; `bin/vortex` MAY declarar path `api`, `tokio` y `axum` (pins ya en el workspace). `check-workspace.sh` y fixtures se actualizan
- **BREAKING** (CLI): `vortex serve` ya no imprime `no implementado`; exige configuración válida y no arranca si falta el path, el archivo o el bind está ocupado
- TDD: happy = router 404 + `vortex serve` 404 + SIGTERM/SIGINT exit 0; unhappy = sin config / archivo inexistente / TOML inválido / bind ocupado

## Non-goals

Este change **no** cubre:

- TLS, `axum-server`, rustls, `vectra.local` — **T-00-16** / T-00-15 (v2) — **No hace** del ticket
- Rutas de negocio bajo `/api/v1` — tickets de catálogo / búsqueda (T-02-06+, T-03-13, …)
- `ApiError` / `problem+json` / 404 de dominio — **T-00-11**
- Middlewares (`tower-http`: body limit, timeout, compresión, `Cache-Control`) — **T-00-12**
- `GET /health` y registro de salud — **T-00-13**
- `GET /metrics` — **T-00-14**
- Estáticos de la SPA y `/media` — **T-00-19**
- Arranque ordenado (logs → Storage → migraciones → salud → API) e `init_logging` en `main` — **T-00-20**
- Supervisor, sidecars, `launchd` — T-00-26, T-00-27, T-00-30 (varios v2)
- Cambios a `docs/openapi.yaml` (el 404 de ruta desconocida es comportamiento del Gateway, no un path nuevo del contrato)

## Capabilities

### New Capabilities

- `vortex-api`: esqueleto HTTP del API Gateway — `build_router`, `AppState`, `vortex serve` en HTTP, 404 de ruta inexistente, apagado ordenado con `SIGTERM` y `Ctrl+C`

### Modified Capabilities

- `cargo-workspace`: `api` MAY declarar `axum` (`workspace = true`) y `tokio` en `[dev-dependencies]`; `bin/vortex` MAY declarar path `api`, `tokio` y `axum` además de la allowlist actual. El resto de libs distintas de `domain`, `storage` y `api` siguen sin deps. `cargo test --workspace` MUST ejecutar también los tests del router y de `vortex serve`
- `vortex-cli`: `serve` deja de ser stub; el resto de subcomandos reconocidos siguen con `no implementado` / exit 1. `--help` no cambia
- `vortex-config`: `vortex serve` **sí** carga `Config` (resolución `--config` / `VORTEX_CONFIG`). Los demás stubs siguen sin abrir el archivo. El escenario que usaba `serve` como stub pasa a otro subcomando (p. ej. `reindex`)

## Impact

- **Repo:** `vortex-core/crates/api/` (`Cargo.toml`, `src/lib.rs`, tests del router), `vortex-core/bin/vortex/` (`Cargo.toml`, `main.rs`, módulo `serve`, tests de CLI/`serve`), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** ninguna ruta nueva; 404 de Axum por defecto (sin `problem+json`)
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `scripts/check-workspace.sh`; `openspec validate T-00-10`
- **Ticket:** cierra T-00-10 (Listo cuando: arranca, 404, SIGTERM limpio)
- **BREAKING** (CLI y harness interno): `vortex serve` exige TOML válido; `check-workspace.sh` deja de fallar si `api` declara `axum` o si `vortex` declara `api`/`tokio`/`axum`; sigue fallando si `api` declara crates fuera de su allowlist o si `catalog` (u otro lib) declara deps
