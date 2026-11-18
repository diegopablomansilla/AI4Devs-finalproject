# Spec Delta

Ticket padre: T-02-24. Change: T-02-24. HU: HU-02-07.

## ADDED Requirements

### Requirement: Adaptador StockLedger.append transaccional

El crate `storage` MUST implementar el puerto `StockLedger` (`append`, `list_by_producto`) contra las tablas `movimiento_stock` y `producto` ya migradas (T-02-15, T-02-14) usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla (`movimiento_stock:`, `producto:`). Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT reindexar Meilisearch (T-03-07). MUST NOT declarar `update` ni `delete` de un movimiento (HU-02-07 inmutabilidad). MUST NOT confirmar una venta (`CREATE venta`, `contador:venta`, varias líneas) — T-08-03. MUST NOT paginar `list_by_producto` con cursor (T-00-21 / T-02-26). MUST NOT reemplazar el movimiento `inicial` de `ProductRepo::create` (sigue inline).

`append` MUST persistir un `NewMovimientoStock` en **una** transacción conforme a [4.5.2](../../../../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción):

1. Validar signo de `cantidad`, `motivo` obligatorio en `ajuste` y regla de `venta` **antes** de abrir `Tx` (mismas reglas que `MovimientoStock::try_new`). Una violación MUST ser `invalid_input` y MUST NOT insertar movimiento, MUST NOT cambiar `producto.stock` ni emitir outbox.
2. Si el `producto` no existe, MUST fallar con `not_found` y MUST NOT escribir.
3. Dentro de la `Tx`, el motor MUST calcular `$nuevo = producto.stock + cantidad`. MUST NOT calcular el stock resultante en un read-modify-write fuera de la transacción.
4. Si `$nuevo < 0`, MUST ejecutar `THROW "stock_negativo"`. El mapper de `Tx` MUST exponer `DomainError` con `code` `negative_stock`. Tras el fallo, `producto.stock` MUST quedar igual que antes, MUST NOT existir el movimiento nuevo y MUST NOT existir el evento de outbox de ese `append`.
5. Si `$nuevo >= 0`: `CREATE movimiento_stock` con `tipo`, `cantidad`, `stock_resultante` = `$nuevo`, `motivo`, `venta` y `terminal` del input; `UPDATE` del producto con `stock` = `$nuevo`. MUST NOT incrementar `producto.version`. MUST NOT escribir otros campos de ficha.
6. MUST emitir en la **misma** transacción un `OutboxEvent` con `entidad` `producto`, `operacion` `upsert`, `registro` igual al `id` del producto y `motivo` `"stock"`.
7. El resultado de éxito MUST ser el `MovimientoStock` persistido (`id` sin prefijo, `stock_resultante` = `$nuevo`, `terminal` igual al input).

`append` MUST aceptar `tipo` `ingreso` y `ajuste` (HU-02-07). MAY persistir `inicial` o `venta` si pasan las reglas de dominio; MUST NOT crear la tabla ni la fila `venta`. Un producto inactivo MUST poder recibir movimientos (el ledger es independiente de `activo`).

`list_by_producto` MUST devolver los movimientos de ese producto ordenados de más reciente a más antiguo (`creado_en` descendente), o una lista vacía si no hay filas. MUST NOT fallar solo porque el producto no tenga movimientos.

#### Scenario: Ingreso incrementa stock y guarda terminal (happy, HU-02-07)

- **WHEN** un producto tiene `stock` `"10"` y `version` 1
- **AND** se llama `append` con `tipo` `ingreso`, `cantidad` `"20"` y `terminal` `"caja-1"`
- **THEN** la operación termina sin error
- **AND** `producto.stock` es `"30"`
- **AND** `producto.version` sigue siendo 1
- **AND** el movimiento tiene `tipo` `ingreso`, `cantidad` `"20"`, `stock_resultante` `"30"` y `terminal` `"caja-1"`
- **AND** el `id` del movimiento no contiene `:`
- **AND** existe un evento en `outbox` con `entidad` `producto`, `operacion` `upsert` y `motivo` `"stock"`

#### Scenario: Ajuste negativo con motivo deja stock en cero (happy, HU-02-07)

- **WHEN** un producto tiene `stock` `"5"`
- **AND** se llama `append` con `tipo` `ajuste`, `cantidad` `"-5"` y `motivo` `"Conteo físico"`
- **THEN** la operación termina sin error
- **AND** `producto.stock` es `"0"`
- **AND** el movimiento tiene `stock_resultante` `"0"` y el `motivo` indicado
- **AND** `producto.version` no cambia

#### Scenario: Dos ingresos concurrentes ambos aplicados (happy, Listo cuando)

