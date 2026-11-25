# Spec Delta

Ticket padre: T-03-07. HU: HU-03-01.

## ADDED Requirements

### Requirement: Adaptador OutboxRepo

El crate `storage` MUST implementar el puerto `OutboxRepo` contra las tablas `outbox` y `sync_cursor` ya migradas (T-03-01). Los ids de fila MUST ser ULID de 26 caracteres **sin** prefijo de tabla. `registro` del evento MUST exponerse como ULID sin `tabla:`. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP. MUST NOT implementar el worker (vive en `sync`). MUST NOT escribir `outbox_fallo`.

La lectura MUST ejecutar el equivalente de `SELECT * FROM outbox WHERE id > $cursor ORDER BY id LIMIT 200` (cursor vacío = todos desde el origen). El avance MUST actualizar `sync_cursor:meilisearch` (`ultimo_evento` = `outbox:<ulid>`). Un id de evento que no existe MUST ser `not_found` y MUST NOT cambiar el cursor.

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

### Requirement: Tests de OutboxRepo en storage

Antes de archive, la suite MUST incluir:

| Test | Ubicación | Tipo |
|---|---|---|
| `storage_outbox_repo_lists_after_cursor` | `vortex-core/crates/storage/tests/` | happy |
| `storage_outbox_repo_advances_meilisearch_cursor` | `vortex-core/crates/storage/tests/` | happy |
| `storage_outbox_repo_limits_batch_to_200` | `vortex-core/crates/storage/tests/` | happy |
| `storage_outbox_repo_advance_unknown_event_is_not_found` | `vortex-core/crates/storage/tests/` | unhappy |

MUST usar `storage::testing::test_db()`. MUST NOT exigir Meilisearch.

#### Scenario: Suite OutboxRepo storage (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p storage storage_outbox_repo`
- **THEN** los cuatro tests de la tabla corren y terminan con código 0
