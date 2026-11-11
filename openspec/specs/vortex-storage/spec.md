# vortex-storage Specification

## Purpose

Abre SurrealDB embebido con RocksDB en el crate `storage`, selecciona el namespace `vectra` y la base `comercio`, aplica migraciones `.surql` embebidas con registro y checksum SHA-256, y expone un harness de tests (`storage::testing`) con motor en memoria o RocksDB temporal.

Ticket padre: T-00-08, T-00-09, T-09-02, T-03-01. HU: HU-00-02, HU-09-01, HU-03-01.

## Requirements

### Requirement: Db::open abre RocksDB embebido y selecciona NS/DB

El crate `storage` MUST exponer `storage::Db::open(path)` asíncrono. `path` MUST ser el directorio del motor (convención de instalación: `<paths.datos>/db`; este change MUST NOT leer `Config`). El motor MUST ser SurrealDB embebido con RocksDB (PM-03). MUST NOT usarse el motor en memoria en esta API. Tras una apertura exitosa, la sesión MUST usar el namespace `vectra` y la base `comercio` ([4.3](../../../docs/4-modelo.md#43-convenciones)). Si el directorio no existe o está vacío, la operación MUST crear lo necesario para esa sesión (directorio, namespace y base) y MUST NOT aplicar migraciones `.surql` ni crear la tabla `migracion` (T-00-09). La versión de `surrealdb` MUST ser la pinneada en `[workspace.dependencies]` (`=2.7.0`).

#### Scenario: Primera apertura en directorio temporal (happy)

- **WHEN** se llama a `Db::open` con un directorio temporal vacío o inexistente
- **THEN** la operación termina sin error
- **AND** la sesión usa namespace `vectra` y base `comercio`

#### Scenario: Reapertura del mismo directorio (happy)

- **WHEN** se abre un directorio con `Db::open`, se cierra esa instancia y se vuelve a llamar a `Db::open` con el mismo path
- **THEN** la segunda apertura termina sin error
- **AND** la sesión sigue usando namespace `vectra` y base `comercio`

### Requirement: Errores de apertura tipados con motivo

`Db::open` MUST devolver un error tipado (`OpenError`) que implementa `std::error::Error` cuando el motor no puede abrir el path. El mensaje MUST incluir un motivo legible (permiso denegado, contenido corrupto u otro fallo de IO/motor). Ese error MUST alinearse conceptualmente con `DomainError::Unavailable` y `code` `unavailable` (T-00-03) y MUST NOT mapearse a HTTP ni a `problem+json` (T-00-11). MUST NOT ampliar las variantes ni los `code` de `DomainError`.

#### Scenario: Directorio corrupto (unhappy)

- **WHEN** se llama a `Db::open` con un path que existe pero no es una base RocksDB válida (archivo en lugar de directorio, o directorio con contenido de motor ilegible)
- **THEN** la operación falla
- **AND** el error es el tipo de apertura tipado
- **AND** el mensaje incluye un motivo (corrupción, formato inválido o equivalente del motor/IO)

#### Scenario: Directorio sin permisos (unhappy)

- **WHEN** se llama a `Db::open` con un directorio existente que el proceso no puede escribir (Unix: modo sin permisos)
- **THEN** la operación falla
- **AND** el error es el tipo de apertura tipado
- **AND** el mensaje incluye un motivo (permiso denegado o equivalente)

#### Scenario: OpenError es std::error::Error (happy)

- **WHEN** se obtiene un `OpenError` de una apertura fallida
- **THEN** el valor implementa `std::error::Error`
- **AND** `Display` no está vacío

### Requirement: Motor de migraciones embebidas con checksum

El crate `storage` MUST incluir migraciones en `migrations/NNNN_nombre.surql` embebidas con `include_str!`. MUST existir `0000_migracion.surql` conforme a [4.10](../../../docs/4-modelo.md#410-operación-del-esquema). `Db::migrate()` MUST aplicar las migraciones embebidas en orden ascendente de `version`, registrar cada una en `migracion` con `checksum` SHA-256 hex del contenido del archivo, y ser idempotente cuando el contenido no cambió. Si una `version` ya registrada tiene un `checksum` distinto al del archivo embebido, la operación MUST fallar sin aplicar más SQL y el error MUST nombrar la versión en `Display`. `Db::open` MUST NOT aplicar migraciones (T-00-08).

#### Scenario: Primera instalación (happy)

- **WHEN** tras `Db::open` se llama `Db::migrate` en un directorio temporal vacío
- **THEN** `migrate` termina sin error
- **AND** existe un registro con `version` 0 y `checksum` igual al SHA-256 de `0000_migracion.surql`

#### Scenario: Re-arranque sin cambios (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia con las mismas migraciones embebidas
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro para la `version` 0

#### Scenario: Migración alterada (unhappy)

- **WHEN** la versión 0 ya está registrada y se intenta `apply_migrations` con SQL distinto en la misma versión
- **THEN** la operación falla
- **AND** el mensaje de error incluye la versión (`0` o `0000`)

### Requirement: Harness de integración storage::testing

El crate `storage` MUST exponer el módulo `storage::testing`. `test_db()` MUST abrir SurrealDB embebido con motor en memoria (`kv-mem`), MUST seleccionar namespace `vectra` y base `comercio`, y MUST aplicar todas las migraciones embebidas (`Db::migrate`) antes de devolver el `Db`. `test_db_rocks()` MUST abrir RocksDB en un directorio temporal único, MUST migrar, y MUST liberar el directorio al descartar el valor retornado. `Db::open(path)` MUST seguir usando únicamente RocksDB y MUST NOT aplicar migraciones. Los tests de integración del crate MUST vivir bajo `vortex-core/crates/storage/tests/`; MUST existir al menos un archivo que use `storage::testing` y verifique el harness en CI (`cargo test --workspace`).

#### Scenario: test_db con esquema de migraciones (happy)

- **WHEN** se llama `storage::testing::test_db().await`
- **THEN** la operación termina sin error
- **AND** `list_migrations()` incluye `version` 0 con el checksum de `0000_migracion.surql`
- **AND** la sesión usa namespace `vectra` y base `comercio`

#### Scenario: test_db_rocks en directorio temporal (happy)

- **WHEN** se llama `storage::testing::test_db_rocks().await`
- **THEN** la operación termina sin error
- **AND** `list_migrations()` incluye `version` 0
- **AND** el directorio del motor existe mientras el valor retornado está vivo

#### Scenario: Instancias en memoria aisladas (happy)

- **WHEN** en una instancia `test_db()` se define una tabla de prueba y se inserta un registro
- **AND** se abre una segunda instancia `test_db()` independiente
- **THEN** la primera instancia tiene un registro en esa tabla y la segunda tiene conteo 0

#### Scenario: Harness sin tablas de negocio (unhappy)

- **WHEN** tras `test_db()` se consulta el conteo de registros en una tabla de negocio no migrada (p. ej. `categoria`) con `Db::record_count`
- **THEN** el conteo es 0

#### Scenario: Db::open sigue rechazando archivo (unhappy)

- **WHEN** se llama `Db::open` con una ruta que es un archivo regular
- **THEN** la operación falla con `OpenError` tipado

### Requirement: Migración outbox, sync_cursor y outbox_fallo

El crate `storage` MUST incluir la migración embebida `0001_outbox.surql`, aplicada por `Db::migrate()` después de `0000_migracion.surql`. El SQL MUST definir las tablas `SCHEMAFULL` `outbox`, `sync_cursor` y `outbox_fallo` conforme a [4.8](../../../docs/4-modelo.md#48-sincronización-outbox-y-cursores), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../docs/4-modelo.md#410-operación-del-esquema)).

`outbox` MUST declarar: `entidad` (`string` con `ASSERT` en `producto`, `categoria`, `marca`, `atributo_def`, `sinonimo`, `nota_tecnica`, `imagen_producto`); `registro` (`record`); `operacion` (`string` con `ASSERT` en `upsert`, `delete`); `motivo` (`option<string>`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`).

`sync_cursor` MUST declarar: `ultimo_evento` (`option<record<outbox>>`); `actualizado_en` (`datetime` con `VALUE time::now()`). La migración MUST crear los registros `sync_cursor:meilisearch` y `sync_cursor:vectores` con `ultimo_evento` vacío si aún no existen.

`outbox_fallo` MUST declarar: `evento` (`record<outbox>`); `consumidor` (`string` con `ASSERT` en `meilisearch`, `vectores`); `intentos` (`int` con `DEFAULT 1`); `ultimo_error` (`string`); `estado` (`string` con `ASSERT` en `reintentando`, `fallido`); `actualizado_en` (`datetime` con `VALUE time::now()`); índice único `uq_outbox_fallo` sobre `(evento, consumidor)`.

MUST NOT habilitar *change feed* en estas tablas (tablas técnicas; el *change feed* aplica a maestras y ledger). MUST NOT crear `OutboxEvent`, `storage::Tx`, worker ni puertos de dominio (T-03-02 / Fase 3).

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
