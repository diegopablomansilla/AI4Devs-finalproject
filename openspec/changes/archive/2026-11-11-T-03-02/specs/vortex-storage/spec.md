# Spec Delta

Ticket padre: T-03-02. Change: T-03-02. HU: HU-03-01.

## ADDED Requirements

### Requirement: Transacción dato + outbox

El crate `storage` MUST exponer `storage::Tx`, obtenida desde un `Db` ya abierto y migrado. `Tx` MUST acumular sentencias SurrealQL con parámetros y uno o más `OutboxEvent`. Al confirmar, MUST ejecutar todas las sentencias y un `CREATE` en `outbox` por cada evento **entre** `BEGIN` y `COMMIT` (una sola transacción). El `CREATE` de outbox MUST usar `entidad` y `operacion` del evento, `registro` como `record` `entidad:ulid` (el ULID de `OutboxEvent.registro`) y `motivo` si está presente. `Tx::commit` MUST devolver `Result<(), DomainError>`. MUST NOT persistir el dato si el `COMMIT` no ocurre. MUST NOT implementar repositorios de catálogo (T-02-03 y siguientes). Los tests MUST usar una tabla auxiliar definida en el propio test, no tablas de negocio.

#### Scenario: Commit escribe dato y evento (happy)

- **WHEN** una `Tx` inserta un registro en una tabla auxiliar y emite un `OutboxEvent` `producto`/`upsert`, y se confirma
- **THEN** `commit` termina sin error
- **AND** el conteo de la tabla auxiliar es 1
- **AND** el conteo de `outbox` es 1
- **AND** el evento tiene `entidad` `producto` y `operacion` `upsert`

#### Scenario: Varios eventos en la misma transacción (happy)

- **WHEN** una `Tx` emite dos `OutboxEvent` y se confirma
- **THEN** `commit` termina sin error
- **AND** el conteo de `outbox` es 2

#### Scenario: THROW no deja dato ni evento (unhappy)

- **WHEN** una `Tx` inserta un registro auxiliar, emite un `OutboxEvent` y ejecuta `THROW "stock_negativo"`, y se intenta confirmar
- **THEN** `commit` falla
- **AND** el conteo de la tabla auxiliar es 0
- **AND** el conteo de `outbox` es 0

### Requirement: OutboxNotifier tras commit exitoso

El crate `storage` MUST exponer `OutboxNotifier` basado en `tokio::sync::Notify`. Un `Db` MUST compartir un `OutboxNotifier`. Tras un `Tx::commit` **exitoso**, MUST disparar el notifier (al menos un waiter en espera MUST despertar). Si `commit` falla, MUST NOT disparar el notifier. MUST NOT arrancar el worker Index Sync (T-03-07).

#### Scenario: Aviso solo tras commit (happy)

- **WHEN** un waiter espera el `OutboxNotifier` y una `Tx` con dato + evento se confirma
- **THEN** el waiter se despierta
- **AND** el dato y el evento existen

#### Scenario: Fallo no avisa (unhappy)

- **WHEN** un waiter espera el `OutboxNotifier` y una `Tx` con dato + evento + `THROW` falla al confirmar
- **THEN** el waiter no se despierta por ese `commit`
- **AND** el dato y el evento no existen

### Requirement: UNIQUE y THROW se traducen a DomainError

