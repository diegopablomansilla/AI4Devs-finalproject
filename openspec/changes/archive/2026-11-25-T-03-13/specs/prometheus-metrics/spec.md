# Spec Delta

Ticket: T-03-13. HU: HU-03-03, HU-03-04.

## ADDED Requirements

### Requirement: Histograma search_duration_seconds

El crate `api` MUST observar cada `GET /api/v1/search` que completa (éxito o error HTTP) en un histograma Prometheus llamado `search_duration_seconds`, sin labels de cardinalidad libre (sin `q` ni path crudo). La unidad MUST ser segundos. El histograma MUST aparecer en `GET /metrics` como tipo `histogram` (series `_bucket`, `_sum` y `_count`) tras al menos una búsqueda. El nombre MUST coincidir con `operation_duration_seconds("search")` (T-00-14). MUST NOT emitir la serie si no hubo ninguna búsqueda en ese proceso. MUST NOT registrar facetas ni ranking. La petición abortada por el cliente MAY no emitir muestra (el future se dropea antes de completar).

#### Scenario: una búsqueda queda en el histograma (happy)

- **WHEN** se envía `GET /api/v1/search?q=perno` al router (con Gateway inyectado) y después se lee `GET /metrics`
- **THEN** el cuerpo incluye `search_duration_seconds`
- **AND** existen las series `_bucket`, `_sum` y `_count` de ese histograma

#### Scenario: sin búsquedas no exige la serie (unhappy)

- **WHEN** se envía solo `GET /health` y después `GET /metrics` sin haber llamado `/search`
- **THEN** el scrape sigue siendo texto Prometheus válido
- **AND** este escenario MUST NOT exigir que `search_duration_seconds` esté ausente si otro test del mismo proceso ya observó una búsqueda (el recorder es global)
