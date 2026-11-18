# vortex-api Specification

## Purpose

Define el esqueleto HTTP del API Gateway de Vortex Core: router con estado de aplicación clonable, traducción de errores de dominio y de Gateway a RFC 9457 (`application/problem+json`), middlewares transversales (límite de cuerpo, timeout, compresión, `Cache-Control: no-store` en `/api/*`, span de tracing), `GET /health` con snapshot agregado, `GET /metrics` Prometheus, estáticos de la SPA (`index.html` con `{{VERSION}}`, `/static/<versión>/`, `sw.js`, *manifest*) y `/media/*` desde `<datos>/media`, `vortex serve` escuchando en HTTP según `server.bind`, respuesta 404 a rutas inexistentes y apagado ordenado con SIGTERM y Ctrl+C, de modo que el proceso arranque y termine limpio antes de TLS o rutas de negocio.

Ticket padre: T-00-10, T-00-11, T-00-12, T-00-13, T-00-14, T-00-19, T-00-20, T-00-21, T-02-25. HU: HU-00-05, HU-00-06, HU-00-09, HU-01-01, HU-01-05, HU-02-07, HU-02-08, HU-02-10, HU-09-02.

## Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST registrar las rutas de categorías (requirement «Endpoints /api/v1/categories»). El router MUST registrar las rutas de marcas (requirement «Endpoints /api/v1/brands»). El router MUST registrar las rutas de definiciones de atributo (requirement «Endpoints /api/v1/attribute-definitions»). El router MUST registrar las rutas de producto (requirement «Endpoints POST/GET/PATCH /api/v1/products»). El router MUST registrar `POST /api/v1/products/{productId}/stock-movements` (requirement «Endpoint POST /api/v1/products/{productId}/stock-movements»). MUST NOT registrar otras rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»). `AppState::new` / `with_health` MAY omitir el servicio de categorías, el de marcas, el de definiciones de atributo, el de producto y el de stock (los tests de `/health` y estáticos siguen construyéndose sin ellos).

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

### Requirement: GET /health expone el snapshot agregado

El router de `build_router` MUST registrar `GET /health`. La respuesta MUST tener `Content-Type: application/json` y un cuerpo alineado al esquema `Health` de `docs/openapi.yaml`: `status` (`ok` | `degraded` | `starting` | `down`), `version` (string), `uptimeSeconds` (entero ≥ 0) y `components` con `storage`, `meilisearch`, `llm`, `speech`, `textEmbeddings` y `sync`, cada uno con `status` de componente. HTTP MUST ser **200** cuando el agregado es `ok` o `degraded`, y **503** cuando es `starting` o `down`. El cuerpo MUST NOT ser `application/problem+json`. Este change MUST NOT registrar rutas de negocio bajo `/api/v1`. `GET /health` MUST NOT enviar `Cache-Control: no-store` por el middleware de `/api/*`.

#### Scenario: GET /health ok es 200 (happy)

- **WHEN** el `HealthReport` inyectado tiene agregado `ok` (p. ej. seis componentes `disabled` o todos `up`)
- **AND** se envía `GET /health` al router de `build_router`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `status` `ok`, `version` no vacía, `uptimeSeconds` ≥ 0
- **AND** `components` incluye los seis nombres requeridos

#### Scenario: GET /health degraded es 200 (happy)

- **WHEN** el `HealthReport` inyectado tiene agregado `degraded` (p. ej. `meilisearch` `down`, `storage` no `down`, nadie `starting`)
- **AND** se envía `GET /health`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `status` `degraded`
- **AND** `Content-Type` es `application/json`

#### Scenario: GET /health starting es 503 (unhappy)

- **WHEN** el `HealthReport` inyectado tiene agregado `starting`
- **AND** se envía `GET /health`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `status` `starting`
- **AND** `Content-Type` es `application/json`
- **AND** `Content-Type` no es `application/problem+json`
- **AND** el JSON no tiene `code` de problema

#### Scenario: GET /health down es 503 (unhappy)

- **WHEN** el `HealthReport` inyectado tiene agregado `down` (`storage` `down`)
- **AND** se envía `GET /health`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `status` `down`
- **AND** `Content-Type` es `application/json`
- **AND** el JSON no tiene `code` de problema

#### Scenario: GET /health no lleva no-store (happy)

- **WHEN** se envía `GET /health`
- **THEN** la respuesta no incluye `Cache-Control: no-store`

#### Scenario: POST /health no es Health 200 (unhappy)

- **WHEN** se envía `POST /health` al router de `build_router`
- **THEN** la respuesta no tiene estado 200
- **AND** el cuerpo no es un `Health` con `status` `ok` servido como éxito de este endpoint

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

### Requirement: Arranque ordenado de vortex serve (paso 1)

`vortex serve` MUST arrancar en este orden, sobre HTTP en `localhost` (sin TLS): (1) cargar y validar `Config`; (2) instalar el subscriber de logging; (3) abrir Storage embebido en `<paths.datos>/db` y aplicar las migraciones embebidas; (4) publicar salud; (5) escuchar en `server.bind`. MUST NOT arrancar sidecars ni el Index Sync Worker (T-00-27). MUST NOT inyectar Storage en `AppState` en este change.

Antes de abrir Storage, el registro de salud MUST publicar `storage` en `starting` (el resto de componentes MAY permanecer `disabled`). Tras migrar con éxito, MUST publicar `storage` en `up`. El listener TCP MUST NOT aceptarse hasta que Storage y migraciones terminen OK. El primer `GET /health` observable contra ese bind MUST tener `status` `ok`, HTTP 200, `components.storage.status` `up`, y los demás componentes en `disabled` (agregado `ok`).

Si falla la configuración, la instalación de logs, la apertura de Storage o las migraciones, el proceso MUST terminar con código distinto de 0, MUST escribir el error en stderr (y en logs si el subscriber ya está instalado), y MUST NOT quedar escuchando HTTP.

Cada fase (config, logging, storage, health, api) MUST registrar su duración en un span o evento `tracing` cuyo nombre empiece por `boot.` (`boot.config`, `boot.logging`, `boot.storage`, `boot.health`, `boot.api`). El JSON bajo `<datos>/logs/` MUST incluir esos nombres cuando el arranque llega a instalar logging.

#### Scenario: vortex serve espera health ok con storage up (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` con `paths.datos` en un directorio temporal escribible y `server.bind` libre en `127.0.0.1`
- **THEN** el proceso queda escuchando en esa dirección
- **AND** `GET /health` contra ese bind recibe HTTP 200
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `status` `ok`
- **AND** `components.storage.status` es `up`
- **AND** el JSON no tiene `code` de `problem+json`

#### Scenario: las fases de boot registran duración (happy)

- **WHEN** el mismo arranque exitoso del escenario anterior
- **THEN** existe al menos un archivo bajo `<paths.datos>/logs/` cuyo nombre empieza por `vortex.log`
- **AND** el JSON incluye spans o eventos `boot.config`, `boot.logging`, `boot.storage`, `boot.health` y `boot.api`
- **AND** cada uno registra una duración (campo de elapsed / `busy` / equivalente de `tracing`)

#### Scenario: base inaccesible no expone la API (unhappy)

