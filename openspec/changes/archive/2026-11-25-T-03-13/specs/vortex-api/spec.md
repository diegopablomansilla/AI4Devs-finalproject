# Spec Delta

Ticket: T-03-13. HU: HU-03-03, HU-03-04.

## ADDED Requirements

### Requirement: Endpoint GET /api/v1/search

El router de `build_router` MUST registrar `GET /api/v1/search` alineado con `docs/openapi.yaml` (`operationId` `searchProducts`). El Gateway MUST delegar en el `SearchGateway` inyectado en `AppState` y MUST NOT implementar el motor, facetas ni respaldo en SurrealDB.

Query (nombres OpenAPI, camelCase):

| Parámetro | Defecto | Dominio |
|---|---|---|
| `q` | `""` (vacío permitido) | `q` (≤ 200) |
| `categoryId` | ausente | `categoria_ids` con ese ULID (un id; la rama está en `categoria_ids` del documento) |
| `brandId` | ausente | `marca_id` |
| `inStock` | ausente (ambos) | `con_stock` |
| `attr[<clave>]` | ausente | `atributos` (`clave` sin prefijo `attr_`) |
| `sort` | `relevance` | `SearchOrden` (`relevance`/`price_asc`/`price_desc`/`stock_desc`) |
| `offset` | `0` (máx. 1000) | `offset` |
| `limit` | `20` (1..=50) | `limit` |
| `facets` | `false` | ignorado |

`attr[<clave>]` MUST parsearse como *deepObject* (`attr[medida]=M8&attr[grado]=5`). MUST NOT exigir una crate extra de query string.

Si `AppState` no tiene servicio de búsqueda, MUST responder `503` `unavailable` (`application/problem+json`). Motor `unavailable` MUST ser `503` `unavailable` (no `200` con `degraded: true`; eso es T-03-30). `invalid_input` de dominio o ULID/sort inválidos MUST ser `400` `invalid_input` con `errors[].field` en inglés (`q`, `offset`, `limit`, `categoryId`, `brandId`, `sort`, `key`). Timeout del motor o de la ruta MUST ser `504` `timeout`.

`docs/openapi.yaml` MUST declarar `'503': { $ref: '#/components/responses/Unavailable' }` en `GET /api/v1/search` (además de `200`/`400`/`504` ya presentes). MUST NOT implementar `GET /api/v1/search/suggestions` ni `GET /api/v1/search/by-code/{code}`. `POST /api/v1/search` MUST NOT ser una ruta de negocio (método no registrado → 405).

`vortex serve` MUST inyectar `SearchService` sobre el adaptador Meilisearch ya construido, con uid `productos`. Contra ese bind, `GET /api/v1/search` MUST NOT ser `404`.

#### Scenario: GET con q devuelve SearchResponse (happy)

- **WHEN** el Gateway de búsqueda está inyectado y configurado para devolver un hit de `"perno"`
- **AND** se envía `GET /api/v1/search?q=perno`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `query` `"perno"`, `hits` no vacío, `estimatedTotalHits`, `offset` 0, `limit` 20, `processingTimeMs`, `degraded` falso
- **AND** `facets` es `null` o está ausente
- **AND** el JSON no tiene claves en español (`nombre`, `estimado_total`)
- **AND** `assert_conforms` de `GET /api/v1/search` 200 `application/json` no falla

#### Scenario: q vacía con inStock explora con filtros (happy)

- **WHEN** se envía `GET /api/v1/search?inStock=true`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el Gateway recibe `q` vacía y `con_stock` verdadero

#### Scenario: attr deepObject se traduce a filtros (happy)

- **WHEN** se envía `GET /api/v1/search?q=perno&attr[medida]=M8&attr[grado]=5`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el Gateway recibe `atributos` `medida` → `"M8"` y `grado` → `"5"`
- **AND** las claves no llevan prefijo `attr_`

#### Scenario: categoryId brandId inStock se traducen (happy)

- **WHEN** se envía `GET /api/v1/search?q=perno&categoryId=<ulid>&brandId=<ulid>&inStock=false`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el Gateway recibe `categoria_ids` con ese id, `marca_id` y `con_stock` falso

#### Scenario: sort price_asc se traduce (happy)

- **WHEN** se envía `GET /api/v1/search?q=perno&sort=price_asc`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el Gateway recibe orden de precio ascendente

#### Scenario: highlight sale de _formatted (happy)

- **WHEN** el Gateway devuelve un hit con `resaltado.nombre` que contiene `<mark>`
- **AND** se envía `GET /api/v1/search?q=perno`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `hits[0].highlight.name` contiene `<mark>`
- **AND** `hits[0].highlight` no es `null`

