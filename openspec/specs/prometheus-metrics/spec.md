# prometheus-metrics Specification

## Purpose
Define el recorder de exposición Prometheus, el histograma HTTP de latencia por ruta coincidente y la convención de nombres reservados para que `/metrics` y los tickets posteriores compartan series estables, sin emitir métricas de dominio en este change.

## Requirements

### Requirement: Histograma HTTP con MatchedPath

El Gateway MUST observar cada petición HTTP (salvo el scrape de `GET /metrics`) en un histograma llamado `http_request_duration_seconds` con exactamente estas labels: `route`, `method`, `status`. `route` MUST ser la ruta coincidente del router (`MatchedPath`): para `GET /health` MUST ser `/health`. `method` MUST ser el método HTTP en mayúsculas (`GET`, `POST`, …). `status` MUST ser el código HTTP decimal como texto (`200`, `404`, `503`, …). La unidad MUST ser segundos. El histograma MUST aparecer en la exposición Prometheus como tipo `histogram` (series `_bucket`, `_sum` y `_count`). Este change MUST NOT registrar métricas de dominio.

#### Scenario: petición a /health queda en el histograma (happy)

- **WHEN** se envía `GET /health` al router y después se lee la exposición Prometheus
- **THEN** existe el histograma `http_request_duration_seconds`
- **AND** alguna serie incluye `route="/health"`, `method="GET"` y `status="200"` (o `status="503"` si el agregado de salud no es `ok`/`degraded`)
- **AND** existen las series `_bucket`, `_sum` y `_count` de ese histograma

#### Scenario: el scrape no se observa a sí mismo (happy)

- **WHEN** se envía solo `GET /metrics` sin ninguna petición previa en ese proceso de test aislado, o se compara el scrape consigo mismo
- **THEN** el cuerpo de esa respuesta MUST NOT ser la única evidencia de `route="/metrics"` como petición de negocio
- **AND** el scrape sigue siendo texto Prometheus válido (requirement «GET /metrics expone texto Prometheus» en `vortex-api`)

### Requirement: Ruta sin match usa label acotado

Si la petición no tiene ruta coincidente (`MatchedPath` ausente, p. ej. fallback 404), `route` MUST ser el valor fijo `unmatched`. `route` MUST NOT ser el URI crudo ni un path libre del cliente. Así la cardinalidad del histograma MUST permanecer acotada a las rutas registradas más `unmatched`.

#### Scenario: 404 no usa el URI crudo (unhappy)

- **WHEN** se envía `GET /no-such-route-xyz` al router y después se lee la exposición Prometheus
- **THEN** el histograma incluye `route="unmatched"` y `status="404"`
- **AND** el texto de exposición no incluye la label `route="/no-such-route-xyz"`

### Requirement: Convención de nombres reservados

El crate `api` MUST publicar constantes con estos nombres Prometheus para que los tickets siguientes no inventen aliases:

| Nombre | Tipo esperado | Ticket dueño |
|---|---|---|
| `outbox_lag` | gauge | T-03-09 |
| `outbox_failed_total` | counter | T-03-09 |
| `backup_failures_total` | counter | T-00-29 |
| `<operacion>_duration_seconds` | histogram | p. ej. `search_duration_seconds` (T-03-13), `recommendations_duration_seconds` (T-04-07), `speech_duration_seconds` (T-06-04) |

Este change MUST NOT emitir series `outbox_lag`, `outbox_failed_total` ni `backup_failures_total`. Las constantes MUST coincidir exactamente con esos strings. El patrón `<operacion>_duration_seconds` MUST usar `snake_case` y el sufijo `_duration_seconds`. El crate `domain` MUST NOT depender del recorder ni de HTTP.

#### Scenario: constantes coinciden con la convención (happy)

- **WHEN** se leen las constantes de nombres reservados en `api`
- **THEN** incluyen `outbox_lag`, `outbox_failed_total` y `backup_failures_total`
- **AND** el helper de duración produce `search_duration_seconds` para la operación `search`

#### Scenario: el scrape no emite series de dominio (unhappy)

- **WHEN** se envía `GET /health` y después `GET /metrics` sin haber registrado métricas de dominio
- **THEN** el texto de exposición no incluye una serie `outbox_lag`
- **AND** no incluye `outbox_failed_total` ni `backup_failures_total`

### Requirement: Tests del histograma y de la convención

Antes de archive, la suite MUST incluir estos tests (nombres estables; crate / tipo). Happy y unhappy path son obligatorios.

| Test | Crate | Tipo |
|---|---|---|
| `http_histogram_records_health_route` | `api` | integración del crate |
| `unmatched_route_uses_unmatched_label` | `api` | integración del crate |
| `reserved_metric_names_match_convention` | `api` | crate (constantes) |
| `scrape_does_not_emit_domain_series` | `api` | integración del crate |

`cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST ejecutarlos.

#### Scenario: la suite del histograma cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core`
- **THEN** corren los tests de la tabla
- **AND** los de `MatchedPath` `/health` y de `unmatched` en 404 pasan
- **AND** los de constantes reservadas y de ausencia de series de dominio pasan
