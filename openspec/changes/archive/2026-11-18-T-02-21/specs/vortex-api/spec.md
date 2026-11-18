# Spec Delta

Ticket padre: T-02-21. Change: T-02-21. HU: HU-02-04, HU-02-05.

## ADDED Requirements

### Requirement: Endpoints POST/GET/PATCH /api/v1/products

El router de `build_router` MUST registrar `POST /api/v1/products` y `GET` y `PATCH /api/v1/products/{productId}` alineados con `docs/openapi.yaml`. El Gateway MUST delegar en el servicio de producto inyectado en `AppState` (capability `catalog-producto`) y MUST NOT implementar validación de atributos, existencia de categoría/marca ni persistencia.

`POST` MUST crear con cuerpo `ProductCreate` (`sku`, `name`, `productType`, `categoryId` y `price` obligatorios; `description`/`brandId`/`attributes`/`codes`/`unitOfMeasure`/`location`/`initialStock` opcionales) y responder **201** con `Product`. MUST enviar la cabecera `Location` con valor `/api/v1/products/{id}` (el `id` del producto creado, ULID sin prefijo). `unitOfMeasure` ausente MUST interpretarse como `unit`. `attributes`/`codes` ausentes MUST interpretarse como `[]`. `initialStock` ausente MUST interpretarse como `"0"`. La cabecera `X-Vectra-Terminal` (OpenAPI `Terminal`, `maxLength` 64) MUST copiarse a `NewProducto.terminal` si está presente y no vacía tras recortar espacios; si falta o queda vacía, `terminal` MUST ser ausente. Un valor de más de 64 caracteres MUST ser `400` `invalid_input` con `errors[].field` `X-Vectra-Terminal`. MUST NOT crear el movimiento `inicial` en el Gateway (lo hace `ProductRepo::create`).

`GET /api/v1/products/{productId}` MUST devolver **200** `Product` o **404** `not_found`. MUST NOT filtrar por `active` (la baja es T-02-22).

`PATCH` MUST aplicar el cuerpo `ProductUpdate` (`version` obligatorio, `minProperties: 2`): mapear los campos presentes a `PatchProducto` y delegar en el servicio. El resultado de éxito MUST ser **200** `Product` con `version` incrementada. `stock` MUST NOT ser editable. El campo `active` MUST producir `400` `invalid_input` con `errors[].field` `active` (reactivación es T-02-22). Cuerpo con solo `version` (sin otro campo de ficha) MUST ser `400` `invalid_input`.

`{productId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `productId`. Id inexistente MUST ser `404` `not_found`. SKU duplicado MUST ser `409` `duplicate_sku`. `version` desactualizada MUST ser `409` `version_conflict`. Valor de atributo incompatible MUST ser `400` `invalid_input` con `errors[].field` `attributes[n].value`. Categoría o marca inexistente o inactiva MUST ser `400` `invalid_input` con `errors[].field` `categoryId` o `brandId`. JSON malformado o `price` como número JSON MUST ser `400` `invalid_input`. Cuerpo `POST` sin `sku`/`name`/`productType`/`categoryId`/`price` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de producto, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`ProductService` + `ProductoRepo` + repos de categoría, marca y `AtributoDef`). MUST NOT registrar `GET /api/v1/products` (listado T-02-27). MUST NOT registrar `DELETE /api/v1/products/{productId}` (T-02-22). MUST NOT reindexar (T-03-07). `docs/openapi.yaml` MUST declarar `503` en `POST /api/v1/products` y en `GET`/`PATCH /api/v1/products/{productId}`. MUST NOT declarar `GET` de listado ni `DELETE` en este change.

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

- **WHEN** se envía `PATCH` con `{ "version": 1, "active": true }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `active`

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

### Requirement: DTO inglés de producto según 5.3

