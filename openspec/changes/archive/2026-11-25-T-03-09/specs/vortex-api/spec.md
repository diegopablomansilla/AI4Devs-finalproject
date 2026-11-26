# Spec Delta

Ticket: T-03-09. HU: HU-03-01, HU-00-05.

## ADDED Requirements

### Requirement: GET /health serializa sync.outboxLag y sync.failedEvents

`GET /health` MUST incluir en `components.sync` los enteros `outboxLag` y `failedEvents` (≥ 0), alineados al esquema `Health` de `docs/openapi.yaml` (campos ya definidos; este change MUST NOT modificar `openapi.yaml`). Los valores MUST copiarse del `HealthSnapshot` inyectado. MUST NOT consultar SurrealDB ni Meilisearch en el handler. Un snapshot sin stats publicadas MUST emitir `0`. Los otros componentes MUST NOT incluir `outboxLag` ni `failedEvents`. El cuerpo MUST seguir siendo `application/json` (no `problem+json`).

#### Scenario: GET /health incluye extras de sync en cero (happy)

- **WHEN** el `HealthReport` inyectado tiene `sync.outboxLag` 0 y `sync.failedEvents` 0
- **AND** se envía `GET /health` al router de `build_router`
- **THEN** el JSON tiene `components.sync.outboxLag` igual a 0
- **AND** `components.sync.failedEvents` igual a 0
- **AND** `components.storage` no tiene esas propiedades

#### Scenario: GET /health refleja fallidos publicados (unhappy path visible)

- **WHEN** el snapshot inyectado tiene `failedEvents` 2 y `outboxLag` 5
- **AND** se envía `GET /health`
- **THEN** el JSON tiene `components.sync.outboxLag` 5
- **AND** `components.sync.failedEvents` 2
- **AND** HTTP sigue 200 u 503 según el agregado (el lag no cambia el código)

### Requirement: Tests de extras de sync en GET /health

| Test | Crate | Tipo |
|---|---|---|
| `get_health_sync_extras_default_zero` | `api` | integración del crate |
| `get_health_sync_extras_reflect_snapshot` | `api` | integración del crate |

`cargo test --workspace` MUST ejecutarlos. Las respuestas MUST poder validarse contra `Health` de `openapi.yaml` (campos adicionales permitidos en `sync`).

#### Scenario: suite health extras (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p api get_health_sync_extras`
- **THEN** ambos tests corren y terminan con código 0
