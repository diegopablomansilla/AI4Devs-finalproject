# Spec Delta

Ticket padre: T-00-08. Change: T-00-08. HU: HU-00-02.

## Purpose

Abre SurrealDB embebido con RocksDB en el crate `storage` y deja seleccionados el namespace `vectra` y la base `comercio`, con errores de apertura tipados que incluyen el motivo, para que T-00-09 pueda migrar sobre una fuente de verdad ya abierta.

## Tests previstos

Crate `storage`, tests de integración `vortex-core/crates/storage/tests/open.rs` (runtime Tokio):

| Test | Camino | Escenario |
|---|---|---|
| `open_empty_dir_selects_ns_and_db` | happy | Primera apertura en directorio temporal |
| `reopen_same_dir_keeps_ns_and_db` | happy | Reapertura del mismo directorio |
| `open_rejects_corrupt_directory` | unhappy | Directorio corrupto |
| `open_rejects_unwritable_directory` | unhappy | Sin permisos de escritura (Unix) |

Harness `scripts/check-workspace.sh` (vía `scripts/fixtures/workspace/run-script-tests.sh`):

| Caso | Camino | Escenario (cargo-workspace) |
|---|---|---|
| `storage` con `surrealdb` + `thiserror` | happy | storage declara la allowlist |
| `storage` con `axum` | unhappy | storage declara una dependencia fuera de la allowlist |
| `api` con `[dependencies]` | unhappy | Un lib distinto de domain declara dependencias |

## ADDED Requirements

### Requirement: Db::open abre RocksDB embebido y selecciona NS/DB

El crate `storage` MUST exponer `storage::Db::open(path)` asíncrono. `path` MUST ser el directorio del motor (convención de instalación: `<paths.datos>/db`; este change MUST NOT leer `Config`). El motor MUST ser SurrealDB embebido con RocksDB (PM-03). MUST NOT usarse el motor en memoria en esta API. Tras una apertura exitosa, la sesión MUST usar el namespace `vectra` y la base `comercio` ([4.3](../../../../docs/4-modelo.md#43-convenciones)). Si el directorio no existe o está vacío, la operación MUST crear lo necesario para esa sesión (directorio, namespace y base) y MUST NOT aplicar migraciones `.surql` ni crear la tabla `migracion` (T-00-09). La versión de `surrealdb` MUST ser la pinneada en `[workspace.dependencies]` (`=2.7.0`).

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
