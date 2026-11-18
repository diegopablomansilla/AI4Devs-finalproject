# Spec Delta

Ticket padre: T-02-27. Change: T-02-27. HU: HU-02-10.

## ADDED Requirements

### Requirement: Endpoint GET /api/v1/products (listado admin)

El router de `build_router` MUST registrar `GET /api/v1/products` alineado con `docs/openapi.yaml` (`listProducts`). El Gateway MUST decodificar `cursor` y `limit` con el helper de T-00-21, mapear query `categoryId`/`brandId`/`active` a `FiltroProducto`, pedir `take = limit + 1` al servicio y armar `ProductSummaryPage` con `paginate`. MUST NOT implementar búsqueda por texto. MUST NOT hidratar `imageUrl` (siempre `null`; T-02-28 es v2). MUST NOT registrar `GET /search`.

`active` ausente MUST equivaler a `true` (solo activos). `active=false` MUST listar solo dados de baja. `categoryId` presente MUST filtrar la rama (categoría y descendientes). `brandId` presente MUST filtrar esa marca. Los tres filtros MUST combinarse con AND. Orden MUST ser por `id` ascendente.

La respuesta de éxito MUST ser **200** `application/json` `ProductSummaryPage`: `items` (array de `ProductSummary`) y `nextCursor` (`string` o `null`). Cada `ProductSummary` MUST tener `id`, `sku`, `name`, `productType`, `price` y `stock` como string (`NonNegativeDecimal`), `active`; `brand` MUST ser el nombre de la marca o `null`; `categoryPath` MUST ser la `ruta` de la categoría; `imageUrl` MUST ser `null`. MUST NOT serializar claves de dominio (`nombre`, `tipo`, `precio`, `activo`, `marca_nombre`, `categoria_ruta`).

Un `categoryId` o `brandId` que no parsea como ULID MUST ser **400** `invalid_input` con `errors[].field` `categoryId` o `brandId`. Cursor inválido MUST ser **400** `invalid_input` field `cursor`. `limit` fuera de 1..=100 MUST ser **400** `invalid_input` field `limit`. Categoría o marca inexistente MUST ser **200** con `items` vacío y `nextCursor` `null` (no 404). El query `q` u otros params de búsqueda MUST ignorarse (no cambian el resultado respecto de no enviarlos).

Si `AppState` no tiene servicio de producto, `GET /api/v1/products` MUST responder **503** `unavailable`. `docs/openapi.yaml` MUST declarar `503` en este GET. `vortex serve` MUST inyectar el mismo `ProductGateway` que la ficha.

#### Scenario: Filtro por rama (happy, HU-02-10, Listo cuando)

- **WHEN** existen productos en `"Fijaciones"` y en su subcategoría `"Pernos"`, y otro fuera de esa rama
- **AND** se envía `GET /api/v1/products?categoryId={id-de-Fijaciones}`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye los productos de `"Fijaciones"` y de `"Pernos"`
- **AND** no incluye el producto de fuera de la rama
- **AND** cada ítem de la rama tiene `categoryPath` que empieza por `"Fijaciones"`
- **AND** `assert_conforms` de `GET /api/v1/products` 200 `application/json` no falla

#### Scenario: Filtro por inactivos (happy, HU-02-10, Listo cuando)

- **WHEN** existe un producto activo y otro dado de baja
- **AND** se envía `GET /api/v1/products?active=false`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye solo el producto inactivo
- **AND** ese ítem tiene `active` falso
- **AND** `assert_conforms` de `GET /api/v1/products` 200 `application/json` no falla

#### Scenario: Default solo activos (happy)

- **WHEN** existe un producto activo y otro inactivo
- **AND** se envía `GET /api/v1/products` sin `active`
- **THEN** `items` incluye solo el activo

#### Scenario: Filtro por marca (happy)

- **WHEN** existen productos de dos marcas
- **AND** se envía `GET /api/v1/products?brandId={id-de-Fixser}`
- **THEN** `items` incluye solo los de esa marca
- **AND** `brand` es el nombre `"Fixser"` (string, no objeto)

#### Scenario: Paginación por cursor (happy)

