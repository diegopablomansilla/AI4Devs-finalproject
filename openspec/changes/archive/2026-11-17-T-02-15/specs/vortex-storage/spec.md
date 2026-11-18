# Spec Delta

Ticket padre: T-02-15. Change: T-02-15. HU: HU-02-07.

## ADDED Requirements

### Requirement: Migración movimiento_stock

El crate `storage` MUST incluir la migración embebida `0006_movimiento_stock.surql`, aplicada por `Db::migrate()` después de `0005_producto.surql`. El SQL MUST definir la tabla `SCHEMAFULL` `movimiento_stock` conforme a [4.5.1](../../../../../docs/4-modelo.md#451-movimiento_stock), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

`movimiento_stock` MUST declarar, todos `READONLY`: `producto` (`record<producto>`); `tipo` (`string` con `ASSERT $value IN ['inicial', 'ingreso', 'ajuste', 'venta']`); `cantidad` (`decimal` con `ASSERT $value != 0`); `stock_resultante` (`decimal` con `ASSERT $value >= 0`); `motivo` (`option<string>`); `venta` (`option<record>` genérico, **sin** `record<venta>` hasta T-08-01); `terminal` (`option<string>`); `creado_en` (`datetime` con `DEFAULT time::now()`). MUST habilitar `CHANGEFEED 3d` (ledger). MUST definir el índice `idx_mov_producto` sobre `(producto, creado_en)`. `idx_mov_producto` MUST NOT ser `UNIQUE`.

MUST NOT sembrar filas de ledger. MUST NOT implementar entidad de dominio, reglas de signo por `tipo`, `motivo` obligatorio en `ajuste`, puerto `StockLedger`, HTTP ni casos de uso (T-02-17 / T-02-24 / T-02-25). Tras `storage::testing::test_db()`, `record_count("movimiento_stock")` MUST ser 0.

#### Scenario: Migración aplica esquema movimiento_stock (happy)

- **WHEN** se abre una base vacía y se llama `Db::migrate` (o `storage::testing::test_db`)
- **THEN** `list_migrations()` incluye `version` 6 con checksum SHA-256 de `0006_movimiento_stock.surql`
- **AND** `record_count("movimiento_stock")` es 0

#### Scenario: Re-arranque no duplica el registro de migración (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro de `migracion` para `version` 6

#### Scenario: Alta mínima de movimiento ingreso (happy)

- **WHEN** tras migrar existe un `producto` y se inserta un `movimiento_stock` con `tipo` `"ingreso"`, `cantidad` `20`, `stock_resultante` `20` y ese `producto`
- **THEN** la inserción termina sin error
- **AND** `record_count("movimiento_stock")` es 1
- **AND** `motivo`, `venta` y `terminal` quedan ausentes

#### Scenario: UPDATE sobre un movimiento falla (unhappy, Listo cuando)

- **WHEN** tras migrar existe un `movimiento_stock` y se intenta `UPDATE` de un campo (p. ej. `motivo`)
- **THEN** la operación falla
- **AND** el movimiento conserva los valores originales

#### Scenario: tipo fuera del enum rechazado (unhappy)

- **WHEN** tras migrar se intenta insertar un `movimiento_stock` con `tipo` `"transferencia"`
- **THEN** la operación falla
- **AND** `movimiento_stock` permanece sin esa fila

#### Scenario: cantidad cero rechazada (unhappy)

- **WHEN** tras migrar se intenta insertar un `movimiento_stock` con `cantidad` `0`
- **THEN** la operación falla
- **AND** `movimiento_stock` permanece sin esa fila

#### Scenario: stock_resultante negativo rechazado (unhappy)

- **WHEN** tras migrar se intenta insertar un `movimiento_stock` con `stock_resultante` menor que `0`
- **THEN** la operación falla
- **AND** `movimiento_stock` permanece sin esa fila

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `migrate_applies_0006_movimiento_stock` | `storage` (`tests/movimiento_stock.rs`) | integración happy |
| `migrate_rerun_keeps_single_0006_record` | `storage` (`tests/movimiento_stock.rs`) | integración happy |
| `movimiento_stock_creates_ingreso` | `storage` (`tests/movimiento_stock.rs`) | integración happy |
| `movimiento_stock_rejects_update` | `storage` (`tests/movimiento_stock.rs`) | integración unhappy (Listo cuando) |
| `movimiento_stock_rejects_invalid_tipo` | `storage` (`tests/movimiento_stock.rs`) | integración unhappy |
| `movimiento_stock_rejects_zero_cantidad` | `storage` (`tests/movimiento_stock.rs`) | integración unhappy |
| `movimiento_stock_rejects_negative_stock_resultante` | `storage` (`tests/movimiento_stock.rs`) | integración unhappy |
| Ajuste `list_migrations` (incluye `version` 6) | `storage` (`tests/migrate.rs`, `tests/harness.rs`) | integración existente |
