# vortex-storage Specification

## Purpose

Abre SurrealDB embebido con RocksDB en el crate `storage`, selecciona el namespace `vectra` y la base `comercio`, y aplica migraciones `.surql` embebidas con registro y checksum SHA-256, con errores tipados de apertura y de drift.

Ticket padre: T-00-08, T-00-09. HU: HU-00-02.

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
