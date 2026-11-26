# Spec Delta

Ticket padre: T-09-02. Change: T-09-02. HU: HU-09-01.

## ADDED Requirements

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
