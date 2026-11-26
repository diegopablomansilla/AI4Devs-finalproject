# Spec Delta

Ticket padre: T-02-23. Change: T-02-23. HU: HU-02-03 (escenario «Atributo en uso»).

## MODIFIED Requirements

### Requirement: Endpoints /api/v1/attribute-definitions

El router de `build_router` MUST registrar `GET` y `POST /api/v1/attribute-definitions` y `PATCH` y `DELETE /api/v1/attribute-definitions/{attributeId}` alineados con `docs/openapi.yaml`. El Gateway MUST delegar en el servicio de definiciones de atributo inyectado en `AppState` (capability `catalog-atributo-def`) y MUST NOT implementar persistencia ni la consulta de occupancy (la aplica `catalog`).

`GET` MUST devolver `{ "items": [...] }` ordenado por `order` ascendente y, en empate, por `key` lexicográfico. Sin query o con `includeInactive=false`, MUST omitir definiciones con `active = false`. Con `includeInactive=true`, MUST incluir inactivas. MUST NOT filtrar por `q` (el contrato no declara ese query). MUST NOT paginar.

`POST` MUST crear con cuerpo `AttributeDefinitionCreate` (`key`, `label` y `type` obligatorios; `unit`/`options` opcionales; `facetable` default `false`; `searchable` default `true`; `order` default `0`) y responder `201` con `AttributeDefinition`. `PATCH` MUST aplicar el cuerpo `AttributeDefinitionUpdate` (`minProperties: 1`): fusionar los campos presentes con la fila actual; `active: true` reactiva; `active: false` da de baja lógica. `DELETE` MUST dar de baja lógica y responder `204` sin cuerpo. `PATCH { "active": false }` MUST ser la misma baja que `DELETE`.

Si ningún producto usa la definición, cambiar `key` o `type` MUST responder **200**. Si algún producto la usa y el `PATCH` cambia `key`, MUST responder **409** `attribute_in_use` (`application/problem+json`, título «El atributo está en uso»). Si cambia `type` u `options` de forma incompatible con valores existentes, MUST responder **409** `attribute_in_use`. Cambiar `label` (u otros campos no estructurales) de una definición en uso MUST responder **200**. `DELETE` de una definición en uso MUST seguir siendo **204** (MUST NOT `409`). `docs/openapi.yaml` ya describe esos `409` en PATCH; este ticket MUST NOT agregar `409` en `DELETE`.

`{attributeId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `attributeId`. Id inexistente MUST ser `404` `not_found`. Clave duplicada MUST ser `409` `conflict`. `enum` sin opciones MUST ser `400` `invalid_input`. Cuerpo JSON vacío en `PATCH` o sin `key`/`label`/`type` en `POST` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de definiciones de atributo, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`AtributoDefRepo` + adaptador `AttributeUsage`). MUST NOT registrar `GET /api/v1/attribute-definitions/{attributeId}`. MUST NOT reindexar productos (T-03-08). `docs/openapi.yaml` MUST declarar `503` en las operaciones de `/api/v1/attribute-definitions` y `/api/v1/attribute-definitions/{attributeId}`. MUST NOT declarar `409` en `DELETE`.

#### Scenario: POST crea grado enum (happy, HU-02-03)

- **WHEN** se envía `POST /api/v1/attribute-definitions` con `{ "key": "grado", "label": "Grado", "type": "enum", "options": ["2", "5", "8"], "facetable": true }`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `key` `"grado"`, `type` `"enum"`, `options` `["2", "5", "8"]`, `facetable` verdadero y `active` verdadero
- **AND** el JSON no tiene claves `clave`, `etiqueta`, `tipo` ni `activo`
- **AND** `assert_conforms` de `POST /api/v1/attribute-definitions` 201 `application/json` no falla

#### Scenario: GET lista sin inactivas (happy)

- **WHEN** existen una definición activa y una inactiva
- **AND** se envía `GET /api/v1/attribute-definitions`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye la activa y no incluye la inactiva
- **AND** `assert_conforms` de `GET /api/v1/attribute-definitions` 200 `application/json` no falla

#### Scenario: GET includeInactive incluye inactivas (happy)

- **WHEN** existe al menos una definición inactiva
- **AND** se envía `GET /api/v1/attribute-definitions?includeInactive=true`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye la inactiva

#### Scenario: GET ordena por order (happy)

- **WHEN** existen definiciones con `order` 10 (`medida`) y `order` 0 (`grado`)
- **AND** se envía `GET /api/v1/attribute-definitions`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items[0].key` es `"grado"`
- **AND** `items[1].key` es `"medida"`

#### Scenario: PATCH actualiza label (happy)