- **WHEN** un producto tiene `stock` `"5"`
- **AND** dos `append` de `tipo` `ingreso` y `cantidad` `"10"` se ejecutan de forma concurrente
- **THEN** ambas operaciones terminan sin error
- **AND** `producto.stock` es `"25"`
- **AND** existen dos movimientos `ingreso` de cantidad `"10"` para ese producto
- **AND** hay dos eventos de outbox con `motivo` `"stock"` correspondientes a esos append
- **AND** `producto.version` no cambia

#### Scenario: Ajuste −8 sobre stock 5 es negative_stock (unhappy, HU-02-07, Listo cuando)

- **WHEN** un producto tiene `stock` `"5"` y un movimiento `inicial` previo
- **AND** se llama `append` con `tipo` `ajuste`, `cantidad` `"-8"` y `motivo` `"Conteo físico"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `negative_stock`
- **AND** `producto.stock` sigue siendo `"5"`
- **AND** no se inserta un movimiento nuevo
- **AND** no se inserta un evento de outbox para ese `append`
- **AND** `producto.version` no cambia

#### Scenario: Ajuste sin motivo es invalid_input (unhappy, HU-02-07)

- **WHEN** un producto tiene `stock` `"5"`
- **AND** se llama `append` con `tipo` `ajuste`, `cantidad` `"-1"` y `motivo` ausente
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `motivo`
- **AND** `producto.stock` sigue siendo `"5"`
- **AND** no se inserta un movimiento nuevo ni un evento de outbox

#### Scenario: append de producto inexistente (unhappy)

- **WHEN** se llama `append` con un `producto` que no existe
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** no se inserta fila en `movimiento_stock` ni en `outbox`

#### Scenario: ingreso con cantidad negativa (unhappy)

- **WHEN** un producto existe
- **AND** se llama `append` con `tipo` `ingreso` y `cantidad` `"-1"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `cantidad`
- **AND** el stock y el ledger no cambian

#### Scenario: list_by_producto ordena del más reciente al más antiguo (happy)

- **WHEN** un producto tiene un movimiento `inicial` y después un `ingreso`
- **AND** se llama `list_by_producto` con su `id`
- **THEN** la operación termina sin error
- **AND** el primer elemento es el `ingreso`
- **AND** el segundo es el `inicial`

#### Scenario: El adaptador no expone update ni delete (unhappy, HU-02-07 inmutabilidad)

- **WHEN** se inspecciona el adaptador de `StockLedger` en `storage`
- **THEN** no expone `update` ni `delete` de un movimiento
- **AND** un error se corrige con un nuevo `append` de tipo `ajuste`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `stock_ledger_append_ingreso_guarda_terminal_caja_1` | `storage` (`tests/stock_ledger.rs`) | integración happy (HU-02-07) |
| `stock_ledger_append_ajuste_a_cero_con_motivo` | `storage` (`tests/stock_ledger.rs`) | integración happy |
| `stock_ledger_append_does_not_increment_version` | `storage` (`tests/stock_ledger.rs`) | integración happy |
| `stock_ledger_concurrent_ingresos_both_apply` | `storage` (`tests/stock_ledger.rs`) | integración happy (Listo cuando) |
| `stock_ledger_list_by_producto_newest_first` | `storage` (`tests/stock_ledger.rs`) | integración happy |
| `stock_ledger_append_ajuste_minus_8_on_stock_5_is_negative_stock` | `storage` (`tests/stock_ledger.rs`) | integración unhappy (Listo cuando, HU-02-07) |
| `stock_ledger_append_ajuste_sin_motivo_is_invalid_input` | `storage` (`tests/stock_ledger.rs`) | integración unhappy (HU-02-07) |
| `stock_ledger_append_unknown_product_is_not_found` | `storage` (`tests/stock_ledger.rs`) | integración unhappy |
| `stock_ledger_append_ingreso_cantidad_negativa_is_invalid_input` | `storage` (`tests/stock_ledger.rs`) | integración unhappy |
| `stock_ledger_adapter_has_no_update_or_delete` | `storage` (`tests/stock_ledger.rs`) | integración unhappy (HU-02-07 inmutabilidad) |

## MODIFIED Requirements

### Requirement: Adaptador ProductRepo

El crate `storage` MUST implementar el puerto `ProductRepo` (`create`, `get`, `update`, `set_activo`) contra las tablas `producto` y `movimiento_stock` ya migradas (T-02-14, T-02-15) usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla (`producto:`). Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT reindexar Meilisearch (T-03-07). MUST NOT declarar listado (T-02-27). MUST NOT reemplazar el movimiento `inicial` de `create` por `StockLedger::append` (el alta sigue inline; el adaptador `StockLedger` es un tipo aparte). MUST NOT comprobar que categoría o marca existan y estén activas más allá de lo que el esquema `record<…>` ya exige (T-02-20). MUST NOT validar valores de atributo contra definiciones cargadas por el caso de uso (T-02-16 / T-02-20). MUST NOT hidratar ni persistir imágenes (T-02-30).

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
