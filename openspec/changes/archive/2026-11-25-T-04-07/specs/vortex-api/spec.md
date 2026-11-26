# Spec Delta

Ticket: T-04-07. HU: HU-04-05.

## ADDED Requirements

### Requirement: Endpoint GET /api/v1/products/{productId}/recommendations

El router de `build_router` MUST registrar `GET /api/v1/products/{productId}/recommendations` alineado con `docs/openapi.yaml` (`operationId` `getRecommendations`). El Gateway MUST delegar en el `RecommendationGateway` inyectado en `AppState` y MUST NOT reimplementar la consulta 4.6.2, filtrar destinos inactivos, filtrar sustitutos por stock ni hidratar imagen.

`{productId}` MUST parsearse como ULID de `Id<Producto>`. El resultado de éxito MUST ser **200** `application/json` con el esquema `Recommendations`. Producto existente sin aristas MUST ser **200** con `similar`, `substitutes` y `complements` vacíos (no `404`). Producto inexistente (`DomainError` `not_found`) MUST ser **404** `not_found` (`application/problem+json`). `{productId}` que no parsea como ULID MUST ser **400** `invalid_input` con `errors[].field` `productId`.

Si `AppState` no tiene servicio de recomendaciones, MUST responder `503` `unavailable` (`application/problem+json`). `Unavailable` del Gateway MUST ser `503` `unavailable`. MUST NOT cachear la respuesta (ni `Cache-Control` distinto del `no-store` de `/api/*`, ni capa in-process). MUST NOT exigir timeout de ruta distinto del default del Gateway (5 s).

`docs/openapi.yaml` MUST declarar `'400': { $ref: '#/components/responses/BadRequest' }` y `'503': { $ref: '#/components/responses/Unavailable' }` en esta operación (además de `200`/`404` ya presentes). `POST`/`PATCH`/`DELETE` sobre esa ruta MUST NOT ser operaciones de negocio (método no registrado → 405).

`vortex serve` MUST inyectar `RecommendationService` sobre el adaptador `RelacionRepo` ya construido. Contra ese bind, `GET /api/v1/products/{productId}/recommendations` MUST NOT ser `404`.

`AppState::new` / `with_health` MAY omitir el servicio de recomendaciones.

#### Scenario: GET tres bloques ordenados (happy, HU-04-05)

- **WHEN** el Gateway de recomendaciones está inyectado y configurado para devolver similares, sustitutos y complementarios de "Perno M8", ordenados por prioridad
- **AND** se envía `GET /api/v1/products/{productId}/recommendations`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `productId`, `highlightSubstitutes`, `similar`, `substitutes` y `complements`
- **AND** cada bloque conserva el orden del Gateway
- **AND** el JSON no tiene claves en español (`producto_id`, `destacar_sustitutos`, `similares`, `sustitutos`, `complementarios`)
- **AND** `assert_conforms` de `GET /api/v1/products/{productId}/recommendations` 200 `application/json` no falla

#### Scenario: Producto sin stock destaca sustitutos (happy, HU-04-06)

- **WHEN** el Gateway devuelve `destacar_sustitutos` verdadero y un solo sustituto con stock 12
- **AND** se envía `GET /api/v1/products/{productId}/recommendations`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `highlightSubstitutes` es verdadero
- **AND** `substitutes` tiene un ítem cuyo `stock` es `"12"` (o el string decimal equivalente)

#### Scenario: Destino inactivo no aparece (happy, HU-04-05)

- **WHEN** el Gateway excluye un destino dado de baja
- **AND** se envía `GET` de recomendaciones del origen
- **THEN** la respuesta tiene estado HTTP 200
- **AND** ningún ítem tiene el `id` de ese destino

#### Scenario: Producto existente sin aristas es 200 vacío (happy)

- **WHEN** el Gateway devuelve tres vectores vacíos y `destacar_sustitutos` falso
- **AND** se envía `GET /api/v1/products/{productId}/recommendations`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `similar`, `substitutes` y `complements` son arrays vacíos
- **AND** `assert_conforms` de 200 `application/json` no falla

#### Scenario: imageUrl es null (happy, recableo MVP)

- **WHEN** un GET 200 incluye al menos un `RecommendedProduct`
- **THEN** ese ítem tiene `imageUrl` `null`
- **AND** no hidrata URL de imagen

