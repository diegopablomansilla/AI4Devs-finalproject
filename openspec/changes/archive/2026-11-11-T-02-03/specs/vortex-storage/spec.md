# Spec Delta

Ticket padre: T-02-03. Change: T-02-03. HU: HU-02-01.

## ADDED Requirements

### Requirement: Adaptador CategoryRepo

El crate `storage` MUST implementar el puerto `CategoryRepo` (`create`, `get`, `list_all`) contra la tabla `categoria` ya migrada (T-02-01) usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla. Los errores MUST ser `DomainError`. MUST NOT declarar ni implementar `update` (T-02-04). MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT reindexar productos (T-03-08).

`create` MUST: generar el *slug* desde `nombre` cuando el input no aporta uno; calcular `ancestros` y `ruta` desde el padre persistido (raíz: `padre` ausente, `ancestros` vacío, `ruta = [nombre]`; hija: `ancestros` = ancestros del padre + `id` del padre, `ruta` = ruta del padre + `nombre`); persistir `activo` verdadero; emitir en la **misma** transacción un `OutboxEvent` con `entidad` `categoria`, `operacion` `upsert` y `registro` igual al `id` de la categoría creada. Si el padre indicado no existe, MUST fallar con `DomainError::NotFound` y MUST NOT insertar fila ni evento. Si la profundidad resultante supera 4, MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT insertar. Una violación del índice único `(padre, slug)` (incluidas dos raíces con el mismo *slug*) MUST fallar con `DomainError::Conflict` y `code` `duplicate_slug`, y MUST NOT dejar la segunda fila ni su evento de outbox.

`get` MUST devolver la `Categoria` persistida o `DomainError::NotFound`. `list_all` MUST devolver todas las filas de `categoria` (incluidas inactivas).

#### Scenario: Alta de raíz (happy)

- **WHEN** se crea una categoría con `nombre` `"Fijaciones"` y sin padre
- **THEN** la operación termina sin error
- **AND** `ruta` es `["Fijaciones"]`
- **AND** `ancestros` está vacío
- **AND** `padre` está ausente
- **AND** `slug` es `"fijaciones"`
- **AND** el `id` no contiene `:`

#### Scenario: Alta de hija Fijaciones > Pernos (happy)

- **WHEN** existe la categoría `"Fijaciones"` y se crea `"Pernos"` con ese padre
- **THEN** la operación termina sin error
- **AND** `ruta` es `["Fijaciones", "Pernos"]`
- **AND** `ancestros` contiene exactamente el `id` de `"Fijaciones"`
- **AND** `slug` es `"pernos"`

#### Scenario: get de categoría existente (happy)

- **WHEN** se persiste una categoría y se llama `get` con su `id`
- **THEN** la operación termina sin error
- **AND** `nombre`, `ruta` y `id` coinciden con lo persistido

#### Scenario: list_all incluye raíz e hija (happy)

- **WHEN** existen `"Fijaciones"` y `"Pernos"` y se llama `list_all`
- **THEN** la operación termina sin error
- **AND** el resultado incluye ambas categorías

#### Scenario: create emite evento de outbox (happy)

- **WHEN** `create` persiste una categoría
- **THEN** existe un evento en `outbox` con `entidad` `categoria` y `operacion` `upsert`
- **AND** `registro` es el `id` de la categoría (sin prefijo de tabla)

#### Scenario: mismo slug bajo padres distintos (happy)

- **WHEN** existen dos padres distintos y se crean dos hijas con el mismo *slug* y padre distinto
- **THEN** ambas altas terminan sin error

#### Scenario: slug duplicado bajo el mismo padre (unhappy)

- **WHEN** ya existe una hija con *slug* `"pernos"` bajo `"Fijaciones"` y se intenta crear otra con el mismo padre y el mismo *slug*
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_slug`
- **AND** no queda una segunda fila ni un segundo evento de outbox para esa alta

#### Scenario: slug duplicado entre raíces (unhappy)

- **WHEN** ya existe una raíz con *slug* `"fijaciones"` y se intenta crear otra raíz con el mismo *slug*
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_slug`
- **AND** no queda una segunda raíz ni su evento de outbox

