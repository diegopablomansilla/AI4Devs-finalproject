# Spec Delta

Ticket padre: T-02-21. Change: T-02-21. HU: HU-02-04.

## MODIFIED Requirements

### Requirement: Puerto ProductRepo sin persistencia

El crate `domain` MUST exponer el puerto `ProductRepo` como *trait* con operaciones `create`, `get` y `update`. Los `id` MUST ser `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `create` MUST aceptar `NewProducto` con `sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `stock_inicial` y `terminal` (`Option<string>`: identificador de terminal para el movimiento `inicial`; ausente si no hubo cabecera HTTP). MUST NOT validar `maxLength` 64 de `X-Vectra-Terminal` (eso es el Gateway). `get` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `FichaProducto`. `update` MUST aceptar `UpdateProducto` (versión esperada + ficha completa) y el resultado de éxito MUST ser `FichaProducto`. El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT declarar `set_activo` ni listado (T-02-22, T-02-27). MUST NOT crear movimientos de stock (T-02-18 / T-02-24). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create y get (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `create`, `get` y `update`
- **AND** el resultado de éxito de `create` usa `Producto` y `Id<Producto>`
- **AND** los resultados de éxito de `get` y `update` usan `FichaProducto`
- **AND** los resultados de error usan `DomainError`

#### Scenario: El puerto declara update (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `update`
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