El Gateway MUST traducir el contrato inglés al dominio español y de vuelta según [5.3](../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos). Campos de respuesta `Product`: `id`, `sku`, `name`, `productType`, `description`, `brand` (`BrandRef` o `null`), `category` (`CategoryRef` con `path`), `attributes` (con `definitionId`, `value`, `key`, `label`, `unit`), `codes`, `unitOfMeasure`, `price`, `location`, `stock`, `active`, `version`, `images`, `createdAt`, `updatedAt`. `price` y `stock` MUST ser strings del esquema `NonNegativeDecimal`. `images` MUST ser `[]` (T-02-30). MUST NOT serializar `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `activo`, `creado_en` ni `actualizado_en`.

`POST`/`PATCH` MUST leer `name` → `nombre`, `productType` → `tipo`, `description` → `descripcion`, `brandId` → `marca`, `categoryId` → `categoria`, `attributes[].definitionId` → `def`, `attributes[].value` → `valor`, `codes[].type` → `Codigo.tipo`, `codes[].value` → `valor`, `unitOfMeasure` → `unidad_medida`, `price` → `precio`, `location.aisle`/`shelf`/`position` → `pasillo`/`estante`/`posicion`, `initialStock` → `stock_inicial`, `version` → `version`.

Enums MUST mapear: `unitOfMeasure` `unit`→`unidad`, `pair`→`par`, `box`→`caja`, `meter`→`metro`, `kilogram`→`kilogramo`, `liter`→`litro`; `codes[].type` `manufacturer`→`fabricante`, `oem`→`oem`, `barcode`→`barras`. Un enum desconocido MUST ser `400` `invalid_input`.

En `problem+json`, `errors[].field` MUST usar nombres de API (`name`, `productType`, `categoryId`, `brandId`, `productId`, `price`, `unitOfMeasure`, `attributes[n].value`, `attributes[n].definitionId`, `codes[n].type`, `initialStock`, `X-Vectra-Terminal`, `active`, `body`). MUST NOT aparecer `nombre`, `tipo`, `categoria`, `marca`, `precio` ni `unidad_medida` en `errors[].field`. El remap global `tipo`→`type` (definiciones de atributo) MUST NOT aplicarse a `productType`: el handler de productos MUST mapear `tipo` de ficha a `productType`.

#### Scenario: respuesta sin claves de dominio (happy)

- **WHEN** `POST` crea el perno M8 x 40
- **THEN** el JSON de 201 tiene `name`, `productType`, `categoryId` ausente (usa `category`), `unitOfMeasure` `"unit"`, `price` string y `stock` string
- **AND** no tiene `nombre`, `tipo`, `precio`, `unidad_medida`, `categoria` ni `activo`

#### Scenario: errors[].field categoryId no categoria (unhappy, 5.3)

- **WHEN** el servicio rechaza una categoría inactiva con `FieldError.field` `categoria`
- **AND** el Gateway traduce ese error a HTTP
- **THEN** `errors[].field` es `categoryId`
- **AND** no aparece `categoria` en el cuerpo

### Requirement: Tests de integración de productos contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-21. Listo cuando: cada respuesta JSON (éxito o error) se valida con `assert_conforms` (T-09-04). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_product_perno_m8x40_returns_201_with_location` | `api` | integración, happy, HU-02-04 |
| `post_product_sends_terminal_on_initial_stock` | `api` | integración, happy |
| `get_product_returns_200` | `api` | integración, happy |
| `patch_product_price_increments_version` | `api` | integración, happy, HU-02-05 |
| `patch_product_partial_keeps_other_fields` | `api` | integración, happy |
| `post_product_duplicate_sku_returns_409` | `api` | integración, unhappy, HU-02-04 / Listo cuando |
| `post_product_cuarenta_on_number_attribute_returns_400` | `api` | integración, unhappy, HU-02-04 / Listo cuando |
| `patch_product_stale_version_returns_409` | `api` | integración, unhappy, HU-02-05 / Listo cuando |
| `get_product_unknown_id_returns_404` | `api` | integración, unhappy |
| `patch_product_unknown_id_returns_404` | `api` | integración, unhappy |
| `product_id_invalid_ulid_returns_400` | `api` | integración, unhappy |
| `post_product_missing_name_returns_400` | `api` | integración, unhappy |
| `post_product_price_as_json_number_returns_400` | `api` | integración, unhappy |
| `patch_product_active_returns_400` | `api` | integración, unhappy, T-02-22 |
| `post_product_terminal_too_long_returns_400` | `api` | integración, unhappy |
| `post_product_without_service_returns_503` | `api` | integración, unhappy |
| `get_products_list_is_not_summary_page` | `api` | integración, unhappy, T-02-27 |
| `product_problem_field_is_category_id_not_categoria` | `api` | integración, unhappy, 5.3 |
| `vortex_serve_get_unknown_product_returns_404` | `vortex` | binario, happy (servicio inyectado) |

