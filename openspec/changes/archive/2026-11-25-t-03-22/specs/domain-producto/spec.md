# Spec Delta

Ticket: T-03-22. HU: HU-03-08.

## MODIFIED Requirements

### Requirement: Puerto ProductRepo sin persistencia

El crate `domain` MUST exponer el puerto `ProductRepo` como *trait* con operaciones `create`, `get`, `update`, `set_activo`, `list` y `find_by_code`. Los `id` MUST ser `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `create` MUST aceptar `NewProducto` con `sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `stock_inicial` y `terminal` (`Option<string>`: identificador de terminal para el movimiento `inicial`; ausente si no hubo cabecera HTTP). MUST NOT validar `maxLength` 64 de `X-Vectra-Terminal` (eso es el Gateway). `get` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `FichaProducto`. `update` MUST aceptar `UpdateProducto` (versión esperada + ficha completa) y el resultado de éxito MUST ser `FichaProducto`. `set_activo` MUST aceptar `Id<Producto>` y `bool` y el resultado de éxito MUST ser `FichaProducto`. `list` MUST seguir el requirement «ProductRepo list». `find_by_code` MUST aceptar `code: String` y devolver `Vec<ResumenProducto>`. El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT crear movimientos de stock (T-02-18 / T-02-24). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create y get (happy)

- **WHEN** se inspecciona el puerto `ProductRepo` en `domain`
- **THEN** expone `create`, `get`, `update`, `set_activo`, `list` y `find_by_code`
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

## ADDED Requirements

### Requirement: ProductRepo find_by_code

`ProductRepo::find_by_code` MUST devolver todos los productos **activos** cuyo `sku` es igual al código pedido **o** tienen al menos un `Codigo.valor` igual al código pedido. La comparación MUST ser igualdad exacta de string (case-sensitive). MUST NOT devolver coincidencias por prefijo ni por un carácter de diferencia. Varios productos con el mismo OEM MUST poder aparecer en el mismo resultado.

#### Scenario: OEM compartido devuelve ambas piezas (happy, Listo cuando)

- **WHEN** existen dos productos activos con `sku` distintos y ambos tienen `codigos` con `valor` `"SHARED-OEM"`
- **AND** se llama `find_by_code` con `"SHARED-OEM"`
- **THEN** el resultado tiene longitud 2
- **AND** los dos `sku` distintos están representados

#### Scenario: HX-0841Z no devuelve HX-0840Z (unhappy, Listo cuando)

- **WHEN** existe un producto activo con código fabricante `"HX-0840Z"`
- **AND** se llama `find_by_code` con `"HX-0841Z"`
- **THEN** el resultado está vacío

#### Scenario: Match por SKU (happy)

- **WHEN** existe un producto activo con `sku` `"PER-M8X40-ZN"`
- **AND** se llama `find_by_code` con `"PER-M8X40-ZN"`
- **THEN** el resultado tiene un elemento cuyo `sku` es `"PER-M8X40-ZN"`

#### Scenario: Inactivo excluido (unhappy)

- **WHEN** un producto inactivo tiene OEM `"OFF-OEM"` y un activo tiene el mismo OEM
- **AND** se llama `find_by_code` con `"OFF-OEM"`
- **THEN** el resultado contiene solo el producto activo

#### Scenario: suite find_by_code en domain (happy)

- **WHEN** se ejecuta `cargo test -p domain product_repo_trait_exposes_find_by_code`
- **THEN** el test termina con código 0