- **WHEN** existe "grado" y se envía `PATCH /api/v1/attribute-definitions/{id}` con `{ "label": "Grado SAE" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `label` `"Grado SAE"`
- **AND** `assert_conforms` de `PATCH /api/v1/attribute-definitions/{attributeId}` 200 `application/json` no falla

#### Scenario: PATCH cambia key sin attribute_in_use (happy)

- **WHEN** existe "medida" y ningún producto la usa
- **AND** se envía `PATCH` con `{ "key": "medida_mm" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `key` `"medida_mm"`
- **AND** el JSON no tiene `code` `attribute_in_use`

#### Scenario: PATCH key de medida en uso (unhappy, HU-02-03, Listo cuando)

- **WHEN** existe "medida" usada por un producto con valor `"M8"`
- **AND** se envía `PATCH` con `{ "key": "medida_mm" }`
- **THEN** la respuesta tiene estado HTTP 409
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `attribute_in_use`
- **AND** `assert_conforms` de `PATCH /api/v1/attribute-definitions/{attributeId}` 409 `application/problem+json` no falla

#### Scenario: PATCH type incompatible de medida en uso (unhappy, HU-02-03)

- **WHEN** existe "medida" (`type` `text`) usada con valor `"M8"`
- **AND** se envía `PATCH` con `{ "type": "number" }`
- **THEN** la respuesta tiene estado HTTP 409
- **AND** el JSON tiene `code` `attribute_in_use`
- **AND** `assert_conforms` de `PATCH /api/v1/attribute-definitions/{attributeId}` 409 `application/problem+json` no falla

#### Scenario: PATCH label de medida en uso (happy)

- **WHEN** existe "medida" usada por un producto
- **AND** se envía `PATCH` con `{ "label": "Medida métrica" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `label` `"Medida métrica"` y `key` `"medida"`

#### Scenario: DELETE de definición en uso (happy)

- **WHEN** existe "medida" usada por un producto
- **AND** se envía `DELETE /api/v1/attribute-definitions/{id}`
- **THEN** la respuesta tiene estado HTTP 204
- **AND** el cuerpo está vacío

#### Scenario: PATCH reactiva (happy)

- **WHEN** "grado" está inactiva
- **AND** se envía `PATCH` con `{ "active": true }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `active` es `true`

#### Scenario: DELETE da de baja (happy)

- **WHEN** existe "grado"
- **AND** se envía `DELETE /api/v1/attribute-definitions/{id}`
- **THEN** la respuesta tiene estado HTTP 204
- **AND** el cuerpo está vacío

#### Scenario: POST clave duplicada (unhappy)

- **WHEN** ya existe una definición con `key` `"grado"`
- **AND** se envía `POST` con esa `key`
- **THEN** la respuesta tiene estado HTTP 409
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `conflict`
- **AND** `assert_conforms` de `POST /api/v1/attribute-definitions` 409 `application/problem+json` no falla

#### Scenario: POST enum sin opciones (unhappy, HU-02-03)

- **WHEN** se envía `POST` con `{ "key": "grado", "label": "Grado", "type": "enum" }` sin `options` o con `options` vacío
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `assert_conforms` de `POST /api/v1/attribute-definitions` 400 `application/problem+json` no falla

#### Scenario: PATCH id desconocido (unhappy)

- **WHEN** se envía `PATCH /api/v1/attribute-definitions/{ulid-inexistente}` con `{ "label": "X" }`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: DELETE id desconocido (unhappy)

- **WHEN** se envía `DELETE /api/v1/attribute-definitions/{ulid-inexistente}`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: POST sin key (unhappy)

- **WHEN** se envía `POST /api/v1/attribute-definitions` con `{ "label": "Grado", "type": "enum", "options": ["2"] }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: PATCH cuerpo vacío (unhappy)

- **WHEN** se envía `PATCH` con `{ }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: attributeId no es ULID (unhappy)

- **WHEN** se envía `DELETE /api/v1/attribute-definitions/not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `attributeId`

#### Scenario: sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de definiciones de atributo
- **AND** se envía `GET /api/v1/attribute-definitions`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`

### Requirement: Tests de integración de definiciones de atributo contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-23 (extiende T-02-12). Listo cuando: tests con un producto que usa "medida"; cada respuesta JSON (éxito o error) se valida con `assert_conforms` (T-09-04) salvo `DELETE` 204 (sin cuerpo). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_attribute_def_grado_returns_201` | `api` | integración, happy, HU-02-03 |
| `get_attribute_defs_omits_inactive_by_default` | `api` | integración, happy |
| `get_attribute_defs_include_inactive_lists_inactive` | `api` | integración, happy |
| `get_attribute_defs_ordered_by_order` | `api` | integración, happy |
| `patch_attribute_def_updates_label` | `api` | integración, happy |
| `patch_attribute_def_changes_key_without_in_use` | `api` | integración, happy |
| `patch_attribute_def_key_in_use_returns_409` | `api` | integración, unhappy, HU-02-03, Listo cuando |
| `patch_attribute_def_type_incompatible_in_use_returns_409` | `api` | integración, unhappy, HU-02-03 |
| `patch_attribute_def_label_in_use_returns_200` | `api` | integración, happy |
| `patch_attribute_def_options_incompatible_in_use_returns_409` | `api` | integración, unhappy |
| `delete_attribute_def_in_use_returns_204` | `api` | integración, happy |
| `patch_attribute_def_reactivates` | `api` | integración, happy |
| `delete_attribute_def_returns_204` | `api` | integración, happy |
| `post_attribute_def_duplicate_key_returns_409` | `api` | integración, unhappy |
| `post_attribute_def_enum_without_options_returns_400` | `api` | integración, unhappy, HU-02-03 |
| `patch_attribute_def_unknown_id_returns_404` | `api` | integración, unhappy |
| `delete_attribute_def_unknown_id_returns_404` | `api` | integración, unhappy |
| `post_attribute_def_missing_key_returns_400` | `api` | integración, unhappy |
| `patch_attribute_def_empty_body_returns_400` | `api` | integración, unhappy |
| `attribute_id_invalid_ulid_returns_400` | `api` | integración, unhappy |
| `get_attribute_defs_without_service_returns_503` | `api` | integración, unhappy |
| `attribute_def_problem_field_is_key_not_clave` | `api` | integración, unhappy, 5.3 |
| `vortex_serve_get_attribute_definitions_returns_items` | `vortex` | binario, happy |

#### Scenario: la suite de attribute-definitions cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 200/201/400/404/409/503 (salvo 204) pasan `assert_conforms`
