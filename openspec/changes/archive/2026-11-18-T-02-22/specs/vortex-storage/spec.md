# Spec Delta

Ticket padre: T-02-22. Change: T-02-22. HU: HU-02-06.

## ADDED Requirements

### Requirement: ProductRepo set_activo conserva historial

El adaptador `ProductRepo` en `storage` MUST implementar `set_activo(id, activo)`. MUST actualizar solo el campo `activo` (y `actualizado_en`) de esa fila `producto` en una `Tx` con un `OutboxEvent` `producto`/`upsert` (`registro` = `id` sin prefijo, `motivo` ausente). MUST NOT hacer `DELETE` físico de la fila `producto`. MUST NOT borrar, alterar ni insertar filas en `movimiento_stock`. MUST NOT borrar aristas de relación ni filas `imagen_producto`. MUST NOT escribir `stock`. MUST NOT incrementar `version`. MUST NOT reindexar Meilisearch (T-03-07).

El resultado de éxito MUST ser la `FichaProducto` rehidratada con el `activo` pedido. `id` desconocido MUST ser `not_found` y MUST NOT emitir outbox. Si el producto ya tiene ese `activo`, `set_activo` MUST ser éxito idempotente (sigue emitiendo el evento de outbox).

#### Scenario: Baja lógica deja el ledger intacto (happy, HU-02-06, Listo cuando)

- **WHEN** existe un producto activo con stock `"50"` y un movimiento `inicial`
- **AND** se llama `set_activo(id, false)`
- **THEN** la operación termina sin error
- **AND** `activo` es falso
- **AND** `stock` sigue siendo `"50"`
- **AND** `version` no cambia
- **AND** el recuento de `movimiento_stock` de ese producto no cambia
- **AND** el movimiento `inicial` conserva `tipo`, `cantidad` y `stock_resultante`
- **AND** existe un evento en `outbox` con `entidad` `producto`, `operacion` `upsert` y `motivo` ausente

#### Scenario: Reactivación restaura activo (happy, HU-02-06, Listo cuando)

- **WHEN** un producto está inactivo
- **AND** se llama `set_activo(id, true)`
- **THEN** la operación termina sin error
- **AND** `activo` es verdadero
- **AND** `stock` y el ledger no cambian
- **AND** existe un evento en `outbox` con `entidad` `producto` y `operacion` `upsert`

#### Scenario: Baja idempotente (happy)

- **WHEN** un producto ya está inactivo
- **AND** se llama `set_activo(id, false)`
- **THEN** la operación termina sin error
- **AND** `activo` sigue falso
- **AND** el ledger no cambia

#### Scenario: set_activo id desconocido (unhappy)

- **WHEN** se llama `set_activo` con un `id` que no existe en `producto`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** no se inserta un evento en `outbox`

## MODIFIED Requirements

### Requirement: Adaptador ProductRepo

El crate `storage` MUST implementar el puerto `ProductRepo` (`create`, `get`, `update`, `set_activo`) contra las tablas `producto` y `movimiento_stock` ya migradas (T-02-14, T-02-15) usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla (`producto:`). Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT reindexar Meilisearch (T-03-07). MUST NOT declarar listado (T-02-27). MUST NOT implementar `StockLedger::append` para `ingreso`/`ajuste`/`venta` (T-02-24). MUST NOT comprobar que categoría o marca existan y estén activas más allá de lo que el esquema `record<…>` ya exige (T-02-20). MUST NOT validar valores de atributo contra definiciones cargadas por el caso de uso (T-02-16 / T-02-20). MUST NOT hidratar ni persistir imágenes (T-02-30).

`create` MUST persistir los campos de `NewProducto` en una sola transacción:

1. `CREATE producto` con `activo` verdadero, `version` 1 y `stock` 0 (el DEFAULT del esquema); MUST NOT escribir `stock` distinto de 0 en el `CONTENT` del alta.
2. Si `stock_inicial > 0`: `CREATE movimiento_stock` con `tipo` `inicial`, `cantidad` = `stock_inicial`, `stock_resultante` = `stock_inicial`, `motivo` y `venta` ausentes, y `terminal` igual a `NewProducto.terminal` (ausente si `None`); y `UPDATE` del producto con `stock` = `stock_inicial`. MUST NOT incrementar `version`.
3. Si `stock_inicial` es 0: MUST NOT insertar fila en `movimiento_stock`.
4. MUST emitir en la **misma** transacción un `OutboxEvent` con `entidad` `producto`, `operacion` `upsert` y `registro` igual al `id` del producto creado. Si hubo movimiento `inicial`, `motivo` del evento MUST ser `"stock"`; si `stock_inicial` es 0, `motivo` MUST ser ausente.