- **WHEN** existen tres productos activos A, B, C ordenados por `id`
- **AND** se envía `GET /api/v1/products?limit=2`
- **THEN** `items` tiene exactamente A y B
- **AND** `nextCursor` decodifica al `id` de B
- **WHEN** se envía `GET /api/v1/products?limit=2&cursor={nextCursor}`
- **THEN** `items` es C
- **AND** `nextCursor` es `null`
- **AND** no se repite A ni B

#### Scenario: Página vacía (happy)

- **WHEN** no hay productos que matcheen el filtro
- **AND** se envía `GET /api/v1/products`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` es `[]`
- **AND** `nextCursor` es `null`

#### Scenario: ProductSummary DTO 5.3 (happy)

- **WHEN** se lista un producto activo con precio y stock
- **THEN** el ítem tiene `name`, `productType`, `price` string, `stock` string, `active`, `imageUrl` `null`
- **AND** no tiene `nombre`, `tipo`, `precio`, `activo`, `marca_nombre` ni `categoria_ruta`

#### Scenario: cursor inválido (unhappy)

- **WHEN** se envía `GET /api/v1/products?cursor=%%%`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `cursor`
- **AND** `assert_conforms` de `GET /api/v1/products` 400 `application/problem+json` no falla

#### Scenario: limit fuera de rango (unhappy)

- **WHEN** se envía `GET /api/v1/products?limit=0`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `limit`
- **WHEN** se envía `GET /api/v1/products?limit=101`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `errors` incluye `field` `limit`

#### Scenario: categoryId no es ULID (unhappy)

- **WHEN** se envía `GET /api/v1/products?categoryId=not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `categoryId`

#### Scenario: brandId no es ULID (unhappy)

- **WHEN** se envía `GET /api/v1/products?brandId=not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `brandId`

#### Scenario: q no dispara búsqueda (unhappy, No hace)

- **WHEN** existen productos cuyo nombre no contiene `"perno"` y otros que sí
- **AND** se envía `GET /api/v1/products?q=perno` sin otros filtros (default activos)
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` no está filtrado por el texto `"perno"` (mismo conjunto que sin `q`)
- **AND** no es una respuesta de `GET /search`

