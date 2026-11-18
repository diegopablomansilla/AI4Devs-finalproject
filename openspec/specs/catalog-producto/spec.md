# catalog-producto Specification

## Purpose

Orquesta en el crate `catalog` los casos de uso de producto (crear, obtener ficha, editar, dar de baja y reactivar) sobre `ProductRepo`, cargando `AtributoDef` para validar valores y exigiendo que categoría y marca existan y estén activas, con tests contra repositorios en memoria y fachada object-safe para el Gateway, sin HTTP.

Ticket padre: T-02-20. Change: T-02-20. HU: HU-02-04, HU-02-05.

## Requirements

### Requirement: Crear y obtener ficha de producto

El crate `catalog` MUST exponer casos de uso de producto que delegan en `ProductRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. Crear MUST aceptar los campos de `NewProducto` (`sku`, `nombre`, `tipo`, `descripcion`, `marca` opcional, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `stock_inicial`, `terminal` opcional). MUST reenviar `terminal` al puerto sin transformarlo. MUST NOT validar `maxLength` 64 (Gateway). MUST NOT escribir el ledger.

Antes de llamar a `ProductRepo::create`, el caso de uso MUST:

- cargar cada `AtributoDef` referenciada en `atributos` y validar los valores contra esas definiciones (mismas reglas que el constructor de `Producto`: tipo, `opciones` de `enum`, `def` no repetido)
- cargar la categoría por `id` y exigir que exista y tenga `activo = true`
- si `marca` está presente, cargar la marca por `id` y exigir que exista y tenga `activo = true`

El resultado de éxito de crear MUST ser la `FichaProducto` hidratada (marca, `ruta` de categoría, atributos con `clave`/`etiqueta`/`unidad`), con `activo` verdadero, `version` 1 y `stock` igual a `stock_inicial`. MUST NOT escribir `stock` fuera de `stock_inicial` pasado al puerto. MUST NOT crear movimientos de ledger por sí mismo (T-02-18 / T-02-24). MUST NOT incluir imágenes.

Obtener MUST delegar en `ProductRepo::get` y devolver la `FichaProducto` o `not_found`. MUST NOT filtrar por `activo` (la ficha admin incluye inactivos para reactivar).

Los errores del puerto (`duplicate_sku`, `invalid_input`, `not_found`) MUST propagarse sin remapeo. Categoría o marca inexistente o inactiva MUST fallar con `DomainError` `code` `invalid_input` y un `FieldError` cuyo `field` es `categoria` o `marca` respectivamente (T-02-21 remapeará a `categoryId` / `brandId`). `AtributoDef` inexistente MUST ser `invalid_input` con `FieldError.field` `attributes[n].definitionId`. Valor incompatible con el tipo de la definición MUST ser `invalid_input` con `FieldError.field` `attributes[n].value`. MUST NOT reindexar (T-03-07).

#### Scenario: Alta completa perno M8 x 40 (happy, HU-02-04)

- **WHEN** existen una categoría activa, una marca activa y definiciones de atributo, y el caso de uso crea "Perno hexagonal M8 x 40 zincado" con SKU `"PER-M8X40-ZN"`, precio, ubicación, atributos válidos y `stock_inicial` 50
- **THEN** la operación termina sin error
- **AND** el producto queda activo con `version` 1
- **AND** `stock` es 50
- **AND** la ficha expone marca, `ruta` de categoría y atributos resueltos

#### Scenario: Alta reenvía terminal al puerto (happy, T-02-21)

- **WHEN** el caso de uso crea un producto válido con `terminal` `"caja-1"`
- **THEN** la operación termina sin error
- **AND** el puerto `create` recibe `NewProducto.terminal` `"caja-1"`

#### Scenario: Obtener ficha existente (happy)

- **WHEN** existe un producto creado y el caso de uso lo obtiene por `id`
- **THEN** la operación termina sin error
- **AND** el resultado es una `FichaProducto` con marca (si hay), `categoria.ruta` y atributos con `clave`/`etiqueta`/`unidad`

#### Scenario: SKU duplicado (unhappy, HU-02-04)

- **WHEN** ya existe un producto con SKU `"PER-M8X40-ZN"`
- **AND** el caso de uso intenta crear otro con el mismo SKU
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `duplicate_sku`

#### Scenario: Atributo number con valor cuarenta (unhappy, HU-02-04)

- **WHEN** existe la definición "largo" de tipo `numero`
- **AND** el caso de uso crea un producto cuyo atributo en índice `n` referencia esa definición con valor texto `"cuarenta"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `attributes[n].value`

