# Spec Delta

## ADDED Requirements

### Requirement: Endpoints /api/v1/categories

El router de `build_router` MUST registrar `GET` y `POST /api/v1/categories` y `PATCH` y `DELETE /api/v1/categories/{categoryId}` alineados con `docs/openapi.yaml`. El Gateway MUST delegar en el servicio de categorías inyectado en `AppState` (capability `catalog-categoria`) y MUST NOT implementar reglas de árbol, H-10 ni persistencia.

`GET` MUST devolver `{ "items": [...] }` con lista plana. Sin query o con `includeInactive=false`, MUST omitir categorías con `active = false`. Con `includeInactive=true`, MUST incluir inactivas. MUST NOT paginar.

`POST` MUST crear con cuerpo `CategoryCreate` (`name` obligatorio; `slug` y `parentId` opcionales) y responder `201` con `Category`. `PATCH` MUST aplicar el cuerpo `CategoryUpdate` (`minProperties: 1`): `name` renombra; `parentId` mueve (incl. `null` = raíz); `active: true` reactiva; `active: false` da de baja lógica. `DELETE` MUST dar de baja lógica y responder `204` sin cuerpo. `PATCH { "active": false }` MUST ser la misma baja que `DELETE`.

`{categoryId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `categoryId`. Id inexistente MUST ser `404` `not_found`. Slug duplicado MUST ser `409` `duplicate_slug`. Ciclo MUST ser `400` `category_cycle`. Baja con hija activa MUST ser `409` `conflict` (H-10). Cuerpo JSON vacío en `PATCH` o sin `name` en `POST` MUST ser `400` `invalid_input`. `PATCH` que solo envía `slug` MUST ser `400` `invalid_input` (`errors[].field` `slug`) — no hay caso de uso de slug en T-02-05.

Si `AppState` no tiene servicio de categorías, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (occupancy stub `false` hasta T-02-18). MUST NOT registrar `GET /api/v1/categories/{categoryId}`. MUST NOT reindexar productos (T-03-08). `docs/openapi.yaml` MUST declarar `409` en `DELETE /api/v1/categories/{categoryId}` (mismo `Conflict` que `PATCH`).

#### Scenario: POST crea subcategoría (happy)

- **WHEN** existe "Fijaciones" y se envía `POST /api/v1/categories` con `{ "name": "Pernos", "parentId": "<id-fijaciones>" }`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `name` `"Pernos"`, `parentId` el id de "Fijaciones" y `path` `["Fijaciones", "Pernos"]`
- **AND** el JSON no tiene claves `nombre`, `padre` ni `ruta`
- **AND** `assert_conforms` de `POST /api/v1/categories` 201 `application/json` no falla

#### Scenario: GET lista plana sin inactivas (happy)

- **WHEN** existen una categoría activa y una inactiva
- **AND** se envía `GET /api/v1/categories`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye la activa y no incluye la inactiva
- **AND** `assert_conforms` de `GET /api/v1/categories` 200 `application/json` no falla

#### Scenario: GET includeInactive incluye inactivas (happy)

- **WHEN** existe al menos una categoría inactiva
- **AND** se envía `GET /api/v1/categories?includeInactive=true`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye la inactiva

#### Scenario: PATCH renombra (happy)

- **WHEN** existe "Fijaciones" y se envía `PATCH /api/v1/categories/{id}` con `{ "name": "Tornillería" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `name` `"Tornillería"`
- **AND** `assert_conforms` de `PATCH /api/v1/categories/{categoryId}` 200 `application/json` no falla

#### Scenario: PATCH mueve (happy)

- **WHEN** "Pernos" es hija de "Fijaciones" y existe "Herramientas"
- **AND** se envía `PATCH` de "Pernos" con `{ "parentId": "<id-herramientas>" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `parentId` es el id de "Herramientas"

#### Scenario: PATCH reactiva (happy)

- **WHEN** "Pernos" está inactiva
- **AND** se envía `PATCH` con `{ "active": true }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `active` es `true`

#### Scenario: DELETE da de baja (happy)

- **WHEN** "Pernos" es hoja sin hijas activas
- **AND** se envía `DELETE /api/v1/categories/{id}`
- **THEN** la respuesta tiene estado HTTP 204
- **AND** el cuerpo está vacío

#### Scenario: POST slug duplicado (unhappy)

- **WHEN** ya existe una categoría con el mismo slug bajo el mismo padre
- **AND** se envía `POST` con ese `name`/`slug` y el mismo `parentId`
- **THEN** la respuesta tiene estado HTTP 409
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `duplicate_slug`
- **AND** `assert_conforms` de `POST /api/v1/categories` 409 `application/problem+json` no falla

#### Scenario: PATCH ciclo (unhappy)

- **WHEN** "Pernos" es hija de "Fijaciones"
- **AND** se envía `PATCH` de "Fijaciones" con `{ "parentId": "<id-pernos>" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `category_cycle`
- **AND** `assert_conforms` de `PATCH /api/v1/categories/{categoryId}` 400 `application/problem+json` no falla

#### Scenario: DELETE con hija activa (unhappy)

- **WHEN** "Pernos" es hija activa de "Fijaciones"
- **AND** se envía `DELETE` de "Fijaciones"
- **THEN** la respuesta tiene estado HTTP 409
- **AND** el JSON tiene `code` `conflict`
- **AND** `assert_conforms` de `DELETE /api/v1/categories/{categoryId}` 409 `application/problem+json` no falla

#### Scenario: PATCH id desconocido (unhappy)

- **WHEN** se envía `PATCH /api/v1/categories/{ulid-inexistente}` con `{ "name": "X" }`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: DELETE id desconocido (unhappy)

- **WHEN** se envía `DELETE /api/v1/categories/{ulid-inexistente}`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: POST sin name (unhappy)

- **WHEN** se envía `POST /api/v1/categories` con `{ "parentId": null }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: PATCH solo slug (unhappy)

- **WHEN** se envía `PATCH` con `{ "slug": "nuevo-slug" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `slug`

#### Scenario: categoryId no es ULID (unhappy)

- **WHEN** se envía `DELETE /api/v1/categories/not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `categoryId`

#### Scenario: sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de categorías
- **AND** se envía `GET /api/v1/categories`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`

### Requirement: DTO inglés de categoría según 5.3

El Gateway MUST traducir el contrato inglés al dominio español y de vuelta. Campos de respuesta `Category`: `id`, `name`, `slug`, `parentId` (ULID o `null`), `ancestorIds`, `path`, `active`. MUST NOT serializar `nombre`, `padre`, `ancestros`, `ruta`, `activo`, `creado_en` ni `actualizado_en`. `POST`/`PATCH` MUST leer `name` → `nombre`, `parentId` → `padre`, `slug` → `slug`, `active` → `activo`. En `problem+json`, `errors[].field` MUST usar nombres de API (`name`, `parentId`, `slug`, `categoryId`, `body`), nunca `nombre` ni `padre`.

#### Scenario: respuesta sin claves de dominio (happy)

- **WHEN** `POST` crea una categoría raíz `"Fijaciones"`
- **THEN** el JSON de 201 tiene `name`, `slug`, `parentId`, `ancestorIds`, `path` y `active`
- **AND** no tiene `nombre`, `padre`, `ancestros`, `ruta` ni `activo`

#### Scenario: errors[].field en inglés (unhappy)

- **WHEN** el servicio rechaza un ciclo con `FieldError.field` `padre`
- **AND** el Gateway traduce ese error a HTTP
- **THEN** `errors[].field` es `parentId`
- **AND** no aparece `padre` en el cuerpo

### Requirement: Tests de integración de categorías contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-06. Listo cuando: cada respuesta JSON (éxito o error) se valida con `assert_conforms` (T-09-04) salvo `DELETE` 204 (sin cuerpo). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_category_child_returns_201_with_path` | `api` | integración, happy, HU-02-01 |
| `get_categories_omits_inactive_by_default` | `api` | integración, happy |
| `get_categories_include_inactive_lists_inactive` | `api` | integración, happy |
| `patch_category_renames` | `api` | integración, happy |
| `patch_category_moves` | `api` | integración, happy |
| `patch_category_reactivates` | `api` | integración, happy |
| `delete_category_returns_204` | `api` | integración, happy |
| `post_category_duplicate_slug_returns_409` | `api` | integración, unhappy, HU-02-01 |
| `patch_category_cycle_returns_400` | `api` | integración, unhappy, HU-02-01 |
| `delete_category_with_active_child_returns_409` | `api` | integración, unhappy, H-10 |
| `patch_category_unknown_id_returns_404` | `api` | integración, unhappy |
| `delete_category_unknown_id_returns_404` | `api` | integración, unhappy |
| `post_category_missing_name_returns_400` | `api` | integración, unhappy |
| `patch_category_slug_only_returns_400` | `api` | integración, unhappy |
| `category_id_invalid_ulid_returns_400` | `api` | integración, unhappy |
| `get_categories_without_service_returns_503` | `api` | integración, unhappy |
| `category_problem_field_is_parent_id_not_padre` | `api` | integración, unhappy, 5.3 |
| `vortex_serve_get_categories_returns_items` | `vortex` | binario, happy |

#### Scenario: la suite de categorías cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 200/201/400/404/409/503 (salvo 204) pasan `assert_conforms`

## MODIFIED Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST registrar las rutas de categorías (requirement «Endpoints /api/v1/categories»). MUST NOT registrar otras rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»). `AppState::new` / `with_health` MAY omitir el servicio de categorías (los tests de `/health` y estáticos siguen construyéndose sin él).

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

### Requirement: vortex serve escucha HTTP en server.bind

`vortex serve` MUST dejar de responder `no implementado`. MUST resolver el path de configuración como en `vortex-config` (`--config` gana sobre `VORTEX_CONFIG`), MUST cargar y validar el TOML, MUST instalar el subscriber de logging (capability `vortex-logging`), MUST abrir Storage y migrar según el requirement «Arranque ordenado de vortex serve (paso 1)», y MUST escuchar HTTP (sin TLS) en la dirección `server.bind` **solo después** de esas fases. MUST cablear un registro de salud que implementa `HealthReport`. MUST inyectar en `AppState` `paths.web` como raíz `web`, `paths.datos` unido a `media` como raíz `media`, y la `version` del binario. MUST inyectar el servicio de categorías (adaptador `CategoryRepo` de `storage` + occupancy stub que no reporta productos activos) y MUST NOT inyectar `Storage` crudo en `AppState`. MUST NOT arrancar sidecars. Una petición HTTP a una ruta no registrada MUST responder 404 con `Content-Type: application/problem+json` y `code` `not_found`. `GET /health` contra ese bind MUST responder según el requirement «GET /health expone el snapshot agregado» y, en un arranque exitoso de este change, MUST coincidir con «Arranque ordenado de vortex serve (paso 1)». `GET /metrics` contra ese bind MUST responder según el requirement «GET /metrics expone texto Prometheus». `GET /` contra ese bind MUST responder según el requirement «Estáticos de la SPA y /media» cuando `{web}/index.html` existe. `GET /api/v1/categories` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`.

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