#### Scenario: sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de producto
- **AND** se envía `GET /api/v1/products`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` de `GET /api/v1/products` 503 `application/problem+json` no falla

Antes de archive, la suite MUST incluir estos tests (nombres estables). Ticket padre: T-02-27. Listo cuando: filtro por rama y por inactivos. Cada JSON (200/400/503) se valida con `assert_conforms` (T-09-04). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `get_products_branch_filter_includes_descendants` | `api` | integración, happy, HU-02-10 / Listo cuando |
| `get_products_active_false_returns_inactive_only` | `api` | integración, happy, HU-02-10 / Listo cuando |
| `get_products_default_excludes_inactive` | `api` | integración, happy |
| `get_products_filters_by_brand` | `api` | integración, happy |
| `get_products_paginates_by_cursor` | `api` | integración, happy |
| `get_products_empty_page` | `api` | integración, happy |
| `get_products_summary_has_english_dto` | `api` | integración, happy, 5.3 |
| `get_products_invalid_cursor_returns_400` | `api` | integración, unhappy |
| `get_products_limit_zero_returns_400` | `api` | integración, unhappy |
| `get_products_limit_101_returns_400` | `api` | integración, unhappy |
| `get_products_invalid_category_id_returns_400` | `api` | integración, unhappy |
| `get_products_invalid_brand_id_returns_400` | `api` | integración, unhappy |
| `get_products_q_does_not_text_search` | `api` | integración, unhappy, No hace |
| `get_products_without_service_returns_503` | `api` | integración, unhappy |
| `storage_list_branch_includes_descendants` | `storage` | integración, happy, HU-02-10 / Listo cuando |
| `storage_list_inactive_only` | `storage` | integración, happy, HU-02-10 / Listo cuando |
| `vortex_serve_get_products_empty_returns_200` | `vortex` | binario, happy (servicio inyectado) |

#### Scenario: la suite de listado cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 200/400/503 pasan `assert_conforms`

## MODIFIED Requirements

### Requirement: Endpoints POST/GET/PATCH /api/v1/products

El router de `build_router` MUST registrar `POST` y `GET /api/v1/products` y `GET`, `PATCH` y `DELETE /api/v1/products/{productId}` alineados con `docs/openapi.yaml`. El Gateway MUST delegar en el servicio de producto inyectado en `AppState` (capability `catalog-producto`) y MUST NOT implementar validación de atributos, existencia de categoría/marca ni persistencia. El GET de colección MUST seguir el requirement «Endpoint GET /api/v1/products (listado admin)».

`POST` MUST crear con cuerpo `ProductCreate` (`sku`, `name`, `productType`, `categoryId` y `price` obligatorios; `description`/`brandId`/`attributes`/`codes`/`unitOfMeasure`/`location`/`initialStock` opcionales) y responder **201** con `Product`. MUST enviar la cabecera `Location` con valor `/api/v1/products/{id}` (el `id` del producto creado, ULID sin prefijo). `unitOfMeasure` ausente MUST interpretarse como `unit`. `attributes`/`codes` ausentes MUST interpretarse como `[]`. `initialStock` ausente MUST interpretarse como `"0"`. La cabecera `X-Vectra-Terminal` (OpenAPI `Terminal`, `maxLength` 64) MUST copiarse a `NewProducto.terminal` si está presente y no vacía tras recortar espacios; si falta o queda vacía, `terminal` MUST ser ausente. Un valor de más de 64 caracteres MUST ser `400` `invalid_input` con `errors[].field` `X-Vectra-Terminal`. MUST NOT crear el movimiento `inicial` en el Gateway (lo hace `ProductRepo::create`).

`GET /api/v1/products/{productId}` MUST devolver **200** `Product` o **404** `not_found`. MUST NOT filtrar por `active` (la ficha admin incluye inactivos).

`PATCH` MUST aplicar el cuerpo `ProductUpdate` (`version` obligatorio, `minProperties: 2`): mapear los campos de ficha presentes a `PatchProducto` y delegar en el servicio; el campo `active` MUST seguir el requirement «DELETE y reactivación de producto». El resultado de éxito de un parche de ficha MUST ser **200** `Product` con `version` incrementada. `stock` MUST NOT ser editable. Cuerpo con solo `version` (sin `active` ni otro campo de ficha) MUST ser `400` `invalid_input`.

`{productId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `productId`. Id inexistente MUST ser `404` `not_found`. SKU duplicado MUST ser `409` `duplicate_sku`. `version` desactualizada en un parche de ficha MUST ser `409` `version_conflict`. Valor de atributo incompatible MUST ser `400` `invalid_input` con `errors[].field` `attributes[n].value`. Categoría o marca inexistente o inactiva MUST ser `400` `invalid_input` con `errors[].field` `categoryId` o `brandId`. JSON malformado o `price` como número JSON MUST ser `400` `invalid_input`. Cuerpo `POST` sin `sku`/`name`/`productType`/`categoryId`/`price` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de producto, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`ProductService` + `ProductoRepo` + repos de categoría, marca y `AtributoDef`). MUST NOT reindexar (T-03-07). `docs/openapi.yaml` MUST declarar `503` en `POST` y `GET /api/v1/products` y en `GET`/`PATCH`/`DELETE /api/v1/products/{productId}`.

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
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON es `ProductSummaryPage` con `items` y `nextCursor`
- **AND** `assert_conforms` de `GET /api/v1/products` 200 `application/json` no falla

### Requirement: Paginación por cursor opaco en el Gateway