#### Scenario: get de id inexistente (unhappy)

- **WHEN** se llama `get` con un `id` que no existe en `categoria`
- **THEN** la operación falla
- **AND** el error es `DomainError::NotFound`

#### Scenario: padre inexistente (unhappy)

- **WHEN** se llama `create` con un `padre` que no existe
- **THEN** la operación falla
- **AND** el error es `DomainError::NotFound`
- **AND** no se inserta fila en `categoria` ni evento en `outbox`

#### Scenario: quinto nivel rechazado (unhappy)

- **WHEN** el padre persistido ya tiene profundidad 4 y se intenta crear una hija
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se inserta la hija ni su evento de outbox

## MODIFIED Requirements

### Requirement: Transacción dato + outbox

El crate `storage` MUST exponer `storage::Tx`, obtenida desde un `Db` ya abierto y migrado. `Tx` MUST acumular sentencias SurrealQL con parámetros y uno o más `OutboxEvent`. Al confirmar, MUST ejecutar todas las sentencias y un `CREATE` en `outbox` por cada evento **entre** `BEGIN` y `COMMIT` (una sola transacción). El `CREATE` de outbox MUST usar `entidad` y `operacion` del evento, `registro` como `record` `entidad:ulid` (el ULID de `OutboxEvent.registro`) y `motivo` si está presente. `Tx::commit` MUST devolver `Result<(), DomainError>`. MUST NOT persistir el dato si el `COMMIT` no ocurre. Los tests de `Tx` MUST usar una tabla auxiliar definida en el propio test, no tablas de negocio. El crate MAY implementar `CategoryRepo` (T-02-03).

#### Scenario: Commit escribe dato y evento (happy)

- **WHEN** una `Tx` inserta un registro en una tabla auxiliar y emite un `OutboxEvent` `producto`/`upsert`, y se confirma
- **THEN** `commit` termina sin error
- **AND** el conteo de la tabla auxiliar es 1
- **AND** el conteo de `outbox` es 1
- **AND** el evento tiene `entidad` `producto` y `operacion` `upsert`

#### Scenario: Varios eventos en la misma transacción (happy)

- **WHEN** una `Tx` emite dos `OutboxEvent` y se confirma
- **THEN** `commit` termina sin error
- **AND** el conteo de `outbox` es 2

#### Scenario: THROW no deja dato ni evento (unhappy)

- **WHEN** una `Tx` inserta un registro auxiliar, emite un `OutboxEvent` y ejecuta `THROW "stock_negativo"`, y se intenta confirmar
- **THEN** `commit` falla
- **AND** el conteo de la tabla auxiliar es 0
- **AND** el conteo de `outbox` es 0

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `category_repo_creates_root_fijaciones` | `storage` (`tests/categoria.rs`) | integración happy |
| `category_repo_creates_child_pernos_with_ruta` | `storage` (`tests/categoria.rs`) | integración happy (Listo cuando) |
| `category_repo_get_returns_persisted` | `storage` (`tests/categoria.rs`) | integración happy |
| `category_repo_list_all_includes_root_and_child` | `storage` (`tests/categoria.rs`) | integración happy |
| `category_repo_create_emits_outbox_upsert` | `storage` (`tests/categoria.rs`) | integración happy |
| `category_repo_allows_same_slug_under_different_padres` | `storage` (`tests/categoria.rs`) | integración happy |
| `category_repo_rejects_duplicate_slug_under_same_padre` | `storage` (`tests/categoria.rs`) | integración unhappy (HU-02-01) |
| `category_repo_rejects_duplicate_root_slug` | `storage` (`tests/categoria.rs`) | integración unhappy |
| `category_repo_get_unknown_id_is_not_found` | `storage` (`tests/categoria.rs`) | integración unhappy |
| `category_repo_create_missing_padre_is_not_found` | `storage` (`tests/categoria.rs`) | integración unhappy |
| `category_repo_rejects_depth_five` | `storage` (`tests/categoria.rs`) | integración unhappy |

Los tests MUST usar `storage::testing::test_db()` y el puerto `CategoryRepo`. MUST NOT llamar HTTP ni el crate `api`. `cargo test --workspace` MUST ejecutarlos.
