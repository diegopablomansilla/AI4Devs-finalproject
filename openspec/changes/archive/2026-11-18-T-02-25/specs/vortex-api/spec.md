# Spec Delta

Ticket padre: T-02-25. Change: T-02-25. HU: HU-02-07.

## ADDED Requirements

### Requirement: Endpoint POST /api/v1/products/{productId}/stock-movements

El router de `build_router` MUST registrar `POST /api/v1/products/{productId}/stock-movements` alineado con `docs/openapi.yaml` (`operationId` `registerStockMovement`). El Gateway MUST delegar en el servicio de stock inyectado en `AppState` (capability `catalog-stock`) y MUST NOT implementar aritmética de stock, outbox ni persistencia.

`POST` MUST aceptar cuerpo `StockMovementCreate` (`type` y `quantity` obligatorios; `reason` opcional) y responder **201** con `StockMovement`. `type` MUST ser solo `inbound` o `adjustment` (`inbound`→`ingreso`, `adjustment`→`ajuste`). `quantity` MUST ser string decimal del esquema `Decimal` (con signo, ≠ 0). `inbound` MUST exigir `quantity` > 0; `adjustment` MAY ser positivo o negativo. `reason` MUST mapearse a `motivo`; si `type` es `adjustment`, `reason` ausente, nulo o solo espacios MUST ser **400** `invalid_input` con `errors[].field` `reason`. La cabecera `X-Vectra-Terminal` (OpenAPI `Terminal`, `maxLength` 64) MUST copiarse a `NewMovimientoStock.terminal` si está presente y no vacía tras recortar espacios; si falta o queda vacía, `terminal` MUST ser ausente. Un valor de más de 64 caracteres MUST ser `400` `invalid_input` con `errors[].field` `X-Vectra-Terminal`. MUST NOT exigir `version`. MUST NOT incrementar `version` de la ficha. MUST NOT crear movimientos `initial` ni `sale`.

