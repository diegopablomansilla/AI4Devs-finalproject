# Spec Delta

Ticket: T-03-09. HU: HU-03-01, HU-00-05.

## ADDED Requirements

### Requirement: sync expone outboxLag y failedEvents

El snapshot de `HealthReport` MUST incluir en el componente `sync`, además de `status`, los enteros `outboxLag` ≥ 0 y `failedEvents` ≥ 0. `outboxLag` MUST ser el lag del consumidor `meilisearch` (eventos de outbox pendientes de ese cursor). `failedEvents` MUST ser el conteo de `outbox_fallo` con `consumidor` `meilisearch` y `estado` `fallido`. Ausencia de publicación MUST serializarse como 0, no como omisión del campo en el modelo de dominio. Un `outboxLag` > 0 MUST NOT por sí solo cambiar el `status` de `sync` ni el agregado ( [5.11](../../../../../docs/5-contract-api.md#511-degradación-vista-desde-la-api): outbox atrasado no es error). Los demás componentes MUST NOT exigir estos campos. El crate `domain` MUST NOT conocer HTTP ni Prometheus.

El `HealthRegistry` MUST inicializar `outboxLag` y `failedEvents` en 0 y MUST permitir publicarlos (junto con el `status` de `sync` o por una operación dedicada). MUST NOT sondear Meilisearch ni abrir SurrealDB para calcularlos: solo expone valores ya publicados (el worker / T-00-27 los actualizan).

#### Scenario: snapshot sync trae lag y fallidos en cero (happy)

- **WHEN** se crea un registro de salud nuevo y se pide el snapshot
- **THEN** `components.sync.outboxLag` es 0
- **AND** `components.sync.failedEvents` es 0

#### Scenario: publicar stats de sync se refleja (happy)

- **WHEN** un registro recibe `outboxLag` = 3 y `failedEvents` = 1
- **THEN** el snapshot tiene esos valores en `components.sync`
- **AND** el `status` de `sync` no pasa a `degraded` solo por el lag

#### Scenario: lag no degrada el agregado (happy)

- **WHEN** los seis componentes están `up` (o `disabled` salvo `storage` `up`) y `outboxLag` > 0
- **THEN** el estado agregado sigue las reglas de T-00-13 (no usa `outboxLag`)

### Requirement: Tests de extras de sync en salud

| Test | Crate | Tipo |
|---|---|---|
| `registry_new_sync_stats_are_zero` | `supervisor` | crate |
| `registry_set_sync_stats_are_reflected` | `supervisor` | crate |
| `aggregate_ignores_outbox_lag` | `domain` | unitario |

`cargo test --workspace` MUST ejecutarlos.

#### Scenario: extras de sync cubren happy (happy)

- **WHEN** se ejecuta `cargo test -p supervisor registry_set_sync_stats` y `cargo test -p domain aggregate_ignores_outbox_lag`
- **THEN** terminan con código 0