#### Scenario: facets=true no calcula facetas (happy, No hace)

- **WHEN** se envía `GET /api/v1/search?q=perno&facets=true`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `facets` es `null` o está ausente
- **AND** `degraded` es falso

#### Scenario: q demasiado larga es 400 (unhappy)

- **WHEN** se envía `GET /api/v1/search?q=` con 201 caracteres
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `q`
- **AND** `assert_conforms` de `GET /api/v1/search` 400 `application/problem+json` no falla

#### Scenario: offset o limit fuera de rango es 400 (unhappy)

- **WHEN** se envía `GET /api/v1/search?offset=1001` o `limit=0` o `limit=51`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `offset` o `limit`

#### Scenario: categoryId o brandId no ULID es 400 (unhappy)

- **WHEN** se envía `GET /api/v1/search?categoryId=no-ulid` o `brandId=no-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `categoryId` o `brandId`

#### Scenario: sort desconocido es 400 (unhappy)

- **WHEN** se envía `GET /api/v1/search?sort=nombre`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `sort`

#### Scenario: clave attr_medida es 400 (unhappy)

- **WHEN** se envía `GET /api/v1/search?attr[attr_medida]=M8`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: motor unavailable es 503 (unhappy)

- **WHEN** el Gateway está configurado para `unavailable`
- **AND** se envía `GET /api/v1/search?q=perno`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** el JSON no tiene `degraded` verdadero
- **AND** `assert_conforms` de `GET /api/v1/search` 503 `application/problem+json` no falla

#### Scenario: sin servicio es 503 (unhappy)

- **WHEN** `AppState` no tiene servicio de búsqueda
- **AND** se envía `GET /api/v1/search?q=perno`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`

### Requirement: Timeout de 2 s y cancelación en GET /search

`GET /api/v1/search` MUST imponer un *timeout* de **2 segundos** independiente del timeout global de 5 s. Si la operación no termina en 2 s, MUST responder `504` con `Content-Type: application/problem+json`, `code` `timeout` y `title` "La operación superó el tiempo de espera". `assert_conforms` de `504` `application/problem+json` MUST no fallar.

Si el cliente cierra la conexión antes de completar, el Gateway MUST descartar el *future* de la petición y MUST NOT dejar el trabajo del `SearchGateway` en curso (el future del caso de uso se dropea). El timeout global de 5 s MUST seguir aplicando al resto de rutas.

#### Scenario: búsqueda lenta es 504 a los 2 s (unhappy)

- **WHEN** el Gateway de búsqueda no completa dentro de 2 s
- **AND** se envía `GET /api/v1/search?q=perno`
- **THEN** la respuesta tiene estado HTTP 504
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `timeout`
- **AND** la respuesta llega antes de 5 s
- **AND** `assert_conforms` de `GET /api/v1/search` 504 `application/problem+json` no falla

#### Scenario: el cliente aborta descarta el trabajo (unhappy, Listo cuando)

- **WHEN** el Gateway de búsqueda está colgado a propósito
- **AND** un cliente TCP envía `GET /api/v1/search?q=perno` y cierra la conexión sin leer la respuesta
- **THEN** el future de búsqueda se dropea
- **AND** el trabajo en curso no queda vivo hasta el timeout de 2 s

### Requirement: DTO inglés de SearchResponse según 5.3

El cuerpo 200 MUST serializar `SearchResponse` / `SearchHit` en inglés según [5.3](../../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) y `docs/openapi.yaml`: `id`, `sku`, `name`, `productType`, `brand`, `codes`, `attributes`, `price`, `unitOfMeasure`, `stock`, `inStock`, `location`, `imageUrl`, `highlight.name` / `highlight.description`. `price` y `stock` MUST ser strings `NonNegativeDecimal` (nunca float JSON). `unitOfMeasure` MUST ser el enum OpenAPI (`unit`, `pair`, …), no el wire español de dominio. `query` MUST repetir el `q` pedido. `estimatedTotalHits`, `offset`, `limit`, `processingTimeMs` MUST copiar la página de dominio. `degraded` MUST ser `false`. `highlight` MUST ser `null` cuando el hit no trae `resaltado`. `categoryPath` MAY omitirse o ir vacío (el hit de dominio no trae ruta de nombres). `errors[].field` MUST usar nombres OpenAPI (`categoryId`, no `categoria`).

#### Scenario: hit mapea campos required (happy)

- **WHEN** el Gateway devuelve un hit con sku, nombre, tipo, precio, unidad, stock y `con_stock`
- **AND** se envía `GET /api/v1/search?q=perno`
- **THEN** `hits[0]` tiene `sku`, `name`, `productType`, `price` (string), `unitOfMeasure`, `stock` (string), `inStock`
- **AND** el JSON no incluye `nombre`, `precio` numérico ni `unidad_medida`

