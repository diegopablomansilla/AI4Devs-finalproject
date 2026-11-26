# Spec Delta

Ticket: T-04-07. HU: HU-04-05.

## ADDED Requirements

### Requirement: Histograma recommendations_duration_seconds

El crate `api` MUST observar cada `GET /api/v1/products/{productId}/recommendations` que completa (éxito o error HTTP) en un histograma Prometheus llamado `recommendations_duration_seconds`, sin labels de cardinalidad libre (sin `productId` ni path crudo). La unidad MUST ser segundos. El histograma MUST aparecer en `GET /metrics` como tipo `histogram` (series `_bucket`, `_sum` y `_count`) tras al menos una consulta de recomendaciones. El nombre MUST coincidir con `operation_duration_seconds("recommendations")` (T-00-14). MUST NOT emitir la serie si no hubo ninguna consulta de recomendaciones en ese proceso. MUST NOT cachear muestras ni registrar UI. La petición abortada por el cliente MAY no emitir muestra (el future se dropea antes de completar).

#### Scenario: una consulta queda en el histograma (happy)

- **WHEN** se envía `GET /api/v1/products/{ulid}/recommendations` al router (con Gateway inyectado) y después se lee `GET /metrics`
- **THEN** el cuerpo incluye `recommendations_duration_seconds`
- **AND** existen las series `_bucket`, `_sum` y `_count` de ese histograma

#### Scenario: sin consultas no exige la serie (unhappy)

- **WHEN** se envía solo `GET /health` y después `GET /metrics` sin haber llamado `/recommendations`
- **THEN** el scrape sigue siendo texto Prometheus válido
- **AND** este escenario MUST NOT exigir que `recommendations_duration_seconds` esté ausente si otro test del mismo proceso ya observó una consulta (el recorder es global)
