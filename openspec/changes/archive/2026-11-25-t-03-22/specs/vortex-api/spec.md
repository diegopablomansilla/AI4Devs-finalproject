# Spec Delta

Ticket: T-03-22. HU: HU-03-08.

## MODIFIED Requirements

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

`docs/openapi.yaml` MUST declarar `'503': { $ref: '#/components/responses/Unavailable' }` en `GET /api/v1/search` (además de `200`/`400`/`504` ya presentes). MUST NOT implementar `GET /api/v1/search/suggestions`. `POST /api/v1/search` MUST NOT ser una ruta de negocio (método no registrado → 405).

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

## ADDED Requirements

### Requirement: Endpoint GET /api/v1/search/by-code/{code}

El router MUST registrar `GET /api/v1/search/by-code/{code}` alineado con `docs/openapi.yaml` (`operationId` `findProductsByCode`). MUST delegar en `ProductGateway::find_by_code` inyectado en `AppState`. MUST NOT usar `SearchGateway` ni Meilisearch.

Path `code`: tras percent-decode y `trim`, MUST tener longitud 1..=64. Vacío o > 64 → `400 invalid_input` con `errors[].field` `code`. Si no hay `ProductGateway` → `503 unavailable`.

Respuesta `200`: JSON `{ "items": ProductSummary[] }` (camelCase). Sin matches → `items: []`.

`docs/openapi.yaml` MUST declarar `'400'` y `'503'` en esta ruta.

#### Scenario: OEM devuelve items (happy)

- **WHEN** el catálogo tiene dos activos con el mismo OEM
- **AND** se envía `GET /api/v1/search/by-code/SHARED-OEM`
- **THEN** HTTP 200 y `items` tiene longitud 2
- **AND** `assert_conforms` de `GET /api/v1/search/by-code/{code}` 200 `application/json` no falla

#### Scenario: HX-0841Z no devuelve HX-0840Z (unhappy, Listo cuando)

- **WHEN** solo existe fabricante `"HX-0840Z"`
- **AND** se envía `GET /api/v1/search/by-code/HX-0841Z`
- **THEN** HTTP 200 y `items` está vacío

#### Scenario: Sin SearchGateway sigue operando (happy, H-05)

- **WHEN** `AppState` tiene `ProductGateway` pero no `SearchGateway`
- **AND** se envía `GET /api/v1/search/by-code/<código válido>`
- **THEN** HTTP 200

#### Scenario: code demasiado largo (unhappy)

- **WHEN** el segmento `code` tiene más de 64 caracteres
- **THEN** HTTP 400 `invalid_input`, `errors[].field` es `code`
- **AND** `assert_conforms` no falla

#### Scenario: Sin catálogo 503 (unhappy)

- **WHEN** `AppState` no tiene `ProductGateway`
- **THEN** HTTP 503 `unavailable`

#### Scenario: suite by-code en api (happy)

- **WHEN** se ejecuta `cargo test -p api --test search by_code`
- **THEN** los tests terminan con código 0
