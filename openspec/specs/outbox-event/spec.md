# outbox-event Specification

## Purpose

Define el tipo de dominio `OutboxEvent` (entidad, registro ULID, operación y motivo) para que las escrituras de catálogo emitan un evento de sincronización sin acoplar `domain` a SurrealDB.

Ticket padre: T-03-02. HU: HU-03-01.

## Requirements

### Requirement: OutboxEvent en domain

El crate `domain` MUST exponer `OutboxEvent` con exactamente estos campos: `entidad`, `registro`, `operacion` y `motivo`. `entidad` MUST ser uno de `producto`, `categoria`, `marca`, `atributo_def`, `sinonimo`, `nota_tecnica`, `imagen_producto` ([4.8.1](../../../docs/4-modelo.md#481-outbox)). `operacion` MUST ser `upsert` o `delete`. `registro` MUST ser un ULID Crockford de 26 caracteres **sin** prefijo de tabla (mismas reglas que `Id<T>`). `motivo` MUST ser `Option` de string (pista para el worker: `stock`, `ficha`, `precio`, u otra). El crate MUST NOT depender de `surrealdb` ni incluir `tabla:` en `registro`. MUST NOT ampliar las variantes ni los `code` de `DomainError`.

#### Scenario: Construcción válida (happy)

- **WHEN** se construye un `OutboxEvent` con `entidad` `producto`, `operacion` `upsert`, `registro` un ULID de 26 caracteres y `motivo` `Some("stock")`
- **THEN** la construcción termina sin error
- **AND** los cuatro campos expuestos coinciden con esos valores

#### Scenario: motivo ausente (happy)

- **WHEN** se construye un `OutboxEvent` con `motivo` vacío (`None`)
- **THEN** la construcción termina sin error
- **AND** `motivo` expuesto es `None`

#### Scenario: Prefijo de tabla en registro (unhappy)

- **WHEN** se intenta construir un `OutboxEvent` con `registro` `producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8` (u otro `tabla:` + ULID)
- **THEN** la operación falla
- **AND** no se produce un `OutboxEvent`

#### Scenario: entidad u operación fuera de lista (unhappy)

- **WHEN** se intenta construir un `OutboxEvent` con `entidad` `foo` o `operacion` `update`
- **THEN** la operación falla
- **AND** no se produce un `OutboxEvent`

### Requirement: Tests de OutboxEvent

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `outbox_event_valid_producto_upsert` | `vortex-core/crates/domain/` (unidad) | happy |
| `outbox_event_motivo_none` | `vortex-core/crates/domain/` (unidad) | happy |
| `outbox_event_rejects_table_prefix` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `outbox_event_rejects_unknown_entidad_or_operacion` | `vortex-core/crates/domain/` (unidad) | unhappy |

`cargo test --workspace` MUST ejecutarlos. MUST NOT usar SurrealDB ni el crate `storage`.

#### Scenario: la suite de OutboxEvent cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p domain` en `vortex-core/`
- **THEN** corren los tests de la tabla
- **AND** los de construcción válida y `motivo` ausente pasan
- **AND** los de prefijo de tabla y entidad/operación inválidas pasan

### Requirement: Puerto OutboxRepo en domain

El crate `domain` MUST exponer el puerto `OutboxRepo` con operaciones para el consumidor Meilisearch (el consumidor `vectores` es T-05-08):

- Leer el lote de eventos con id de fila de `outbox` **estrictamente mayor** que el cursor actual, ordenados por id, con límite 200 ([4.8.1](../../../../../docs/4-modelo.md#481-outbox)).
- Leer el cursor `sync_cursor:meilisearch` (`ultimo_evento` ausente = origen).
- Avanzar `sync_cursor:meilisearch.ultimo_evento` al id de un evento ya confirmado.
- Registrar `outbox_fallo` `fallido` del consumidor `meilisearch`, leer lag de `meilisearch` y leer el conteo de `fallido` (requirement «OutboxRepo registra fallos, lag y creado_en»).

Cada fila leída MUST incluir el id ULID de la fila `outbox` (sin prefijo de tabla), el `OutboxEvent` (`entidad`, `registro` ULID sin prefijo, `operacion`, `motivo`) y `creado_en`. El puerto MUST devolver `DomainError` en fallos. MUST NOT abrir sockets, MUST NOT depender de `surrealdb` ni de `meilisearch-sdk`, MUST NOT purgar (T-03-10).

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
### Requirement: OutboxRepo registra fallos, lag y creado_en

El puerto `OutboxRepo` MUST además:

- Exponer `creado_en` (datetime RFC 3339) en cada fila leída, para `sync_propagation_seconds`.
- Registrar un fallo permanente del consumidor `meilisearch`: upsert de `outbox_fallo` con `evento`, `consumidor` = `meilisearch`, `estado` = `fallido`, `ultimo_error` no vacío e `intentos` ≥ 1. Un par `(evento, consumidor)` ya existente MUST actualizarse (índice único), no insertar duplicados.
- Devolver el lag del consumidor `meilisearch`: cantidad de filas `outbox` con id estrictamente mayor que `sync_cursor:meilisearch.ultimo_evento` (cursor vacío = todas las filas). MUST NOT incluir el cursor `vectores`.
- Devolver el conteo de filas `outbox_fallo` con `consumidor` `meilisearch` y `estado` `fallido`.

Registrar un fallo cuyo `evento` no existe en `outbox` MUST ser `DomainError` `code` `not_found` y MUST NOT insertar fila. MUST NOT purgar (T-03-10). MUST NOT escribir `estado` `reintentando`. MUST NOT operar el consumidor `vectores` (T-05-08).

#### Scenario: Registro de fallido (unhappy path persistido)

- **WHEN** se registra un fallo para un evento de outbox existente con mensaje de error no vacío
- **THEN** la operación termina sin error
- **AND** el conteo de `fallido` de `meilisearch` aumenta en 1 (o permanece 1 si ya existía esa pareja)

#### Scenario: Evento inexistente al registrar fallo (unhappy)

- **WHEN** se pide registrar un fallo para un ULID que no existe en `outbox`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** el conteo de `fallido` no cambia

#### Scenario: Lag cuenta pendientes de meilisearch (happy)

- **WHEN** el cursor `meilisearch` apunta a un evento y hay N eventos posteriores
- **THEN** el lag de `meilisearch` es N
- **AND** no usa el cursor `vectores`

#### Scenario: Fila leída incluye creado_en (happy)

- **WHEN** se lee un lote de outbox
- **THEN** cada fila expone `creado_en`

### Requirement: Tests de OutboxRepo fallos y lag (puerto)

| Test | Ubicación | Tipo |
|---|---|---|
| `outbox_repo_dummy_records_fallido` | `vortex-core/crates/domain/` | happy |
| `outbox_repo_dummy_lag_counts_pending` | `vortex-core/crates/domain/` | happy |
| `outbox_repo_dummy_record_unknown_event_is_not_found` | `vortex-core/crates/domain/` | unhappy |

`cargo test -p domain outbox_repo_dummy` MUST ejecutarlos.

#### Scenario: Dummy cubre fallido y lag (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p domain outbox_repo_dummy`
- **THEN** los tests de la tabla corren y terminan con código 0