#### Scenario: productId inválido es 400 (unhappy)

- **WHEN** se envía `GET /api/v1/products/not-a-ulid/recommendations`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `productId`
- **AND** `assert_conforms` de `GET /api/v1/products/{productId}/recommendations` 400 `application/problem+json` no falla

#### Scenario: producto inexistente es 404 (unhappy)

- **WHEN** se envía `GET /api/v1/products/{ulid-inexistente}/recommendations`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** `assert_conforms` de 404 `application/problem+json` no falla

#### Scenario: sin servicio es 503 (unhappy)

- **WHEN** `AppState` no tiene servicio de recomendaciones
- **AND** se envía `GET /api/v1/products/{ulid}/recommendations`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` de 503 `application/problem+json` no falla

#### Scenario: Gateway unavailable es 503 (unhappy)

- **WHEN** el Gateway inyectado falla con `unavailable`
- **AND** se envía `GET /api/v1/products/{ulid}/recommendations`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` de 503 `application/problem+json` no falla

#### Scenario: no hay caché de recomendaciones (unhappy, No hace)

- **WHEN** se envía un GET 200 de recomendaciones
- **THEN** la respuesta no incluye `Cache-Control` con `max-age` ni `public`
- **AND** el handler no sirve un resultado memorizado de una petición anterior

### Requirement: DTO inglés de Recommendations según 5.3

El cuerpo 200 MUST serializar `Recommendations` en inglés: `productId`, `highlightSubstitutes` (boolean), `similar`, `substitutes`, `complements`. Cada ítem MUST ser `RecommendedProduct`: `relationId`, `id`, `sku`, `name`, `stock` y `price` como string decimal (`NonNegativeDecimal`), `location` (`string` o `null`: línea formateada del dominio, p. ej. `P3 · E-B · 12`), `imageUrl` (`null` en MVP), `priority` (entero JSON, no string), `note` (`string` o `null`). MUST NOT serializar `producto_id`, `destacar_sustitutos`, `similares`, `sustitutos`, `complementarios`, `relacion_id`, `nombre`, `precio`, `ubicacion`, `prioridad` ni `nota`.

`errors[].field` MUST usar el nombre OpenAPI `productId` (no `producto`).

#### Scenario: GET no filtra claves de dominio (happy)

- **WHEN** un GET 200 tiene éxito con al menos un ítem
- **THEN** el JSON tiene `productId`, `highlightSubstitutes`, `similar`, `substitutes`, `complements`
- **AND** el ítem tiene `relationId`, `name`, `stock`, `price`, `priority`
- **AND** no tiene `producto_id`, `destacar_sustitutos`, `similares`, `nombre` ni `prioridad`

#### Scenario: problem field es productId no producto (unhappy)

- **WHEN** un GET produce `400 invalid_input` por ULID inválido
- **THEN** `errors[].field` es `productId`
- **AND** no es `producto`

### Requirement: Tests de api de recomendaciones

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-04-07. Listo cuando: respuesta validada contra `openapi.yaml`. Cada respuesta JSON (éxito o error) MUST validarse con `assert_conforms` (T-09-04). `cargo test --workspace` MUST ejecutarlos.

| Test | Módulo | Tipo |
|---|---|---|
| `get_recommendations_three_blocks_returns_200` | `crates/api/tests/recommendations.rs` | integración HTTP / happy |
| `get_recommendations_highlights_substitutes_when_stock_zero` | `crates/api/tests/recommendations.rs` | integración HTTP / happy |
| `get_recommendations_excludes_inactive_target` | `crates/api/tests/recommendations.rs` | integración HTTP / happy |
| `get_recommendations_empty_when_no_edges_returns_200` | `crates/api/tests/recommendations.rs` | integración HTTP / happy |
| `get_recommendations_image_url_is_null` | `crates/api/tests/recommendations.rs` | integración HTTP / happy |
| `get_recommendations_json_is_english` | `crates/api/tests/recommendations.rs` | integración HTTP / happy |
| `get_recommendations_records_duration_histogram` | `crates/api/tests/recommendations.rs` | integración HTTP / happy |
| `get_recommendations_invalid_ulid_returns_400` | `crates/api/tests/recommendations.rs` | integración HTTP / unhappy |
| `get_recommendations_unknown_product_returns_404` | `crates/api/tests/recommendations.rs` | integración HTTP / unhappy |
| `get_recommendations_without_service_returns_503` | `crates/api/tests/recommendations.rs` | integración HTTP / unhappy |
| `get_recommendations_unavailable_returns_503` | `crates/api/tests/recommendations.rs` | integración HTTP / unhappy |
| `get_recommendations_has_no_max_age_cache` | `crates/api/tests/recommendations.rs` | integración HTTP / unhappy |
| `vortex_serve_get_recommendations_is_not_404` | `bin/vortex/tests/serve.rs` | integración / happy |