#### Scenario: highlight null cuando no hay resaltado (happy)

- **WHEN** el Gateway devuelve un hit con `resaltado` ausente
- **AND** se envía `GET /api/v1/search?q=perno`
- **THEN** `hits[0].highlight` es `null`

### Requirement: Tests de integración de search contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-03-13. Listo cuando: cada JSON de éxito o `problem+json` se valida con `assert_conforms` (T-09-04) y existe el test de cancelación. `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `get_search_q_perno_returns_200` | `api` | integración; `assert_conforms` 200 |
| `get_search_empty_q_with_in_stock_returns_200` | `api` | integración; `assert_conforms` 200 |
| `get_search_maps_attr_deep_object` | `api` | integración |
| `get_search_maps_category_brand_in_stock` | `api` | integración |
| `get_search_maps_sort_price_asc` | `api` | integración |
| `get_search_highlight_from_formatted` | `api` | integración; `assert_conforms` 200 |
| `get_search_facets_true_returns_null_facets` | `api` | integración |
| `get_search_client_abort_drops_work` | `api` | integración (Listo cuando: cancelación) |
| `get_search_q_too_long_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_offset_1001_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_limit_zero_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_limit_51_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_invalid_category_id_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_invalid_brand_id_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_invalid_sort_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_attr_prefix_clave_returns_400` | `api` | integración; `assert_conforms` 400 |
| `get_search_timeout_returns_504` | `api` | integración; `assert_conforms` 504 |
| `get_search_unavailable_returns_503` | `api` | integración; `assert_conforms` 503 |
| `get_search_without_service_returns_503` | `api` | integración; `assert_conforms` 503 |
| `get_search_records_duration_histogram` | `api` | integración (`/metrics`) |
| `vortex_serve_get_search_is_not_404` | `vortex` | binario |

#### Scenario: la suite de search cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core`
- **THEN** corren los tests de la tabla
- **AND** los JSON de 200/400/504/503 pasan `assert_conforms`
- **AND** el test de cancelación pasa

## MODIFIED Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST registrar las rutas de categorías (requirement «Endpoints /api/v1/categories»). El router MUST registrar las rutas de marcas (requirement «Endpoints /api/v1/brands»). El router MUST registrar las rutas de definiciones de atributo (requirement «Endpoints /api/v1/attribute-definitions»). El router MUST registrar las rutas de producto (requirement «Endpoints POST/GET/PATCH /api/v1/products»). El router MUST registrar `POST /api/v1/products/{productId}/stock-movements` (requirement «Endpoint POST /api/v1/products/{productId}/stock-movements»). El router MUST registrar `GET /api/v1/search` (requirement «Endpoint GET /api/v1/search»). MUST NOT registrar otras rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»). `AppState::new` / `with_health` MAY omitir el servicio de categorías, el de marcas, el de definiciones de atributo, el de producto, el de stock y el de búsqueda (los tests de `/health` y estáticos siguen construyéndose sin ellos).

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

#### Scenario: GET /api/v1/search está registrado (happy)

- **WHEN** se envía `GET /api/v1/search` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`

### Requirement: Timeout por defecto 5 s

Si el Gateway no termina una petición en **5 segundos**, MUST responder `504` con `Content-Type: application/problem+json`, `code` `timeout`, `title` "La operación superó el tiempo de espera" y `type` `https://vectra.local/problems/timeout`. `instance` MUST ser la ruta pedida. El JSON MUST NOT incluir `errors`. Una petición que termina antes de 5 s MUST NOT recibir `504` por este timeout. `GET /api/v1/search` MUST usar el timeout de 2 s del requirement «Timeout de 2 s y cancelación en GET /search»; el default de 5 s MUST NOT aplicar a esa ruta. Este change MUST NOT acortar ni alargar el timeout de otras rutas (speech 10 s, SSE 90 s, etc.).

#### Scenario: handler lento es 504 timeout (unhappy)

- **WHEN** una petición llega a un handler que no responde dentro de 5 s (sonda de test; no forma parte del router de producción)
- **THEN** la respuesta tiene estado HTTP 504
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `timeout` y `title` "La operación superó el tiempo de espera"
- **AND** el JSON no incluye `errors`

#### Scenario: handler rápido no es 504 (happy)

- **WHEN** una petición a una ruta inexistente (404 inmediato) se envía al router de `build_router`
- **THEN** la respuesta tiene estado 404
- **AND** el JSON tiene `code` `not_found`
- **AND** la respuesta no tiene estado 504