El crate `api` MUST exponer un helper de paginación por cursor para listados de administración, alineado con [5.6](../../../../../docs/5-contract-api.md#56-paginación) y con los parámetros `Cursor` / `Limit` de `docs/openapi.yaml`. El helper MUST NOT registrar rutas HTTP (el GET de listado lo registra el requirement «Endpoint GET /api/v1/products (listado admin)»). MUST NOT registrar `GET /api/v1/products/{productId}/stock-movements`. MUST NOT implementar paginación por *offset* (`offset`, `estimatedTotalHits`).

El cursor MUST ser opaco: internamente es el último ULID visto (26 caracteres Crockford, sin prefijo de tabla), serializado en **base64url sin padding** (RFC 4648 §5). Un cursor ausente MUST significar la primera página. Codificar un ULID canónico y decodificar el cursor resultante MUST devolver el mismo ULID. El valor codificado MUST NOT ser el ULID en texto plano.

`limit` ausente MUST equivaler a **50**. `limit` MUST aceptarse solo en el rango **1..=100**. Un `limit` menor que 1 o mayor que 100 MUST ser `invalid_input` con `errors[].field` `limit`.

Un cursor presente que no sea base64url sin padding, que decodifique a bytes que no sean un ULID canónico de 26 caracteres, o que esté vacío MUST ser `invalid_input` con `errors[].field` `cursor`. Ese error MUST traducirse vía `ApiError` a HTTP **400**, `Content-Type: application/problem+json` y `code` `invalid_input`.

Dada una secuencia de ítems ya ordenada por el llamador (hasta `limit + 1` elementos) y un `limit` válido: si hay más ítems que `limit`, la página MUST devolver exactamente `limit` ítems y `nextCursor` MUST ser el cursor del último ítem devuelto; si hay `limit` ítems o menos (incluida la secuencia vacía), `nextCursor` MUST ser `null`. El helper MUST NOT imponer dirección de orden (ascendente o descendente): eso lo define el listado consumidor (T-02-27, T-02-26).

MUST NOT agregar crates a la allowlist de `api` para esta codificación.

#### Scenario: Roundtrip ULID y cursor opaco (happy, Listo cuando)

- **WHEN** se codifica el ULID canónico `01J9ZKA7M3Q8R2T5V9X1Z4B6D8` y se decodifica el cursor resultante
- **THEN** el ULID recuperado es `01J9ZKA7M3Q8R2T5V9X1Z4B6D8`
- **AND** el cursor no es igual al ULID en texto plano
- **AND** el cursor solo usa el alfabeto base64url (`A–Z`, `a–z`, `0–9`, `-`, `_`) y no contiene `=`

#### Scenario: Cursor ausente es primera página (happy)

- **WHEN** se decodifica un cursor ausente
- **THEN** el resultado es éxito sin ULID (primera página)

#### Scenario: limit por defecto 50 y tope 100 (happy, Listo cuando)

- **WHEN** se interpreta `limit` ausente
- **THEN** el valor efectivo es 50
- **WHEN** se interpreta `limit` 1 y `limit` 100
- **THEN** ambos se aceptan con esos valores

#### Scenario: nextCursor del último ítem si hay más páginas (happy)

- **WHEN** se arma una página con `limit` 2 a partir de tres ítems con ULID `A`, `B`, `C` (en ese orden)
- **THEN** la página contiene exactamente los ítems `A` y `B`
- **AND** `nextCursor` decodifica al ULID de `B`

#### Scenario: nextCursor null al final (happy)

- **WHEN** se arma una página con `limit` 2 a partir de dos ítems
- **THEN** `nextCursor` es `null`
- **WHEN** se arma una página con `limit` 2 a partir de cero ítems
- **THEN** `nextCursor` es `null` y la lista de ítems está vacía

#### Scenario: cursor inválido no es base64url (unhappy)

- **WHEN** se decodifica el cursor `%%%`
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: cursor con padding o alfabeto estándar (unhappy)

- **WHEN** se decodifica un cursor base64url con padding `=` o con alfabeto estándar (`+` o `/`)
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: cursor que no decodifica a ULID (unhappy)

- **WHEN** se decodifica un cursor base64url válido cuyo payload no es un ULID canónico de 26 caracteres (p. ej. texto `not-a-ulid`, ULID en minúsculas, o ULID con prefijo de tabla)
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: cursor vacío (unhappy)

- **WHEN** se decodifica el cursor `""`
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: limit fuera de rango (unhappy, Listo cuando)

- **WHEN** se interpreta `limit` 0
- **THEN** el error es `invalid_input` con `errors[].field` `limit`
- **WHEN** se interpreta `limit` 101
- **THEN** el error es `invalid_input` con `errors[].field` `limit`

#### Scenario: cursor inválido se traduce a problem+json (unhappy)

- **WHEN** el helper rechaza un cursor inválido y el error se traduce con `ApiError`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `cursor`
