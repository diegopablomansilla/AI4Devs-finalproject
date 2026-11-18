# catalog-producto Specification

## Purpose

Orquesta en el crate `catalog` los casos de uso de producto (crear, obtener ficha y editar) sobre `ProductRepo`, cargando `AtributoDef` para validar valores y exigiendo que categoría y marca existan y estén activas, con tests contra repositorios en memoria y fachada object-safe para el Gateway, sin HTTP ni baja lógica.

Ticket padre: T-02-20. Change: T-02-20. HU: HU-02-04, HU-02-05.

## Requirements

### Requirement: Crear y obtener ficha de producto

El crate `catalog` MUST exponer casos de uso de producto que delegan en `ProductRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. Crear MUST aceptar los campos de `NewProducto` (`sku`, `nombre`, `tipo`, `descripcion`, `marca` opcional, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `stock_inicial`, `terminal` opcional). MUST reenviar `terminal` al puerto sin transformarlo. MUST NOT validar `maxLength` 64 (Gateway). MUST NOT escribir el ledger.

Antes de llamar a `ProductRepo::create`, el caso de uso MUST:

- cargar cada `AtributoDef` referenciada en `atributos` y validar los valores contra esas definiciones (mismas reglas que el constructor de `Producto`: tipo, `opciones` de `enum`, `def` no repetido)
- cargar la categoría por `id` y exigir que exista y tenga `activo = true`
- si `marca` está presente, cargar la marca por `id` y exigir que exista y tenga `activo = true`

El resultado de éxito de crear MUST ser la `FichaProducto` hidratada (marca, `ruta` de categoría, atributos con `clave`/`etiqueta`/`unidad`), con `activo` verdadero, `version` 1 y `stock` igual a `stock_inicial`. MUST NOT escribir `stock` fuera de `stock_inicial` pasado al puerto. MUST NOT crear movimientos de ledger por sí mismo (T-02-18 / T-02-24). MUST NOT incluir imágenes.

Obtener MUST delegar en `ProductRepo::get` y devolver la `FichaProducto` o `not_found`. MUST NOT filtrar por `activo` (la baja es T-02-22).

Los errores del puerto (`duplicate_sku`, `invalid_input`, `not_found`) MUST propagarse sin remapeo. Categoría o marca inexistente o inactiva MUST fallar con `DomainError` `code` `invalid_input` y un `FieldError` cuyo `field` es `categoria` o `marca` respectivamente (T-02-21 remapeará a `categoryId` / `brandId`). `AtributoDef` inexistente MUST ser `invalid_input` con `FieldError.field` `attributes[n].definitionId`. Valor incompatible con el tipo de la definición MUST ser `invalid_input` con `FieldError.field` `attributes[n].value`. MUST NOT reindexar (T-03-07). MUST NOT dar de baja ni reactivar (T-02-22).

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

Editar MUST cargar la ficha actual, fusionar los campos de ficha enviados (al menos un campo de ficha además de `version`) y persistir vía `ProductRepo::update` con el snapshot completo y la `version` leída. Arrays `atributos` y `codigos`, si se envían, MUST reemplazarse enteros. MUST NOT escribir `stock`. MUST NOT cambiar `activo` (T-02-22). MUST NOT incluir `stock` ni `activo` en el parche de ficha.

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

### Requirement: Tests en memoria de cada caso

Los tests del crate `catalog` MUST usar repositorios en memoria (sin SurrealDB) para producto, categoría, marca y definiciones de atributo. MUST cubrir crear, obtener y editar, incluido el camino no feliz (Listo cuando). El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`. MUST NOT declarar tests de baja o reactivación de producto.

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

#### Scenario: Suite en memoria (happy)

- **WHEN** se ejecuta `cargo test -p catalog` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** ninguno abre SurrealDB

### Requirement: Fachada object-safe para el Gateway

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) de los casos de uso de producto (crear, obtener, editar) para que el Gateway la inyecte como `Arc<dyn …>`. El servicio de producto MUST implementar esa fachada. MUST NOT mapear a HTTP ni abrir SurrealDB. MUST NOT exponer baja, reactivación ni listado.

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye el servicio de producto con repositorios en memoria
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones
- **AND** crear un producto válido vía la fachada termina sin error