#### Scenario: Categoría inactiva (unhappy)

- **WHEN** la categoría referenciada existe y tiene `activo` falso
- **AND** el caso de uso intenta crear un producto en esa categoría
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `categoria`
- **AND** no se persiste el producto

#### Scenario: Marca inactiva (unhappy)

- **WHEN** la marca referenciada existe y tiene `activo` falso
- **AND** el caso de uso intenta crear un producto con esa marca
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `marca`
- **AND** no se persiste el producto

#### Scenario: Categoría inexistente (unhappy)

- **WHEN** el `id` de categoría no existe
- **AND** el caso de uso intenta crear un producto con esa categoría
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `categoria`

#### Scenario: Obtener id desconocido (unhappy)

- **WHEN** el caso de uso obtiene un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Editar ficha con bloqueo optimista

Editar MUST cargar la ficha actual, fusionar los campos de ficha enviados (al menos un campo de ficha además de `version`) y persistir vía `ProductRepo::update` con el snapshot completo y la `version` leída. Arrays `atributos` y `codigos`, si se envían, MUST reemplazarse enteros. MUST NOT escribir `stock`. MUST NOT cambiar `activo` (usar `deactivate`/`reactivate`). MUST NOT incluir `stock` ni `activo` en el parche de ficha.

Antes de `update`, el caso de uso MUST repetir las mismas comprobaciones de categoría/marca activas y de `AtributoDef` que en el alta, usando los valores **resultantes** del merge. El resultado de éxito MUST ser la `FichaProducto` con `version` incrementada en 1. Una `version` desactualizada MUST ser `DomainError` `code` `version_conflict` (propagada del puerto). `id` desconocido MUST ser `not_found`. MUST NOT mapear a HTTP.

#### Scenario: Edición exitosa de precio (happy, HU-02-05)

- **WHEN** existe un producto con `version` 1 y precio `"120.50"`
- **AND** el caso de uso envía un cambio de precio `"130.00"` con `version` 1
- **THEN** la operación termina sin error
- **AND** el precio persistido es `"130.00"`
- **AND** `version` es 2
- **AND** `stock` no cambia

#### Scenario: Parche parcial conserva el resto de la ficha (happy)

- **WHEN** existe un producto con nombre, SKU y categoría
- **AND** el caso de uso envía solo `version` y un nuevo precio
- **THEN** nombre, SKU y categoría se conservan
- **AND** el precio es el nuevo
- **AND** `version` se incrementa en 1

#### Scenario: Conflicto de versión (unhappy, HU-02-05)

- **WHEN** existe un producto con `version` 1
- **AND** otra edición ya persistió `version` 2
- **AND** el caso de uso envía un cambio con `version` 1
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `version_conflict`
- **AND** el precio persistido es el de la edición que sí aplicó

#### Scenario: Editar id desconocido (unhappy)

- **WHEN** el caso de uso edita un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

#### Scenario: Editar con categoría inactiva (unhappy)

- **WHEN** el merge resultante apunta a una categoría inactiva
- **AND** el caso de uso intenta editar
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `categoria`
- **AND** la `version` persistida no cambia

### Requirement: Dar de baja y reactivar producto