#### Scenario: suite acordada de recomendaciones (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p api --test recommendations` y `cargo test -p vortex vortex_serve_get_recommendations_is_not_404`
- **THEN** todos los tests de la tabla pasan
- **AND** los JSON de éxito y error llaman `assert_conforms`

## MODIFIED Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST registrar las rutas de categorías (requirement «Endpoints /api/v1/categories»). El router MUST registrar las rutas de marcas (requirement «Endpoints /api/v1/brands»). El router MUST registrar las rutas de definiciones de atributo (requirement «Endpoints /api/v1/attribute-definitions»). El router MUST registrar las rutas de producto (requirement «Endpoints POST/GET/PATCH /api/v1/products»). El router MUST registrar `POST /api/v1/products/{productId}/stock-movements` (requirement «Endpoint POST /api/v1/products/{productId}/stock-movements»). El router MUST registrar `GET /api/v1/search` (requirement «Endpoint GET /api/v1/search»). El router MUST registrar las rutas de relaciones (requirement «Endpoints de relaciones»). El router MUST registrar `GET /api/v1/products/{productId}/recommendations` (requirement «Endpoint GET /api/v1/products/{productId}/recommendations»). MUST NOT registrar otras rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»). `AppState::new` / `with_health` MAY omitir el servicio de categorías, el de marcas, el de definiciones de atributo, el de producto, el de stock, el de búsqueda, el de relaciones y el de recomendaciones (los tests de `/health` y estáticos siguen construyéndose sin ellos).

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

#### Scenario: GET recommendations está registrado (happy)

- **WHEN** se envía `GET /api/v1/products/{ulid}/recommendations` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no tiene estado 404
- **AND** el JSON no tiene `code` `not_found`

### Requirement: Endpoints de relaciones

El router de `build_router` MUST registrar `POST /api/v1/relations`, `PATCH` y `DELETE /api/v1/relations/{relationType}/{relationId}` y `GET /api/v1/products/{productId}/relations` alineados con `docs/openapi.yaml` (`createRelation`, `updateRelation`, `deleteRelation`, `listProductRelations`). El Gateway MUST delegar en el `RelationGateway` inyectado en `AppState` (capability `vortex-recommend`) y MUST NOT implementar persistencia, bidireccionalidad ni existencia de producto.

`POST` MUST aceptar `RelationCreate` (`type`, `sourceProductId`, `targetProductId` obligatorios; `priority` defecto 1; `note` opcional ≤ 500; `bidirectional` opcional) y responder **201** `{ items }` con 1 `Relation` (unidireccional) o 2 (bidireccional, mismo `bidirectionalGroup`). `type` API MUST traducirse `similar`→`similar_a`, `substitute`→`sustituible_por`, `complement`→`complementa`. Si `bidirectional` se omite, MUST usarse el default del tipo (`true` para `similar`, `false` para el resto).

`PATCH` MUST aceptar `RelationUpdate` (`minProperties: 1`: `priority` y/o `note`) y responder **200** `{ items }` con la arista pedida y, si es bidireccional, la pareja. `DELETE` MUST responder **204** sin cuerpo; el grupo bidireccional se elimina entero.

`GET /api/v1/products/{productId}/relations` MUST responder **200** `{ items }` (admin: destinos inactivos y sin stock incluidos). Query `type` MUST filtrar por `RelationType`. Query `direction` MUST ser `outgoing` (defecto) o `incoming`. Producto inexistente MUST ser **404** `not_found`. Producto existente sin aristas MUST ser **200** `items: []`.