- **WHEN** `paths.datos` es un directorio escribible (los logs pueden crearse)
- **AND** `<paths.datos>/db` es un archivo regular (no un directorio de RocksDB)
- **AND** se ejecuta `vortex serve` con ese config y un `server.bind` libre
- **THEN** el proceso termina con código distinto de 0
- **AND** el proceso no queda escuchando HTTP en `server.bind`
- **AND** stderr o el log indican un fallo de Storage o de apertura de la base

#### Scenario: logs no escribibles no expone la API (unhappy)

- **WHEN** `paths.datos` apunta a un archivo regular
- **AND** se ejecuta `vortex serve` con ese config y un `server.bind` libre
- **THEN** el proceso termina con código distinto de 0
- **AND** el proceso no queda escuchando HTTP en `server.bind`
- **AND** stderr nombra la clave `paths.datos` o `logs`

### Requirement: Apagado ordenado con SIGTERM y Ctrl+C

Con el servidor HTTP en ejecución, una señal `SIGTERM` o `SIGINT` (`Ctrl+C`) MUST dejar de aceptar conexiones nuevas, MUST soltar el puerto de `server.bind` y MUST terminar el proceso con código **0**. MUST NOT exigir `SIGKILL`. En este change MUST NOT apagar sidecars (no hay supervisor).

#### Scenario: SIGTERM termina limpio (happy)

- **WHEN** `vortex serve` está escuchando en un bind de prueba
- **AND** el proceso recibe `SIGTERM`
- **THEN** el proceso termina con código 0
- **AND** el puerto de `server.bind` queda libre para un nuevo bind

#### Scenario: SIGINT termina limpio (happy)

- **WHEN** `vortex serve` está escuchando en un bind de prueba
- **AND** el proceso recibe `SIGINT`
- **THEN** el proceso termina con código 0
- **AND** el puerto de `server.bind` queda libre

### Requirement: ApiError traduce DomainError a problem+json

