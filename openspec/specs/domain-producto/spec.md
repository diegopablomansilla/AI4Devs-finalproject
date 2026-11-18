# domain-producto Specification

## Purpose

Define la entidad `Producto`, los value objects de ficha (`Sku`, `Codigo`, `Ubicacion`, `UnidadMedida`, `ValorAtributo`), la validación de atributos contra `AtributoDef` y el puerto `ProductRepo` en el crate `domain`, para que el repositorio (T-02-18) implemente un contrato hexagonal sin persistir en este ticket.

## Requirements

### Requirement: Entidad Producto con identidad tipada

El crate `domain` MUST exponer una entidad `Producto` cuyos campos cubren el modelo [4.4.4](../../../docs/4-modelo.md#444-producto) con nombres en español ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id` (`Id<Producto>`), `sku` (`Sku`), `nombre` (`string`), `tipo` (`string`, sustantivo principal), `descripcion` (`Option<string>`), `marca` (`Option<Id<Marca>>`), `categoria` (`Id<Categoria>`), `atributos` (lista de pares `def` + `valor`), `codigos` (lista de `Codigo`), `unidad_medida` (`UnidadMedida`), `precio` (`Decimal` ≥ 0), `ubicacion` (`Option<Ubicacion>`), `stock` (`Decimal` ≥ 0), `activo` (`bool`), `version` (`int` ≥ 1), `creado_en` y `actualizado_en` (`Timestamp`). El `id` MUST ser un ULID de 26 caracteres **sin** prefijo de tabla (`producto:`).

Construir un `Producto` MUST exigir:

- `sku` válido (1..=64 caracteres tras recortar espacios; OpenAPI `ProductCreate.sku`)
- `nombre` no vacío tras recortar espacios y de longitud ≤ 200 (OpenAPI `name`)
- `tipo` no vacío tras recortar espacios y de longitud ≤ 60 (OpenAPI `productType`)
- `descripcion`, si está presente, de longitud ≤ 4000
- `precio` ≥ 0 y `stock` ≥ 0
- `version` ≥ 1
- `atributos` válidos contra las `AtributoDef` aportadas (requirement «Validación de atributos contra AtributoDef»)

MUST NOT persistir ni abrir SurrealDB. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT usar los nombres de API en inglés (`name`, `productType`, `unitOfMeasure`, `brandId`) como campos de la entidad. MUST NOT incluir imágenes (T-02-28). MUST NOT escribir `stock` desde un caso de uso de ficha (el campo es proyección del ledger; T-02-18 / T-02-19); el constructor MAY aceptarlo para hidratar un snapshot.

#### Scenario: Construcción válida perno M8 x 40 (happy, HU-02-04)

- **WHEN** se construye un `Producto` con `sku` `"PER-M8X40-ZN"`, `nombre` `"Perno hexagonal M8 x 40 zincado"`, `tipo` `"Perno"`, una `categoria`, una `marca` opcional, `unidad_medida` `unidad`, `precio` `"120.50"`, `stock` `"0"`, `version` 1, `activo` verdadero y atributos cuyos valores coinciden con las `AtributoDef` aportadas
- **THEN** la construcción termina sin error
- **AND** el `id` no contiene `:`
- **AND** `sku` es `"PER-M8X40-ZN"`
- **AND** `nombre` es `"Perno hexagonal M8 x 40 zincado"`
- **AND** `activo` es verdadero
- **AND** `version` es 1

#### Scenario: sku vacío (unhappy)

- **WHEN** se intenta construir un `Producto` con `sku` vacío o solo espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `Producto`

#### Scenario: precio negativo (unhappy)

- **WHEN** se intenta construir un `Producto` con `precio` menor que 0
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `Producto`

### Requirement: Value objects Sku, Codigo, Ubicacion y UnidadMedida

El crate `domain` MUST exponer:

- `Sku`: cadena 1..=64 tras recortar espacios
- `Codigo`: `tipo` uno de `fabricante`, `oem`, `barras` (modelo [4.4.4](../../../docs/4-modelo.md#444-producto); API `manufacturer`/`oem`/`barcode` se traduce en T-02-21) y `valor` 1..=64 tras recortar espacios. MUST NOT exigir unicidad de `oem` (varios productos MAY compartir OEM)
- `Ubicacion`: objeto con `pasillo`, `estante` y `posicion` opcionales (`Option<string>`)
- `UnidadMedida`: uno de `unidad`, `par`, `caja`, `metro`, `kilogramo`, `litro` (API `unit`/`pair`/`box`/`meter`/`kilogram`/`liter` se traduce en T-02-21)

Parsear un `tipo` de código o una `UnidadMedida` fuera del enum MUST fallar con `DomainError` `code` `invalid_input`. MUST NOT mapear a HTTP.

#### Scenario: Codigo fabricante válido (happy)

- **WHEN** se construye un `Codigo` con `tipo` `fabricante` y `valor` `"HX-0840Z"`
- **THEN** la construcción termina sin error
- **AND** `tipo` es `fabricante`
- **AND** `valor` es `"HX-0840Z"`

#### Scenario: UnidadMedida unidad (happy)

- **WHEN** se parsea `unidad` como `UnidadMedida`
- **THEN** la operación termina sin error
- **AND** el valor es `unidad`

#### Scenario: tipo de codigo inválido (unhappy)

- **WHEN** se intenta construir un `Codigo` con `tipo` `"ean"` u otro valor fuera de `fabricante`/`oem`/`barras`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `Codigo`

#### Scenario: UnidadMedida inválida (unhappy)

- **WHEN** se intenta parsear `"unit"` o `"docena"` como `UnidadMedida`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce una `UnidadMedida`

### Requirement: Validación de atributos contra AtributoDef

Al construir un `Producto`, cada elemento de `atributos` MUST referenciar un `AtributoDef` aportado (`def` = `Id<AtributoDef>`) y su `valor` MUST coincidir con el `tipo` de esa definición:

- `texto` → valor texto
- `numero` (API `number`) → valor numérico (`Decimal`)
- `booleano` → valor booleano
- `enum` → valor texto que MUST estar en `opciones` de la definición (comparación exacta tras recortar espacios)

Un mismo `def` MUST NOT repetirse en el array. Un `def` ausente de las definiciones aportadas MUST fallar.

El mismatch de tipo o de valor (incluido un texto no numérico frente a `numero`, y un valor fuera de `opciones` frente a `enum`) MUST producir `DomainError::InvalidInput` con `code` `invalid_input` y un `FieldError` cuyo `field` es `attributes[n].value` (índice 0-based, ruta del contrato OpenAPI / HU-02-04). Un `def` repetido MUST producir `invalid_input` con `FieldError.field` `attributes[n].definitionId` para la ocurrencia duplicada. MUST NOT persistir. MUST NOT comprobar que categoría o marca existan o estén activas (T-02-20).

#### Scenario: Atributos coinciden con texto, numero y enum (happy)

- **WHEN** se construye un `Producto` con tres atributos: `medida` `texto` valor `"M8"`, `largo` `numero` valor `40`, `grado` `enum` valor `"5"` (presente en `opciones`)
- **THEN** la construcción termina sin error
- **AND** los tres pares `def`/`valor` quedan en `atributos`

#### Scenario: cuarenta en atributo number (unhappy, HU-02-04, Listo cuando)

- **WHEN** se intenta construir un `Producto` cuyo atributo en índice `n` referencia una `AtributoDef` de tipo `numero` y el valor es el texto `"cuarenta"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `attributes[n].value`
- **AND** no se produce un `Producto`

#### Scenario: def repetido (unhappy, Listo cuando)

- **WHEN** se intenta construir un `Producto` con dos entradas de `atributos` que referencian el mismo `Id<AtributoDef>`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `attributes[n].definitionId` donde `n` es el índice de la ocurrencia duplicada
- **AND** no se produce un `Producto`

#### Scenario: valor de enum fuera de opciones (unhappy)

- **WHEN** se intenta construir un `Producto` con un atributo `enum` cuyo valor texto no está en `opciones` de su `AtributoDef`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `attributes[n].value`
- **AND** no se produce un `Producto`

### Requirement: Puerto ProductRepo sin persistencia

El crate `domain` MUST exponer el puerto `ProductRepo` como *trait* con operaciones `create`, `get`, `update`, `set_activo` y `list`. Los `id` MUST ser `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `create` MUST aceptar `NewProducto` con `sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `stock_inicial` y `terminal` (`Option<string>`: identificador de terminal para el movimiento `inicial`; ausente si no hubo cabecera HTTP). MUST NOT validar `maxLength` 64 de `X-Vectra-Terminal` (eso es el Gateway). `get` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `FichaProducto`. `update` MUST aceptar `UpdateProducto` (versión esperada + ficha completa) y el resultado de éxito MUST ser `FichaProducto`. `set_activo` MUST aceptar `Id<Producto>` y `bool` y el resultado de éxito MUST ser `FichaProducto`. `list` MUST seguir el requirement «ProductRepo list». El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT crear movimientos de stock (T-02-18 / T-02-24). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create y get (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `create`, `get`, `update`, `set_activo` y `list`
- **AND** el resultado de éxito de `create` usa `Producto` y `Id<Producto>`
- **AND** los resultados de éxito de `get`, `update` y `set_activo` usan `FichaProducto`
- **AND** los resultados de error usan `DomainError`

#### Scenario: El puerto declara update (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `update`
- **AND** el resultado de éxito usa `FichaProducto`
- **AND** los resultados de error usan `DomainError`

#### Scenario: El puerto declara set_activo (happy, T-02-22)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `set_activo`
- **AND** acepta `Id<Producto>` y `bool`
- **AND** el resultado de éxito usa `FichaProducto`
- **AND** los resultados de error usan `DomainError`

#### Scenario: NewProducto incluye terminal opcional (happy, T-02-21)

- **WHEN** se construye un `NewProducto` con `terminal` `Some("caja-1")` y el resto de campos válidos
- **THEN** el valor se conserva en el input de `create`
- **AND** `terminal` `None` también es un input válido

#### Scenario: domain no persiste productos (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `CREATE` o `UPDATE` sobre la tabla `producto`

### Requirement: FichaProducto y UpdateProducto

El crate `domain` MUST exponer:

- `AtributoFicha`: `def` (`Id<AtributoDef>`), `clave`, `etiqueta`, `unidad` (`Option<string>`), `valor` (`ValorAtributo`)
- `FichaProducto`: `producto` (`Producto`), `marca` (`Option<Marca>`), `categoria` (`Categoria` con `ruta`), `atributos` (`Vec<AtributoFicha>`). MUST NOT incluir imágenes (T-02-30)
- `UpdateProducto`: `id` (`Id<Producto>`), `version` (`int` ≥ 1, versión leída por el caller), `sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos` (`Vec<AtributoValor>`), `codigos` (`Vec<Codigo>`), `unidad_medida`, `precio`, `ubicacion`

`UpdateProducto` MUST NOT declarar `stock` ni `activo` (stock es proyección del ledger; `activo` se cambia solo con `ProductRepo::set_activo`). MUST NOT mapear a HTTP ni usar nombres de API en inglés (`brandId`, `categoryId`, `key`/`label`/`unit`). MUST NOT persistir.

#### Scenario: FichaProducto expone marca, ruta y atributos resueltos (happy)

- **WHEN** se construye una `FichaProducto` con una `Marca`, una `Categoria` cuya `ruta` es `["Fijaciones", "Pernos"]` y un `AtributoFicha` con `clave` `"medida"`, `etiqueta` `"Medida"` y `unidad` `"mm"`
- **THEN** `marca` está presente
- **AND** `categoria.ruta` es `["Fijaciones", "Pernos"]`
- **AND** el atributo resuelto expone `clave`, `etiqueta` y `unidad`
- **AND** no hay campo de imágenes

#### Scenario: UpdateProducto no incluye stock ni activo (happy)

- **WHEN** se inspecciona el tipo `UpdateProducto` en `domain`
- **THEN** declara `id`, `version` y los campos de ficha (`sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`)
- **AND** no declara `stock` ni `activo`

### Requirement: FiltroProducto y ResumenProducto

El crate `domain` MUST exponer:

- `FiltroProducto`: `categoria` (`Option<Id<Categoria>>`), `marca` (`Option<Id<Marca>>`), `activo` (`bool`). MUST NOT incluir texto de búsqueda (`q`, nombre, SKU). MUST NOT usar nombres de API (`categoryId`, `brandId`, `active`).
- `ResumenProducto`: `id` (`Id<Producto>`), `sku`, `nombre`, `tipo`, `precio` (`Decimal`), `stock` (`Decimal`), `activo`, `marca_nombre` (`Option<string>`: nombre de la marca o ausente), `categoria_ruta` (`Vec<string>`: `ruta` de la categoría del producto). MUST NOT incluir atributos, códigos, versión, ubicación, descripción, imágenes ni `imageUrl`.

MUST NOT persistir ni mapear a HTTP.

#### Scenario: FiltroProducto declara categoria, marca y activo (happy)

- **WHEN** se construye un `FiltroProducto` con `categoria` presente, `marca` ausente y `activo` verdadero
- **THEN** el valor se conserva
- **AND** no hay campo de texto de búsqueda

#### Scenario: ResumenProducto expone nombre de marca y ruta (happy)

- **WHEN** se construye un `ResumenProducto` con `marca_nombre` `"Fixser"` y `categoria_ruta` `["Fijaciones", "Pernos"]`
- **THEN** `marca_nombre` es `"Fixser"`
- **AND** `categoria_ruta` es `["Fijaciones", "Pernos"]`
- **AND** no hay campo de imágenes ni `imageUrl`

### Requirement: ProductRepo list

`ProductRepo::list` MUST aceptar `FiltroProducto`, un cursor de dominio `after` (`Option<Id<Producto>>`: ausente = primera página) y `take` (`u32`: cantidad máxima de filas a devolver, el llamador pide `limit + 1`). El resultado de éxito MUST ser `Vec<ResumenProducto>` con **a lo sumo** `take` elementos, ordenados por `id` **ascendente** (ULID, orden de alta). Si `after` está presente, MUST excluir ese `id` y todos los anteriores (`id` estrictamente mayor). MUST NOT paginar con offset. MUST NOT buscar por texto.

Filtros MUST combinarse con AND:

- `activo` MUST coincidir exactamente con `producto.activo`
- `marca` presente MUST coincidir con `producto.marca`; ausente MUST NOT filtrar por marca
- `categoria` presente MUST incluir productos cuya categoría es ese `id` **o** cuyos `categoria.ancestros` contienen ese `id` (toda la rama, [4.4.1](../../../../../docs/4-modelo.md#441-categoria--árbol)); ausente MUST NOT filtrar por categoría

Una categoría o marca inexistente MUST producir lista vacía (no `not_found`). MUST NOT emitir outbox ni escribir. MUST NOT hidratar imágenes. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP.

#### Scenario: El puerto declara list (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `list`
- **AND** acepta `FiltroProducto`, `after` opcional y `take`
- **AND** el resultado de éxito usa `Vec<ResumenProducto>`
- **AND** los resultados de error usan `DomainError`

#### Scenario: list no busca por texto (unhappy)

- **WHEN** se inspecciona `FiltroProducto` y `ProductRepo::list`
- **THEN** no hay parámetro de consulta de texto (`q`, nombre, SKU)
- **AND** el puerto no declara un listado por relevancia

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `producto_constructs_perno_m8x40_zn` | `domain` (`src/producto.rs`) | unitario happy (HU-02-04) |
| `producto_rejects_blank_sku` | `domain` | unitario unhappy |
| `producto_rejects_negative_precio` | `domain` | unitario unhappy |
| `codigo_constructs_fabricante` | `domain` | unitario happy |
| `unidad_medida_parses_unidad` | `domain` | unitario happy |
| `codigo_rejects_unknown_tipo` | `domain` | unitario unhappy |
| `unidad_medida_rejects_english_or_unknown` | `domain` | unitario unhappy |
| `producto_accepts_matching_atributos` | `domain` | unitario happy |
| `producto_rejects_cuarenta_on_number_attribute` | `domain` | unitario unhappy (Listo cuando, HU-02-04) |
| `producto_rejects_duplicate_atributo_def` | `domain` | unitario unhappy (Listo cuando) |
| `producto_rejects_enum_value_not_in_opciones` | `domain` | unitario unhappy |
| `product_repo_trait_exposes_create_get` | `domain` | unitario happy (dummy) |
