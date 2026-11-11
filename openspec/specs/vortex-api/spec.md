# vortex-api Specification

## Purpose

Define el esqueleto HTTP del API Gateway de Vortex Core: router con estado de aplicación clonable, traducción de errores de dominio y de Gateway a RFC 9457 (`application/problem+json`), middlewares transversales (límite de cuerpo, timeout, compresión, `Cache-Control: no-store` en `/api/*`, span de tracing), `GET /health` con snapshot agregado, `GET /metrics` Prometheus, estáticos de la SPA (`index.html` con `{{VERSION}}`, `/static/<versión>/`, `sw.js`, *manifest*) y `/media/*` desde `<datos>/media`, `vortex serve` escuchando en HTTP según `server.bind`, respuesta 404 a rutas inexistentes y apagado ordenado con SIGTERM y Ctrl+C, de modo que el proceso arranque y termine limpio antes de TLS o rutas de negocio.

Ticket padre: T-00-10, T-00-11, T-00-12, T-00-13, T-00-14, T-00-19, T-00-20. HU: HU-00-05, HU-00-06, HU-00-09, HU-01-01, HU-01-05, HU-09-02.

## Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). `AppState` MUST incluir un `HealthReport` (inyectado; MAY usar un snapshot por defecto con los seis componentes en `disabled` para tests). `AppState` MUST incluir raíces `web` y `media` y una `version` (requirement «Estáticos de la SPA y /media»). El router MUST registrar `GET /health` (requirement «GET /health expone el snapshot agregado»). El router MUST registrar `GET /metrics` (requirement «GET /metrics expone texto Prometheus»). El router MUST registrar los GET de estáticos y `/media` (requirement «Estáticos de la SPA y /media»). El router MUST NOT registrar rutas de negocio bajo `/api/v1`. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»).

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

`vortex serve` MUST dejar de responder `no implementado`. MUST resolver el path de configuración como en `vortex-config` (`--config` gana sobre `VORTEX_CONFIG`), MUST cargar y validar el TOML, MUST instalar el subscriber de logging (capability `vortex-logging`), MUST abrir Storage y migrar según el requirement «Arranque ordenado de vortex serve (paso 1)», y MUST escuchar HTTP (sin TLS) en la dirección `server.bind` **solo después** de esas fases. MUST cablear un registro de salud que implementa `HealthReport`. MUST inyectar en `AppState` `paths.web` como raíz `web`, `paths.datos` unido a `media` como raíz `media`, y la `version` del binario. MUST NOT arrancar sidecars. Una petición HTTP a una ruta no registrada MUST responder 404 con `Content-Type: application/problem+json` y `code` `not_found`. `GET /health` contra ese bind MUST responder según el requirement «GET /health expone el snapshot agregado» y, en un arranque exitoso de este change, MUST coincidir con «Arranque ordenado de vortex serve (paso 1)». `GET /metrics` contra ese bind MUST responder según el requirement «GET /metrics expone texto Prometheus». `GET /` contra ese bind MUST responder según el requirement «Estáticos de la SPA y /media» cuando `{web}/index.html` existe.

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