El crate `api` MUST exponer `ApiError` que traduce `DomainError` a una respuesta RFC 9457. El `Content-Type` MUST ser `application/problem+json`. El cuerpo MUST incluir `type`, `title`, `status` y `code` (esquema `Problem` de `docs/openapi.yaml`). `type` MUST ser `https://vectra.local/problems/{kebab}` donde `{kebab}` es el `code` en snake_case con `_` reemplazado por `-`. `title` MUST estar en español según la tabla de este requirement. `status` y `code` MUST seguir la tabla de [5.4](../../../docs/5-contract-api.md#54-errores):

| Variante de dominio | HTTP | `code` |
|---|---|---|
| `InvalidInput` | `400` | `invalid_input`, `self_relation`, `category_cycle`, `negative_stock` |
| `NotFound` | `404` | `not_found` |
| `Conflict` | `409` | `conflict`, `version_conflict`, `duplicate_sku`, `duplicate_slug`, `duplicate_relation`, `attribute_in_use` |
| `Unavailable` | `503` | `unavailable`, `ai_unavailable`, `speech_unavailable` |
| `Timeout` | `504` | `timeout` |

Títulos (`title`) MUST ser:

| `code` | `title` |
|---|---|
| `invalid_input` | La solicitud no es válida |
| `self_relation` | Un producto no puede relacionarse consigo mismo |
| `category_cycle` | La categoría formaría un ciclo |
| `negative_stock` | El stock no puede ser negativo |
| `not_found` | Recurso no encontrado |
| `conflict` | Conflicto con el estado actual |
| `version_conflict` | El producto fue modificado por otra terminal |
| `duplicate_sku` | El SKU ya existe |
| `duplicate_slug` | El slug ya existe |
| `duplicate_relation` | La relación ya existe |
| `attribute_in_use` | El atributo está en uso |
| `unavailable` | El servicio no está disponible |
| `ai_unavailable` | El modo IA no está disponible |
| `speech_unavailable` | El reconocimiento de voz no está disponible |
| `timeout` | La operación superó el tiempo de espera |
| `internal` | Error interno |

`detail` MUST estar en español y MUST NOT incluir trazas, nombres de tipos Rust ni el `Display` técnico de `DomainError`. Para `InvalidInput`, el cuerpo MUST incluir `errors` como lista de `{ field, message }` (la lista MAY estar vacía). Para el resto de variantes el cuerpo MUST NOT incluir `errors`. `Unavailable` MUST enviar la cabecera `Retry-After` con un entero de segundos. `instance`, cuando la traducción ocurre en una petición HTTP, MUST ser la ruta de esa petición. El crate `domain` MUST NOT conocer HTTP ni `problem+json`.

#### Scenario: NotFound a 404 problem+json (happy)

- **WHEN** el Gateway traduce un `DomainError::NotFound`
- **THEN** la respuesta tiene estado 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`, `status` 404, `title` "Recurso no encontrado" y `type` `https://vectra.local/problems/not-found`
- **AND** el JSON no incluye `errors`

#### Scenario: InvalidInput a 400 con errors[] (happy)

- **WHEN** el Gateway traduce un `DomainError::InvalidInput` con `code` `negative_stock` y un `FieldError` `field` `quantity` / `message` no vacío
- **THEN** la respuesta tiene estado 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `negative_stock`, `title` "El stock no puede ser negativo"
- **AND** `errors` contiene exactamente ese par `field` / `message`

#### Scenario: Conflict version_conflict a 409 (happy)

- **WHEN** el Gateway traduce un `DomainError::Conflict` con `code` `version_conflict`
- **THEN** la respuesta tiene estado 409
- **AND** el JSON tiene `code` `version_conflict` y `title` "El producto fue modificado por otra terminal"
- **AND** el JSON no incluye `errors`

#### Scenario: Unavailable incluye Retry-After (happy)

- **WHEN** el Gateway traduce un `DomainError::Unavailable` con `code` `ai_unavailable`
- **THEN** la respuesta tiene estado 503
- **AND** el JSON tiene `code` `ai_unavailable`
- **AND** la cabecera `Retry-After` está presente con un entero de segundos

#### Scenario: Timeout a 504 (happy)

- **WHEN** el Gateway traduce un `DomainError::Timeout`
- **THEN** la respuesta tiene estado 504
- **AND** el JSON tiene `code` `timeout` y `Content-Type` `application/problem+json`

#### Scenario: Cada code de dominio se traduce (happy)

- **WHEN** el Gateway traduce un `DomainError` por cada `code` de dominio de la tabla de este requirement
- **THEN** cada respuesta tiene el `status` de su fila, `Content-Type` `application/problem+json` y el `code` pedido
- **AND** `title` coincide con la tabla de títulos
- **AND** `type` usa el kebab del `code`

#### Scenario: Conflict no expone errors (unhappy)

- **WHEN** el Gateway traduce un `DomainError::Conflict` (cualquier `code` de esa fila)
- **THEN** el JSON no incluye la clave `errors`

### Requirement: Rechazos JSON a 400 invalid_input

Cuando un extractor JSON del Gateway rechaza el cuerpo (JSON malformado, tipos incompatibles, cuerpo vacío o `Content-Type` ausente/incorrecto), la respuesta MUST ser `400` con `code` `invalid_input`, `Content-Type` `application/problem+json` y `errors` con al menos un `{ field, message }`. Si el rechazo identifica una ruta de campo, `field` MUST ser esa ruta; si no, `field` MUST ser `body`. MUST NOT responder `415` en este change. MUST NOT filtrar el cuerpo crudo de la petición en `detail`.

#### Scenario: JSON malformado (unhappy)

- **WHEN** se envía un cuerpo que no es JSON válido a un endpoint que espera JSON
- **THEN** la respuesta tiene estado 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` tiene al menos un elemento con `field` no vacío y `message` no vacío

#### Scenario: JSON con campo inválido (unhappy)

- **WHEN** se envía JSON sintácticamente válido cuyo tipo no coincide con el esquema esperado (p. ej. un número donde va un objeto)
- **THEN** la respuesta tiene estado 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors[0].field` identifica el cuerpo o la ruta del campo

#### Scenario: Content-Type ausente en JSON (unhappy)

- **WHEN** se envía un cuerpo a un endpoint que espera JSON sin `Content-Type: application/json`
- **THEN** la respuesta tiene estado 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** no tiene estado 415

### Requirement: 404 y 500 genéricos sin detalles internos

Una petición a una ruta que el router no registra MUST responder `404` con `code` `not_found`, `Content-Type` `application/problem+json`, `title` "Recurso no encontrado" e `instance` igual a la ruta pedida. Un error no previsto MUST responder `500` con `code` `internal`, el `title` de la tabla (`Error interno`) y un `detail` genérico en español. El 500 MUST NOT incluir el mensaje de la causa, trazas ni nombres de tipos. El 404 y el 500 MUST NOT incluir `errors`.

#### Scenario: ruta inexistente es problem+json not_found (happy)

- **WHEN** se envía `GET /no-such-route` al router de `build_router`
- **THEN** la respuesta tiene estado 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found` e `instance` `/no-such-route`

#### Scenario: 500 internal no filtra la causa (unhappy)

- **WHEN** el Gateway traduce un error no previsto cuya causa contiene un secreto o un mensaje técnico (p. ej. `secret=hunter2`)
- **THEN** la respuesta tiene estado 500
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `internal`
- **AND** el cuerpo y las cabeceras no contienen el texto de esa causa

### Requirement: Límite de cuerpo por defecto 256 KiB

El Gateway MUST rechazar un cuerpo de petición cuyo tamaño supere **256 KiB** (262144 bytes). La respuesta MUST tener estado HTTP `413`, `Content-Type: application/problem+json`, `code` `payload_too_large`, `title` "El cuerpo es demasiado grande" y `type` `https://vectra.local/problems/payload-too-large`. `instance`, cuando la petición tiene path, MUST ser esa ruta. El JSON MUST NOT incluir `errors`. Un cuerpo de **exactamente** 256 KiB MUST NOT producir `413` por este límite. Este change MUST NOT aplicar límites distintos por ruta (search, imágenes, speech, etc.): esos valores viven en [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway) y los fija cada ticket de endpoint.

#### Scenario: cuerpo sobre el límite es 413 payload_too_large (unhappy)

- **WHEN** se envía `POST` con `Content-Length` de 262145 bytes (256 KiB + 1) a una ruta del router de `build_router`
- **THEN** la respuesta tiene estado HTTP 413
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `payload_too_large`, `status` 413 y `title` "El cuerpo es demasiado grande"
- **AND** el JSON no incluye `errors`

#### Scenario: cuerpo en el límite no es 413 (happy)

- **WHEN** se envía `POST` con `Content-Length` de 262144 bytes (256 KiB) a una ruta inexistente del router de `build_router`
- **THEN** la respuesta no tiene estado 413
- **AND** el JSON no tiene `code` `payload_too_large`

### Requirement: Timeout por defecto 5 s

Si el Gateway no termina una petición en **5 segundos**, MUST responder `504` con `Content-Type: application/problem+json`, `code` `timeout`, `title` "La operación superó el tiempo de espera" y `type` `https://vectra.local/problems/timeout`. `instance` MUST ser la ruta pedida. El JSON MUST NOT incluir `errors`. Una petición que termina antes de 5 s MUST NOT recibir `504` por este timeout. Este change MUST NOT acortar ni alargar el timeout por ruta (search 2 s, speech 10 s, SSE 90 s, etc.).

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

### Requirement: Compresión gzip/brotli excepto SSE

Cuando el cliente envía `Accept-Encoding` con `gzip` o `br`, el Gateway MUST poder comprimir respuestas JSON con ese algoritmo. Una respuesta con `Content-Type` `text/event-stream` MUST NOT llevar `Content-Encoding` `gzip` ni `br`, aunque el cliente pida compresión. Este change MUST NOT comprimir por defecto las respuestas SSE de tickets futuros (`POST /api/v1/ai/ask`).

#### Scenario: JSON acepta gzip (happy)

- **WHEN** se pide una respuesta JSON (sonda de test con cuerpo lo bastante grande para comprimir) con cabecera `Accept-Encoding: gzip`
- **THEN** la respuesta incluye `Content-Encoding: gzip`
- **AND** el `Content-Type` sigue siendo JSON (`application/json` o `application/problem+json`)

#### Scenario: JSON acepta brotli (happy)

- **WHEN** se pide una respuesta JSON comprimible con cabecera `Accept-Encoding: br`
- **THEN** la respuesta incluye `Content-Encoding: br`

#### Scenario: SSE no se comprime (unhappy)

- **WHEN** se pide una respuesta `text/event-stream` (sonda de test) con cabecera `Accept-Encoding: gzip`
- **THEN** la respuesta no incluye `Content-Encoding: gzip` ni `Content-Encoding: br`
- **AND** `Content-Type` indica `text/event-stream`

### Requirement: Cache-Control no-store en /api/*

Una respuesta cuya ruta empieza por `/api/` o es exactamente `/api` MUST incluir la cabecera `Cache-Control: no-store`. Una respuesta cuya ruta no está bajo `/api` MUST NOT incluir `Cache-Control: no-store` por este middleware (los estáticos de T-00-19 definen su propia política). El 404 de una ruta `/api/...` inexistente MUST llevar `no-store`.

#### Scenario: /api/* envía no-store (happy)

- **WHEN** se envía `GET /api/v1/no-such-route` al router de `build_router`
- **THEN** la respuesta incluye `Cache-Control: no-store`
- **AND** el estado es 404 con `code` `not_found`

#### Scenario: path fuera de /api no lleva no-store (unhappy)

- **WHEN** se envía `GET /no-such-route` al router de `build_router`
- **THEN** la respuesta no incluye `Cache-Control: no-store`
- **AND** el estado es 404 con `code` `not_found`

### Requirement: Span de tracing por petición HTTP

Cada petición atendida por `build_router` MUST abrir un span de `tracing` que registre al menos el método HTTP, la ruta (path o ruta coincidente), el estado de la respuesta y la duración. El crate `api` MUST NOT instalar el subscriber global; `vortex serve` lo instala (capability `vortex-logging`). Si hay un subscriber activo (p. ej. en un test del crate `api`), el span MUST ser observable.

#### Scenario: span incluye método, ruta, estado y duración (happy)

- **WHEN** se envía `GET /no-such-route` con un subscriber de tracing de test instalado
- **THEN** existe un span de la petición cuyo método es `GET`
- **AND** el span registra la ruta `/no-such-route` (o equivalente)
- **AND** el span registra el estado 404
- **AND** el span registra una duración

### Requirement: Tests de contrato del servidor HTTP

Antes de archive, la suite MUST incluir estos tests (nombres estables; crate / tipo). Happy y unhappy path son obligatorios.

| Test | Crate | Tipo |
|---|---|---|
| `build_router_unknown_path_returns_404` | `api` | integración del crate |
| `domain_error_variants_map_to_problem_json` | `api` | crate (tabla de `DomainError`) |
| `json_rejection_returns_400_invalid_input` | `api` | integración del crate |
| `internal_error_does_not_leak_cause` | `api` | crate |
| `vortex_serve_starts_and_returns_404` | `vortex` | binario |
| `vortex_serve_exits_clean_on_sigterm` | `vortex` | binario (unix) |
| `vortex_serve_exits_clean_on_sigint` | `vortex` | binario (unix) |
| `vortex_serve_without_config_fails` | `vortex` | binario |
| `vortex_serve_missing_config_file_fails` | `vortex` | binario |
| `vortex_serve_invalid_config_fails` | `vortex` | binario |
| `vortex_serve_bind_in_use_fails` | `vortex` | binario |
| `oversized_body_returns_413_payload_too_large` | `api` | integración del crate |
| `body_at_limit_is_not_413` | `api` | integración del crate |
| `slow_handler_returns_504_timeout` | `api` | integración del crate |
| `fast_request_is_not_504` | `api` | integración del crate |
| `json_response_is_gzip_compressed` | `api` | integración del crate |
| `json_response_is_brotli_compressed` | `api` | integración del crate |
| `sse_response_is_not_compressed` | `api` | integración del crate |
| `api_path_sends_cache_control_no_store` | `api` | integración del crate |
| `non_api_path_omits_cache_control_no_store` | `api` | integración del crate |
| `http_request_span_includes_method_route_status_duration` | `api` | crate (subscriber de test) |
| `get_health_ok_returns_200` | `api` | integración del crate |
| `get_health_degraded_returns_200` | `api` | integración del crate |
| `get_health_starting_returns_503` | `api` | integración del crate |
| `get_health_down_returns_503` | `api` | integración del crate |
| `get_health_is_not_problem_json` | `api` | integración del crate |
| `post_health_is_not_200_health` | `api` | integración del crate |
| `vortex_serve_get_health_returns_json` | `vortex` | binario |
| `get_metrics_returns_prometheus_text` | `api` | integración del crate |
| `get_metrics_includes_prior_request_histogram` | `api` | integración del crate |
| `get_metrics_is_not_problem_json` | `api` | integración del crate |
| `get_metrics_omits_cache_control_no_store` | `api` | integración del crate |
| `post_metrics_is_not_prometheus_200` | `api` | integración del crate |
| `vortex_serve_get_metrics_returns_prometheus` | `vortex` | binario |
| `index_html_replaces_version_and_sends_no_cache` | `api` | integración del crate |
| `index_html_missing_returns_404` | `api` | integración del crate |
| `static_asset_sends_immutable_cache` | `api` | integración del crate |
| `static_asset_wrong_version_returns_404` | `api` | integración del crate |
| `static_path_traversal_returns_404` | `api` | integración del crate |
| `sw_js_sends_no_cache` | `api` | integración del crate |
| `manifest_sends_no_cache` | `api` | integración del crate |
| `manifest_replaces_version` | `api` | integración del crate |
| `media_file_sends_long_cache` | `api` | integración del crate |
| `media_missing_returns_404` | `api` | integración del crate |
| `media_path_traversal_returns_404` | `api` | integración del crate |
| `post_index_is_not_200_html` | `api` | integración del crate |
| `vortex_serve_serves_index_from_paths_web` | `vortex` | binario |
| `vortex_serve_health_ok_with_storage_up` | `vortex` | binario |
| `vortex_serve_boot_logs_phase_durations` | `vortex` | binario |
| `vortex_serve_inaccessible_storage_exits` | `vortex` | binario |
| `vortex_serve_unwritable_datos_exits` | `vortex` | binario |

`cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST ejecutarlos. Los tests de `problem+json` MUST asertar estado HTTP, `Content-Type: application/problem+json` y `code`. Los de middleware MUST cubrir cuerpo excedido, timeout y cabeceras (`Cache-Control`, `Content-Encoding`). Los de `/health` MUST asertar 200/503, `Content-Type: application/json` y `status` agregado. Los de `/metrics` MUST asertar 200, `text/plain` y el histograma de una petición previa. Los de estáticos MUST asertar `Cache-Control` (`no-cache` vs `max-age=31536000` + `immutable`), la sustitución de `{{VERSION}}` en `index.html` y en `manifest.webmanifest`, y el 404 `not_found` de archivo ausente, versión distinta y *path traversal*. Los de arranque ordenado MUST asertar `/health` `ok` con `storage` `up`, spans `boot.*` en el log, y que una base inaccesible o `paths.datos` no escribible terminan con error sin dejar el puerto escuchando. Los stubs de los demás subcomandos MUST seguir cubiertos.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core`
- **THEN** corren los tests de la tabla
- **AND** los de 404 / `problem+json` / apagado ordenado pasan
- **AND** los de JSON inválido, 500 sin leak, config ausente/inválida y bind ocupado pasan
- **AND** los de cuerpo excedido, timeout, cabeceras y span HTTP pasan
- **AND** los de `GET /health` (200 `ok`/`degraded`, 503 `starting`/`down`, no `problem+json`) pasan
- **AND** los de `GET /metrics` (200 `text/plain`, histograma de petición previa, no `problem+json`, `POST` no es scrape) pasan
- **AND** los de estáticos (`no-cache` / inmutable, `{{VERSION}}` en `index.html` y en el *manifest*, 404 de ausente / versión / *traversal*) pasan
- **AND** los de arranque ordenado (`health` `ok` + `storage` `up`, duraciones `boot.*`, Storage inaccesible, `datos` no escribible) pasan

### Requirement: GET /metrics expone texto Prometheus

El router de `build_router` MUST registrar `GET /metrics`. La respuesta MUST tener estado HTTP **200** y `Content-Type` que empiece por `text/plain` (MAY incluir parámetros `version` y `charset`). El cuerpo MUST ser texto de exposición Prometheus válido: al menos una línea `# TYPE` o una serie con nombre Prometheus, sin JSON de `problem+json`. Tras una petición HTTP previa distinta del scrape (p. ej. `GET /health`), el cuerpo MUST incluir el histograma `http_request_duration_seconds` de esa petición (capability `prometheus-metrics`). `GET /metrics` MUST NOT enviar `Cache-Control: no-store` por el middleware de `/api/*`. El cuerpo MUST NOT ser `application/problem+json`. Este change MUST NOT registrar rutas de negocio bajo `/api/v1`. El path y el content type MUST alinearse con `GET /metrics` en `docs/openapi.yaml` (sin modificar ese archivo).

#### Scenario: GET /metrics es 200 text/plain (happy)

- **WHEN** se envía `GET /metrics` al router de `build_router`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` empieza por `text/plain`
- **AND** el cuerpo no está vacío
- **AND** el cuerpo no es JSON con `code` de problema

#### Scenario: GET /metrics incluye el histograma de una petición previa (happy)

- **WHEN** se envía `GET /health` al router
- **AND** después se envía `GET /metrics`
- **THEN** la respuesta de `/metrics` tiene estado 200
- **AND** el cuerpo incluye `http_request_duration_seconds`
- **AND** incluye `route="/health"` y `method="GET"`

#### Scenario: GET /metrics no lleva no-store (happy)

- **WHEN** se envía `GET /metrics`
- **THEN** la respuesta no incluye `Cache-Control: no-store`

#### Scenario: GET /metrics no es problem+json (unhappy)

- **WHEN** se envía `GET /metrics`
- **THEN** `Content-Type` no es `application/problem+json`
- **AND** el cuerpo no tiene `code` de problema

#### Scenario: POST /metrics no es scrape 200 (unhappy)

- **WHEN** se envía `POST /metrics` al router de `build_router`
- **THEN** la respuesta no tiene estado 200 con cuerpo de exposición Prometheus
- **AND** el cuerpo no se presenta como scrape exitoso de este endpoint

### Requirement: Estáticos de la SPA y /media

`AppState` MUST incluir una raíz `web` (directorio de `paths.web`), una raíz `media` (`<datos>/media`) y una cadena `version` no vacía. El constructor por defecto (`AppState::new` / `with_health`) MAY usar raíces inexistentes y `version` igual a `CARGO_PKG_VERSION` del crate `api`. `vortex serve` MUST inyectar `paths.web`, `paths.datos` unido a `media`, y la misma `version` que publica `/health`.

El router de `build_router` MUST registrar estos GET (y MUST NOT exigir que los archivos existan al construir el router):

| Ruta | Archivo en disco | `Cache-Control` | Cuerpo |
|---|---|---|---|
| `/` y `/index.html` | `{web}/index.html` | `no-cache` | HTML con cada `{{VERSION}}` reemplazado por `version` |
| `/sw.js` | `{web}/sw.js` | `no-cache` | texto con cada `{{VERSION}}` reemplazado por `version` |
| `/manifest.webmanifest` | `{web}/manifest.webmanifest` | `no-cache` | texto con cada `{{VERSION}}` reemplazado por `version` |
| `/static/{version}/{*path}` | `{web}/{path}` | `max-age=31536000, immutable` | bytes del archivo, solo si `{version}` es exactamente `AppState.version` y `{path}` es un archivo regular bajo `{web}` |
| `/media/{*path}` | `{media}/{path}` | `max-age=31536000, immutable` | bytes del archivo, solo si `{path}` es un archivo regular bajo `{media}` |

`Content-Type` de `/` e `/index.html` MUST ser `text/html` (MAY incluir `charset=utf-8`). Las demás rutas MUST enviar un `Content-Type` coherente con el recurso (p. ej. `text/javascript` o `application/javascript` para `.js`; `application/manifest+json` para el *manifest*; tipo de imagen o `application/octet-stream` para `/media`). Estas respuestas MUST NOT enviar `Cache-Control: no-store` (el middleware de `/api/*` no aplica). `{{VERSION}}` MUST sustituirse de forma literal, todas las ocurrencias, en `index.html`, `manifest.webmanifest` **y** `sw.js`; MUST NOT interpretarse como plantilla fuera de esos tres archivos.

Una petición GET cuyo archivo no existe, cuyo `{path}` no es un archivo regular (directorio, raíz vacía), cuyo `{version}` de `/static/` no coincide, o que intente salir de `{web}` / `{media}` (`..`, `%2e%2e`, barra inicial, enlace que resuelva fuera de la raíz) MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found`. MUST NOT devolver el contenido de un archivo fuera de esas raíces. MUST NOT listar directorios. Un método distinto de GET sobre estas rutas MUST NOT ser `200` con el cuerpo del estático. Los tests de este requirement MUST usar un árbol temporal de fixtures (MUST NOT depender del árbol `vectra-web/` real).

#### Scenario: index.html sustituye VERSION y envía no-cache (happy)

- **WHEN** `AppState` apunta a un `{web}` que contiene `index.html` con el texto `{{VERSION}}`
- **AND** se envía `GET /` al router de `build_router`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` empieza por `text/html`
- **AND** el cuerpo contiene el valor de `version` y no contiene `{{VERSION}}`
- **AND** `Cache-Control` es `no-cache`
- **AND** `Cache-Control` no es `no-store`

#### Scenario: GET /index.html es el mismo documento (happy)

- **WHEN** el mismo `{web}` del escenario anterior
- **AND** se envía `GET /index.html`
- **THEN** la respuesta tiene estado 200
- **AND** el cuerpo no contiene `{{VERSION}}`
- **AND** `Cache-Control` es `no-cache`

#### Scenario: activo versionado envía caché inmutable (happy)

- **WHEN** `{web}` contiene un archivo regular `app.js` (o equivalente) con cuerpo no vacío
- **AND** se envía `GET /static/<version>/app.js` con `<version>` igual a `AppState.version`
- **THEN** la respuesta tiene estado 200
- **AND** el cuerpo coincide con el archivo
- **AND** `Cache-Control` incluye `max-age=31536000` e `immutable`

#### Scenario: sw.js envía no-cache (happy)

- **WHEN** `{web}` contiene `sw.js`
- **AND** se envía `GET /sw.js`
- **THEN** la respuesta tiene estado 200
- **AND** `Cache-Control` es `no-cache`
- **AND** `Cache-Control` no es `no-store`

#### Scenario: sw.js sustituye VERSION (happy)

- **WHEN** `{web}` contiene `sw.js` con el texto `{{VERSION}}` (p. ej. en el nombre de caché o en una URL de precache)
- **AND** se envía `GET /sw.js`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `text/javascript` o `application/javascript`
- **AND** el cuerpo contiene el valor de `version` y no contiene `{{VERSION}}`
- **AND** `Cache-Control` es `no-cache`

#### Scenario: manifest envía no-cache (happy)

- **WHEN** `{web}` contiene `manifest.webmanifest`
- **AND** se envía `GET /manifest.webmanifest`
- **THEN** la respuesta tiene estado 200
- **AND** `Cache-Control` es `no-cache`
- **AND** `Cache-Control` no es `no-store`

#### Scenario: manifest sustituye VERSION (happy)

- **WHEN** `{web}` contiene `manifest.webmanifest` con el texto `{{VERSION}}` (p. ej. en un `src` de ícono)
- **AND** se envía `GET /manifest.webmanifest`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `Content-Type` es `application/manifest+json`
- **AND** el cuerpo contiene el valor de `version` y no contiene `{{VERSION}}`
- **AND** `Cache-Control` es `no-cache`

#### Scenario: /media envía caché larga (happy)

- **WHEN** `{media}` contiene un archivo regular cuyo nombre es un digest hex (p. ej. `3f2a9c.jpg` bajo un subdirectorio)
- **AND** se envía `GET /media/<esa-ruta-relativa>`
- **THEN** la respuesta tiene estado 200
- **AND** el cuerpo coincide con el archivo
- **AND** `Cache-Control` incluye `max-age=31536000` e `immutable`

#### Scenario: index.html ausente es 404 problem+json (unhappy)

- **WHEN** `{web}` no contiene `index.html` (raíz inexistente o directorio vacío)
- **AND** se envía `GET /`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`

#### Scenario: activo con versión distinta es 404 (unhappy)

- **WHEN** `{web}` contiene `app.js`
- **AND** se envía `GET /static/not-the-version/app.js` con un segmento de versión distinto de `AppState.version`
- **THEN** la respuesta tiene estado 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`
- **AND** el cuerpo no es el contenido de `app.js`

#### Scenario: path traversal en /static es 404 (unhappy)

- **WHEN** `{web}` contiene `app.js` y el directorio padre de `{web}` contiene un archivo `secret.txt` fuera de `{web}`
- **AND** se envía `GET /static/<version>/../secret.txt` o un path equivalente (`..`, `%2e%2e`)
- **THEN** la respuesta tiene estado 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`
- **AND** el cuerpo no contiene el contenido de `secret.txt`

#### Scenario: path traversal en /media es 404 (unhappy)

- **WHEN** `{media}` está bajo `{datos}` y existe un archivo fuera de `{media}` accesible vía `..`
- **AND** se envía `GET /media/../<archivo-fuera>`
- **THEN** la respuesta tiene estado 404
- **AND** el JSON tiene `code` `not_found`
- **AND** el cuerpo no es ese archivo

#### Scenario: /media inexistente es 404 (unhappy)

- **WHEN** se envía `GET /media/no-such-object.bin` y ese path no existe bajo `{media}`
- **THEN** la respuesta tiene estado 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`

#### Scenario: POST / no es HTML 200 (unhappy)

- **WHEN** `{web}` contiene `index.html`
- **AND** se envía `POST /` al router de `build_router`
- **THEN** la respuesta no tiene estado 200 con `Content-Type` `text/html` y el documento sustituido
- **AND** el cuerpo no se presenta como éxito de este estático

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

### Requirement: Endpoints /api/v1/brands

El router de `build_router` MUST registrar `GET` y `POST /api/v1/brands` y `PATCH` y `DELETE /api/v1/brands/{brandId}` alineados con `docs/openapi.yaml`. El Gateway MUST delegar en el servicio de marcas inyectado en `AppState` (capability `catalog-marca`) y MUST NOT implementar persistencia ni occupancy.

`GET` MUST devolver `{ "items": [...] }` ordenado por `name` (orden lexicográfico de `nombre`). Sin query o con `includeInactive=false`, MUST omitir marcas con `active = false`. Con `includeInactive=true`, MUST incluir inactivas. El query `q` (string, `maxLength` 100) MUST filtrar por prefijo **case-insensitive** de `name`. `q` vacío o ausente MUST no filtrar por nombre. `q` de más de 100 caracteres MUST ser `400` `invalid_input` con `errors[].field` `q`. MUST NOT paginar.

`POST` MUST crear con cuerpo `BrandCreate` (`name` obligatorio; `slug` opcional) y responder `201` con `Brand`. `PATCH` MUST aplicar el cuerpo `BrandUpdate` (`minProperties: 1`): `name` renombra; `slug` actualiza el slug; `active: true` reactiva; `active: false` da de baja lógica. `DELETE` MUST dar de baja lógica y responder `204` sin cuerpo. `PATCH { "active": false }` MUST ser la misma baja que `DELETE`. La baja MUST permitirse aunque la marca tenga productos asociados: MUST NOT responder `409` por ocupación (HU-02-02).

`{brandId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `brandId`. Id inexistente MUST ser `404` `not_found`. Slug duplicado MUST ser `409` `duplicate_slug`. Cuerpo JSON vacío en `PATCH` o sin `name` en `POST` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de marcas, estas rutas MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`MarcaRepo`). MUST NOT registrar `GET /api/v1/brands/{brandId}`. MUST NOT reindexar productos (T-03-08). `docs/openapi.yaml` MUST declarar `503` en las operaciones de `/api/v1/brands` y `/api/v1/brands/{brandId}`. MUST NOT declarar `409` en `DELETE /api/v1/brands/{brandId}`.

#### Scenario: POST crea Fixser (happy, HU-02-02)

- **WHEN** se envía `POST /api/v1/brands` con `{ "name": "Fixser" }`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `name` `"Fixser"` y `active` verdadero
- **AND** el JSON no tiene claves `nombre` ni `activo`
- **AND** `assert_conforms` de `POST /api/v1/brands` 201 `application/json` no falla

#### Scenario: GET lista sin inactivas (happy)

- **WHEN** existen una marca activa y una inactiva
- **AND** se envía `GET /api/v1/brands`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye la activa y no incluye la inactiva
- **AND** `assert_conforms` de `GET /api/v1/brands` 200 `application/json` no falla

#### Scenario: GET includeInactive incluye inactivas (happy)

- **WHEN** existe al menos una marca inactiva
- **AND** se envía `GET /api/v1/brands?includeInactive=true`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye la inactiva

#### Scenario: GET q filtra por prefijo (happy)

- **WHEN** existen marcas `"Fixser"` y `"Tramontina"`
- **AND** se envía `GET /api/v1/brands?q=fix`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items` incluye `"Fixser"` y no incluye `"Tramontina"`

#### Scenario: GET ordena por name (happy)

- **WHEN** existen marcas `"Tramontina"` y `"Fixser"`
- **AND** se envía `GET /api/v1/brands`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `items[0].name` es `"Fixser"`
- **AND** `items[1].name` es `"Tramontina"`

#### Scenario: PATCH renombra (happy)

- **WHEN** existe "Fixser" y se envía `PATCH /api/v1/brands/{id}` con `{ "name": "Fixser Pro" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `name` `"Fixser Pro"`
- **AND** `assert_conforms` de `PATCH /api/v1/brands/{brandId}` 200 `application/json` no falla

#### Scenario: PATCH actualiza slug (happy)

- **WHEN** existe "Fixser" y se envía `PATCH` con `{ "slug": "fixser-pro" }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** el JSON tiene `slug` `"fixser-pro"`

#### Scenario: PATCH reactiva (happy)

- **WHEN** "Fixser" está inactiva
- **AND** se envía `PATCH` con `{ "active": true }`
- **THEN** la respuesta tiene estado HTTP 200
- **AND** `active` es `true`

#### Scenario: DELETE da de baja (happy, HU-02-02)

- **WHEN** existe "Fixser"
- **AND** se envía `DELETE /api/v1/brands/{id}`
- **THEN** la respuesta tiene estado HTTP 204
- **AND** el cuerpo está vacío

#### Scenario: DELETE con productos asociados (happy, HU-02-02)

- **WHEN** el fake o el servicio no rechaza ocupación
- **AND** se envía `DELETE /api/v1/brands/{id}` de una marca existente
- **THEN** la respuesta tiene estado HTTP 204
- **AND** la respuesta no tiene estado 409

#### Scenario: POST slug duplicado (unhappy)

- **WHEN** ya existe una marca con el mismo slug
- **AND** se envía `POST` con ese `name`/`slug`
- **THEN** la respuesta tiene estado HTTP 409
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `duplicate_slug`
- **AND** `assert_conforms` de `POST /api/v1/brands` 409 `application/problem+json` no falla

#### Scenario: PATCH id desconocido (unhappy)

- **WHEN** se envía `PATCH /api/v1/brands/{ulid-inexistente}` con `{ "name": "X" }`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: DELETE id desconocido (unhappy)

- **WHEN** se envía `DELETE /api/v1/brands/{ulid-inexistente}`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`

#### Scenario: POST sin name (unhappy)

- **WHEN** se envía `POST /api/v1/brands` con `{ }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: PATCH cuerpo vacío (unhappy)

- **WHEN** se envía `PATCH` con `{ }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: brandId no es ULID (unhappy)

- **WHEN** se envía `DELETE /api/v1/brands/not-a-ulid`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `brandId`

#### Scenario: q demasiado largo (unhappy)

- **WHEN** se envía `GET /api/v1/brands?q=` seguido de más de 100 caracteres
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `q`

#### Scenario: sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de marcas
- **AND** se envía `GET /api/v1/brands`
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`

### Requirement: DTO inglés de marca según 5.3

El Gateway MUST traducir el contrato inglés al dominio español y de vuelta. Campos de respuesta `Brand`: `id`, `name`, `slug`, `active`. MUST NOT serializar `nombre`, `activo`, `creado_en` ni `actualizado_en`. `POST`/`PATCH` MUST leer `name` → `nombre`, `slug` → `slug`, `active` → `activo`. En `problem+json`, `errors[].field` MUST usar nombres de API (`name`, `slug`, `brandId`, `q`, `body`), nunca `nombre`.

#### Scenario: respuesta sin claves de dominio (happy)

- **WHEN** `POST` crea la marca `"Fixser"`
- **THEN** el JSON de 201 tiene `id`, `name`, `slug` y `active`
- **AND** no tiene `nombre` ni `activo`

#### Scenario: errors[].field en inglés (unhappy)

- **WHEN** el servicio rechaza un `nombre` inválido con `FieldError.field` `nombre`
- **AND** el Gateway traduce ese error a HTTP
- **THEN** `errors[].field` es `name`
- **AND** no aparece `nombre` en el cuerpo

### Requirement: Tests de integración de marcas contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-09. Listo cuando: cada respuesta JSON (éxito o error) se valida con `assert_conforms` (T-09-04) salvo `DELETE` 204 (sin cuerpo). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_brand_fixser_returns_201` | `api` | integración, happy, HU-02-02 |
| `get_brands_omits_inactive_by_default` | `api` | integración, happy |
| `get_brands_include_inactive_lists_inactive` | `api` | integración, happy |
| `get_brands_q_filters_prefix` | `api` | integración, happy |
| `get_brands_ordered_by_name` | `api` | integración, happy |
| `patch_brand_renames` | `api` | integración, happy |
| `patch_brand_updates_slug` | `api` | integración, happy |
| `patch_brand_reactivates` | `api` | integración, happy |
| `delete_brand_returns_204` | `api` | integración, happy, HU-02-02 |
| `delete_brand_with_associated_products_returns_204` | `api` | integración, happy, HU-02-02 (baja permitida) |
| `post_brand_duplicate_slug_returns_409` | `api` | integración, unhappy |
| `patch_brand_unknown_id_returns_404` | `api` | integración, unhappy |
| `delete_brand_unknown_id_returns_404` | `api` | integración, unhappy |
| `post_brand_missing_name_returns_400` | `api` | integración, unhappy |
| `patch_brand_empty_body_returns_400` | `api` | integración, unhappy |
| `brand_id_invalid_ulid_returns_400` | `api` | integración, unhappy |
| `get_brands_q_too_long_returns_400` | `api` | integración, unhappy |
| `get_brands_without_service_returns_503` | `api` | integración, unhappy |
| `brand_problem_field_is_name_not_nombre` | `api` | integración, unhappy, 5.3 |
| `vortex_serve_get_brands_returns_items` | `vortex` | binario, happy |

#### Scenario: la suite de marcas cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 200/201/400/404/409/503 (salvo 204) pasan `assert_conforms`

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

### Requirement: Endpoint POST /api/v1/products/{productId}/stock-movements

El router de `build_router` MUST registrar `POST /api/v1/products/{productId}/stock-movements` alineado con `docs/openapi.yaml` (`operationId` `registerStockMovement`). El Gateway MUST delegar en el servicio de stock inyectado en `AppState` (capability `catalog-stock`) y MUST NOT implementar aritmética de stock, outbox ni persistencia.

`POST` MUST aceptar cuerpo `StockMovementCreate` (`type` y `quantity` obligatorios; `reason` opcional) y responder **201** con `StockMovement`. `type` MUST ser solo `inbound` o `adjustment` (`inbound`→`ingreso`, `adjustment`→`ajuste`). `quantity` MUST ser string decimal del esquema `Decimal` (con signo, ≠ 0). `inbound` MUST exigir `quantity` > 0; `adjustment` MAY ser positivo o negativo. `reason` MUST mapearse a `motivo`; si `type` es `adjustment`, `reason` ausente, nulo o solo espacios MUST ser **400** `invalid_input` con `errors[].field` `reason`. La cabecera `X-Vectra-Terminal` (OpenAPI `Terminal`, `maxLength` 64) MUST copiarse a `NewMovimientoStock.terminal` si está presente y no vacía tras recortar espacios; si falta o queda vacía, `terminal` MUST ser ausente. Un valor de más de 64 caracteres MUST ser `400` `invalid_input` con `errors[].field` `X-Vectra-Terminal`. MUST NOT exigir `version`. MUST NOT incrementar `version` de la ficha. MUST NOT crear movimientos `initial` ni `sale`.

`{productId}` MUST ser un ULID de 26 caracteres (esquema `Ulid`). Un path que no parsea MUST ser `400` `invalid_input` con `errors[].field` `productId`. Id inexistente MUST ser `404` `not_found`. Un ajuste que dejaría stock negativo MUST ser `400` `negative_stock` (`application/problem+json`, título «El stock no puede ser negativo») y MUST NOT cambiar el stock. `type` fuera de `inbound`/`adjustment` (incl. `initial` y `sale`) MUST ser `400` `invalid_input`. `quantity` como número JSON MUST ser `400` `invalid_input`. Cuerpo sin `type` o `quantity` MUST ser `400` `invalid_input`.

Si `AppState` no tiene servicio de stock, esta ruta MUST responder `503` `unavailable` (`application/problem+json`). `vortex serve` MUST inyectar el servicio (`StockService` + adaptador `StockLedger` / `MovimientoStockRepo` de `storage`). MUST NOT registrar `GET /api/v1/products/{productId}/stock-movements` (historial T-02-26). MUST NOT registrar `PATCH` ni `DELETE` sobre un movimiento (HU-02-07 inmutabilidad). MUST NOT reindexar (T-03-07). `docs/openapi.yaml` MUST declarar `503` en `POST /api/v1/products/{productId}/stock-movements`. MUST NOT declarar `GET` como implementado en este change (el path GET permanece en el YAML como contrato v2).

#### Scenario: Ingreso desde caja-1 (happy, HU-02-07, Listo cuando)

- **WHEN** existe un producto con stock `"10"`
- **AND** se envía `POST /api/v1/products/{id}/stock-movements` con `{ "type": "inbound", "quantity": "20" }` y cabecera `X-Vectra-Terminal: caja-1`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** `Content-Type` es `application/json`
- **AND** el JSON tiene `type` `"inbound"`, `quantity` `"20"`, `resultingStock` `"30"` y `terminal` `"caja-1"`
- **AND** el JSON no tiene claves `tipo`, `cantidad`, `stock_resultante` ni `motivo`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 201 `application/json` no falla

#### Scenario: Ajuste a cero con motivo (happy, HU-02-07)

- **WHEN** existe un producto con stock `"5"`
- **AND** se envía `POST` con `{ "type": "adjustment", "quantity": "-5", "reason": "Conteo físico" }`
- **THEN** la respuesta tiene estado HTTP 201
- **AND** el JSON tiene `type` `"adjustment"`, `quantity` `"-5"`, `resultingStock` `"0"` y `reason` `"Conteo físico"`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 201 `application/json` no falla

#### Scenario: Ajuste sin motivo (unhappy, HU-02-07, Listo cuando)

- **WHEN** se envía `POST` con `{ "type": "adjustment", "quantity": "-1" }` sin `reason` (o `reason` nulo / solo espacios)
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `reason`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 400 `application/problem+json` no falla

#### Scenario: Stock negativo (unhappy, HU-02-07, Listo cuando)

- **WHEN** existe un producto con stock `"5"`
- **AND** se envía `POST` con `{ "type": "adjustment", "quantity": "-8", "reason": "Conteo físico" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `negative_stock` y `title` "El stock no puede ser negativo"
- **AND** un `GET /api/v1/products/{id}` posterior sigue teniendo `stock` `"5"`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 400 `application/problem+json` no falla

#### Scenario: Producto desconocido (unhappy)

- **WHEN** se envía `POST /api/v1/products/{ulid-inexistente}/stock-movements` con un cuerpo `inbound` válido
- **THEN** la respuesta tiene estado HTTP 404
- **AND** el JSON tiene `code` `not_found`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 404 `application/problem+json` no falla

#### Scenario: productId no es ULID (unhappy)

- **WHEN** se envía `POST /api/v1/products/not-a-ulid/stock-movements` con un cuerpo válido
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `productId`

#### Scenario: inbound con quantity negativa (unhappy)

- **WHEN** se envía `POST` con `{ "type": "inbound", "quantity": "-1" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `quantity`

#### Scenario: type initial (unhappy)

- **WHEN** se envía `POST` con `{ "type": "initial", "quantity": "1" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: type sale (unhappy)

- **WHEN** se envía `POST` con `{ "type": "sale", "quantity": "-1" }`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: quantity como número JSON (unhappy)

- **WHEN** se envía `POST` con `"quantity": 20` (número, no string)
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`

#### Scenario: X-Vectra-Terminal demasiado largo (unhappy)

- **WHEN** se envía `POST` con `X-Vectra-Terminal` de más de 64 caracteres
- **THEN** la respuesta tiene estado HTTP 400
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `X-Vectra-Terminal`

#### Scenario: sin servicio inyectado (unhappy)

- **WHEN** `AppState` no tiene servicio de stock
- **AND** se envía `POST /api/v1/products/{ulid}/stock-movements` con un cuerpo `inbound` mínimo
- **THEN** la respuesta tiene estado HTTP 503
- **AND** el JSON tiene `code` `unavailable`
- **AND** `assert_conforms` de `POST /api/v1/products/{productId}/stock-movements` 503 `application/problem+json` no falla

#### Scenario: GET historial no está registrado (unhappy, T-02-26)

- **WHEN** se envía `GET /api/v1/products/{id}/stock-movements`
- **THEN** la respuesta no es 200 con `StockMovementPage`
- **AND** el JSON no se presenta como historial paginado exitoso

#### Scenario: PATCH de un movimiento no existe (unhappy, HU-02-07 inmutabilidad)

- **WHEN** se envía `PATCH` a `/api/v1/products/{id}/stock-movements/{movementId}` o al collection path
- **THEN** la respuesta no es 200 de edición de movimiento
- **AND** no existe opción HTTP de editar un movimiento

#### Scenario: DELETE de un movimiento no existe (unhappy, HU-02-07 inmutabilidad)

- **WHEN** se envía `DELETE` a `/api/v1/products/{id}/stock-movements/{movementId}` o al collection path
- **THEN** la respuesta no es 204 ni 200 de borrado
- **AND** no existe opción HTTP de borrar un movimiento

### Requirement: DTO inglés de movimiento de stock según 5.3

El Gateway MUST traducir el contrato inglés al dominio español y de vuelta según [5.3](../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos). Campos de respuesta `StockMovement`: `id`, `productId`, `type`, `quantity`, `resultingStock`, `reason`, `saleId`, `terminal`, `createdAt`. `quantity` MUST ser string del esquema `Decimal`; `resultingStock` MUST ser string del esquema `NonNegativeDecimal`. MUST NOT serializar `producto`, `tipo`, `cantidad`, `stock_resultante`, `motivo`, `venta` ni `creado_en`.

`POST` MUST leer `type` → `tipo` (`inbound`→`ingreso`, `adjustment`→`ajuste`), `quantity` → `cantidad`, `reason` → `motivo`. Enums desconocidos MUST ser `400` `invalid_input`.

En `problem+json`, `errors[].field` MUST usar nombres de API (`type`, `quantity`, `reason`, `productId`, `X-Vectra-Terminal`, `body`). MUST NOT aparecer `tipo`, `cantidad` ni `motivo` en `errors[].field`.

#### Scenario: respuesta sin claves de dominio (happy)

- **WHEN** `POST` registra un inbound de `"20"`
- **THEN** el JSON de 201 tiene `id`, `productId`, `type` `"inbound"`, `quantity` string y `resultingStock` string
- **AND** no tiene `tipo`, `cantidad`, `stock_resultante` ni `motivo`

#### Scenario: errors[].field reason no motivo (unhappy, 5.3)

- **WHEN** el servicio rechaza un ajuste sin motivo con `FieldError.field` `motivo`
- **AND** el Gateway traduce ese error a HTTP
- **THEN** `errors[].field` es `reason`
- **AND** no aparece `motivo` en el cuerpo

### Requirement: Tests de integración de stock-movements contra OpenAPI

Antes de archive, la suite de `api` MUST incluir estos tests (nombres estables). Ticket padre: T-02-25. Listo cuando: escenarios de HU-02-07 como tests de `api` validados contra `openapi.yaml`. Cada respuesta JSON (éxito o error) MUST validarse con `assert_conforms` (T-09-04). `cargo test --workspace` MUST ejecutarlos.

| Test | Crate | Tipo |
|---|---|---|
| `post_stock_inbound_20_on_10_returns_201_with_terminal` | `api` | integración, happy, HU-02-07 / Listo cuando |
| `post_stock_adjustment_to_zero_returns_201` | `api` | integración, happy, HU-02-07 |
| `post_stock_adjustment_without_reason_returns_400` | `api` | integración, unhappy, HU-02-07 / Listo cuando |
| `post_stock_adjustment_minus_8_on_5_returns_400_negative_stock` | `api` | integración, unhappy, HU-02-07 / Listo cuando |
| `post_stock_unknown_product_returns_404` | `api` | integración, unhappy |
| `post_stock_invalid_product_id_returns_400` | `api` | integración, unhappy |
| `post_stock_inbound_negative_quantity_returns_400` | `api` | integración, unhappy |
| `post_stock_type_initial_returns_400` | `api` | integración, unhappy |
| `post_stock_type_sale_returns_400` | `api` | integración, unhappy |
| `post_stock_quantity_as_json_number_returns_400` | `api` | integración, unhappy |
| `post_stock_terminal_too_long_returns_400` | `api` | integración, unhappy |
| `post_stock_without_service_returns_503` | `api` | integración, unhappy |
| `get_stock_movements_is_not_page` | `api` | integración, unhappy, T-02-26 |
| `patch_stock_movement_is_not_registered` | `api` | integración, unhappy, HU-02-07 inmutabilidad |
| `delete_stock_movement_is_not_registered` | `api` | integración, unhappy, HU-02-07 inmutabilidad |
| `stock_problem_field_is_reason_not_motivo` | `api` | integración, unhappy, 5.3 |
| `vortex_serve_post_stock_unknown_product_returns_404` | `vortex` | binario, happy (servicio inyectado) |

#### Scenario: la suite de stock-movements cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** los JSON de 201/400/404/503 pasan `assert_conforms`

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