`{productId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `productId`. Id inexistente MUST ser `404` `not_found`. Un ajuste que dejaría stock negativo MUST ser `400` `negative_stock` (`application/problem+json`, título «El stock no puede ser negativo») y MUST NOT cambiar el stock. `type` fuera de `inbound`/`adjustment` (incl. `initial` y `sale`) MUST ser `400` `invalid_input`. `quantity` como número JSON MUST ser `400` `invalid_input`. Cuerpo sin `type` o `quantity` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de stock, esta ruta MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`StockService` + adaptador `StockLedger` / `MovimientoStockRepo` de `storage`). MUST NOT registrar `GET /api/v1/products/{productId}/stock-movements` (historial T-02-26). MUST NOT registrar `PATCH` ni `DELETE` sobre un movimiento (HU-02-07 inmutabilidad). MUST NOT reindexar (T-03-07). `docs/openapi.yaml` MUST declarar `503` en `POST /api/v1/products/{productId}/stock-movements`. MUST NOT declarar `GET` como implementado en este change (el path GET permanece en el YAML como contrato v2).

#### Scenario: Ingreso desde caja-1 (happy, HU-02-07, Listo cuando)

- **WHEN** existe un producto con stock `"10"`
- **AND** se envía `POST /api/v1/products/{id}/stock-movements` con `{ "type": "inbound", "quantity": "20" }` y cabecera `X-Vectra-Terminal: caja-1`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `type` `"inbound"`, `quantity` `"20"`, `resultingStock` `"30"` y `terminal` `"caja-1"`
- **AND** el JSON no tiene claves `tipo`, `cantidad`, `stock_resultante` ni `motivo`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 201 `application/json` no falla

#### Scenario: Ajuste a cero con motivo (happy, HU-02-07)

- **WHEN** existe un producto con stock `"5"`
- **AND** se envía `POST` con `{ "type": "adjustment", "quantity": "-5", "reason": "Conteo físico" }`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** el JSON tiene `type` `"adjustment"`, `quantity` `"-5"`, `resultingStock` `"0"` y `reason` `"Conteo físico"`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 201 `application/json` no falla

#### Scenario: Ajuste sin motivo (unhappy, HU-02-07, Listo cuando)

- **WHEN** se envía `POST` con `{ "type": "adjustment", "quantity": "-1" }` sin `reason` (o `reason` nulo / solo espacios)
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `reason`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 400 `application/problem+json` no falla

#### Scenario: Stock negativo (unhappy, HU-02-07, Listo cuando)

- **WHEN** existe un producto con stock `"5"`
- **AND** se envía `POST` con `{ "type": "adjustment", "quantity": "-8", "reason": "Conteo físico" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `negative_stock` y `title` "El stock no puede ser negativo"
- **AND** un `GET /api/v1/products/{id}` posterior sigue teniendo `stock` `"5"`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 400 `application/problem+json` no falla

#### Scenario: Producto desconocido (unhappy)

- **WHEN** se envía `POST /api/v1/products/{ulid-inexistente}/stock-movements` con un cuerpo `inbound` válido
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 404 `application/problem+json` no falla

#### Scenario: productId no es ULID (unhappy)

- **WHEN** se envía `POST /api/v1/products/not-a-ulid/stock-movements` con un cuerpo válido
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `productId`

#### Scenario: inbound con quantity negativa (unhappy)

- **WHEN** se envía `POST` con `{ "type": "inbound", "quantity": "-1" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `quantity`

#### Scenario: type initial (unhappy)

- **WHEN** se envía `POST` con `{ "type": "initial", "quantity": "1" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: type sale (unhappy)

- **WHEN** se envía `POST` con `{ "type": "sale", "quantity": "-1" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: quantity como número JSON (unhappy)

- **WHEN** se envía `POST` con `"quantity": 20` (número, no string)
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: X-Vectra-Terminal demasiado largo (unhappy)

- **WHEN** se envía `POST` con `X-Vectra-Terminal` de más de 64 caracteres
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `X-Vectra-Terminal`

#### Scenario: sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de stock
- **AND** se envía `POST /api/v1/products/{ulid}/stock-movements` con un cuerpo `inbound` mínimo
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 503 `application/problem+json` no falla

#### Scenario: GET historial no está registrado (unhappy, T-02-26)

- **WHEN** se envía `GET /api/v1/products/{id}/stock-movements`
- **THEN** la respuesta no es 200 con `StockMovementPage`
- **AND** el JSON no se presenta como historial paginado exitoso

#### Scenario: PATCH de un movimiento no existe (unhappy, HU-02-07 inmutabilidad)

- **WHEN** se envía `PATCH` a `/api/v1/products/{id}/stock-movements/{movementId}` o al collection path
- **THEN** la respuesta no es 200 de edición de movimiento
- **AND** no existe opción HTTP de editar un movimiento

#### Scenario: DELETE de un movimiento no existe (unhappy, HU-02-07 inmutabilidad)

- **WHEN** se envía `DELETE` a `/api/v1/products/{id}/stock-movements/{movementId}` o al collection path
- **THEN** la respuesta no es 204 ni 200 de borrado
- **AND** no existe opción HTTP de borrar un movimiento

### Requirement: DTO inglés de movimiento de stock según 5.3

El Gateway MUST traducir el contrato inglés al dominio español y de vuelta según [5.3](../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos). Campos de respuesta `StockMovement`: `id`, `productId`, `type`, `quantity`, `resultingStock`, `reason`, `saleId`, `terminal`, `createdAt`. `quantity` MUST ser string del esquema `Decimal`; `resultingStock` MUST ser string del esquema `NonNegativeDecimal`. MUST NOT serializar `producto`, `tipo`, `cantidad`, `stock_resultante`, `motivo`, `venta` ni `creado_en`.

`POST` MUST leer `type` → `tipo` (`inbound`→`ingreso`, `adjustment`→`ajuste`), `quantity` → `cantidad`, `reason` → `motivo`. Enums desconocidos MUST ser `400` `invalid_input`.

En `problem+json`, `errors[].field` MUST usar nombres de API (`type`, `quantity`, `reason`, `productId`, `X-Vectra-Terminal`, `body`). MUST NOT aparecer `tipo`, `cantidad` ni `motivo` en `errors[].field`.

#### Scenario: respuesta sin claves de dominio (happy)

- **WHEN** `POST` registra un inbound de `"20"`
- **THEN** el JSON de 201 tiene `id`, `productId`, `type` `"inbound"`, `quantity` string y `resultingStock` string
- **AND** no tiene `tipo`, `cantidad`, `stock_resultante` ni `motivo`

#### Scenario: errors[].field reason no motivo (unhappy, 5.3)

- **WHEN** el servicio rechaza un ajuste sin motivo con `FieldError.field` `motivo`
- **AND** el Gateway traduce ese error a HTTP
- **THEN** `errors[].field` es `reason`
- **AND** no aparece `motivo` en el cuerpo

### Requirement: Tests de integración de stock-movements contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-25. Listo cuando: escenarios de HU-02-07 como tests de `api` validados contra `openapi.yaml`. Cada respuesta JSON (éxito o error) MUST validarse con `assert_conforms` (T-09-04). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_stock_inbound_20_on_10_returns_201_with_terminal` | `api` | integración, happy, HU-02-07 / Listo cuando |
| `post_stock_adjustment_to_zero_returns_201` | `api` | integración, happy, HU-02-07 |
| `post_stock_adjustment_without_reason_returns_400` | `api` | integración, unhappy, HU-02-07 / Listo cuando |
| `post_stock_adjustment_minus_8_on_5_returns_400_negative_stock` | `api` | integración, unhappy, HU-02-07 / Listo cuando |
| `post_stock_unknown_product_returns_404` | `api` | integración, unhappy |
| `post_stock_invalid_product_id_returns_400` | `api` | integración, unhappy |
| `post_stock_inbound_negative_quantity_returns_400` | `api` | integración, unhappy |
| `post_stock_type_initial_returns_400` | `api` | integración, unhappy |
| `post_stock_type_sale_returns_400` | `api` | integración, unhappy |
| `post_stock_quantity_as_json_number_returns_400` | `api` | integración, unhappy |
| `post_stock_terminal_too_long_returns_400` | `api` | integración, unhappy |
| `post_stock_without_service_returns_503` | `api` | integración, unhappy |
| `get_stock_movements_is_not_page` | `api` | integración, unhappy, T-02-26 |
| `patch_stock_movement_is_not_registered` | `api` | integración, unhappy, HU-02-07 inmutabilidad |
| `delete_stock_movement_is_not_registered` | `api` | integración, unhappy, HU-02-07 inmutabilidad |
| `stock_problem_field_is_reason_not_motivo` | `api` | integración, unhappy, 5.3 |
| `vortex_serve_post_stock_unknown_product_returns_404` | `vortex` | binario, happy (servicio inyectado) |

#### Scenario: la suite de stock-movements cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 201/400/404/503 pasan `assert_conforms`

## MODIFIED Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST registrar las rutas de categorías (requirement «Endpoints /api/v1/categories»). El router MUST registrar las rutas de marcas (requirement «Endpoints /api/v1/brands»). El router MUST registrar las rutas de definiciones de atributo (requirement «Endpoints /api/v1/attribute-definitions»). El router MUST registrar las rutas de producto (requirement «Endpoints POST/GET/PATCH /api/v1/products»). El router MUST registrar `POST /api/v1/products/{productId}/stock-movements` (requirement «Endpoint POST /api/v1/products/{productId}/stock-movements»). MUST NOT registrar otras rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»). `AppState::new` / `with_health` MAY omitir el servicio de categorías, el de marcas, el de definiciones de atributo, el de producto y el de stock (los tests de `/health` y estáticos siguen construyéndose sin ellos).

#### Scenario: ruta inexistente responde 404 (happy)

- **WHEN** se envía `GET` a una ruta que el router no registra (p. ej. `/no-such-route`) sobre el router de `build_router`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`

#### Scenario: AppState se comparte entre peticiones (happy)

- **WHEN** se clona un `AppState` y se construyen peticiones contra el mismo router
- **THEN** ambas peticiones se atienden sin error de estado
- **AND** no hace falta un servicio de dominio de catálogo para construir el `AppState`

#### Scenario: GET /health está registrado (happy)

- **WHEN** se envía `GET /health` al router de `build_router` con el `AppState` por defecto
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`

#### Scenario: GET /metrics está registrado (happy)

- **WHEN** se envía `GET /metrics` al router de `build_router`
- **THEN** la respuesta no tiene estado 404
- **AND** el cuerpo no tiene `code` `not_found`

#### Scenario: GET / está registrado cuando hay index.html (happy)

- **WHEN** `AppState` apunta a un `{web}` con `index.html`
- **AND** se envía `GET /` al router de `build_router`
- **THEN** la respuesta no tiene estado 404
- **AND** el cuerpo no tiene `code` `not_found`

#### Scenario: GET /api/v1/categories está registrado (happy)

- **WHEN** se envía `GET /api/v1/categories` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`

#### Scenario: GET /api/v1/brands está registrado (happy)

- **WHEN** se envía `GET /api/v1/brands` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`

#### Scenario: GET /api/v1/attribute-definitions está registrado (happy)

- **WHEN** se envía `GET /api/v1/attribute-definitions` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`

#### Scenario: POST /api/v1/products está registrado (happy)

- **WHEN** se envía `POST /api/v1/products` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`

#### Scenario: POST stock-movements está registrado (happy)

- **WHEN** se envía `POST /api/v1/products/{ulid}/stock-movements` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`
