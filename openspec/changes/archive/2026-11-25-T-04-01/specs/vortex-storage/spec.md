# Spec Delta

Ticket padre: T-04-01. Change: T-04-01. HU: HU-04-01.

## ADDED Requirements

### Requirement: Migración relaciones

El crate `storage` MUST incluir la migración embebida `0007_relaciones.surql`, aplicada por `Db::migrate()` después de `0006_movimiento_stock.surql`. El SQL MUST definir las tablas `SCHEMAFULL` `similar_a`, `sustituible_por` y `complementa` conforme a [4.6.1](../../../../../docs/4-modelo.md#461-tablas-de-relación) (PM-05), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

Cada tabla MUST ser `TYPE RELATION IN producto OUT producto ENFORCED` y MUST declarar: `prioridad` (`int` con `DEFAULT 1` y `ASSERT $value >= 1`); `nota` (`option<string>`); `grupo_bidireccional` (`option<string>`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`); `actualizado_en` (`datetime` con `VALUE time::now()`). MUST habilitar `CHANGEFEED 3d`. MUST definir un índice único sobre `(in, out)`: `uq_similar_a`, `uq_sustituible_por` y `uq_complementa`. El contrato HTTP sigue usando `substitute`; el nombre de tabla MUST ser `sustituible_por`.

MUST NOT sembrar aristas. MUST NOT implementar entidad de dominio, `self_relation`, bidireccionalidad, puerto `RelationRepo`, consulta de recomendaciones, HTTP ni casos de uso (T-04-02 / T-04-03 / T-04-05 / T-04-06). MUST NOT rechazar `in = out` en el esquema. Tras `storage::testing::test_db()`, `record_count` de `similar_a`, `sustituible_por` y `complementa` MUST ser 0.

#### Scenario: Migración aplica esquema de relaciones (happy)

- **WHEN** se abre una base vacía y se llama `Db::migrate` (o `storage::testing::test_db`)
- **THEN** `list_migrations()` incluye `version` 7 con checksum SHA-256 de `0007_relaciones.surql`
- **AND** `record_count("similar_a")`, `record_count("sustituible_por")` y `record_count("complementa")` son 0

#### Scenario: Re-arranque no duplica el registro de migración (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro de `migracion` para `version` 7

#### Scenario: RELATE entre dos productos existentes (happy)

- **WHEN** tras migrar existen productos `A` y `B` y se ejecuta `RELATE` de `A` hacia `B` en `similar_a`, `sustituible_por` y `complementa`
- **THEN** las tres operaciones terminan sin error
- **AND** cada tabla queda con `record_count` 1
- **AND** `prioridad` queda en `1` cuando no se envía

#### Scenario: mismo par en tablas distintas (happy)

- **WHEN** tras migrar existe `RELATE A -> similar_a -> B` y se ejecuta `RELATE A -> sustituible_por -> B`
- **THEN** ambas aristas existen
- **AND** el índice único de cada tabla no las trata como duplicado

#### Scenario: ENFORCED rechaza destino inexistente (unhappy, Listo cuando)

- **WHEN** tras migrar existe un producto `A` y se ejecuta `RELATE` de `A` hacia un `producto` que no existe en `sustituible_por`
- **THEN** la operación falla
- **AND** `sustituible_por` permanece sin esa arista

#### Scenario: índice único (in, out) rechaza duplicado (unhappy, Listo cuando)

- **WHEN** tras migrar existe `RELATE A -> sustituible_por -> B` y se intenta el mismo `RELATE` otra vez
- **THEN** la segunda operación falla
- **AND** `record_count("sustituible_por")` sigue siendo 1

#### Scenario: prioridad menor que 1 rechazada (unhappy)

- **WHEN** tras migrar existen productos `A` y `B` y se ejecuta `RELATE A -> similar_a -> B` con `prioridad` `0`
- **THEN** la operación falla
- **AND** `similar_a` permanece sin esa arista

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `migrate_applies_0007_relaciones` | `storage` (`tests/relaciones.rs`) | integración happy |
| `migrate_rerun_keeps_single_0007_record` | `storage` (`tests/relaciones.rs`) | integración happy |
| `relacion_relates_existing_products` | `storage` (`tests/relaciones.rs`) | integración happy |
| `relacion_allows_same_pair_on_distinct_tables` | `storage` (`tests/relaciones.rs`) | integración happy |
| `relacion_enforced_rejects_missing_target` | `storage` (`tests/relaciones.rs`) | integración unhappy (Listo cuando) |
| `relacion_unique_rejects_duplicate_edge` | `storage` (`tests/relaciones.rs`) | integración unhappy (Listo cuando) |
| `relacion_rejects_prioridad_below_one` | `storage` (`tests/relaciones.rs`) | integración unhappy |
| Ajuste `list_migrations` (incluye `version` 7) | `storage` (`tests/migrate.rs`, `tests/harness.rs`) | integración existente |