`Tx::commit` MUST traducir errores del motor a `DomainError` existente (T-00-03) y MUST NOT mapear a HTTP ni `problem+json`. `THROW "stock_negativo"` MUST ser `InvalidInput` con `code` `negative_stock` ([4.5.2](../../../../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción), [5.4](../../../../../docs/5-contract-api.md#54-errores)). Un `THROW` cuyo texto es un `code` de dominio conocido MUST usar esa variante/`code`. Una violación de índice `UNIQUE` MUST ser `Conflict`: `duplicate_sku` si el índice o el mensaje identifican `sku`; `duplicate_slug` si identifican `slug`; `duplicate_relation` si identifican `relation`; en cualquier otro UNIQUE, `conflict`. Un fallo de motor no clasificado MUST ser `Unavailable` con `code` `unavailable`. MUST NOT agregar variantes ni `code` nuevos a `DomainError`.

#### Scenario: THROW stock_negativo (unhappy)

- **WHEN** una `Tx` ejecuta `THROW "stock_negativo"` y se confirma
- **THEN** `commit` falla con `DomainError::InvalidInput`
- **AND** el `code` expuesto es `negative_stock`

#### Scenario: UNIQUE sku (unhappy)

- **WHEN** una `Tx` inserta dos filas que violan un índice `UNIQUE` cuyo nombre incluye `sku` y se confirma
- **THEN** `commit` falla con `DomainError::Conflict`
- **AND** el `code` expuesto es `duplicate_sku`
- **AND** no queda la segunda fila ni su evento de outbox (si se había encolado en esa `Tx`)

#### Scenario: Error de motor no clasificado (unhappy)

- **WHEN** una `Tx` ejecuta una sentencia que el motor rechaza sin ser `THROW` de código conocido ni UNIQUE (p. ej. SQL inválido)
- **THEN** `commit` falla con `DomainError::Unavailable`
- **AND** el `code` expuesto es `unavailable`

### Requirement: Tests de Tx y notifier

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. **Listo cuando** del ticket: rollback y notify.

| Test | Ubicación | Tipo |
|---|---|---|
| `tx_commit_writes_data_and_outbox` | `vortex-core/crates/storage/tests/tx.rs` | integración / happy |
| `tx_commit_writes_multiple_outbox_events` | `tests/tx.rs` | integración / happy |
| `tx_commit_notifies_waiter` | `tests/tx.rs` | integración / happy |
| `tx_throw_rolls_back_data_and_outbox` | `tests/tx.rs` | integración / unhappy (Listo cuando) |
| `tx_failed_does_not_notify` | `tests/tx.rs` | integración / unhappy (Listo cuando) |
| `tx_throw_stock_negativo_is_negative_stock` | `tests/tx.rs` | integración / unhappy |
| `tx_unique_sku_is_duplicate_sku` | `tests/tx.rs` | integración / unhappy |
| `tx_unclassified_error_is_unavailable` | `tests/tx.rs` | integración / unhappy |

Los tests MUST usar `storage::testing::test_db()`. MUST NOT crear repositorios de catálogo ni tablas `categoria`/`producto` de negocio. `cargo test --workspace` MUST ejecutarlos.

#### Scenario: la suite de Tx cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p storage --test tx` en `vortex-core/`
- **THEN** corren los tests de la tabla
- **AND** commit escribe dato + outbox y avisa
- **AND** THROW deja conteos en 0 y no avisa

## MODIFIED Requirements

### Requirement: Migración outbox, sync_cursor y outbox_fallo

El crate `storage` MUST incluir la migración embebida `0001_outbox.surql`, aplicada por `Db::migrate()` después de `0000_migracion.surql`. El SQL MUST definir las tablas `SCHEMAFULL` `outbox`, `sync_cursor` y `outbox_fallo` conforme a [4.8](../../../../../docs/4-modelo.md#48-sincronización-outbox-y-cursores), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

`outbox` MUST declarar: `entidad` (`string` con `ASSERT` en `producto`, `categoria`, `marca`, `atributo_def`, `sinonimo`, `nota_tecnica`, `imagen_producto`); `registro` (`record`); `operacion` (`string` con `ASSERT` en `upsert`, `delete`); `motivo` (`option<string>`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`).

`sync_cursor` MUST declarar: `ultimo_evento` (`option<record<outbox>>`); `actualizado_en` (`datetime` con `VALUE time::now()`). La migración MUST crear los registros `sync_cursor:meilisearch` y `sync_cursor:vectores` con `ultimo_evento` vacío si aún no existen.

`outbox_fallo` MUST declarar: `evento` (`record<outbox>`); `consumidor` (`string` con `ASSERT` en `meilisearch`, `vectores`); `intentos` (`int` con `DEFAULT 1`); `ultimo_error` (`string`); `estado` (`string` con `ASSERT` en `reintentando`, `fallido`); `actualizado_en` (`datetime` con `VALUE time::now()`); índice único `uq_outbox_fallo` sobre `(evento, consumidor)`.

MUST NOT habilitar *change feed* en estas tablas (tablas técnicas; el *change feed* aplica a maestras y ledger). MUST NOT implementar el worker Index Sync ni puertos de repositorio de catálogo (T-03-07 / T-02-03).

#### Scenario: Migración aplica esquema y siembra cursores (happy)

- **WHEN** se abre una base vacía y se llama `Db::migrate` (o `storage::testing::test_db`)
- **THEN** `list_migrations()` incluye `version` 1 con checksum SHA-256 de `0001_outbox.surql`
- **AND** existen los registros `sync_cursor:meilisearch` y `sync_cursor:vectores`
- **AND** `outbox` y `outbox_fallo` están vacías

#### Scenario: Re-arranque no duplica cursores (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro de `migracion` para `version` 1
- **AND** sigue habiendo exactamente dos registros en `sync_cursor`

#### Scenario: Evento válido se inserta (happy)

- **WHEN** tras migrar se inserta un evento `outbox` con `entidad` `producto`, `operacion` `upsert` y un `registro` válido
- **THEN** la inserción termina sin error
- **AND** el conteo de `outbox` es 1

#### Scenario: entidad inválida rechazada (unhappy)

- **WHEN** tras migrar se intenta insertar un evento `outbox` con `entidad` fuera de la lista permitida (p. ej. `foo`)
- **THEN** la operación falla
- **AND** `outbox` permanece vacía

#### Scenario: operacion inválida rechazada (unhappy)

- **WHEN** tras migrar se intenta insertar un evento `outbox` con `operacion` distinta de `upsert` y `delete` (p. ej. `update`)
- **THEN** la operación falla
- **AND** `outbox` permanece vacía

#### Scenario: consumidor inválido en outbox_fallo (unhappy)

- **WHEN** tras migrar se intenta insertar `outbox_fallo` con `consumidor` fuera de `meilisearch` y `vectores`
- **THEN** la operación falla
- **AND** `outbox_fallo` permanece vacía
