# Spec Delta

Ticket padre: T-02-22. Change: T-02-22. HU: HU-02-06.

## ADDED Requirements

### Requirement: DELETE y reactivación de producto

El router de `build_router` MUST registrar `DELETE /api/v1/products/{productId}` alineado con `docs/openapi.yaml`. `DELETE` MUST dar de baja lógica (`activo = false`) vía `ProductGateway::deactivate` y responder **204** sin cuerpo. MUST NOT reindexar (T-03-07). MUST NOT borrar ledger, relaciones ni imágenes.

`PATCH /api/v1/products/{productId}` MUST aceptar `ProductUpdate.active`: `true` reactiva vía `reactivate`; `false` da de baja vía `deactivate`. `{ "version": N, "active": true }` (sin otros campos de ficha) MUST ser **200** `Product` con `active` verdadero. `{ "version": N, "active": false }` MUST ser la misma baja lógica que `DELETE` pero **200** `Product`. `version` MUST estar presente (esquema OpenAPI); un parche solo de `active` MUST NOT exigir que `version` coincida ni incrementarla. Si el parche trae `active` y campos de ficha, MUST aplicar `set_activo` y después el `update` de ficha (este sí usa `version` y la incrementa).

`GET` de un producto inactivo MUST seguir siendo **200** `Product` con `active` falso. `{productId}` inválido MUST ser **400** `invalid_input` field `productId`. Id inexistente MUST ser **404** `not_found`. Sin servicio inyectado, `DELETE` MUST ser **503** `unavailable`. `docs/openapi.yaml` MUST declarar `503` en `DELETE /api/v1/products/{productId}`.

#### Scenario: DELETE da de baja (happy, HU-02-06, Listo cuando)

- **WHEN** existe un producto activo
- **AND** se envía `DELETE /api/v1/products/{id}`
- **THEN** la respuesta tiene estado HTTP 204
- **AND** no hay cuerpo
- **AND** un `GET` posterior del mismo `id` es 200 con `active` falso

#### Scenario: PATCH reactiva (happy, HU-02-06, Listo cuando)

- **WHEN** existe un producto inactivo con `version` 1
- **AND** se envía `PATCH /api/v1/products/{id}` con `{ "version": 1, "active": true }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `active` verdadero
- **AND** `version` no cambia
- **AND** `stock` no cambia
- **AND** `assert_conforms` de `PATCH /api/v1/products/{productId}` 200 `application/json` no falla

#### Scenario: PATCH active false da de baja (happy)

- **WHEN** existe un producto activo con `version` 1
- **AND** se envía `PATCH` con `{ "version": 1, "active": false }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `active` falso

#### Scenario: GET de producto inactivo (happy)

- **WHEN** un producto fue dado de baja
- **AND** se envía `GET /api/v1/products/{id}`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `active` falso
- **AND** `assert_conforms` de `GET /api/v1/products/{productId}` 200 `application/json` no falla

#### Scenario: DELETE id desconocido (unhappy)

- **WHEN** se envía `DELETE /api/v1/products/{ulid-inexistente}`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** `assert_conforms` de `DELETE /api/v1/products/{productId}` 404 `application/problem+json` no falla

#### Scenario: DELETE productId no es ULID (unhappy)

- **WHEN** se envía `DELETE /api/v1/products/not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `productId`

#### Scenario: DELETE sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de producto
- **AND** se envía `DELETE /api/v1/products/{ulid}`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` de `DELETE /api/v1/products/{productId}` 503 `application/problem+json` no falla

#### Scenario: PATCH reactiva id desconocido (unhappy)

- **WHEN** se envía `PATCH` con `{ "version": 1, "active": true }` a un ULID inexistente
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

## MODIFIED Requirements

### Requirement: Endpoints POST/GET/PATCH /api/v1/products

El router de `build_router` MUST registrar `POST /api/v1/products` y `GET`, `PATCH` y `DELETE /api/v1/products/{productId}` alineados con `docs/openapi.yaml`. El Gateway MUST delegar en el servicio de producto inyectado en `AppState` (capability `catalog-producto`) y MUST NOT implementar validación de atributos, existencia de categoría/marca ni persistencia.

`POST` MUST crear con cuerpo `ProductCreate` (`sku`, `name`, `productType`, `categoryId` y `price` obligatorios; `description`/`brandId`/`attributes`/`codes`/`unitOfMeasure`/`location`/`initialStock` opcionales) y responder **201** con `Product`. MUST enviar la cabecera `Location` con valor `/api/v1/products/{id}` (el `id` del producto creado, ULID sin prefijo). `unitOfMeasure` ausente MUST interpretarse como `unit`. `attributes`/`codes` ausentes MUST interpretarse como `[]`. `initialStock` ausente MUST interpretarse como `"0"`. La cabecera `X-Vectra-Terminal` (OpenAPI `Terminal`, `maxLength` 64) MUST copiarse a `NewProducto.terminal` si está presente y no vacía tras recortar espacios; si falta o queda vacía, `terminal` MUST ser ausente. Un valor de más de 64 caracteres MUST ser `400` `invalid_input` con `errors[].field` `X-Vectra-Terminal`. MUST NOT crear el movimiento `inicial` en el Gateway (lo hace `ProductRepo::create`).

