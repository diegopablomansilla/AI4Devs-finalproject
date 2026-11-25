# Spec Delta

Ticket padre: T-00-27. Change: T-00-27. HU: HU-00-06.

## ADDED Requirements

### Requirement: Arranque ordenado de vortex serve (paso 2)

Tras Storage y migraciones OK (requirement «Arranque ordenado de vortex serve (paso 1)»), y **antes** de aceptar el listener TCP, `vortex serve` MUST, sobre HTTP en `localhost` (sin TLS, Meilisearch externo [2.7](../../../../../docs/2-arquitectura.md#27-red-local-y-seguridad-mínima)):

1. Instanciar el adaptador de búsqueda de texto con `meilisearch.url` y `meilisearch.api_key` del `Config`.
2. Publicar `meilisearch` en `starting`.
3. Esperar a que la sonda de salud del adaptador reporte el componente `up`, con un límite de **20 segundos**. Si vence el límite o la sonda no reporta `up`, MUST publicar `meilisearch` en `down` y **seguir** (modo degradado). MUST NOT terminar el proceso ni omitir el bind HTTP solo porque Meilisearch esté caído.
4. Arrancar el Index Sync Worker inyectando el uid de índice `productos`, el `OutboxNotifier` de Storage, los repositorios de outbox/producto/`atributo_def` y la telemetría Prometheus de sync. MUST publicar `sync` en `up` cuando la tarea queda en ejecución (también si Meilisearch está `down`; el worker ya reintenta).
5. Recién entonces escuchar en `server.bind`.

MUST NOT lanzar el binario de Meilisearch ni ningún sidecar (T-00-25). MUST NOT cargar modelos de IA ni publicar `llm` / `speech` / `textEmbeddings` distintos de `disabled` (T-05-10, T-05-13, T-06-08). MUST NOT aplicar `vortex reindex` ni `swapIndexes` (T-03-11).

Si Meilisearch está `up` al terminar la espera, el primer `GET /health` observable MUST tener HTTP 200, `status` `ok`, `components.storage.status` `up`, `components.meilisearch.status` `up` y `components.sync.status` `up`. Si Meilisearch está `down`, el primer `GET /health` observable MUST tener HTTP 200 (no 503), `status` `degraded`, `components.storage.status` `up`, `components.meilisearch.status` `down` y `components.sync.status` `up`. El cuerpo MUST NOT ser `application/problem+json`.

Las fases de este paso MUST registrar duración en eventos o spans `boot.meilisearch` y `boot.sync` (mismo patrón `boot.*` del paso 1).

#### Scenario: Meilisearch disponible arranca ok (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` con `meilisearch.url` apuntando a un Meilisearch que responde `available` antes de 20 s
- **AND** `paths.datos` escribible y `server.bind` libre en `127.0.0.1`
- **THEN** el proceso queda escuchando en esa dirección
- **AND** `GET /health` recibe HTTP 200 y `Content-Type` `application/json`
- **AND** el JSON tiene `status` `ok`
- **AND** `components.storage.status` es `up`
- **AND** `components.meilisearch.status` es `up`
- **AND** `components.sync.status` es `up`
- **AND** el JSON no tiene `code` de `problem+json`

#### Scenario: Meilisearch detenido no impide la API (unhappy, Listo cuando)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` con `meilisearch.url` inalcanzable (Meilisearch detenido o puerto cerrado)
- **AND** `paths.datos` escribible y `server.bind` libre en `127.0.0.1`
- **THEN** el proceso queda escuchando en esa dirección (no termina con error)
- **AND** `GET /health` recibe HTTP 200 y `Content-Type` `application/json`
- **AND** el JSON tiene `status` `degraded`
- **AND** `components.meilisearch.status` es `down`
- **AND** `components.storage.status` es `up`
- **AND** `components.sync.status` es `up`
- **AND** el JSON no tiene `code` de `problem+json`

#### Scenario: las fases meilisearch y sync registran duración (happy)

- **WHEN** un arranque de `vortex serve` llega a instalar logging y completa el paso 2
- **THEN** el JSON bajo `<paths.datos>/logs/` incluye eventos o spans `boot.meilisearch` y `boot.sync`
- **AND** cada uno registra una duración

### Requirement: Tests del arranque ordenado (paso 2)

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Crate | Tipo | Motor |
|---|---|---|---|
| `vortex_serve_health_ok_when_meilisearch_up` | `vortex` (`bin/vortex/tests/serve.rs`) | happy | `#[ignore = "meilisearch"]` |
| `vortex_serve_starts_degraded_when_meilisearch_down` | `vortex` (`bin/vortex/tests/serve.rs`) | unhappy, Listo cuando | URL inalcanzable; sin sidecar |
| `wait_meilisearch_returns_true_when_probe_is_up` | `vortex` (`bin/vortex/src/`) | happy | unidad (sonda falsa) |
| `wait_meilisearch_returns_false_when_probe_stays_down` | `vortex` (`bin/vortex/src/`) | unhappy | unidad (sonda falsa; timeout corto) |

`cargo test --workspace` **sin** `--include-ignored` MUST ejecutar el Listo cuando y los tests de la sonda, y MUST terminar 0 aunque Meilisearch no esté levantado. `cargo test --workspace -- --include-ignored meilisearch` MUST ejecutar `vortex_serve_health_ok_when_meilisearch_up`.

#### Scenario: Listo cuando sin sidecar (unhappy)

- **WHEN** se ejecuta `cargo test -p vortex vortex_serve_starts_degraded_when_meilisearch_down` **sin** `--include-ignored`
- **THEN** el test corre y termina con código 0

#### Scenario: Happy con sidecar (happy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test -p vortex -- --include-ignored meilisearch`
- **THEN** `vortex_serve_health_ok_when_meilisearch_up` corre y termina con código 0

## MODIFIED Requirements

### Requirement: vortex serve escucha HTTP en server.bind

`vortex serve` MUST dejar de responder `no implementado`. MUST resolver el path de configuración como en `vortex-config` (`--config` gana sobre `VORTEX_CONFIG`), MUST cargar y validar el TOML, MUST instalar el subscriber de logging (capability `vortex-logging`), MUST abrir Storage y migrar según el requirement «Arranque ordenado de vortex serve (paso 1)», MUST verificar Meilisearch y arrancar el Index Sync Worker según el requirement «Arranque ordenado de vortex serve (paso 2)», y MUST escuchar HTTP (sin TLS) en la dirección `server.bind` **solo después** de esas fases. MUST cablear un registro de salud que implementa `HealthReport`. MUST inyectar en `AppState` `paths.web` como raíz `web`, `paths.datos` unido a `media` como raíz `media`, y la `version` del binario. MUST inyectar el servicio de categorías (adaptador `CategoryRepo` de `storage` + occupancy stub que no reporta productos activos) y MUST inyectar el servicio de marcas (adaptador `BrandRepo` / `MarcaRepo` de `storage`) y MUST inyectar el servicio de definiciones de atributo (adaptador `AttributeDefRepo` / `AtributoDefRepo` de `storage`) y MUST inyectar el servicio de producto (`ProductService` con adaptadores `ProductRepo` / `ProductoRepo`, `CategoryRepo`, `BrandRepo` y `AttributeDefRepo` de `storage`) y MUST NOT inyectar `Storage` crudo en `AppState`. MUST NOT arrancar sidecars (Meilisearch se consume por URL). Una petición HTTP a una ruta no registrada MUST responder 404 con `Content-Type: application/problem+json` y `code` `not_found`. `GET /health` contra ese bind MUST responder según el requirement «GET /health expone el snapshot agregado» y, en un arranque que completa el paso 2, MUST coincidir con «Arranque ordenado de vortex serve (paso 2)». `GET /metrics` contra ese bind MUST responder según el requirement «GET /metrics expone texto Prometheus». `GET /` contra ese bind MUST responder según el requirement «Estáticos de la SPA y /media» cuando `{web}/index.html` existe. `GET /api/v1/categories` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/brands` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/attribute-definitions` contra ese bind MUST responder `200` con `{ "items": ... }` (lista vacía si no hay filas), no `503`. `GET /api/v1/products/{ulid-válido-inexistente}` contra ese bind MUST responder `404` `not_found`, no `503`.

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
- **AND** HTTP es 200
- **AND** el JSON tiene `status` (`ok` o `degraded`), `version`, `uptimeSeconds` y `components` con los seis nombres
- **AND** `components.storage.status` es `up`
- **AND** `components.meilisearch.status` es `up` o `down` (no `disabled`)
- **AND** `components.sync.status` es `up`
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

`vortex serve` MUST arrancar en este orden, sobre HTTP en `localhost` (sin TLS): (1) cargar y validar `Config`; (2) instalar el subscriber de logging; (3) abrir Storage embebido en `<paths.datos>/db` y aplicar las migraciones embebidas; (4) publicar salud; (5) verificar Meilisearch y arrancar el Index Sync Worker según el requirement «Arranque ordenado de vortex serve (paso 2)»; (6) escuchar en `server.bind`. MUST NOT lanzar procesos sidecar. MUST NOT inyectar Storage crudo en `AppState`.

Antes de abrir Storage, el registro de salud MUST publicar `storage` en `starting`. Tras migrar con éxito, MUST publicar `storage` en `up`. El listener TCP MUST NOT aceptarse hasta que Storage y migraciones terminen OK **y** el paso 2 haya publicado `meilisearch` (`up` o `down`) y arrancado el worker. El primer `GET /health` observable contra ese bind MUST tener HTTP 200, `components.storage.status` `up`, y el resto según el paso 2 (ya no se exige agregado `ok` ni los demás componentes `disabled`).

Si falla la configuración, la instalación de logs, la apertura de Storage o las migraciones, el proceso MUST terminar con código distinto de 0, MUST escribir el error en stderr (y en logs si el subscriber ya está instalado), y MUST NOT quedar escuchando HTTP. Un Meilisearch caído MUST NOT usarse como fallo de Storage.

Cada fase (config, logging, storage, health, api) MUST registrar su duración en un span o evento `tracing` cuyo nombre empiece por `boot.` (`boot.config`, `boot.logging`, `boot.storage`, `boot.health`, `boot.api`). El JSON bajo `<datos>/logs/` MUST incluir esos nombres cuando el arranque llega a instalar logging. Las fases `boot.meilisearch` y `boot.sync` las exige el paso 2.

#### Scenario: vortex serve espera health ok con storage up (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` con `paths.datos` en un directorio temporal escribible y `server.bind` libre en `127.0.0.1`
- **THEN** el proceso queda escuchando en esa dirección
- **AND** `GET /health` contra ese bind recibe HTTP 200
- **AND** `Content-Type` es `application/json`
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
