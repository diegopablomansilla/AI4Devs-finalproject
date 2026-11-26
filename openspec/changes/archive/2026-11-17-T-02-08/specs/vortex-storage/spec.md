# Spec Delta

Ticket padre: T-02-08. Change: T-02-08. HU: HU-02-02.

## ADDED Requirements

### Requirement: Migración marca

El crate `storage` MUST incluir la migración embebida `0003_marca.surql`, aplicada por `Db::migrate()` después de `0002_categoria.surql`. El SQL MUST definir la tabla `SCHEMAFULL` `marca` conforme a [4.4.2](../../../../../docs/4-modelo.md#442-marca), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

`marca` MUST declarar: `nombre` (`string` con `ASSERT string::len($value) > 0`); `slug` (`string`); `activo` (`bool` con `DEFAULT true`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`); `actualizado_en` (`datetime` con `VALUE time::now()`). MUST habilitar `CHANGEFEED 3d` (tabla maestra). MUST definir el índice único `uq_marca_slug` sobre `slug`.

MUST NOT sembrar filas de catálogo. MUST NOT implementar HTTP ni casos de uso en `catalog` (T-02-09). Tras `storage::testing::test_db()`, `record_count("marca")` MUST ser 0.

#### Scenario: Migración aplica esquema marca (happy)

- **WHEN** se abre una base vacía y se llama `Db::migrate` (o `storage::testing::test_db`)
- **THEN** `list_migrations()` incluye `version` 3 con checksum SHA-256 de `0003_marca.surql`
- **AND** `record_count("marca")` es 0

#### Scenario: Re-arranque no duplica el registro de migración (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro de `migracion` para `version` 3

#### Scenario: Índice único slug rechaza duplicado (unhappy)

- **WHEN** tras migrar existe una marca con un `slug` y se intenta insertar otra con el mismo `slug`
- **THEN** la segunda inserción falla
- **AND** no queda una segunda fila con ese `slug`

#### Scenario: nombre vacío rechazado (unhappy)

- **WHEN** tras migrar se intenta insertar una `marca` con `nombre` de longitud 0
- **THEN** la operación falla
- **AND** `marca` permanece sin esa fila

### Requirement: Adaptador BrandRepo

El crate `storage` MUST implementar el puerto `BrandRepo` (`create`, `get`, `list_all`, `update`, `set_activo`) contra la tabla `marca` ya migrada usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT reindexar productos (T-03-08). MUST NOT consultar `producto` ni rechazar baja por ocupación.

`create` MUST: generar el *slug* desde `nombre` cuando el input no aporta uno; persistir `activo` verdadero; emitir en la **misma** transacción un `OutboxEvent` con `entidad` `marca`, `operacion` `upsert` y `registro` igual al `id` de la marca creada. Una violación del índice único `slug` MUST fallar con `DomainError::Conflict` y `code` `duplicate_slug`, y MUST NOT dejar la segunda fila ni su evento de outbox. `nombre` vacío o inválido MUST ser `invalid_input` y MUST NOT insertar.

`get` MUST devolver la `Marca` persistida o `DomainError::NotFound`. `list_all` MUST devolver todas las filas de `marca` (incluidas inactivas).

`update` MUST cambiar `nombre` y el *slug* (explícito o regenerado desde `nombre`) de esa fila en una `Tx` con un `OutboxEvent` `marca`/`upsert`. Colisión de *slug* MUST ser `duplicate_slug`. `id` desconocido MUST ser `not_found`. MUST NOT tocar otras filas.

`set_activo` MUST actualizar solo el campo `activo` (y `actualizado_en`) de esa fila en una `Tx` con un `OutboxEvent` `marca`/`upsert`. MUST NOT hacer `DELETE` físico. `id` desconocido MUST ser `not_found`. Baja lógica de una marca MUST estar permitida aunque más adelante existan productos que la referencien (HU-02-02; este ticket no consulta `producto`).

#### Scenario: Alta de marca Fixser (happy, HU-02-02, Listo cuando)

- **WHEN** se crea una marca con `nombre` `"Fixser"` y sin *slug* explícito
- **THEN** la operación termina sin error
- **AND** `nombre` es `"Fixser"`
- **AND** `slug` es `"fixser"`
- **AND** `activo` es verdadero
- **AND** el `id` no contiene `:`

#### Scenario: get de marca existente (happy)

- **WHEN** se persiste una marca y se llama `get` con su `id`
- **THEN** la operación termina sin error
- **AND** `nombre`, `slug` e `id` coinciden con lo persistido

#### Scenario: list_all incluye activas e inactivas (happy)

- **WHEN** existen una marca activa y otra inactiva y se llama `list_all`
- **THEN** la operación termina sin error
- **AND** el resultado incluye ambas marcas

#### Scenario: create emite evento de outbox (happy)

- **WHEN** `create` persiste una marca
- **THEN** existe un evento en `outbox` con `entidad` `marca` y `operacion` `upsert`
- **AND** `registro` es el `id` de la marca (sin prefijo de tabla)

#### Scenario: Edición regenera slug (happy, Listo cuando)

- **WHEN** existe la marca `"Fixser"` y `update` cambia el `nombre` a `"Fischer"` sin *slug* explícito
- **THEN** la operación termina sin error
- **AND** `nombre` es `"Fischer"`
- **AND** `slug` es `"fischer"`
- **AND** existe un evento de outbox `marca`/`upsert` adicional para ese `id`

#### Scenario: Baja lógica persiste inactiva (happy, HU-02-02, Listo cuando)

- **WHEN** existe una marca activa y `set_activo` se llama con `false`
- **THEN** la operación termina sin error
- **AND** `activo` es falso
- **AND** la fila sigue existiendo
- **AND** existe un evento de outbox `marca`/`upsert` (no `delete`)

#### Scenario: slug duplicado en alta (unhappy)

- **WHEN** ya existe una marca con *slug* `"fixser"` y se intenta crear otra con el mismo *slug*
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_slug`
- **AND** no queda una segunda fila ni un segundo evento de outbox para esa alta

#### Scenario: slug duplicado en edición (unhappy)

- **WHEN** existen `"Fixser"` y `"Fischer"` y `update` intenta asignar a `"Fischer"` el *slug* de `"Fixser"`
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_slug`
- **AND** `"Fischer"` conserva su *slug* original

#### Scenario: get de id inexistente (unhappy)

- **WHEN** se llama `get` con un `id` que no existe en `marca`
- **THEN** la operación falla
- **AND** el error es `DomainError::NotFound`

#### Scenario: update de id inexistente (unhappy)

- **WHEN** se llama `update` con un `id` que no existe
- **THEN** la operación falla
- **AND** el error es `DomainError::NotFound`

#### Scenario: set_activo de id desconocido (unhappy)

- **WHEN** `set_activo` se llama con un `id` que no existe
- **THEN** la operación falla
- **AND** el error es `DomainError::NotFound`

#### Scenario: create con nombre vacío (unhappy)

- **WHEN** se llama `create` con `nombre` vacío o solo espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se inserta fila ni evento de outbox

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `migrate_applies_0003_marca` | `storage` (`tests/marca.rs`) | integración happy |
| `migrate_rerun_keeps_single_0003_record` | `storage` (`tests/marca.rs`) | integración happy |
| `marca_rejects_duplicate_slug` | `storage` (`tests/marca.rs`) | integración unhappy (esquema) |
| `marca_rejects_empty_nombre` | `storage` (`tests/marca.rs`) | integración unhappy (esquema) |
| `brand_repo_creates_fixser` | `storage` (`tests/marca.rs`) | integración happy (Listo cuando, HU-02-02) |
| `brand_repo_get_returns_persisted` | `storage` (`tests/marca.rs`) | integración happy |
| `brand_repo_list_all_includes_inactive` | `storage` (`tests/marca.rs`) | integración happy |
| `brand_repo_create_emits_outbox_upsert` | `storage` (`tests/marca.rs`) | integración happy |
| `brand_repo_update_renames_and_regenerates_slug` | `storage` (`tests/marca.rs`) | integración happy (Listo cuando) |
| `brand_repo_set_activo_persists_inactive` | `storage` (`tests/marca.rs`) | integración happy (Listo cuando, HU-02-02) |
| `brand_repo_rejects_duplicate_slug` | `storage` (`tests/marca.rs`) | integración unhappy |
| `brand_repo_update_duplicate_slug_is_duplicate_slug` | `storage` (`tests/marca.rs`) | integración unhappy |
| `brand_repo_get_unknown_id_is_not_found` | `storage` (`tests/marca.rs`) | integración unhappy |
| `brand_repo_update_unknown_id_is_not_found` | `storage` (`tests/marca.rs`) | integración unhappy |
| `brand_repo_set_activo_unknown_id_is_not_found` | `storage` (`tests/marca.rs`) | integración unhappy |
| `brand_repo_create_blank_nombre_is_invalid_input` | `storage` (`tests/marca.rs`) | integración unhappy |
| Ajuste `list_migrations` (incluye `version` 3) | `storage` (`tests/migrate.rs`, `tests/harness.rs`) | integración existente |