#### Scenario: la suite de productos cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 200/201/400/404/409/503 pasan `assert_conforms`

## MODIFIED Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST registrar las rutas de categorías (requirement «Endpoints /api/v1/categories»). El router MUST registrar las rutas de marcas (requirement «Endpoints /api/v1/brands»). El router MUST registrar las rutas de definiciones de atributo (requirement «Endpoints /api/v1/attribute-definitions»). El router MUST registrar las rutas de producto (requirement «Endpoints POST/GET/PATCH /api/v1/products»). MUST NOT registrar otras rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»). `AppState::new` / `with_health` MAY omitir el servicio de categorías, el de marcas, el de definiciones de atributo y el de producto (los tests de `/health` y estáticos siguen construyéndose sin ellos).

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

### Requirement: vortex serve escucha HTTP en server.bind

`vortex serve` MUST dejar de responder `no implementado`. MUST resolver el path de configuración como en `vortex-config` (`--config` gana sobre `VORTEX_CONFIG`), MUST cargar y validar el TOML, MUST instalar el subscriber de logging (capability `vortex-logging`), MUST abrir Storage y migrar según el requirement «Arranque ordenado de vortex serve (paso 1)», y MUST escuchar HTTP (sin TLS) en la dirección `server.bind` **solo después** de esas fases. MUST cablear un registro de salud que implementa `HealthReport`. MUST inyectar en `AppState` `paths.web` como raíz `web`, `paths.datos` unido a `media` como raíz `media`, y la `version` del binario. MUST inyectar el servicio de categorías (adaptador `CategoryRepo` de `storage` + occupancy stub que no reporta productos activos) y MUST inyectar el servicio de marcas (adaptador `BrandRepo` / `MarcaRepo` de `storage`) y MUST inyectar el servicio de definiciones de atributo (adaptador `AttributeDefRepo` / `AtributoDefRepo` de `storage`) y MUST inyectar el servicio de producto (`ProductService` con adaptadores `ProductRepo` / `ProductoRepo`, `CategoryRepo`, `BrandRepo` y `AttributeDefRepo` de `storage`) y MUST NOT inyectar `Storage` crudo en `AppState`. MUST NOT arrancar sidecars. Una petición HTTP a una ruta no registrada MUST responder 404 con `Content-Type: application/problem+json` y `code` `not_found`. `GET /health` contra ese bind MUST responder según el requirement «GET /health expone el snapshot agregado» y, en un arranque exitoso de este change, MUST coincidir con «Arranque ordenado de vortex serve (paso 1)». `GET /metrics` contra ese bind MUST responder según el requirement «GET /metrics expone texto Prometheus». `GET /` contra ese bind MUST responder según el requirement «Estáticos de la SPA y /media» cuando `{web}/index.html` existe. `GET /api/v1/categories` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/brands` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/attribute-definitions` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/products/{ulid-válido-inexistente}` contra ese bind MUST responder `404` `not_found`, no `503`.

#### Scenario: vortex serve arranca y responde 404 (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` con `server.bind` en `127.0.0.1` y un puerto libre
- **THEN** el proceso queda en ejecución escuchando en esa dirección
- **AND** `GET /no-such-route` contra ese bind recibe estado HTTP 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`

#### Scenario: vortex serve GET /health responde Health (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` escuchando en un bind de prueba
- **AND** se envía `GET /health` contra ese bind
- **THEN** la respuesta tiene `Content-Type: application/json`
- **AND** el JSON tiene `status` `ok`, `version`, `uptimeSeconds` y `components` con los seis nombres
- **AND** `components.storage.status` es `up`
- **AND** el JSON no tiene `code` de `problem+json`

