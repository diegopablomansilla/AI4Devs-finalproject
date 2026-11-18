# Spec Delta

## Purpose

Define la entidad `Producto`, los value objects de ficha (`Sku`, `Codigo`, `Ubicacion`, `UnidadMedida`, `ValorAtributo`), la validación de atributos contra `AtributoDef` y el puerto `ProductRepo` en el crate `domain`, para que el repositorio (T-02-18) implemente un contrato hexagonal sin persistir en este ticket.

Ticket padre: T-02-16. Change: T-02-16. HU: HU-02-04.

## ADDED Requirements

### Requirement: Entidad Producto con identidad tipada

El crate `domain` MUST exponer una entidad `Producto` cuyos campos cubren el modelo [4.4.4](../../../../../docs/4-modelo.md#444-producto) con nombres en español ([5.3](../../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id` (`Id<Producto>`), `sku` (`Sku`), `nombre` (`string`), `tipo` (`string`, sustantivo principal), `descripcion` (`Option<string>`), `marca` (`Option<Id<Marca>>`), `categoria` (`Id<Categoria>`), `atributos` (lista de pares `def` + `valor`), `codigos` (lista de `Codigo`), `unidad_medida` (`UnidadMedida`), `precio` (`Decimal` ≥ 0), `ubicacion` (`Option<Ubicacion>`), `stock` (`Decimal` ≥ 0), `activo` (`bool`), `version` (`int` ≥ 1), `creado_en` y `actualizado_en` (`Timestamp`). El `id` MUST ser un ULID de 26 caracteres **sin** prefijo de tabla (`producto:`).

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
- `Codigo`: `tipo` uno de `fabricante`, `oem`, `barras` (modelo [4.4.4](../../../../../docs/4-modelo.md#444-producto); API `manufacturer`/`oem`/`barcode` se traduce en T-02-21) y `valor` 1..=64 tras recortar espacios. MUST NOT exigir unicidad de `oem` (varios productos MAY compartir OEM)
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

El crate `domain` MUST exponer el puerto `ProductRepo` como *trait* con operaciones `create` y `get`. Los `id` MUST ser `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `create` MUST aceptar `NewProducto` con `sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion` y `stock_inicial`. El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT declarar `update`, `set_activo` ni listado (T-02-19, T-02-22, T-02-27). MUST NOT crear movimientos de stock ni eventos de outbox (T-02-18). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create y get (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `create` y `get`
- **AND** los resultados de éxito usan `Producto` y `Id<Producto>`
- **AND** los resultados de error usan `DomainError`

#### Scenario: domain no persiste productos (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `CREATE` sobre la tabla `producto`

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
