# Spec Delta

Ticket padre: T-02-14. Change: T-02-14. HU: HU-02-04.

## ADDED Requirements

### Requirement: Migración producto

El crate `storage` MUST incluir la migración embebida `0005_producto.surql`, aplicada por `Db::migrate()` después de `0004_atributo_def.surql`. El SQL MUST definir la tabla `SCHEMAFULL` `producto` conforme a [4.4.4](../../../../../docs/4-modelo.md#444-producto), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

`producto` MUST declarar: `sku` (`string` con `ASSERT string::len($value) > 0`); `nombre` (`string` con `ASSERT string::len($value) > 0`); `tipo` (`string`); `descripcion` (`option<string>`); `marca` (`option<record<marca>>`); `categoria` (`record<categoria>`); `atributos` (`array<object>` con `DEFAULT []`); `atributos[*].def` (`record<atributo_def>`); `atributos[*].valor` (`string | number | bool`); `codigos` (`array<object>` con `DEFAULT []`); `codigos[*].tipo` (`string` con `ASSERT $value IN ['fabricante', 'oem', 'barras']`); `codigos[*].valor` (`string`); `unidad_medida` (`string` con `DEFAULT 'unidad'` y `ASSERT $value IN ['unidad', 'par', 'caja', 'metro', 'kilogramo', 'litro']`); `precio` (`decimal` con `ASSERT $value >= 0`); `ubicacion` (`option<object>`); `ubicacion.pasillo` / `ubicacion.estante` / `ubicacion.posicion` (`option<string>`); `stock` (`decimal` con `DEFAULT 0` y `ASSERT $value >= 0`); `activo` (`bool` con `DEFAULT true`); `version` (`int` con `DEFAULT 1`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`); `actualizado_en` (`datetime` con `VALUE time::now()`). MUST habilitar `CHANGEFEED 3d` (tabla maestra).

MUST definir el índice único `uq_producto_sku` sobre `sku`, el índice `idx_producto_categoria` sobre `categoria`, el índice `idx_producto_marca` sobre `marca`, el analizador `an_respaldo` (`TOKENIZERS blank, class FILTERS lowercase, ascii, edgengram(2, 15)`), el índice de búsqueda `ft_producto_nombre` sobre `nombre` (`SEARCH ANALYZER an_respaldo BM25`) y el índice no único `idx_producto_codigos` sobre `codigos[*].valor` (hallazgo H-05; se documenta en 4.4.4). `idx_producto_codigos` MUST NOT ser `UNIQUE` (los códigos OEM se pueden compartir entre productos).

MUST NOT sembrar filas de catálogo. MUST NOT implementar entidad de dominio, value objects, puerto `ProductRepo`, ledger, HTTP ni casos de uso (T-02-16 / T-02-18 / T-02-21). Tras `storage::testing::test_db()`, `record_count("producto")` MUST ser 0.

#### Scenario: Migración aplica esquema producto (happy)

- **WHEN** se abre una base vacía y se llama `Db::migrate` (o `storage::testing::test_db`)
- **THEN** `list_migrations()` incluye `version` 5 con checksum SHA-256 de `0005_producto.surql`
- **AND** `record_count("producto")` es 0

#### Scenario: Re-arranque no duplica el registro de migración (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro de `migracion` para `version` 5

#### Scenario: Alta mínima de producto (happy)

- **WHEN** tras migrar existe una `categoria` y se inserta un `producto` con `sku` `"PER-M8X40-ZN"`, `nombre` no vacío, `tipo` `"Perno"`, esa `categoria` y `precio` `120.50`
- **THEN** la inserción termina sin error
- **AND** `record_count("producto")` es 1
- **AND** `stock` queda en `0`, `activo` en verdadero y `version` en `1`

#### Scenario: precio cero aceptado (happy)

- **WHEN** tras migrar existe una `categoria` y se inserta un `producto` con `precio` `0`
- **THEN** la inserción termina sin error
- **AND** queda una fila con ese `precio`

#### Scenario: mismo código OEM en dos productos (happy, H-05)

- **WHEN** tras migrar existen dos productos con `sku` distintos y ambos tienen `codigos` con `{ tipo: "oem", valor: "HX-0840Z" }`
- **THEN** ambas inserciones terminan sin error
- **AND** `record_count("producto")` es 2

#### Scenario: Índice único sku rechaza duplicado (unhappy, Listo cuando)

- **WHEN** tras migrar existe un `producto` con `sku` `"PER-M8X40-ZN"` y se intenta insertar otro con el mismo `sku`
- **THEN** la segunda inserción falla
- **AND** no queda una segunda fila con ese `sku`

#### Scenario: precio negativo rechazado (unhappy, Listo cuando)

- **WHEN** tras migrar se intenta insertar un `producto` con `precio` menor que `0`
- **THEN** la operación falla
- **AND** `producto` permanece sin esa fila

#### Scenario: tipo de código fuera del enum rechazado (unhappy, Listo cuando)

- **WHEN** tras migrar se intenta insertar un `producto` con `codigos` que incluye `{ tipo: "ean", valor: "123" }`
- **THEN** la operación falla
- **AND** `producto` permanece sin esa fila

#### Scenario: sku vacío rechazado (unhappy)

- **WHEN** tras migrar se intenta insertar un `producto` con `sku` de longitud 0
- **THEN** la operación falla
- **AND** `producto` permanece sin esa fila

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `migrate_applies_0005_producto` | `storage` (`tests/producto.rs`) | integración happy |
| `migrate_rerun_keeps_single_0005_record` | `storage` (`tests/producto.rs`) | integración happy |
| `producto_creates_minimal_sku` | `storage` (`tests/producto.rs`) | integración happy |
| `producto_accepts_zero_precio` | `storage` (`tests/producto.rs`) | integración happy |
| `producto_allows_duplicate_oem_code` | `storage` (`tests/producto.rs`) | integración happy (H-05) |
| `producto_rejects_duplicate_sku` | `storage` (`tests/producto.rs`) | integración unhappy (Listo cuando) |
| `producto_rejects_negative_precio` | `storage` (`tests/producto.rs`) | integración unhappy (Listo cuando) |
| `producto_rejects_invalid_codigo_tipo` | `storage` (`tests/producto.rs`) | integración unhappy (Listo cuando) |
| `producto_rejects_empty_sku` | `storage` (`tests/producto.rs`) | integración unhappy |
| Ajuste `list_migrations` (incluye `version` 5) | `storage` (`tests/migrate.rs`, `tests/harness.rs`) | integración existente |
