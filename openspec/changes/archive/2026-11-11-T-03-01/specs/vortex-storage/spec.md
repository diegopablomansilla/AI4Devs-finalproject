# Spec Delta

Ticket padre: T-03-01. Change: T-03-01. HU: HU-03-01.

## ADDED Requirements

### Requirement: Migración outbox, sync_cursor y outbox_fallo

El crate `storage` MUST incluir la migración embebida `0001_outbox.surql`, aplicada por `Db::migrate()` después de `0000_migracion.surql`. El SQL MUST definir las tablas `SCHEMAFULL` `outbox`, `sync_cursor` y `outbox_fallo` conforme a [4.8](../../../../../docs/4-modelo.md#48-sincronización-outbox-y-cursores), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

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

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `migrate_applies_0001_outbox_and_seeds_cursors` | `storage` (`tests/outbox.rs`) | integración happy |
| `migrate_rerun_does_not_duplicate_cursors` | `storage` (`tests/outbox.rs`) | integración happy |
| `outbox_accepts_valid_entidad_and_operacion` | `storage` (`tests/outbox.rs`) | integración happy |
| `outbox_rejects_invalid_entidad` | `storage` (`tests/outbox.rs`) | integración unhappy |
| `outbox_rejects_invalid_operacion` | `storage` (`tests/outbox.rs`) | integración unhappy |
| `outbox_fallo_rejects_invalid_consumidor` | `storage` (`tests/outbox.rs`) | integración unhappy |
| Ajuste `list_migrations` (incluye `version` 1) | `storage` (`tests/migrate.rs`, `tests/harness.rs`) | integración existente |
