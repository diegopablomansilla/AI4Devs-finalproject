# Spec Delta

## Purpose

Define la entidad `MovimientoStock`, las reglas de signo por tipo, el `motivo` obligatorio en `ajuste` y el puerto inmutable `StockLedger` en el crate `domain`, para que el adaptador (T-02-24) implemente un contrato hexagonal sin persistir en este ticket.

Ticket padre: T-02-17. Change: T-02-17. HU: HU-02-07.

## ADDED Requirements

### Requirement: Entidad MovimientoStock con identidad tipada

El crate `domain` MUST exponer una entidad `MovimientoStock` cuyos campos cubren el modelo [4.5.1](../../../../../docs/4-modelo.md#451-movimiento_stock) con nombres en español ([5.3](../../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id` (`Id<MovimientoStock>`), `producto` (`Id<Producto>`), `tipo` (`TipoMovimiento`), `cantidad` (`Decimal` ≠ 0), `stock_resultante` (`Decimal` ≥ 0), `motivo` (`Option<string>`), `venta` (`Option<Id<Venta>>`), `terminal` (`Option<string>`), `creado_en` (`Timestamp`). El `id` MUST ser un ULID de 26 caracteres **sin** prefijo de tabla (`movimiento_stock:`).

Construir un `MovimientoStock` MUST exigir las reglas de signo, `motivo` y `venta` de los requirements siguientes. `stock_resultante` menor que 0 MUST producir `DomainError` con `code` `negative_stock` y MUST NOT producir un `MovimientoStock`. `motivo`, si está presente, MUST tener longitud 1..=500 tras recortar espacios. MUST NOT persistir ni abrir SurrealDB. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT usar los nombres de API en inglés (`productId`, `quantity`, `resultingStock`, `reason`, `saleId`) como campos de la entidad.

#### Scenario: Construcción válida ingreso con terminal (happy, HU-02-07)

- **WHEN** se construye un `MovimientoStock` con `tipo` `ingreso`, `cantidad` `"20"`, `stock_resultante` `"30"`, `terminal` `"caja-1"` y sin `motivo`
- **THEN** la construcción termina sin error
- **AND** el `id` no contiene `:`
- **AND** `tipo` es `ingreso`
- **AND** `cantidad` es `"20"`
- **AND** `stock_resultante` es `"30"`
- **AND** `terminal` es `"caja-1"`

#### Scenario: stock_resultante negativo (unhappy)

- **WHEN** se intenta construir un `MovimientoStock` con `stock_resultante` menor que 0
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `negative_stock`
- **AND** no se produce un `MovimientoStock`

### Requirement: TipoMovimiento y reglas de signo por tipo

El crate `domain` MUST exponer `TipoMovimiento` con exactamente los valores del modelo [4.5.1](../../../../../docs/4-modelo.md#451-movimiento_stock): `inicial`, `ingreso`, `ajuste`, `venta` (API `initial`/`inbound`/`adjustment`/`sale` se traduce en T-02-25). Parsear un valor fuera de ese conjunto, incluido un valor en inglés del contrato, MUST fallar con `DomainError` `code` `invalid_input`.

Al construir un `MovimientoStock`, `cantidad` MUST cumplir:

- `inicial` e `ingreso`: `cantidad > 0`
- `ajuste`: `cantidad ≠ 0` (positivo o negativo)
- `venta`: `cantidad < 0`

`cantidad == 0` MUST fallar para cualquier tipo. Una violación de signo o de cero MUST producir `DomainError` `code` `invalid_input` con `FieldError.field` `cantidad` y MUST NOT producir un `MovimientoStock`. MUST NOT persistir. MUST NOT mapear a HTTP.

#### Scenario: inicial con cantidad positiva (happy)

- **WHEN** se construye un `MovimientoStock` con `tipo` `inicial` y `cantidad` `"50"`
- **THEN** la construcción termina sin error
- **AND** `tipo` es `inicial`
- **AND** `cantidad` es `"50"`

#### Scenario: ajuste negativo con motivo (happy)

- **WHEN** se construye un `MovimientoStock` con `tipo` `ajuste`, `cantidad` `"-2"` y `motivo` `"Conteo físico del pasillo 3"`
- **THEN** la construcción termina sin error
- **AND** `tipo` es `ajuste`
- **AND** `cantidad` es `"-2"`

#### Scenario: venta con cantidad negativa (happy)

- **WHEN** se construye un `MovimientoStock` con `tipo` `venta`, `cantidad` `"-1"` y un `venta` presente
- **THEN** la construcción termina sin error
- **AND** `tipo` es `venta`
- **AND** `cantidad` es `"-1"`

#### Scenario: ingreso con cantidad negativa (unhappy, Listo cuando)

- **WHEN** se intenta construir un `MovimientoStock` con `tipo` `ingreso` y `cantidad` `"-5"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `cantidad`
- **AND** no se produce un `MovimientoStock`

#### Scenario: inicial con cantidad negativa (unhappy, Listo cuando)

- **WHEN** se intenta construir un `MovimientoStock` con `tipo` `inicial` y `cantidad` `"-1"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `cantidad`
- **AND** no se produce un `MovimientoStock`

#### Scenario: venta con cantidad positiva (unhappy, Listo cuando)

- **WHEN** se intenta construir un `MovimientoStock` con `tipo` `venta` y `cantidad` `"3"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `cantidad`
- **AND** no se produce un `MovimientoStock`

#### Scenario: cantidad cero (unhappy, Listo cuando)

- **WHEN** se intenta construir un `MovimientoStock` con `cantidad` `"0"` (cualquier `tipo`)
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `cantidad`
- **AND** no se produce un `MovimientoStock`

#### Scenario: TipoMovimiento en inglés del contrato (unhappy)

- **WHEN** se intenta parsear `"inbound"` o `"adjustment"` como `TipoMovimiento`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `TipoMovimiento`

### Requirement: motivo obligatorio en ajuste

Cuando `tipo` es `ajuste`, `motivo` MUST estar presente y MUST ser no vacío tras recortar espacios. Un `ajuste` sin `motivo`, con `motivo` nulo o solo espacios MUST producir `DomainError` `code` `invalid_input` con `FieldError.field` `motivo` y MUST NOT producir un `MovimientoStock`. Para `inicial`, `ingreso` y `venta`, `motivo` MAY ser ausente. MUST NOT persistir. MUST NOT mapear a HTTP.

#### Scenario: ajuste con motivo (happy, HU-02-07)

- **WHEN** se construye un `MovimientoStock` con `tipo` `ajuste`, `cantidad` `"-8"` y `motivo` `"Conteo físico"`
- **THEN** la construcción termina sin error
- **AND** `motivo` es `"Conteo físico"`

#### Scenario: ajuste sin motivo (unhappy, HU-02-07, Listo cuando)

- **WHEN** se intenta construir un `MovimientoStock` con `tipo` `ajuste` y sin `motivo` (o con `motivo` vacío / solo espacios)
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `motivo`
- **AND** no se produce un `MovimientoStock`

### Requirement: venta obligatoria solo en tipo venta

Cuando `tipo` es `venta`, `venta` MUST ser un `Id<Venta>` presente (marcador de identidad; la entidad `Venta` llega en T-08-01). Cuando `tipo` no es `venta`, `venta` MUST ser ausente. Una violación MUST producir `DomainError` `code` `invalid_input` con `FieldError.field` `venta` y MUST NOT producir un `MovimientoStock`. MUST NOT persistir la tabla `venta` ni abrir SurrealDB.

#### Scenario: venta con referencia (happy)

- **WHEN** se construye un `MovimientoStock` con `tipo` `venta`, `cantidad` `"-1"` y un `Id<Venta>` presente
- **THEN** la construcción termina sin error
- **AND** `venta` está presente

#### Scenario: venta sin referencia (unhappy)

- **WHEN** se intenta construir un `MovimientoStock` con `tipo` `venta` y sin `venta`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `venta`
- **AND** no se produce un `MovimientoStock`

#### Scenario: ingreso con referencia de venta (unhappy)

- **WHEN** se intenta construir un `MovimientoStock` con `tipo` `ingreso` y un `venta` presente
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `venta`
- **AND** no se produce un `MovimientoStock`

### Requirement: Puerto StockLedger sin persistencia ni mutación

El crate `domain` MUST exponer el puerto `StockLedger` como *trait* con operaciones `append` y `list_by_producto`. Los `id` MUST ser `Id<MovimientoStock>` / `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `append` MUST aceptar un input con `producto`, `tipo`, `cantidad`, `motivo`, `venta` y `terminal` (sin `stock_resultante`: lo calcula el adaptador en T-02-24). El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT declarar `update`, `delete` ni cualquier operación que mutue un movimiento existente (HU-02-07 inmutabilidad; un error se corrige con un nuevo `ajuste`). MUST NOT escribir `producto.stock` ni eventos de outbox (T-02-24). MUST NOT paginar con cursor (T-00-21 / T-02-26). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara append y list_by_producto (happy)

- **WHEN** se inspecciona el puerto `StockLedger` en `domain`
- **THEN** expone `append` y `list_by_producto`
- **AND** los resultados de éxito usan `MovimientoStock` y `Id<Producto>`
- **AND** los resultados de error usan `DomainError`

#### Scenario: El puerto no expone update ni delete (unhappy, HU-02-07 inmutabilidad)

- **WHEN** se inspecciona el puerto `StockLedger` en `domain`
- **THEN** no expone `update` ni `delete`
- **AND** no existe un caso de uso de modificación ni borrado de un movimiento

#### Scenario: domain no persiste el ledger (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `CREATE` sobre la tabla `movimiento_stock`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `movimiento_ingreso_guarda_terminal_caja_1` | `domain` (`src/movimiento_stock.rs`) | unitario happy (HU-02-07) |
| `movimiento_inicial_cantidad_positiva` | `domain` | unitario happy |
| `movimiento_ajuste_negativo_con_motivo` | `domain` | unitario happy |
| `movimiento_venta_cantidad_negativa` | `domain` | unitario happy |
| `movimiento_rejects_ingreso_cantidad_negativa` | `domain` | unitario unhappy (Listo cuando) |
| `movimiento_rejects_inicial_cantidad_negativa` | `domain` | unitario unhappy (Listo cuando) |
| `movimiento_rejects_venta_cantidad_positiva` | `domain` | unitario unhappy (Listo cuando) |
| `movimiento_rejects_cantidad_cero` | `domain` | unitario unhappy (Listo cuando) |
| `tipo_movimiento_rejects_english_or_unknown` | `domain` | unitario unhappy |
| `movimiento_rejects_ajuste_sin_motivo` | `domain` | unitario unhappy (Listo cuando, HU-02-07) |
| `movimiento_rejects_stock_resultante_negativo` | `domain` | unitario unhappy |
| `movimiento_rejects_venta_sin_referencia` | `domain` | unitario unhappy |
| `movimiento_rejects_ingreso_con_venta` | `domain` | unitario unhappy |
| `stock_ledger_trait_exposes_append_and_list` | `domain` | unitario happy (dummy) |
| `stock_ledger_trait_has_no_update_or_delete` | `domain` | unitario unhappy (HU-02-07 inmutabilidad) |
