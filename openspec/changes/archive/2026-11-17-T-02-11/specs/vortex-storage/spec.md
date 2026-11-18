# Spec Delta

Ticket padre: T-02-11. Change: T-02-11. HU: HU-02-03.

## ADDED Requirements

### Requirement: Migración atributo_def

El crate `storage` MUST incluir la migración embebida `0004_atributo_def.surql`, aplicada por `Db::migrate()` después de `0003_marca.surql`. El SQL MUST definir la tabla `SCHEMAFULL` `atributo_def` conforme a [4.4.3](../../../../../docs/4-modelo.md#443-atributo_def--definición-de-atributos-técnicos), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../../../docs/4-modelo.md#410-operación-del-esquema)).

`atributo_def` MUST declarar: `clave` (`string`); `etiqueta` (`string`); `tipo` (`string` con `ASSERT $value IN ['texto', 'numero', 'booleano', 'enum']`); `unidad` (`option<string>`); `opciones` (`option<array<string>>`); `facetable` (`bool` con `DEFAULT false`); `buscable` (`bool` con `DEFAULT true`); `orden` (`int` con `DEFAULT 0`); `activo` (`bool` con `DEFAULT true`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`); `actualizado_en` (`datetime` con `VALUE time::now()`). MUST habilitar `CHANGEFEED 3d` (tabla maestra). MUST definir el índice único `uq_atributo_clave` sobre `clave`.

MUST NOT sembrar filas de catálogo. MUST NOT implementar HTTP ni casos de uso en `catalog` (T-02-12). Tras `storage::testing::test_db()`, `record_count("atributo_def")` MUST ser 0.

#### Scenario: Migración aplica esquema atributo_def (happy)

- **WHEN** se abre una base vacía y se llama `Db::migrate` (o `storage::testing::test_db`)
- **THEN** `list_migrations()` incluye `version` 4 con checksum SHA-256 de `0004_atributo_def.surql`
- **AND** `record_count("atributo_def")` es 0

#### Scenario: Re-arranque no duplica el registro de migración (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro de `migracion` para `version` 4

#### Scenario: Índice único clave rechaza duplicado (unhappy)

- **WHEN** tras migrar existe un `atributo_def` con una `clave` y se intenta insertar otro con la misma `clave`
- **THEN** la segunda inserción falla
- **AND** no queda una segunda fila con esa `clave`

#### Scenario: tipo fuera del enum rechazado (unhappy)

- **WHEN** tras migrar se intenta insertar un `atributo_def` con `tipo` `"text"` (u otro valor fuera de `texto`/`numero`/`booleano`/`enum`)
- **THEN** la operación falla
- **AND** `atributo_def` permanece sin esa fila

### Requirement: Adaptador AttributeDefRepo

El crate `storage` MUST implementar el puerto `AttributeDefRepo` (`create`, `get`, `list_all`, `update`, `set_activo`) contra la tabla `atributo_def` ya migrada usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT reindexar productos ni actualizar `filterableAttributes` (T-03-08). MUST NOT consultar `producto` ni rechazar cambios por `attribute_in_use` (T-02-23).

`create` MUST: persistir los campos de `NewAtributoDef`; usar `activo` verdadero; emitir en la **misma** transacción un `OutboxEvent` con `entidad` `atributo_def`, `operacion` `upsert` y `registro` igual al `id` de la definición creada. Una violación del índice único `clave` MUST fallar con `DomainError::Conflict` y `code` `conflict` (equivalente HTTP `409` en T-02-12), y MUST NOT dejar la segunda fila ni su evento de outbox. `tipo` `enum` sin opciones MUST ser `invalid_input` y MUST NOT insertar. `clave` inválida o `etiqueta` vacía MUST ser `invalid_input`.

`get` MUST devolver la `AtributoDef` persistida o `DomainError::NotFound`. `list_all` MUST devolver todas las filas de `atributo_def` (incluidas inactivas), ordenadas por `orden` ascendente y `clave` como desempate (OpenAPI: listado ordenado por `order`).

`update` MUST cambiar los campos editables de esa fila en una `Tx` con un `OutboxEvent` `atributo_def`/`upsert`. Colisión de `clave` MUST ser `conflict`. `id` desconocido MUST ser `not_found`. MUST NOT aplicar `attribute_in_use`. MUST NOT tocar otras filas.

`set_activo` MUST actualizar solo el campo `activo` (y `actualizado_en`) de esa fila en una `Tx` con un `OutboxEvent` `atributo_def`/`upsert`. MUST NOT hacer `DELETE` físico. `id` desconocido MUST ser `not_found`.

#### Scenario: Alta de atributo grado (happy, HU-02-03)

- **WHEN** se crea una definición con `clave` `"grado"`, `etiqueta` `"Grado"`, `tipo` `enum`, `opciones` `["2", "5", "8"]` y `facetable` verdadero
- **THEN** la operación termina sin error
- **AND** `clave` es `"grado"`
- **AND** `tipo` es `enum`
- **AND** `opciones` son `["2", "5", "8"]`
- **AND** `facetable` es verdadero
- **AND** `activo` es verdadero
- **AND** el `id` no contiene `:`

#### Scenario: get de definición existente (happy)

- **WHEN** se persiste una definición y se llama `get` con su `id`
- **THEN** la operación termina sin error
- **AND** `clave`, `etiqueta`, `tipo` e `id` coinciden con lo persistido

#### Scenario: list_all ordena por orden e incluye inactivas (happy)

- **WHEN** existen definiciones con `orden` 10 y 0, una de ellas inactiva, y se llama `list_all`
- **THEN** la operación termina sin error
- **AND** el resultado incluye ambas
- **AND** la de `orden` 0 aparece antes que la de `orden` 10

#### Scenario: create emite evento de outbox (happy)

- **WHEN** `create` persiste una definición
- **THEN** existe un evento en `outbox` con `entidad` `atributo_def` y `operacion` `upsert`
- **AND** `registro` es el `id` de la definición (sin prefijo de tabla)

#### Scenario: Edición cambia etiqueta (happy)

- **WHEN** existe la definición `"grado"` y `update` cambia `etiqueta` a `"Grado SAE"`
- **THEN** la operación termina sin error
- **AND** `etiqueta` es `"Grado SAE"`
- **AND** `clave` sigue siendo `"grado"`
- **AND** existe un evento de outbox `atributo_def`/`upsert` adicional para ese `id`

#### Scenario: Baja lógica persiste inactiva (happy)

- **WHEN** existe una definición activa y `set_activo` se llama con `false`
- **THEN** la operación termina sin error
- **AND** `activo` es falso
- **AND** la fila sigue existiendo
- **AND** existe un evento de outbox `atributo_def`/`upsert` (no `delete`)

#### Scenario: clave duplicada en alta (unhappy, Listo cuando → 409)

- **WHEN** ya existe una definición con `clave` `"grado"` y se intenta crear otra con la misma `clave`
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `conflict`
- **AND** no queda una segunda fila ni un segundo evento de outbox para esa alta

#### Scenario: clave duplicada en edición (unhappy)

- **WHEN** existen `"grado"` y `"medida"` y `update` intenta asignar a `"medida"` la `clave` `"grado"`
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `conflict`
- **AND** `"medida"` conserva su `clave` original

#### Scenario: enum sin opciones en alta (unhappy, HU-02-03, Listo cuando)

- **WHEN** se llama `create` con `tipo` `enum` y `opciones` ausentes o vacías
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se inserta fila ni evento de outbox

#### Scenario: get de id inexistente (unhappy)

- **WHEN** se llama `get` con un `id` que no existe en `atributo_def`
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

#### Scenario: create con clave inválida (unhappy)

- **WHEN** se llama `create` con `clave` `"Grado"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se inserta fila ni evento de outbox

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `migrate_applies_0004_atributo_def` | `storage` (`tests/atributo_def.rs`) | integración happy |
| `migrate_rerun_keeps_single_0004_record` | `storage` (`tests/atributo_def.rs`) | integración happy |
| `atributo_def_rejects_duplicate_clave` | `storage` (`tests/atributo_def.rs`) | integración unhappy (esquema) |
| `atributo_def_rejects_invalid_tipo` | `storage` (`tests/atributo_def.rs`) | integración unhappy (esquema) |
| `attribute_def_repo_creates_grado` | `storage` (`tests/atributo_def.rs`) | integración happy (HU-02-03) |
| `attribute_def_repo_get_returns_persisted` | `storage` (`tests/atributo_def.rs`) | integración happy |
| `attribute_def_repo_list_all_ordered_by_orden` | `storage` (`tests/atributo_def.rs`) | integración happy |
| `attribute_def_repo_create_emits_outbox_upsert` | `storage` (`tests/atributo_def.rs`) | integración happy |
| `attribute_def_repo_update_changes_etiqueta` | `storage` (`tests/atributo_def.rs`) | integración happy |
| `attribute_def_repo_set_activo_persists_inactive` | `storage` (`tests/atributo_def.rs`) | integración happy |
| `attribute_def_repo_rejects_duplicate_clave` | `storage` (`tests/atributo_def.rs`) | integración unhappy (Listo cuando → 409) |
| `attribute_def_repo_update_duplicate_clave_is_conflict` | `storage` (`tests/atributo_def.rs`) | integración unhappy |
| `attribute_def_repo_create_enum_without_options_is_invalid_input` | `storage` (`tests/atributo_def.rs`) | integración unhappy (Listo cuando) |
| `attribute_def_repo_get_unknown_id_is_not_found` | `storage` (`tests/atributo_def.rs`) | integración unhappy |
| `attribute_def_repo_update_unknown_id_is_not_found` | `storage` (`tests/atributo_def.rs`) | integración unhappy |
| `attribute_def_repo_set_activo_unknown_id_is_not_found` | `storage` (`tests/atributo_def.rs`) | integración unhappy |
| `attribute_def_repo_create_invalid_clave_is_invalid_input` | `storage` (`tests/atributo_def.rs`) | integración unhappy |
| Ajuste `list_migrations` (incluye `version` 4) | `storage` (`tests/migrate.rs`, `tests/harness.rs`) | integración existente |