`{productId}` / `{relationId}` que no parsean como ULID MUST ser **400** `invalid_input` con `errors[].field` `productId` o `relationId`. `{relationType}` fuera de `similar`/`substitute`/`complement` MUST ser **400** `invalid_input` field `relationType`. Relación consigo mismo MUST ser **400** `self_relation`. Duplicada MUST ser **409** `duplicate_relation`. Producto o arista inexistente MUST ser **404** `not_found`. JSON malformado, `priority` &lt; 1 o `note` &gt; 500 MUST ser **400** `invalid_input`. `errors[].field` MUST usar nombres OpenAPI (`sourceProductId`, `targetProductId`, `priority`, `note`, `type`).

Si `AppState` no tiene servicio de relaciones, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`RelationService` + `RelacionRepo` + `ProductoRepo`). `GET /api/v1/products/{productId}/recommendations` vive en el requirement «Endpoint GET /api/v1/products/{productId}/recommendations». `docs/openapi.yaml` MUST declarar `503` en las cuatro operaciones. `AppState::new` / `with_health` MAY omitir el servicio de relaciones.

#### Scenario: POST complement Perno a Arandela (happy, HU-04-01)

- **WHEN** existen "Perno M8" y "Arandela M8"
- **AND** se envía `POST /api/v1/relations` con `type` `complement`, origen Perno, destino Arandela, `priority` 1
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `Content-Type` es `application/json`
- **AND** `items` tiene 1 elemento con `type` `complement`, `sourceProductId` del perno, `targetProductId` de la arandela, `priority` 1
- **AND** el JSON no tiene claves `origen`, `destino`, `prioridad` ni `tipo` en español
- **AND** `assert_conforms` de `POST /api/v1/relations` 201 `application/json` no falla

#### Scenario: POST similar bidireccional por defecto (happy, HU-04-02)

- **WHEN** existen productos A y B
- **AND** se envía `POST /api/v1/relations` con `type` `similar` entre A y B sin `bidirectional`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `items` tiene 2 elementos con el mismo `bidirectionalGroup`
- **AND** `GET /api/v1/products/{A}/relations` incluye a B y `GET` de B incluye a A
- **AND** `assert_conforms` de `POST /api/v1/relations` 201 `application/json` no falla

#### Scenario: PATCH prioridad del grupo desde B (happy, HU-04-02)

- **WHEN** existe un par bidireccional similar A↔B
- **AND** se envía `PATCH /api/v1/relations/similar/{id-B→A}` con `{ "priority": 3 }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye ambas aristas con `priority` 3
- **AND** `assert_conforms` de `PATCH /api/v1/relations/{relationType}/{relationId}` 200 `application/json` no falla

#### Scenario: DELETE del grupo desde B (happy, HU-04-02)

- **WHEN** existe un par bidireccional similar A↔B
- **AND** se envía `DELETE /api/v1/relations/similar/{id-B→A}`
- **THEN** la respuesta tiene estado HTTP 204
- **AND** `GET` de A y de B ya no listan esa relación

#### Scenario: GET listado admin agrupable por tipo (happy, HU-04-03)

- **WHEN** un producto tiene aristas de los tres tipos (incl. destino inactivo)
- **AND** se envía `GET /api/v1/products/{productId}/relations`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye las tres, con `priority`, `note` y `bidirectionalGroup` (null o ULID)
- **AND** `target.sku` y `target.name` están presentes; `target.active` es falso en el inactivo
- **AND** `assert_conforms` de `GET /api/v1/products/{productId}/relations` 200 `application/json` no falla

#### Scenario: GET direction incoming (happy)

- **WHEN** existe una arista saliente de A hacia B
- **AND** se envía `GET /api/v1/products/{B}/relations?direction=incoming`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye esa arista (source A, target B)

#### Scenario: POST self_relation (unhappy, HU-04-01, Listo cuando)

- **WHEN** se envía `POST /api/v1/relations` con el mismo ULID en `sourceProductId` y `targetProductId`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `self_relation`
- **AND** `errors[].field` es `targetProductId`
- **AND** `assert_conforms` de `POST /api/v1/relations` 400 `application/problem+json` no falla

#### Scenario: POST duplicate_relation (unhappy, HU-04-01, Listo cuando)

- **WHEN** ya existe `complement` de A hacia B
- **AND** se envía `POST` con el mismo par y tipo
- **THEN** la respuesta tiene estado HTTP 409
- **AND** el JSON tiene `code` `duplicate_relation`
- **AND** `assert_conforms` de `POST /api/v1/relations` 409 `application/problem+json` no falla

#### Scenario: POST producto inexistente (unhappy)

- **WHEN** se envía `POST` con un `targetProductId` que no existe
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** `assert_conforms` de `POST /api/v1/relations` 404 `application/problem+json` no falla

#### Scenario: GET producto inexistente (unhappy)

- **WHEN** se envía `GET /api/v1/products/{ulid-inexistente}/relations`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** `assert_conforms` de `GET /api/v1/products/{productId}/relations` 404 `application/problem+json` no falla

#### Scenario: PATCH id desconocido (unhappy)

- **WHEN** se envía `PATCH /api/v1/relations/similar/{ulid-inexistente}` con `{ "priority": 2 }`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: productId o relationId no ULID (unhappy)

- **WHEN** se envía `GET /api/v1/products/not-a-ulid/relations` o `DELETE /api/v1/relations/similar/not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `productId` o `relationId`

