# Spec Delta

Ticket padre: T-02-01. Change: T-02-01. HU: HU-02-01.

## ADDED Requirements

### Requirement: Migración categoria

El crate `storage` MUST incluir la migración embebida `0002_categoria.surql`, aplicada por `Db::migrate()` después de `0001_outbox.surql`. El SQL MUST definir la tabla `SCHEMAFULL` `categoria` conforme a [4.4.1](../../../../../docs/4-modelo.md#441-categoria--árbol), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

`categoria` MUST declarar: `nombre` (`string` con `ASSERT string::len($value) > 0`); `slug` (`string`); `padre` (`option<record<categoria>>`); `ancestros` (`array<record<categoria>>` con `DEFAULT []`); `ruta` (`array<string>`); `activo` (`bool` con `DEFAULT true`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`); `actualizado_en` (`datetime` con `VALUE time::now()`). MUST habilitar `CHANGEFEED 3d` (tabla maestra). MUST definir el índice único `uq_categoria_slug` sobre `(padre, slug)` y el índice `idx_categoria_ancestros` sobre `ancestros`.

MUST NOT sembrar filas de catálogo. MUST NOT implementar entidad de dominio, *slug*, profundidad, ciclos ni `CategoryRepo` (T-02-02 / T-02-03). Tras `storage::testing::test_db()`, `record_count("categoria")` MUST ser 0.

#### Scenario: Migración aplica esquema categoria (happy)

- **WHEN** se abre una base vacía y se llama `Db::migrate` (o `storage::testing::test_db`)
- **THEN** `list_migrations()` incluye `version` 2 con checksum SHA-256 de `0002_categoria.surql`
- **AND** `record_count("categoria")` es 0

#### Scenario: Re-arranque no duplica el registro de migración (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro de `migracion` para `version` 2

#### Scenario: Mismo slug bajo padres distintos (happy)

- **WHEN** tras migrar existen dos categorías padre distintas y se insertan dos hijas con el mismo `slug` y `padre` distinto
- **THEN** ambas inserciones terminan sin error
- **AND** el conteo de `categoria` incluye esas dos hijas

#### Scenario: Índice único (padre, slug) rechaza duplicado (unhappy)

- **WHEN** tras migrar existe una categoría con un `padre` y un `slug` y se intenta insertar otra con el mismo par `(padre, slug)`
- **THEN** la segunda inserción falla
- **AND** no queda una segunda fila con ese par

#### Scenario: nombre vacío rechazado (unhappy)

- **WHEN** tras migrar se intenta insertar una `categoria` con `nombre` de longitud 0
- **THEN** la operación falla
- **AND** `categoria` permanece sin esa fila

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `migrate_applies_0002_categoria` | `storage` (`tests/categoria.rs`) | integración happy |
| `migrate_rerun_keeps_single_0002_record` | `storage` (`tests/categoria.rs`) | integración happy |
| `categoria_allows_same_slug_under_different_padres` | `storage` (`tests/categoria.rs`) | integración happy |
| `categoria_rejects_duplicate_padre_slug` | `storage` (`tests/categoria.rs`) | integración unhappy (Listo cuando) |
| `categoria_rejects_empty_nombre` | `storage` (`tests/categoria.rs`) | integración unhappy |
| Ajuste `list_migrations` (incluye `version` 2) | `storage` (`tests/migrate.rs`, `tests/harness.rs`) | integración existente |