El crate `catalog` MUST exponer casos de uso `deactivate` y `reactivate` que delegan en `ProductRepo::set_activo` (`false` / `true`). MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT borrar ledger, relaciones ni imágenes. MUST NOT reindexar (T-03-07). MUST NOT exigir occupancy ni rechazar la baja por stock, relaciones o imágenes.

`deactivate` MUST dejar `activo` falso. Si el producto ya está inactivo, MUST ser éxito idempotente. `reactivate` MUST dejar `activo` verdadero. El resultado de éxito MUST ser la `FichaProducto`. `id` desconocido MUST ser `not_found`. Obtener MUST seguir devolviendo fichas inactivas (sin filtrar por `activo`).

`PatchProducto` MUST NOT incluir `activo`. Editar ficha MUST NOT cambiar `activo`.

#### Scenario: Baja lógica (happy, HU-02-06)

- **WHEN** existe un producto activo
- **AND** el caso de uso lo da de baja
- **THEN** la operación termina sin error
- **AND** la ficha tiene `activo` falso
- **AND** `stock` no cambia

#### Scenario: Reactivación (happy, HU-02-06, Listo cuando)

- **WHEN** existe un producto inactivo
- **AND** el caso de uso lo reactiva
- **THEN** la operación termina sin error
- **AND** la ficha tiene `activo` verdadero
- **AND** `stock` no cambia

#### Scenario: Baja idempotente (happy)

- **WHEN** el producto ya está inactivo
- **AND** el caso de uso lo da de baja
- **THEN** la operación termina sin error
- **AND** `activo` sigue falso

#### Scenario: Obtener ficha inactiva (happy)

- **WHEN** existe un producto inactivo
- **AND** el caso de uso lo obtiene por `id`
- **THEN** la operación termina sin error
- **AND** la ficha tiene `activo` falso

#### Scenario: Baja id desconocido (unhappy)

- **WHEN** el caso de uso da de baja un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

#### Scenario: Reactivar id desconocido (unhappy)

- **WHEN** el caso de uso reactiva un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Tests en memoria de cada caso