Si `stock_inicial < 0`, `create` MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT insertar producto, movimiento ni evento. Una violación del índice único `sku` MUST fallar con `DomainError::Conflict` y `code` `duplicate_sku`, y MUST NOT dejar la segunda fila, su movimiento ni su evento de outbox. `sku` o `nombre` vacíos MUST ser `invalid_input` y MUST NOT insertar.

`get` MUST devolver `FichaProducto` o `DomainError::NotFound`:

- `producto` con los campos persistidos de [4.4.4](../../../../../docs/4-modelo.md#444-producto)
- `marca` hidratada (`Marca` con `nombre`) cuando el producto tiene `marca`; ausente si no hay marca
- `categoria` hidratada con `ruta` (nombres desde la raíz hasta la categoría del producto)
- `atributos` resueltos: cada par `def`/`valor` MUST incluir `clave`, `etiqueta` y `unidad` de su `AtributoDef`
- MUST NOT incluir imágenes
- MUST NOT filtrar por `activo` (GET de ficha admin incluye inactivos)

`update` MUST persistir la ficha de `UpdateProducto` en una sola transacción:

1. `UPDATE` del producto con los campos de ficha (`sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `unidad_medida`, `precio`, `ubicacion`) y `version += 1` **WHERE** `version` es la `version` del input ([4.3](../../../../../docs/4-modelo.md#43-convenciones), [5.5](../../../../../docs/5-contract-api.md#55-concurrencia-bloqueo-optimista))
2. `atributos` y `codigos` MUST reemplazarse por los arrays completos del input (no se fusionan elemento a elemento)
3. MUST NOT escribir `stock` ni `activo`
4. MUST emitir en la **misma** transacción un `OutboxEvent` con `entidad` `producto`, `operacion` `upsert`, `registro` igual al `id` y `motivo` ausente
5. El resultado de éxito MUST ser la `FichaProducto` rehidratada (marca, ruta, atributos resueltos) con `version` incrementada en 1

Si el `id` no existe, `update` MUST fallar con `DomainError` `code` `not_found` y MUST NOT emitir outbox. Si el `UPDATE` no afecta filas (versión desactualizada), MUST fallar con `DomainError::Conflict` y `code` `version_conflict`, MUST NOT cambiar la fila ni emitir outbox. Una violación del índice único `sku` MUST fallar con `DomainError::Conflict` y `code` `duplicate_sku`. `sku` o `nombre` vacíos MUST ser `invalid_input`.

`set_activo` MUST seguir el requirement «ProductRepo set_activo conserva historial».

#### Scenario: Alta con stock inicial 50 (happy, HU-02-04, Listo cuando)

- **WHEN** existe una categoría (y opcionalmente marca y definiciones de atributo) y se llama `create` con SKU `"PER-M8X40-ZN"`, nombre `"Perno hexagonal M8 x 40 zincado"`, precio `"120.50"` y `stock_inicial` `"50"`
- **THEN** la operación termina sin error
- **AND** `version` es 1
- **AND** `activo` es verdadero
- **AND** `stock` es `"50"`
- **AND** el `id` no contiene `:`
- **AND** existe un movimiento `movimiento_stock` con `tipo` `inicial`, `cantidad` `"50"` y `stock_resultante` `"50"`
- **AND** el movimiento tiene `terminal` ausente
- **AND** existe un evento en `outbox` con `entidad` `producto`, `operacion` `upsert` y `motivo` `"stock"`
- **AND** `registro` del evento es el `id` del producto (sin prefijo de tabla)

#### Scenario: Alta con terminal en el movimiento inicial (happy, T-02-21)

- **WHEN** se llama `create` con `stock_inicial` `"50"` y `terminal` `"caja-1"`
- **THEN** la operación termina sin error
- **AND** el movimiento `inicial` tiene `terminal` `"caja-1"`

#### Scenario: Alta con stock inicial 0 no crea movimiento (happy)

- **WHEN** se llama `create` con `stock_inicial` `"0"` y un SKU único
- **THEN** la operación termina sin error
- **AND** `stock` es `"0"`
- **AND** `version` es 1
- **AND** no existe fila en `movimiento_stock` para ese producto
- **AND** existe un evento en `outbox` con `entidad` `producto`, `operacion` `upsert` y `motivo` ausente

#### Scenario: get de producto existente (happy)

- **WHEN** se persiste un producto y se llama `get` con su `id`
- **THEN** la operación termina sin error
- **AND** `sku`, `nombre`, `stock`, `version` e `id` coinciden con lo persistido

#### Scenario: get hidrata marca, ruta de categoría y atributos (happy, HU-02-05)

- **WHEN** existe una categoría hija con `ruta` `["Fijaciones", "Pernos"]`, una marca `"Fixser"`, un `AtributoDef` `medida` (`etiqueta` `"Medida"`, `unidad` `"mm"`) y un producto asociado a esos maestros con atributo valor `"M8"`
- **AND** se llama `get` con el `id` del producto
- **THEN** la operación termina sin error
- **AND** `marca.nombre` es `"Fixser"`
- **AND** `categoria.ruta` es `["Fijaciones", "Pernos"]`
- **AND** el atributo resuelto tiene `clave` `"medida"`, `etiqueta` `"Medida"`, `unidad` `"mm"` y valor `"M8"`
- **AND** no se hidratan imágenes

#### Scenario: Edición incrementa version (happy, HU-02-05)

- **WHEN** un producto tiene `version` 1 y se llama `update` con esa `version` y un precio `"130.00"`
- **THEN** la operación termina sin error
- **AND** el precio persistido es `"130.00"`
- **AND** `version` es 2
- **AND** existe un evento en `outbox` con `entidad` `producto`, `operacion` `upsert` y `motivo` ausente

#### Scenario: Edición no escribe stock (happy, HU-02-05)

- **WHEN** un producto tiene `stock` `"50"` y `version` 1
- **AND** se llama `update` con esa `version` cambiando el precio
- **THEN** la operación termina sin error
- **AND** `stock` sigue siendo `"50"`
- **AND** no se inserta un movimiento en `movimiento_stock` por esa edición

#### Scenario: Edición reemplaza atributos y códigos completos (happy)

- **WHEN** un producto tiene un código `fabricante` `"HX-0840Z"` y un atributo
- **AND** se llama `update` con `codigos` `[{ tipo: oem, valor: "OEM-1" }]` y `atributos` vacío
- **THEN** la operación termina sin error
- **AND** `codigos` es solo el OEM
- **AND** `atributos` es vacío

#### Scenario: Dos ediciones con la misma version (unhappy, HU-02-05, Listo cuando)

- **WHEN** un producto tiene `version` 1
- **AND** dos `update` se presentan con `version` 1 (ediciones concurrentes o la segunda tras el commit de la primera)
- **THEN** exactamente una operación termina sin error y deja `version` 2
- **AND** la otra falla
- **AND** el error de la que falla es `DomainError::Conflict` con `code` `version_conflict`
- **AND** el precio (u otro campo de ficha) persistido es el de la edición que pasó
- **AND** no hay un segundo evento de outbox de la edición que falló

#### Scenario: SKU duplicado (unhappy, HU-02-04)

- **WHEN** ya existe un producto con SKU `"PER-M8X40-ZN"` y se intenta crear otro con el mismo SKU
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_sku`
- **AND** no queda una segunda fila en `producto`
- **AND** no queda un segundo movimiento ni un segundo evento de outbox para esa alta

#### Scenario: stock_inicial negativo (unhappy)

- **WHEN** se llama `create` con `stock_inicial` menor que 0
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se inserta fila en `producto`, `movimiento_stock` ni `outbox`

#### Scenario: get de id inexistente (unhappy)

- **WHEN** se llama `get` con un `id` que no existe en `producto`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

#### Scenario: update de id inexistente (unhappy)

- **WHEN** se llama `update` con un `id` que no existe en `producto`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** no se inserta un evento en `outbox`

#### Scenario: update con SKU duplicado (unhappy)

- **WHEN** existen dos productos con SKU distintos y `update` cambia el SKU del segundo al del primero
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_sku`
- **AND** el SKU del segundo producto no cambia
- **AND** no se inserta un evento de outbox para esa edición