`GET /api/v1/products/{productId}` MUST devolver **200** `Product` o **404** `not_found`. MUST NOT filtrar por `active` (la ficha admin incluye inactivos).

`PATCH` MUST aplicar el cuerpo `ProductUpdate` (`version` obligatorio, `minProperties: 2`): mapear los campos de ficha presentes a `PatchProducto` y delegar en el servicio; el campo `active` MUST seguir el requirement «DELETE y reactivación de producto». El resultado de éxito de un parche de ficha MUST ser **200** `Product` con `version` incrementada. `stock` MUST NOT ser editable. Cuerpo con solo `version` (sin `active` ni otro campo de ficha) MUST ser `400` `invalid_input`.

`{productId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `productId`. Id inexistente MUST ser `404` `not_found`. SKU duplicado MUST ser `409` `duplicate_sku`. `version` desactualizada en un parche de ficha MUST ser `409` `version_conflict`. Valor de atributo incompatible MUST ser `400` `invalid_input` con `errors[].field` `attributes[n].value`. Categoría o marca inexistente o inactiva MUST ser `400` `invalid_input` con `errors[].field` `categoryId` o `brandId`. JSON malformado o `price` como número JSON MUST ser `400` `invalid_input`. Cuerpo `POST` sin `sku`/`name`/`productType`/`categoryId`/`price` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de producto, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`ProductService` + `ProductoRepo` + repos de categoría, marca y `AtributoDef`). MUST NOT registrar `GET /api/v1/products` (listado T-02-27). MUST NOT reindexar (T-03-07). `docs/openapi.yaml` MUST declarar `503` en `POST /api/v1/products` y en `GET`/`PATCH`/`DELETE /api/v1/products/{productId}`. MUST NOT declarar `GET` de listado.

#### Scenario: POST alta completa perno M8 x 40 (happy, HU-02-04)

- **WHEN** existen una categoría activa, una marca activa y definiciones de atributo
- **AND** se envía `POST /api/v1/products` con el ejemplo OpenAPI de `ProductCreate` (SKU `"PER-M8X40-ZN"`, name `"Perno hexagonal M8 x 40 zincado"`, `price` `"120.50"`, `initialStock` `"50"`)
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `Content-Type` es `application/json`
- **AND** la cabecera `Location` es `/api/v1/products/{id}` con el `id` del JSON
- **AND** el JSON tiene `sku` `"PER-M8X40-ZN"`, `name` `"Perno hexagonal M8 x 40 zincado"`, `active` verdadero, `version` 1, `stock` `"50"` y `price` string `"120.50"`
- **AND** el JSON no tiene claves `nombre`, `tipo`, `precio`, `categoria` ni `marca`
- **AND** `images` es `[]`
- **AND** `assert_conforms` de `POST /api/v1/products` 201 `application/json` no falla

#### Scenario: POST con X-Vectra-Terminal (happy)

- **WHEN** se envía `POST /api/v1/products` con `initialStock` `"50"` y cabecera `X-Vectra-Terminal: caja-1`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** el movimiento inicial persiste `terminal` `"caja-1"` (vía `NewProducto.terminal`)

#### Scenario: GET ficha (happy)

- **WHEN** existe un producto creado
- **AND** se envía `GET /api/v1/products/{id}`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON es `Product` con `id`, `sku`, `name`, `category.path`, `version` y `stock` como string
- **AND** `assert_conforms` de `GET /api/v1/products/{productId}` 200 `application/json` no falla

#### Scenario: PATCH precio incrementa version (happy, HU-02-05)

- **WHEN** existe un producto con `version` 1 y `price` `"120.50"`
- **AND** se envía `PATCH /api/v1/products/{id}` con `{ "version": 1, "price": "130.00" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `price` `"130.00"` y `version` 2
- **AND** `stock` no cambia
- **AND** `assert_conforms` de `PATCH /api/v1/products/{productId}` 200 `application/json` no falla

#### Scenario: PATCH parcial conserva el resto (happy)

- **WHEN** existe un producto con `name` y `sku`
- **AND** se envía `PATCH` solo con `version` y un nuevo `price`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `name` y `sku` se conservan
- **AND** `version` se incrementa en 1

#### Scenario: POST SKU duplicado (unhappy, HU-02-04, Listo cuando)

- **WHEN** ya existe un producto con SKU `"PER-M8X40-ZN"`
- **AND** se envía `POST` con el mismo `sku`
- **THEN** la respuesta tiene estado HTTP 409
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `duplicate_sku`
- **AND** `assert_conforms` de `POST /api/v1/products` 409 `application/problem+json` no falla

#### Scenario: POST atributo cuarenta (unhappy, HU-02-04, Listo cuando)

- **WHEN** existe la definición "largo" de tipo `number`
- **AND** se envía `POST` cuyo `attributes[n].value` es `"cuarenta"`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `attributes[n].value`
- **AND** `assert_conforms` de `POST /api/v1/products` 400 `application/problem+json` no falla

#### Scenario: PATCH version_conflict (unhappy, HU-02-05, Listo cuando)

- **WHEN** existe un producto con `version` 1
- **AND** otra edición ya persistió `version` 2
- **AND** se envía `PATCH` con `{ "version": 1, "price": "140.00" }`
- **THEN** la respuesta tiene estado HTTP 409
- **AND** el JSON tiene `code` `version_conflict`
- **AND** `assert_conforms` de `PATCH /api/v1/products/{productId}` 409 `application/problem+json` no falla

#### Scenario: GET id desconocido (unhappy)

- **WHEN** se envía `GET /api/v1/products/{ulid-inexistente}`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** `assert_conforms` de `GET /api/v1/products/{productId}` 404 `application/problem+json` no falla

#### Scenario: PATCH id desconocido (unhappy)

- **WHEN** se envía `PATCH /api/v1/products/{ulid-inexistente}` con `{ "version": 1, "price": "1.00" }`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: productId no es ULID (unhappy)

- **WHEN** se envía `GET /api/v1/products/not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `productId`

#### Scenario: POST sin name (unhappy)

- **WHEN** se envía `POST /api/v1/products` con JSON que omite `name`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: POST price como número JSON (unhappy)

- **WHEN** se envía `POST` con `"price": 120.5` (número, no string)
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: PATCH con active (unhappy, T-02-22)

- **WHEN** se envía `PATCH` con `{ "version": 1, "active": true }` a un producto existente
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `active` verdadero
- **AND** `assert_conforms` de `PATCH /api/v1/products/{productId}` 200 `application/json` no falla

#### Scenario: X-Vectra-Terminal demasiado largo (unhappy)

- **WHEN** se envía `POST` con `X-Vectra-Terminal` de más de 64 caracteres
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `X-Vectra-Terminal`

#### Scenario: sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de producto
- **AND** se envía `POST /api/v1/products` con un cuerpo `ProductCreate` mínimo
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` de `POST /api/v1/products` 503 `application/problem+json` no falla

#### Scenario: GET listado no está registrado (unhappy, T-02-27)

- **WHEN** se envía `GET /api/v1/products` (sin `{productId}`)
- **THEN** la respuesta no es 200 con `ProductSummaryPage`
- **AND** el JSON no se presenta como listado admin exitoso

### Requirement: Tests de integración de productos contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-22. Listo cuando: 204, ledger intacto (storage) y reactivación; cada respuesta JSON (éxito o error) se valida con `assert_conforms` (T-09-04) salvo `DELETE` 204 (sin cuerpo). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_product_perno_m8x40_returns_201_with_location` | `api` | integración, happy, HU-02-04 |
| `post_product_sends_terminal_on_initial_stock` | `api` | integración, happy |
| `get_product_returns_200` | `api` | integración, happy |
| `patch_product_price_increments_version` | `api` | integración, happy, HU-02-05 |
| `patch_product_partial_keeps_other_fields` | `api` | integración, happy |
| `post_product_duplicate_sku_returns_409` | `api` | integración, unhappy, HU-02-04 |
| `post_product_cuarenta_on_number_attribute_returns_400` | `api` | integración, unhappy, HU-02-04 |
| `patch_product_stale_version_returns_409` | `api` | integración, unhappy, HU-02-05 |
| `get_product_unknown_id_returns_404` | `api` | integración, unhappy |
| `patch_product_unknown_id_returns_404` | `api` | integración, unhappy |
| `product_id_invalid_ulid_returns_400` | `api` | integración, unhappy |
| `post_product_missing_name_returns_400` | `api` | integración, unhappy |
| `post_product_price_as_json_number_returns_400` | `api` | integración, unhappy |
| `post_product_terminal_too_long_returns_400` | `api` | integración, unhappy |
| `post_product_without_service_returns_503` | `api` | integración, unhappy |
| `get_products_list_is_not_summary_page` | `api` | integración, unhappy, T-02-27 |
| `product_problem_field_is_category_id_not_categoria` | `api` | integración, unhappy, 5.3 |
| `delete_product_returns_204` | `api` | integración, happy, HU-02-06 / Listo cuando |
| `patch_product_active_true_reactivates` | `api` | integración, happy, HU-02-06 / Listo cuando |
| `patch_product_active_false_deactivates` | `api` | integración, happy |
| `get_inactive_product_returns_200` | `api` | integración, happy |
| `delete_product_unknown_id_returns_404` | `api` | integración, unhappy |
| `delete_product_invalid_ulid_returns_400` | `api` | integración, unhappy |
| `delete_product_without_service_returns_503` | `api` | integración, unhappy |
| `patch_product_active_unknown_id_returns_404` | `api` | integración, unhappy |
| `vortex_serve_get_unknown_product_returns_404` | `vortex` | binario, happy (servicio inyectado) |
| `vortex_serve_delete_unknown_product_returns_404` | `vortex` | binario, happy (servicio inyectado) |

#### Scenario: la suite de productos cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 200/201/400/404/409/503 pasan `assert_conforms`
- **AND** `DELETE` 204 no tiene cuerpo JSON
