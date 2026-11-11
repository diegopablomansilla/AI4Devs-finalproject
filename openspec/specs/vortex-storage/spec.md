# vortex-storage Specification

## Purpose

Abre SurrealDB embebido con RocksDB en el crate `storage`, selecciona el namespace `vectra` y la base `comercio`, aplica migraciones `.surql` embebidas con registro y checksum SHA-256 (incl. `categoria`), expone un harness de tests (`storage::testing`) con motor en memoria o RocksDB temporal, ofrece transacciones atómicas dato + outbox con aviso post-commit, e implementa el adaptador `CategoryRepo` (`create`/`get`/`list_all`).

Ticket padre: T-00-08, T-00-09, T-09-02, T-03-01, T-03-02, T-02-01, T-02-03. HU: HU-00-02, HU-09-01, HU-03-01, HU-02-01.

## Requirements

### Requirement: Db::open abre RocksDB embebido y selecciona NS/DB

El crate `storage` MUST exponer `storage::Db::open(path)` asíncrono. `path` MUST ser el directorio del motor (convención de instalación: `<paths.datos>/db`; este change MUST NOT leer `Config`). El motor MUST ser SurrealDB embebido con RocksDB (PM-03). MUST NOT usarse el motor en memoria en esta API. Tras una apertura exitosa, la sesión MUST usar el namespace `vectra` y la base `comercio` ([4.3](../../../docs/4-modelo.md#43-convenciones)). Si el directorio no existe o está vacío, la operación MUST crear lo necesario para esa sesión (directorio, namespace y base) y MUST NOT aplicar migraciones `.surql` ni crear la tabla `migracion` (T-00-09). La versión de `surrealdb` MUST ser la pinneada en `[workspace.dependencies]` (`=2.7.0`).

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

### Requirement: Motor de migraciones embebidas con checksum

El crate `storage` MUST incluir migraciones en `migrations/NNNN_nombre.surql` embebidas con `include_str!`. MUST existir `0000_migracion.surql` conforme a [4.10](../../../docs/4-modelo.md#410-operación-del-esquema). `Db::migrate()` MUST aplicar las migraciones embebidas en orden ascendente de `version`, registrar cada una en `migracion` con `checksum` SHA-256 hex del contenido del archivo, y ser idempotente cuando el contenido no cambió. Si una `version` ya registrada tiene un `checksum` distinto al del archivo embebido, la operación MUST fallar sin aplicar más SQL y el error MUST nombrar la versión en `Display`. `Db::open` MUST NOT aplicar migraciones (T-00-08).

#### Scenario: Primera instalación (happy)

- **WHEN** tras `Db::open` se llama `Db::migrate` en un directorio temporal vacío
- **THEN** `migrate` termina sin error
- **AND** existe un registro con `version` 0 y `checksum` igual al SHA-256 de `0000_migracion.surql`

#### Scenario: Re-arranque sin cambios (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia con las mismas migraciones embebidas
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro para la `version` 0

#### Scenario: Migración alterada (unhappy)

- **WHEN** la versión 0 ya está registrada y se intenta `apply_migrations` con SQL distinto en la misma versión
- **THEN** la operación falla
- **AND** el mensaje de error incluye la versión (`0` o `0000`)

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

### Requirement: Migración outbox, sync_cursor y outbox_fallo

El crate `storage` MUST incluir la migración embebida `0001_outbox.surql`, aplicada por `Db::migrate()` después de `0000_migracion.surql`. El SQL MUST definir las tablas `SCHEMAFULL` `outbox`, `sync_cursor` y `outbox_fallo` conforme a [4.8](../../../docs/4-modelo.md#48-sincronización-outbox-y-cursores), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../docs/4-modelo.md#410-operación-del-esquema)).

`outbox` MUST declarar: `entidad` (`string` con `ASSERT` en `producto`, `categoria`, `marca`, `atributo_def`, `sinonimo`, `nota_tecnica`, `imagen_producto`); `registro` (`record`); `operacion` (`string` con `ASSERT` en `upsert`, `delete`); `motivo` (`option<string>`); `creado_en` (`datetime` con `DEFAULT time::now()` y `READONLY`).

`sync_cursor` MUST declarar: `ultimo_evento` (`option<record<outbox>>`); `actualizado_en` (`datetime` con `VALUE time::now()`). La migración MUST crear los registros `sync_cursor:meilisearch` y `sync_cursor:vectores` con `ultimo_evento` vacío si aún no existen.

`outbox_fallo` MUST declarar: `evento` (`record<outbox>`); `consumidor` (`string` con `ASSERT` en `meilisearch`, `vectores`); `intentos` (`int` con `DEFAULT 1`); `ultimo_error` (`string`); `estado` (`string` con `ASSERT` en `reintentando`, `fallido`); `actualizado_en` (`datetime` con `VALUE time::now()`); índice único `uq_outbox_fallo` sobre `(evento, consumidor)`.

MUST NOT habilitar *change feed* en estas tablas (tablas técnicas; el *change feed* aplica a maestras y ledger). MUST NOT implementar el worker Index Sync ni puertos de repositorio de catálogo (T-03-07 / T-02-03).

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

### Requirement: Migración categoria

El crate `storage` MUST incluir la migración embebida `0002_categoria.surql`, aplicada por `Db::migrate()` después de `0001_outbox.surql`. El SQL MUST definir la tabla `SCHEMAFULL` `categoria` conforme a [4.4.1](../../../docs/4-modelo.md#441-categoria--árbol), usando `DEFINE … IF NOT EXISTS` ([4.10](../../../docs/4-modelo.md#410-operación-del-esquema)).

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

### Requirement: Tests de CategoryRepo

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. **Listo cuando** del ticket: Fijaciones > Pernos.

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

#### Scenario: la suite de CategoryRepo cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p storage --test categoria` en `vortex-core/`
- **THEN** corren los tests de la tabla
- **AND** el alta de hija produce `ruta = ["Fijaciones", "Pernos"]`
- **AND** los unhappy de `duplicate_slug`, `not_found` y profundidad 5 pasan

### Requirement: OutboxNotifier tras commit exitoso

El crate `storage` MUST exponer `OutboxNotifier` basado en `tokio::sync::Notify`. Un `Db` MUST compartir un `OutboxNotifier`. Tras un `Tx::commit` **exitoso**, MUST disparar el notifier (al menos un waiter en espera MUST despertar). Si `commit` falla, MUST NOT disparar el notifier. MUST NOT arrancar el worker Index Sync (T-03-07).

#### Scenario: Aviso solo tras commit (happy)

- **WHEN** un waiter espera el `OutboxNotifier` y una `Tx` con dato + evento se confirma
- **THEN** el waiter se despierta
- **AND** el dato y el evento existen

#### Scenario: Fallo no avisa (unhappy)

- **WHEN** un waiter espera el `OutboxNotifier` y una `Tx` con dato + evento + `THROW` falla al confirmar
- **THEN** el waiter no se despierta por ese `commit`
- **AND** el dato y el evento no existen

### Requirement: UNIQUE y THROW se traducen a DomainError

`Tx::commit` MUST traducir errores del motor a `DomainError` existente (T-00-03) y MUST NOT mapear a HTTP ni `problem+json`. `THROW "stock_negativo"` MUST ser `InvalidInput` con `code` `negative_stock` ([4.5.2](../../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción), [5.4](../../../docs/5-contract-api.md#54-errores)). Un `THROW` cuyo texto es un `code` de dominio conocido MUST usar esa variante/`code`. Una violación de índice `UNIQUE` MUST ser `Conflict`: `duplicate_sku` si el índice o el mensaje identifican `sku`; `duplicate_slug` si identifican `slug`; `duplicate_relation` si identifican `relation`; en cualquier otro UNIQUE, `conflict`. Un fallo de motor no clasificado MUST ser `Unavailable` con `code` `unavailable`. MUST NOT agregar variantes ni `code` nuevos a `DomainError`.

#### Scenario: THROW stock_negativo (unhappy)

- **WHEN** una `Tx` ejecuta `THROW "stock_negativo"` y se confirma
- **THEN** `commit` falla con `DomainError::InvalidInput`
- **AND** el `code` expuesto es `negative_stock`

#### Scenario: UNIQUE sku (unhappy)

- **WHEN** una `Tx` inserta dos filas que violan un índice `UNIQUE` cuyo nombre incluye `sku` y se confirma
- **THEN** `commit` falla con `DomainError::Conflict`
- **AND** el `code` expuesto es `duplicate_sku`
- **AND** no queda la segunda fila ni su evento de outbox (si se había encolado en esa `Tx`)

#### Scenario: Error de motor no clasificado (unhappy)

- **WHEN** una `Tx` ejecuta una sentencia que el motor rechaza sin ser `THROW` de código conocido ni UNIQUE (p. ej. SQL inválido)
- **THEN** `commit` falla con `DomainError::Unavailable`
- **AND** el `code` expuesto es `unavailable`

### Requirement: Tests de Tx y notifier

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. **Listo cuando** del ticket: rollback y notify.

| Test | Ubicación | Tipo |
|---|---|---|
| `tx_commit_writes_data_and_outbox` | `vortex-core/crates/storage/tests/tx.rs` | integración / happy |
| `tx_commit_writes_multiple_outbox_events` | `tests/tx.rs` | integración / happy |
| `tx_commit_notifies_waiter` | `tests/tx.rs` | integración / happy |
| `tx_throw_rolls_back_data_and_outbox` | `tests/tx.rs` | integración / unhappy (Listo cuando) |
| `tx_failed_does_not_notify` | `tests/tx.rs` | integración / unhappy (Listo cuando) |
| `tx_throw_stock_negativo_is_negative_stock` | `tests/tx.rs` | integración / unhappy |
| `tx_unique_sku_is_duplicate_sku` | `tests/tx.rs` | integración / unhappy |
| `tx_unclassified_error_is_unavailable` | `tests/tx.rs` | integración / unhappy |

Los tests MUST usar `storage::testing::test_db()`. MUST NOT crear repositorios de catálogo ni tablas `categoria`/`producto` de negocio. `cargo test --workspace` MUST ejecutarlos.

#### Scenario: la suite de Tx cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p storage --test tx` en `vortex-core/`
- **THEN** corren los tests de la tabla
- **AND** commit escribe dato + outbox y avisa
- **AND** THROW deja conteos en 0 y no avisa

### Requirement: CategoryRepo update con recálculo de rama

El adaptador `CategoryRepo` en `storage` MUST implementar `update(UpdateCategoria)`. Al cambiar `nombre` y/o `padre`, MUST recalcular `slug`, `ancestros` y `ruta` del nodo y de todo su subárbol en una sola `Tx`. MUST emitir un `OutboxEvent` `categoria`/`upsert` por cada categoría actualizada. MUST usar `reject_cycle` del dominio antes de escribir. MUST NOT reindexar productos (T-03-08).

#### Scenario: Renombrar rama de 3 niveles (happy)

- **WHEN** existen Fijaciones > Pernos > Hex y `update` renombra la raíz a Tornillería sin cambiar `padre`
- **THEN** las rutas persistidas son `["Tornillería"]`, `["Tornillería", "Pernos"]` y `["Tornillería", "Pernos", "Hex"]`
- **AND** existen tres eventos outbox adicionales

#### Scenario: Mover debajo de descendiente (unhappy)

- **WHEN** Pernos es hija de Fijaciones y `update` asigna padre Pernos a Fijaciones
- **THEN** la operación falla con `code` `category_cycle`
- **AND** no se añaden filas ni eventos outbox

#### Scenario: Suite de integración update (happy)

- **WHEN** se ejecuta `cargo test -p storage --test categoria` filtrando tests `category_repo_renames_three_level_branch_ruta`, `category_repo_update_emits_outbox_per_affected` y unhappy `category_repo_move_under_descendant_is_category_cycle`
- **THEN** todos pasan

### Requirement: CategoryRepo set_activo

El adaptador `CategoryRepo` en `storage` MUST implementar `set_activo(id, activo)`. MUST actualizar solo el campo `activo` (y `actualizado_en`) de esa fila en una `Tx` con un `OutboxEvent` `categoria`/`upsert`. MUST NOT recalcular `ancestros`/`ruta` ni tocar hijas. `id` desconocido MUST ser `not_found`. MUST NOT reindexar productos (T-03-08). MUST NOT consultar `producto`.

#### Scenario: Baja lógica persistida (happy)

- **WHEN** existe una categoría activa y `set_activo` se llama con `false`
- **THEN** `get` devuelve `activo` falso
- **AND** existe un evento outbox `categoria`/`upsert` adicional

#### Scenario: set_activo de id desconocido (unhappy)

- **WHEN** `set_activo` se llama con un `id` que no existe
- **THEN** la operación falla con `code` `not_found`
- **AND** no se añade evento outbox

#### Scenario: Suite de integración set_activo (happy)

- **WHEN** se ejecuta `cargo test -p storage --test categoria` filtrando `category_repo_set_activo_persists_inactive` y `category_repo_set_activo_unknown_id_is_not_found`
- **THEN** ambos pasan