#### Scenario: vortex serve GET /metrics responde Prometheus (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` escuchando en un bind de prueba
- **AND** se envía `GET /health` contra ese bind
- **AND** se envía `GET /metrics` contra ese bind
- **THEN** la respuesta de `/metrics` tiene estado HTTP 200
- **AND** `Content-Type` empieza por `text/plain`
- **AND** el cuerpo incluye `http_request_duration_seconds`

#### Scenario: vortex serve GET / sirve index.html desde paths.web (happy)

- **WHEN** el TOML de prueba pone `paths.web` en un directorio temporal con `index.html` que contiene `{{VERSION}}`
- **AND** se ejecuta `vortex serve --config <ese-toml>` escuchando en un bind de prueba
- **AND** se envía `GET /` contra ese bind
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` empieza por `text/html`
- **AND** `Cache-Control` es `no-cache`
- **AND** el cuerpo no contiene `{{VERSION}}`

#### Scenario: vortex serve GET /api/v1/categories lista items (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` escuchando en un bind de prueba
- **AND** se envía `GET /api/v1/categories` contra ese bind
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `items` (arreglo, posiblemente vacío)
- **AND** el JSON no tiene `code` `unavailable`

#### Scenario: vortex serve GET /api/v1/brands lista items (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` escuchando en un bind de prueba
- **AND** se envía `GET /api/v1/brands` contra ese bind
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `items` (arreglo, posiblemente vacío)
- **AND** el JSON no tiene `code` `unavailable`

#### Scenario: vortex serve GET /api/v1/attribute-definitions lista items (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` escuchando en un bind de prueba
- **AND** se envía `GET /api/v1/attribute-definitions` contra ese bind
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `items` (arreglo, posiblemente vacío)
- **AND** el JSON no tiene `code` `unavailable`

#### Scenario: vortex serve GET producto desconocido es 404 no 503 (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` escuchando en un bind de prueba
- **AND** se envía `GET /api/v1/products/01J9ZKA7M3Q8R2T5V9X1Z4B6D8` contra ese bind
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** el JSON no tiene `code` `unavailable`

#### Scenario: vortex serve sin path de configuración (unhappy)

- **WHEN** se ejecuta `vortex serve` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr indica que falta el path (`--config` o `VORTEX_CONFIG`)
- **AND** stderr no contiene `no implementado`
- **AND** el proceso no queda escuchando HTTP

#### Scenario: vortex serve con archivo inexistente (unhappy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml serve`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`
- **AND** el mensaje nombra el path o indica que el archivo no existe o no se pudo leer
- **AND** el proceso no queda escuchando HTTP

#### Scenario: vortex serve con configuración inválida (unhappy)

- **WHEN** se ejecuta `vortex serve --config` apuntando a un TOML con un valor ilegal (p. ej. `negocio.moneda = "peso"`)
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr contiene la clave y un motivo
- **AND** stderr no contiene `no implementado`
- **AND** el proceso no queda escuchando HTTP

#### Scenario: vortex serve con bind ocupado (unhappy)

- **WHEN** `server.bind` apunta a una dirección TCP ya ocupada
- **AND** se ejecuta `vortex serve` con ese config
- **THEN** el proceso termina con código distinto de 0
- **AND** el proceso no queda colgado
- **AND** stderr indica un fallo al escuchar o al enlazar