#### Scenario: relationType desconocido (unhappy)

- **WHEN** se envía `PATCH /api/v1/relations/sibling/{ulid}` con `{ "priority": 1 }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `relationType`

#### Scenario: POST priority 0 (unhappy)

- **WHEN** se envía `POST /api/v1/relations` con `priority` 0
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[].field` es `priority`

#### Scenario: sin servicio es 503 (unhappy)

- **WHEN** `AppState` no tiene servicio de relaciones
- **AND** se envía `POST /api/v1/relations` o `GET /api/v1/products/{ulid}/relations`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` del path/método 503 `application/problem+json` no falla

#### Scenario: GET recommendations no está registrado (unhappy, No hace)

- **WHEN** se envía `GET /api/v1/products/{ulid}/recommendations` al router de `build_router` (con o sin servicio)
- **THEN** la respuesta no es un 404 de ruta no registrada
- **AND** el comportamiento de negocio está en el requirement «Endpoint GET /api/v1/products/{productId}/recommendations» (T-04-07)

### Requirement: Tests de api de relaciones

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-04-05. Listo cuando: escenarios de HU-04-01 y HU-04-02 como tests de `api` validados contra `openapi.yaml`. Cada respuesta JSON (éxito o error) MUST validarse con `assert_conforms` (T-09-04). `cargo test --workspace` MUST ejecutarlos. El test `get_recommendations_is_not_registered` MUST NOT formar parte de esta suite (T-04-07 registra la ruta; ver «Tests de api de recomendaciones»).

| Test | Módulo | Tipo |
|---|---|---|
| `post_relation_complement_perno_arandela_returns_201` | `crates/api/tests/relations.rs` | integración HTTP / happy |
| `post_relation_similar_omits_bidirectional_creates_pair` | `crates/api/tests/relations.rs` | integración HTTP / happy |
| `patch_relation_priority_from_b_updates_both` | `crates/api/tests/relations.rs` | integración HTTP / happy |
| `delete_relation_from_b_removes_both` | `crates/api/tests/relations.rs` | integración HTTP / happy |
| `get_product_relations_lists_three_types` | `crates/api/tests/relations.rs` | integración HTTP / happy |
| `get_product_relations_incoming` | `crates/api/tests/relations.rs` | integración HTTP / happy |
| `post_relation_self_returns_400` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `post_relation_duplicate_returns_409` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `post_relation_missing_product_returns_404` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `get_product_relations_unknown_product_returns_404` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `patch_relation_unknown_id_returns_404` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `relation_id_invalid_ulid_returns_400` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `relation_type_unknown_returns_400` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `post_relation_priority_zero_returns_400` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `post_relation_without_service_returns_503` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `relation_problem_field_is_target_product_id` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `vortex_serve_get_unknown_product_relations_returns_404` | `bin/vortex/tests/serve.rs` | integración / unhappy |

#### Scenario: suite acordada de relaciones (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p api --test relations` y `cargo test -p vortex vortex_serve_get_unknown_product_relations_returns_404`
- **THEN** todos los tests de la tabla pasan
- **AND** los JSON de éxito y error llaman `assert_conforms`
