# Spec Delta

Ticket: T-03-09. HU: HU-03-01.

## ADDED Requirements

### Requirement: Adaptador OutboxRepo escribe outbox_fallo y mide lag

El adaptador `OutboxStore` MUST implementar las operaciones nuevas de `OutboxRepo` contra las tablas ya migradas en T-03-01 (`outbox_fallo`, `outbox`, `sync_cursor`). MUST NOT crear una migración nueva. `creado_en` MUST leerse de `outbox`. El upsert de `outbox_fallo` MUST respetar `uq_outbox_fallo` (`evento`, `consumidor`). Lag MUST ser `SELECT count` (o equivalente) de `outbox` con `id > sync_cursor:meilisearch.ultimo_evento`. MUST NOT actualizar `sync_cursor:vectores`.

#### Scenario: Persiste fallido (happy)

- **WHEN** existe una fila `outbox` y se registra un fallo `meilisearch`
- **THEN** hay una fila `outbox_fallo` con `estado` `fallido` y `consumidor` `meilisearch` para ese evento
- **AND** un segundo registro del mismo par no duplica la fila

#### Scenario: Lag en Surreal (happy)

- **WHEN** el cursor `meilisearch` está detrás de N eventos
- **THEN** el lag devuelto es N

#### Scenario: Fallo de evento inexistente (unhappy)

- **WHEN** se registra un fallo para un ULID ausente en `outbox`
- **THEN** el error es `DomainError` `code` `not_found`
- **AND** no queda fila nueva en `outbox_fallo`

### Requirement: Tests de OutboxRepo fallos en storage

| Test | Ubicación | Tipo |
|---|---|---|
| `storage_outbox_repo_records_fallido` | `vortex-core/crates/storage/tests/` | happy |
| `storage_outbox_repo_lag_counts_pending` | `vortex-core/crates/storage/tests/` | happy |
| `storage_outbox_repo_failed_count` | `vortex-core/crates/storage/tests/` | happy |
| `storage_outbox_repo_record_unknown_event_is_not_found` | `vortex-core/crates/storage/tests/` | unhappy |

MUST usar `storage::testing::test_db()`. MUST NOT exigir Meilisearch.

#### Scenario: Suite storage fallos y lag (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p storage storage_outbox_repo`
- **THEN** los tests de la tabla de este requirement y los ya existentes de T-03-07 corren y terminan con código 0

## MODIFIED Requirements

### Requirement: Adaptador OutboxRepo

El crate `storage` MUST implementar el puerto `OutboxRepo` contra las tablas `outbox`, `sync_cursor` y `outbox_fallo` ya migradas (T-03-01). Los ids de fila MUST ser ULID de 26 caracteres **sin** prefijo de tabla. `registro` del evento MUST exponerse como ULID sin `tabla:`. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP. MUST NOT implementar el worker (vive en `sync`). MUST escribir `outbox_fallo` solo vía las operaciones del puerto (requirement «Adaptador OutboxRepo escribe outbox_fallo y mide lag»).

La lectura MUST ejecutar el equivalente de `SELECT * FROM outbox WHERE id > $cursor ORDER BY id LIMIT 200` (cursor vacío = todos desde el origen) e incluir `creado_en`. El avance MUST actualizar `sync_cursor:meilisearch` (`ultimo_evento` = `outbox:<ulid>`). Un id de evento que no existe MUST ser `not_found` y MUST NOT cambiar el cursor.

#### Scenario: Lee eventos posteriores al cursor (happy)

- **WHEN** hay al menos dos filas en `outbox` y el cursor `meilisearch` apunta a la primera
- **THEN** la lectura termina sin error
- **AND** no incluye la primera
- **AND** incluye la segunda (y posteriores) en orden de id
- **AND** `registro` no lleva prefijo de tabla

#### Scenario: Avanza el cursor meilisearch (happy)

- **WHEN** se avanza el cursor al id de un evento persistido
- **THEN** la operación termina sin error
- **AND** `sync_cursor:meilisearch.ultimo_evento` apunta a ese evento
- **AND** una lectura posterior omite ese evento

#### Scenario: Avance a evento inexistente (unhappy)

- **WHEN** se intenta avanzar el cursor a un ULID que no existe en `outbox`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** `ultimo_evento` queda como antes

#### Scenario: Lote acotado a 200 (happy)

- **WHEN** existen más de 200 eventos posteriores al cursor
- **THEN** la lectura devuelve exactamente 200 filas
- **AND** son las 200 de id menor entre las posteriores
