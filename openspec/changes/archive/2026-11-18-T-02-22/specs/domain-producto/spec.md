# Spec Delta

Ticket padre: T-02-22. Change: T-02-22. HU: HU-02-06.

## MODIFIED Requirements

### Requirement: Puerto ProductRepo sin persistencia

El crate `domain` MUST exponer el puerto `ProductRepo` como *trait* con operaciones `create`, `get`, `update` y `set_activo`. Los `id` MUST ser `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `create` MUST aceptar `NewProducto` con `sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `stock_inicial` y `terminal` (`Option<string>`: identificador de terminal para el movimiento `inicial`; ausente si no hubo cabecera HTTP). MUST NOT validar `maxLength` 64 de `X-Vectra-Terminal` (eso es el Gateway). `get` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `FichaProducto`. `update` MUST aceptar `UpdateProducto` (versión esperada + ficha completa) y el resultado de éxito MUST ser `FichaProducto`. `set_activo` MUST aceptar `Id<Producto>` y `bool` y el resultado de éxito MUST ser `FichaProducto`. El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT declarar listado (T-02-27). MUST NOT crear movimientos de stock (T-02-18 / T-02-24). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create y get (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `create`, `get`, `update` y `set_activo`
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
