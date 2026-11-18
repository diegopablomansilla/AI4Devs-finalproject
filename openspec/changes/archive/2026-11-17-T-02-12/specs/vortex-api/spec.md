# Spec Delta

## ADDED Requirements

### Requirement: Endpoints /api/v1/attribute-definitions

El router de `build_router` MUST registrar `GET` y `POST /api/v1/attribute-definitions` y `PATCH` y `DELETE /api/v1/attribute-definitions/{attributeId}` alineados con `docs/openapi.yaml`. El Gateway MUST delegar en el servicio de definiciones de atributo inyectado en `AppState` (capability `catalog-atributo-def`) y MUST NOT implementar persistencia ni `attribute_in_use`.

`GET` MUST devolver `{ "items": [...] }` ordenado por `order` ascendente y, en empate, por `key` lexicográfico. Sin query o con `includeInactive=false`, MUST omitir definiciones con `active = false`. Con `includeInactive=true`, MUST incluir inactivas. MUST NOT filtrar por `q` (el contrato no declara ese query). MUST NOT paginar.

`POST` MUST crear con cuerpo `AttributeDefinitionCreate` (`key`, `label` y `type` obligatorios; `unit`/`options` opcionales; `facetable` default `false`; `searchable` default `true`; `order` default `0`) y responder `201` con `AttributeDefinition`. `PATCH` MUST aplicar el cuerpo `AttributeDefinitionUpdate` (`minProperties: 1`): fusionar los campos presentes con la fila actual; `active: true` reactiva; `active: false` da de baja lógica. `DELETE` MUST dar de baja lógica y responder `204` sin cuerpo. `PATCH { "active": false }` MUST ser la misma baja que `DELETE`. Cambiar `key` o `type` MUST suceder en este ticket: MUST NOT responder `409 attribute_in_use` (T-02-23).

`{attributeId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `attributeId`. Id inexistente MUST ser `404` `not_found`. Clave duplicada MUST ser `409` `conflict`. `enum` sin opciones MUST ser `400` `invalid_input`. Cuerpo JSON vacío en `PATCH` o sin `key`/`label`/`type` en `POST` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de definiciones de atributo, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`AtributoDefRepo`). MUST NOT registrar `GET /api/v1/attribute-definitions/{attributeId}`. MUST NOT reindexar productos (T-03-08). `docs/openapi.yaml` MUST declarar `503` en las operaciones de `/api/v1/attribute-definitions` y `/api/v1/attribute-definitions/{attributeId}`. MUST NOT declarar `409` en `DELETE`.

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

- **WHEN** existe "medida" y se envía `PATCH` con `{ "key": "medida_mm" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `key` `"medida_mm"`
- **AND** el JSON no tiene `code` `attribute_in_use`

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

### Requirement: DTO inglés de definición de atributo según 5.3

El Gateway MUST traducir el contrato inglés al dominio español y de vuelta. Campos de respuesta `AttributeDefinition`: `id`, `key`, `label`, `type`, `unit`, `options`, `facetable`, `searchable`, `order`, `active`. MUST NOT serializar `clave`, `etiqueta`, `tipo`, `unidad`, `opciones`, `buscable`, `orden`, `activo`, `creado_en` ni `actualizado_en`. `POST`/`PATCH` MUST leer `key` → `clave`, `label` → `etiqueta`, `type` → `tipo` (`text`→`texto`, `number`→`numero`, `boolean`→`booleano`, `enum`→`enum`), `unit` → `unidad`, `options` → `opciones`, `searchable` → `buscable`, `order` → `orden`, `active` → `activo`. En `problem+json`, `errors[].field` MUST usar nombres de API (`key`, `label`, `type`, `options`, `unit`, `attributeId`, `body`), nunca `clave`, `etiqueta`, `tipo` ni `opciones`.

#### Scenario: respuesta sin claves de dominio (happy)

- **WHEN** `POST` crea la definición `"grado"`
- **THEN** el JSON de 201 tiene `id`, `key`, `label`, `type`, `facetable`, `searchable`, `order` y `active`
- **AND** no tiene `clave`, `etiqueta`, `tipo` ni `activo`

#### Scenario: errors[].field en inglés (unhappy)

- **WHEN** el servicio rechaza una `clave` inválida con `FieldError.field` `clave`
- **AND** el Gateway traduce ese error a HTTP
- **THEN** `errors[].field` es `key`
- **AND** no aparece `clave` en el cuerpo

### Requirement: Tests de integración de definiciones de atributo contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-12. Listo cuando: cada respuesta JSON (éxito o error) se valida con `assert_conforms` (T-09-04) salvo `DELETE` 204 (sin cuerpo). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_attribute_def_grado_returns_201` | `api` | integración, happy, HU-02-03 |
| `get_attribute_defs_omits_inactive_by_default` | `api` | integración, happy |
| `get_attribute_defs_include_inactive_lists_inactive` | `api` | integración, happy |
| `get_attribute_defs_ordered_by_order` | `api` | integración, happy |
| `patch_attribute_def_updates_label` | `api` | integración, happy |
| `patch_attribute_def_changes_key_without_in_use` | `api` | integración, happy |
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

## MODIFIED Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST registrar las rutas de categorías (requirement «Endpoints /api/v1/categories»). El router MUST registrar las rutas de marcas (requirement «Endpoints /api/v1/brands»). El router MUST registrar las rutas de definiciones de atributo (requirement «Endpoints /api/v1/attribute-definitions»). MUST NOT registrar otras rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»). `AppState::new` / `with_health` MAY omitir el servicio de categorías, el de marcas y el de definiciones de atributo (los tests de `/health` y estáticos siguen construyéndose sin ellos).

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

### Requirement: vortex serve escucha HTTP en server.bind

`vortex serve` MUST dejar de responder `no implementado`. MUST resolver el path de configuración como en `vortex-config` (`--config` gana sobre `VORTEX_CONFIG`), MUST cargar y validar el TOML, MUST instalar el subscriber de logging (capability `vortex-logging`), MUST abrir Storage y migrar según el requirement «Arranque ordenado de vortex serve (paso 1)», y MUST escuchar HTTP (sin TLS) en la dirección `server.bind` **solo después** de esas fases. MUST cablear un registro de salud que implementa `HealthReport`. MUST inyectar en `AppState` `paths.web` como raíz `web`, `paths.datos` unido a `media` como raíz `media`, y la `version` del binario. MUST inyectar el servicio de categorías (adaptador `CategoryRepo` de `storage` + occupancy stub que no reporta productos activos) y MUST inyectar el servicio de marcas (adaptador `BrandRepo` / `MarcaRepo` de `storage`) y MUST inyectar el servicio de definiciones de atributo (adaptador `AttributeDefRepo` / `AtributoDefRepo` de `storage`) y MUST NOT inyectar `Storage` crudo en `AppState`. MUST NOT arrancar sidecars. Una petición HTTP a una ruta no registrada MUST responder 404 con `Content-Type: application/problem+json` y `code` `not_found`. `GET /health` contra ese bind MUST responder según el requirement «GET /health expone el snapshot agregado» y, en un arranque exitoso de este change, MUST coincidir con «Arranque ordenado de vortex serve (paso 1)». `GET /metrics` contra ese bind MUST responder según el requirement «GET /metrics expone texto Prometheus». `GET /` contra ese bind MUST responder según el requirement «Estáticos de la SPA y /media» cuando `{web}/index.html` existe. `GET /api/v1/categories` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/brands` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/attribute-definitions` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`.

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
