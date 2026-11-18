# Spec Delta

Ticket padre: T-02-27. Change: T-02-27. HU: HU-02-10.

## ADDED Requirements

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

## MODIFIED Requirements

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
