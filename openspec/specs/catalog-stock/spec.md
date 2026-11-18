# catalog-stock Specification

## Purpose

Orquesta en el crate `catalog` el registro de ingresos y ajustes de stock sobre el puerto `StockLedger`, con fachada object-safe para el Gateway, sin HTTP ni persistencia.

Ticket padre: T-02-25. Change: T-02-25. HU: HU-02-07.

## Requirements

### Requirement: Registrar ingreso o ajuste de stock

El crate `catalog` MUST exponer un caso de uso que registra un movimiento de stock delegando en `StockLedger::append`. MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT validar `maxLength` 64 de la terminal (Gateway). MUST NOT escribir `producto.stock` ni el outbox por sí mismo (lo hace el adaptador T-02-24).

El input MUST incluir `producto` (`Id<Producto>`), `tipo` (`TipoMovimiento`), `cantidad` (`Decimal`), `motivo` opcional y `terminal` opcional. MUST reenviar `terminal` al puerto sin transformarlo. MUST NOT aceptar `venta` (el POS es T-08-03).

`tipo` MUST ser `ingreso` o `ajuste`. `inicial` o `venta` MUST fallar con `DomainError` `code` `invalid_input` y `FieldError.field` `tipo`, sin llamar a `append`. Las reglas de signo, `motivo` obligatorio en `ajuste` y `cantidad ≠ 0` MUST propagarse del puerto / dominio (`invalid_input` con `field` `cantidad` o `motivo`). `negative_stock` y `not_found` del puerto MUST propagarse sin remapeo.

El resultado de éxito MUST ser el `MovimientoStock` que devuelve `append` (con `stock_resultante` calculado por el ledger). MUST NOT incrementar `version` de la ficha. MUST NOT listar el historial (T-02-26). MUST NOT reindexar (T-03-07).

#### Scenario: Ingreso con terminal (happy, HU-02-07)

- **WHEN** existe un producto con stock 10
- **AND** el caso de uso registra `tipo` `ingreso`, `cantidad` `"20"` y `terminal` `"caja-1"`
- **THEN** la operación termina sin error
- **AND** `tipo` es `ingreso`
- **AND** `cantidad` es `"20"`
- **AND** `stock_resultante` es `"30"`
- **AND** `terminal` es `"caja-1"`

#### Scenario: Ajuste a cero con motivo (happy, HU-02-07)

- **WHEN** existe un producto con stock 5
- **AND** el caso de uso registra `tipo` `ajuste`, `cantidad` `"-5"` y `motivo` `"Conteo físico"`
- **THEN** la operación termina sin error
- **AND** `stock_resultante` es `"0"`
- **AND** `motivo` es `"Conteo físico"`

#### Scenario: Ajuste sin motivo (unhappy, HU-02-07)

- **WHEN** el caso de uso registra `tipo` `ajuste` sin `motivo` (o con `motivo` vacío / solo espacios)
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `motivo`
- **AND** no se llama a `append` con un movimiento inválido, o el puerto rechaza y no persiste

#### Scenario: Ajuste que deja stock negativo (unhappy, HU-02-07)

- **WHEN** existe un producto con stock 5
- **AND** el caso de uso registra `tipo` `ajuste`, `cantidad` `"-8"` y `motivo` no vacío
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `negative_stock`
- **AND** el stock del producto sigue en 5

#### Scenario: Tipo inicial rechazado (unhappy)

- **WHEN** el caso de uso recibe `tipo` `inicial`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `tipo`
- **AND** no se persiste un movimiento

#### Scenario: Tipo venta rechazado (unhappy)

- **WHEN** el caso de uso recibe `tipo` `venta`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `tipo`
- **AND** no se persiste un movimiento

#### Scenario: Producto inexistente (unhappy)

- **WHEN** el `id` de producto no existe
- **AND** el caso de uso registra un `ingreso` válido
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

#### Scenario: Ingreso con cantidad negativa (unhappy)

- **WHEN** el caso de uso registra `tipo` `ingreso` y `cantidad` `"-1"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `cantidad`

### Requirement: Tests en memoria del caso de uso

Los tests del crate `catalog` MUST usar un `StockLedger` en memoria (sin SurrealDB). MUST cubrir ingreso, ajuste, caminos no felices y el rechazo de `inicial`/`venta`. El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`.

| Test | Crate | Tipo |
|---|---|---|
| `catalog_register_inbound_sets_resulting_stock_and_terminal` | `catalog` | unitario (memoria), happy, HU-02-07 |
| `catalog_register_adjustment_to_zero_with_reason` | `catalog` | unitario (memoria), happy, HU-02-07 |
| `catalog_register_ajuste_sin_motivo_is_invalid_input` | `catalog` | unitario (memoria), unhappy, HU-02-07 |
| `catalog_register_ajuste_minus_8_on_stock_5_is_negative_stock` | `catalog` | unitario (memoria), unhappy, HU-02-07 |
| `catalog_register_inicial_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_register_venta_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_register_unknown_product_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_register_ingreso_cantidad_negativa_is_invalid_input` | `catalog` | unitario (memoria), unhappy |

#### Scenario: Suite en memoria (happy)

- **WHEN** se ejecuta `cargo test -p catalog` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** ninguno abre SurrealDB

### Requirement: Fachada object-safe para el Gateway

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) del caso de uso de registro de stock para que el Gateway la inyecte como `Arc<dyn …>`. El servicio MUST implementar esa fachada. MUST NOT mapear a HTTP ni abrir SurrealDB. MUST NOT exponer `list_by_producto`, `update` ni `delete` (HU-02-07 inmutabilidad; historial es T-02-26).

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye el servicio de stock con un `StockLedger` en memoria
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones
- **AND** registrar un ingreso válido vía la fachada termina sin error

#### Scenario: la fachada no expone update ni delete (unhappy, HU-02-07)

- **WHEN** se inspecciona la fachada de stock en `catalog`
- **THEN** no expone `update` ni `delete` de un movimiento
- **AND** no expone listado del historial