Los tests del crate `catalog` MUST usar repositorios en memoria (sin SurrealDB) para producto, categoría, marca y definiciones de atributo. MUST cubrir crear, obtener, editar, dar de baja, reactivar y listar, incluido el camino no feliz (Listo cuando). El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`.

| Test | Crate | Tipo |
|---|---|---|
| `catalog_create_perno_m8x40_sets_version_1` | `catalog` | unitario (memoria), happy, HU-02-04 / Listo cuando |
| `catalog_get_returns_ficha_with_marca_ruta_and_attributes` | `catalog` | unitario (memoria), happy |
| `catalog_update_price_increments_version` | `catalog` | unitario (memoria), happy, HU-02-05 |
| `catalog_update_partial_price_keeps_other_fields` | `catalog` | unitario (memoria), happy |
| `catalog_create_duplicate_sku_is_duplicate_sku` | `catalog` | unitario (memoria), unhappy, HU-02-04 |
| `catalog_create_cuarenta_on_number_attribute_is_invalid_input` | `catalog` | unitario (memoria), unhappy, HU-02-04 |
| `catalog_create_inactive_categoria_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_create_inactive_marca_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_create_unknown_categoria_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_get_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_update_stale_version_is_version_conflict` | `catalog` | unitario (memoria), unhappy, HU-02-05 / Listo cuando |
| `catalog_update_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_update_inactive_categoria_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_deactivate_sets_inactive` | `catalog` | unitario (memoria), happy, HU-02-06 |
| `catalog_reactivate_sets_active` | `catalog` | unitario (memoria), happy, HU-02-06 / Listo cuando |
| `catalog_deactivate_idempotent_when_already_inactive` | `catalog` | unitario (memoria), happy |
| `catalog_get_inactive_product_returns_ficha` | `catalog` | unitario (memoria), happy |
| `catalog_deactivate_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_reactivate_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_list_branch_includes_descendants` | `catalog` | unitario (memoria), happy, HU-02-10 / Listo cuando |
| `catalog_list_inactive_only` | `catalog` | unitario (memoria), happy, HU-02-10 / Listo cuando |
| `catalog_list_filters_by_brand` | `catalog` | unitario (memoria), happy |
| `catalog_list_orders_by_id_and_after` | `catalog` | unitario (memoria), happy |
| `catalog_list_unknown_category_is_empty` | `catalog` | unitario (memoria), happy |

#### Scenario: Suite en memoria (happy)

- **WHEN** se ejecuta `cargo test -p catalog` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** ninguno abre SurrealDB

### Requirement: Fachada object-safe para el Gateway

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) de los casos de uso de producto (crear, obtener, editar, dar de baja, reactivar, listar) para que el Gateway la inyecte como `Arc<dyn …>`. El servicio de producto MUST implementar esa fachada. MUST NOT mapear a HTTP ni abrir SurrealDB.

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye el servicio de producto con repositorios en memoria
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones
- **AND** crear un producto válido vía la fachada termina sin error

#### Scenario: la fachada expone list (happy)

- **WHEN** se inspecciona la fachada object-safe de producto
- **THEN** expone `list`
- **AND** listar con un filtro vacío de categoría/marca y `activo` verdadero termina sin error

### Requirement: Listar productos admin

El crate `catalog` MUST exponer un caso de uso `list` que delega en `ProductRepo::list` y MUST NOT abrir SurrealDB ni mapear a HTTP. MUST aceptar `FiltroProducto`, `after` y `take` y devolver `Vec<ResumenProducto>`. MUST NOT validar `cursor`/`limit` HTTP (Gateway / helper T-00-21). MUST NOT exigir que categoría o marca existan o estén activas (un filtro que no matchea MUST ser lista vacía). MUST NOT buscar por texto. MUST NOT reindexar (T-03-07). MUST NOT hidratar imágenes. Los errores del puerto MUST propagarse sin remapeo.

#### Scenario: Filtro por rama incluye descendientes (happy, HU-02-10, Listo cuando)

- **WHEN** existe la categoría `"Fijaciones"` con hija `"Pernos"`
- **AND** hay un producto en `"Fijaciones"` y otro en `"Pernos"`, ambos activos, y un tercero en otra rama
- **AND** el caso de uso lista con `categoria` = id de `"Fijaciones"`, `activo` verdadero y `marca` ausente
- **THEN** el resultado incluye los productos de `"Fijaciones"` y de `"Pernos"`
- **AND** no incluye el producto de la otra rama

#### Scenario: Filtro por inactivos (happy, HU-02-10, Listo cuando)

- **WHEN** existe un producto activo y otro dado de baja
- **AND** el caso de uso lista con `activo` falso
- **THEN** el resultado incluye solo el producto inactivo

#### Scenario: Filtro por marca (happy)

- **WHEN** existen productos de marca `"Fixser"` y de otra marca, todos activos
- **AND** el caso de uso lista con `marca` = id de `"Fixser"` y `activo` verdadero
- **THEN** el resultado incluye solo los de `"Fixser"`

#### Scenario: Orden por id y after (happy)

- **WHEN** existen tres productos activos A, B, C ordenados por `id` ascendente
- **AND** el caso de uso lista con `take` 2 y `after` ausente
- **THEN** el resultado es A, B en ese orden
- **WHEN** lista de nuevo con `after` = id de B y `take` 2
- **THEN** el resultado es C
- **AND** no incluye A ni B

#### Scenario: Categoría inexistente es lista vacía (happy)

- **WHEN** el caso de uso lista con `categoria` igual a un `Id` que no existe
- **THEN** el resultado es una lista vacía
- **AND** no es `not_found`

#### Scenario: list no busca por texto (unhappy)

- **WHEN** se inspecciona el caso de uso `list`
- **THEN** no acepta `q` ni otro texto de búsqueda
- **AND** no llama a un índice de búsqueda
