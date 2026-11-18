# Spec Delta

Ticket padre: T-02-21. Change: T-02-21. HU: HU-02-04.

## MODIFIED Requirements

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
