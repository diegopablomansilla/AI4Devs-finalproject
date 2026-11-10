# Proposal

## Ticket padre

- **Ticket:** [T-00-13](../../../docs/7-tickets.md) — Registro de salud y `GET /health`
- **Change:** `T-00-13`
- **HU:** [HU-00-05](../../../docs/6-historias.md#hu-00-05--salud-y-métricas)
- **Depende de:** T-00-10 (cerrado en `develop`; T-00-11 y T-00-12 ya están en el Gateway)
- **Hace:** puerto `HealthReport` en `domain`; `HealthRegistry` en `supervisor` con estado por componente (`storage`, `meilisearch`, `llm`, `speech`, `textEmbeddings`, `sync`); agregación: `storage` caído → `down`; algún `starting` → `starting` (503); algún `down` o `degraded` → `degraded`; si no, `ok`. Incluye `version` y `uptimeSeconds`
- **No hace:** chequeos reales de sidecars (cada ticket de adaptador reporta su estado)
- **Listo cuando:** tests del agregador cubren los cuatro estados y el 503 en `starting`

## Why

El Gateway ya escucha HTTP (T-00-10) y aplica middlewares (T-00-12), pero no hay señal de salud: `build_router` sigue prohibiendo `/health` y no existe un registro de componentes. HU-00-05 y el contrato (`GET /health`, esquema `Health` en `docs/openapi.yaml`) necesitan un agregador inyectable ahora, antes del arranque ordenado (T-00-20) y de que cada adaptador publique su estado. Sin este puerto, T-09-04 no tiene un `GET /health` contra el que validar OpenAPI.

## What Changes

- Puerto `HealthReport` y tipos de snapshot en `domain` (sin HTTP): componentes fijos, estados de componente (`up` / `starting` / `degraded` / `down` / `disabled`) y estado agregado (`ok` / `degraded` / `starting` / `down`)
- Función pura de agregación (prioridad: `storage` `down` → `down`; si no, algún `starting` → `starting`; si no, algún `down` o `degraded` → `degraded`; si no, `ok`). `disabled` no degrada el agregado
- `HealthRegistry` en `supervisor`: estado por componente, `version`, `uptimeSeconds`; implementa `HealthReport`; los callers publican con `set` (sin sondear sidecars)
- `GET /health` en `api::build_router`: cuerpo JSON del esquema `Health`; HTTP **200** si el agregado es `ok` o `degraded`; HTTP **503** si es `starting` o `down` (no `problem+json`)
- `AppState` inyecta `Arc<dyn HealthReport>`; `vortex serve` cablea un `HealthRegistry` (composition root). Default de tests: snapshot con los seis componentes en `disabled` → agregado `ok`
- Allowlist: `supervisor` MAY depender de `domain`; `vortex` MAY depender de `supervisor`
- TDD: happy = los cuatro estados del agregador, `GET /health` 200 `ok`/`degraded` con `version` y `uptimeSeconds`; unhappy = 503 en `starting` y `down`, `storage` `down` gana sobre `starting`, `disabled` no dispara `degraded`, método distinto de GET no es 200 `Health`

## Non-goals

Este change **no** cubre:

- Chequeos reales de sidecars, procesos hijos ni HTTP a Meilisearch/Ollama — **No hace** del ticket; cada adaptador reporta en su ticket (T-00-25, T-03-04, T-05-10, T-05-13, T-06-02, …)
- `GET /metrics` ni Prometheus — **T-00-14**
- Arranque ordenado ni `/health` en `starting` durante el boot — **T-00-20**
- Campos `sync.outboxLag` / `failedEvents` — **T-03-09**
- Capacidad `vision` en `/health` — **T-07-03** (v2)
- UI de estado del sistema — **T-01-12**
- Validador de respuestas contra OpenAPI — **T-09-04** (depende de este change)
- HTTPS, `vectra.local`, estáticos, sidecars, Storage al arrancar — T-00-16, T-00-17, T-00-19, T-00-08 ya existe pero no se cablea aquí
- Cambiar `docs/openapi.yaml` (el path `/health` y el esquema `Health` ya están)

## Capabilities

### New Capabilities

- `health-report`: puerto `HealthReport`, tipos de componente/agregado, agregación de los cuatro estados y `HealthRegistry` en `supervisor` (publicación in-memory, sin probes)

### Modified Capabilities

- `vortex-api`: `build_router` registra `GET /health`; `AppState` lleva `Arc<dyn HealthReport>`; 200/503 según el agregado; el cuerpo es el esquema `Health`, no `problem+json`
- `cargo-workspace`: `supervisor` MAY declarar path `domain`; `vortex` MAY declarar path `supervisor`; la suite MUST incluir tests del agregador y de `GET /health`

## Impact

- **Repo:** `vortex-core/crates/domain/` (puerto y agregación), `vortex-core/crates/supervisor/` (`HealthRegistry`, `Cargo.toml`), `vortex-core/crates/api/` (`AppState`, ruta `/health`, DTO JSON), `vortex-core/bin/vortex/` (`serve` + `Cargo.toml`), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** se implementa `GET /health` ya definido; sin cambios de schema
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `scripts/check-workspace.sh`; `scripts/check-domain-deps.sh`; `openspec validate T-00-13` (o `t-00-13` si el CLI exige minúsculas)
- **Ticket:** cierra T-00-13 (Listo cuando: tests del agregador cubren los cuatro estados y el 503 en `starting`)
- **BREAKING** (router): `GET /health` deja de ser 404 `not_found` y pasa a JSON `Health`
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `supervisor` declara `domain` o si `vortex` declara `supervisor`; sigue fallando si esas crates salen de la nueva allowlist
