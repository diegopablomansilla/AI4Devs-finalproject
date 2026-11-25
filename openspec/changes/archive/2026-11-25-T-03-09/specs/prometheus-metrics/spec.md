# Spec Delta

Ticket: T-03-09. HU: HU-03-01, HU-00-05.

## ADDED Requirements

### Requirement: Series de sync en /metrics

El crate `api` MUST implementar el puerto de telemetría de sync (nombres exactos de T-00-14) con el recorder Prometheus ya instalado:

- gauge `outbox_lag` con label `consumidor` (`meilisearch` en el MVP)
- counter `outbox_failed_total`
- histogram `sync_propagation_seconds` (unidad segundos)

Tras registrar lag, un `GET /metrics` MUST incluir una serie `outbox_lag` con `consumidor="meilisearch"`. Tras un incremento de fallidos, MUST incluir `outbox_failed_total`. Tras observar una duración, MUST incluir `sync_propagation_seconds` (`_bucket` / `_sum` / `_count`). El crate `api` MUST NOT depender de `sync`. MUST NOT emitir `backup_failures_total` (T-00-29). MUST NOT emitir `outbox_lag` con `consumidor="vectores"`.

#### Scenario: scrape muestra lag publicado (happy)

- **WHEN** la telemetría Prometheus de sync publica `outbox_lag{consumidor="meilisearch"}` = 4
- **AND** se envía `GET /metrics`
- **THEN** el cuerpo incluye `outbox_lag` y `consumidor="meilisearch"`
- **AND** el valor observable es 4

#### Scenario: scrape muestra fallidos (unhappy path visible)

- **WHEN** la telemetría incrementa `outbox_failed_total`
- **AND** se envía `GET /metrics`
- **THEN** el cuerpo incluye `outbox_failed_total`
- **AND** el counter es ≥ 1

#### Scenario: scrape no inventa vectores ni backup (unhappy)

- **WHEN** solo se publicó telemetría de `meilisearch`
- **AND** se envía `GET /metrics`
- **THEN** el cuerpo no incluye `outbox_lag` con `consumidor="vectores"`
- **AND** no incluye `backup_failures_total` como serie de negocio emitida por este change

### Requirement: Tests de series de sync

| Test | Crate | Tipo |
|---|---|---|
| `scrape_includes_outbox_lag_after_telemetry` | `api` | integración del crate |
| `scrape_includes_outbox_failed_total_after_increment` | `api` | integración del crate |
| `scrape_does_not_emit_vectores_lag_or_backup_failures` | `api` | integración del crate |

`cargo test --workspace` MUST ejecutarlos.

#### Scenario: suite series sync (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p api scrape_includes_outbox scrape_does_not_emit_vectores`
- **THEN** los tests de la tabla corren y terminan con código 0

## MODIFIED Requirements

### Requirement: Convención de nombres reservados

El crate `api` MUST publicar constantes con estos nombres Prometheus para que los tickets siguientes no inventen aliases:

| Nombre | Tipo esperado | Ticket dueño |
|---|---|---|
| `outbox_lag` | gauge | T-03-09 (este change **emite** la serie) |
| `outbox_failed_total` | counter | T-03-09 (este change **emite** la serie) |
| `backup_failures_total` | counter | T-00-29 |
| `<operacion>_duration_seconds` | histogram | p. ej. `search_duration_seconds` (T-03-13), `recommendations_duration_seconds` (T-04-07), `speech_duration_seconds` (T-06-04); `sync_propagation_seconds` lo emite T-03-09 |

Las constantes MUST coincidir exactamente con esos strings. El patrón `<operacion>_duration_seconds` MUST usar `snake_case` y el sufijo `_duration_seconds`. El crate `domain` MUST NOT depender del recorder ni de HTTP. Este change MUST NOT emitir `backup_failures_total`.

#### Scenario: constantes coinciden con la convención (happy)

- **WHEN** se leen las constantes de nombres reservados en `api`
- **THEN** incluyen `outbox_lag`, `outbox_failed_total` y `backup_failures_total`
- **AND** el helper de duración produce `search_duration_seconds` para la operación `search`

#### Scenario: el scrape no emite series de dominio (unhappy)

- **WHEN** se envía `GET /health` y después `GET /metrics` sin haber registrado telemetría de sync ni fallos de backup
- **THEN** el texto de exposición no incluye `backup_failures_total`
- **AND** MAY incluir `outbox_lag` / `outbox_failed_total` solo si otro test del mismo proceso ya publicó telemetría (el recorder es global); este escenario MUST NOT exigir su ausencia absoluta
- **AND** MUST NOT incluir `backup_failures_total`

### Requirement: Tests del histograma y de la convención

Antes de archive, la suite MUST incluir estos tests (nombres estables; crate / tipo). Happy y unhappy path son obligatorios.

| Test | Crate | Tipo |
|---|---|---|
| `http_histogram_records_health_route` | `api` | integración del crate |
| `unmatched_route_uses_unmatched_label` | `api` | integración del crate |
| `reserved_metric_names_match_convention` | `api` | crate (constantes) |
| `scrape_does_not_emit_backup_failures_total` | `api` | integración del crate |

MUST NOT conservar `scrape_does_not_emit_domain_series` (T-03-09 emite `outbox_lag` / `outbox_failed_total` cuando hay telemetría; el test de ausencia queda limitado a `backup_failures_total`). `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST ejecutarlos.

#### Scenario: la suite del histograma cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core`
- **THEN** corren los tests de la tabla
- **AND** los de `MatchedPath` `/health` y de `unmatched` en 404 pasan
- **AND** los de constantes reservadas y de ausencia de `backup_failures_total` pasan
