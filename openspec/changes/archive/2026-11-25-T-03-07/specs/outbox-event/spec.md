# Spec Delta

Ticket padre: T-03-07. HU: HU-03-01.

## ADDED Requirements

### Requirement: Puerto OutboxRepo en domain

El crate `domain` MUST exponer el puerto `OutboxRepo` con operaciones para el consumidor Meilisearch (el consumidor `vectores` es T-05-08):

- Leer el lote de eventos con id de fila de `outbox` **estrictamente mayor** que el cursor actual, ordenados por id, con límite 200 ([4.8.1](../../../../../docs/4-modelo.md#481-outbox)).
- Leer el cursor `sync_cursor:meilisearch` (`ultimo_evento` ausente = origen).
- Avanzar `sync_cursor:meilisearch.ultimo_evento` al id de un evento ya confirmado.

Cada fila leída MUST incluir el id ULID de la fila `outbox` (sin prefijo de tabla) y el `OutboxEvent` (`entidad`, `registro` ULID sin prefijo, `operacion`, `motivo`). El puerto MUST devolver `DomainError` en fallos. MUST NOT abrir sockets, MUST NOT depender de `surrealdb` ni de `meilisearch-sdk`, MUST NOT escribir `outbox_fallo` (T-03-09), MUST NOT purgar (T-03-10).

#### Scenario: Lote desde el origen (happy)

- **WHEN** el cursor `meilisearch` no tiene `ultimo_evento` y existen eventos en `outbox`
- **THEN** la lectura devuelve esos eventos en orden de id
- **AND** a lo sumo 200 filas
- **AND** cada fila expone id de outbox y `OutboxEvent`

#### Scenario: Lote después del cursor (happy)

- **WHEN** `ultimo_evento` apunta a un evento existente y hay eventos posteriores
- **THEN** la lectura **no** incluye el evento del cursor
- **AND** incluye los posteriores en orden, hasta 200

#### Scenario: Avance de cursor (happy)

- **WHEN** se avanza el cursor `meilisearch` al id de un evento
- **THEN** la operación termina sin error
- **AND** una lectura posterior empieza **después** de ese id

#### Scenario: Cursor desconocido (unhappy)

- **WHEN** se pide avanzar a un id de `outbox` que no existe
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** `ultimo_evento` no cambia

### Requirement: Tests de OutboxRepo (puerto)

Antes de archive, la suite MUST incluir un doble de prueba del puerto en `domain` (happy de construcción del lote en memoria y unhappy de cursor inexistente). Los tests contra SurrealDB viven en `vortex-storage`.

| Test | Ubicación | Tipo |
|---|---|---|
| `outbox_repo_dummy_lists_after_cursor` | `vortex-core/crates/domain/` | happy |
| `outbox_repo_dummy_advance_not_found` | `vortex-core/crates/domain/` | unhappy |

#### Scenario: Dummy OutboxRepo cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p domain outbox_repo_dummy`
- **THEN** ambos tests de la tabla corren y terminan con código 0
