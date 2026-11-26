# Spec Delta

Ticket padre: T-04-05. Change: T-04-05. HU: HU-04-01, HU-04-02, HU-04-03.

## ADDED Requirements

### Requirement: Endpoints de relaciones

El router de `build_router` MUST registrar `POST /api/v1/relations`, `PATCH` y `DELETE /api/v1/relations/{relationType}/{relationId}` y `GET /api/v1/products/{productId}/relations` alineados con `docs/openapi.yaml` (`createRelation`, `updateRelation`, `deleteRelation`, `listProductRelations`). El Gateway MUST delegar en el `RelationGateway` inyectado en `AppState` (capability `vortex-recommend`) y MUST NOT implementar persistencia, bidireccionalidad ni existencia de producto.

`POST` MUST aceptar `RelationCreate` (`type`, `sourceProductId`, `targetProductId` obligatorios; `priority` defecto 1; `note` opcional ≤ 500; `bidirectional` opcional) y responder **201** `{ items }` con 1 `Relation` (unidireccional) o 2 (bidireccional, mismo `bidirectionalGroup`). `type` API MUST traducirse `similar`→`similar_a`, `substitute`→`sustituible_por`, `complement`→`complementa`. Si `bidirectional` se omite, MUST usarse el default del tipo (`true` para `similar`, `false` para el resto).

`PATCH` MUST aceptar `RelationUpdate` (`minProperties: 1`: `priority` y/o `note`) y responder **200** `{ items }` con la arista pedida y, si es bidireccional, la pareja. `DELETE` MUST responder **204** sin cuerpo; el grupo bidireccional se elimina entero.

`GET /api/v1/products/{productId}/relations` MUST responder **200** `{ items }` (admin: destinos inactivos y sin stock incluidos). Query `type` MUST filtrar por `RelationType`. Query `direction` MUST ser `outgoing` (defecto) o `incoming`. Producto inexistente MUST ser **404** `not_found`. Producto existente sin aristas MUST ser **200** `items: []`.

`{productId}` / `{relationId}` que no parsean como ULID MUST ser **400** `invalid_input` con `errors[].field` `productId` o `relationId`. `{relationType}` fuera de `similar`/`substitute`/`complement` MUST ser **400** `invalid_input` field `relationType`. Relación consigo mismo MUST ser **400** `self_relation`. Duplicada MUST ser **409** `duplicate_relation`. Producto o arista inexistente MUST ser **404** `not_found`. JSON malformado, `priority` &lt; 1 o `note` &gt; 500 MUST ser **400** `invalid_input`. `errors[].field` MUST usar nombres OpenAPI (`sourceProductId`, `targetProductId`, `priority`, `note`, `type`).

Si `AppState` no tiene servicio de relaciones, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`RelationService` + `RelacionRepo` + `ProductoRepo`). MUST NOT registrar `GET /api/v1/products/{productId}/recommendations` (T-04-07). `docs/openapi.yaml` MUST declarar `503` en las cuatro operaciones. `AppState::new` / `with_health` MAY omitir el servicio de relaciones.

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

- **WHEN** se envía `GET /api/v1/products/{ulid}/recommendations`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

### Requirement: DTO inglés de Relation según 5.3

El cuerpo 201/200 MUST serializar `Relation` en inglés: `id`, `type` (`similar`/`substitute`/`complement`), `sourceProductId`, `targetProductId`, `priority`, `note`, `bidirectionalGroup`, `createdAt`, `updatedAt`, `target` (`ProductSummary` mínimo: `id`, `sku`, `name`, `active` desde `DestinoResumen`; `productType` `""`; `price` y `stock` `"0"`; `brand` null; `categoryPath` `[]`; `imageUrl` null). `priority` MUST ser entero JSON (no string). Timestamps MUST ser RFC 3339 UTC. MUST NOT serializar `origen`, `destino`, `prioridad`, `nota`, `grupo_bidireccional` ni `similar_a`.

`errors[].field` MUST remapear `origen`→`sourceProductId`, `destino`→`targetProductId`, `prioridad`→`priority`, `nota`→`note`; `tipo` MAY quedar `type` (remap global).

#### Scenario: POST no filtra claves de dominio (happy)

- **WHEN** un POST 201 tiene éxito
- **THEN** el JSON tiene `sourceProductId`, `targetProductId`, `priority`, `type`
- **AND** no tiene `origen`, `destino`, `prioridad` ni `similar_a`

#### Scenario: problem field es targetProductId no destino (unhappy)

- **WHEN** un POST produce `self_relation`
- **THEN** `errors[].field` es `targetProductId`
- **AND** no es `destino`

### Requirement: Tests de api de relaciones

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-04-05. Listo cuando: escenarios de HU-04-01 y HU-04-02 como tests de `api` validados contra `openapi.yaml`. Cada respuesta JSON (éxito o error) MUST validarse con `assert_conforms` (T-09-04). `cargo test --workspace` MUST ejecutarlos.

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
| `get_recommendations_is_not_registered` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `relation_problem_field_is_target_product_id` | `crates/api/tests/relations.rs` | integración HTTP / unhappy |
| `vortex_serve_get_unknown_product_relations_returns_404` | `bin/vortex/tests/serve.rs` | integración / unhappy |

#### Scenario: suite acordada de relaciones (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p api --test relations` y `cargo test -p vortex vortex_serve_get_unknown_product_relations_returns_404`
- **THEN** todos los tests de la tabla pasan
- **AND** los JSON de éxito y error llaman `assert_conforms`
