# vortex-api Specification

## Purpose

Define el esqueleto HTTP del API Gateway de Vortex Core: router con estado de aplicación clonable, traducción de errores de dominio y de Gateway a RFC 9457 (`application/problem+json`), `vortex serve` escuchando en HTTP según `server.bind`, respuesta 404 a rutas inexistentes y apagado ordenado con SIGTERM y Ctrl+C, de modo que el proceso arranque y termine limpio antes de TLS o rutas de negocio.

Ticket padre: T-00-10, T-00-11. HU: HU-00-06, HU-01-05, HU-09-02.

## Requirements

### Requirement: build_router expone un router HTTP con AppState clonable

El crate `api` MUST exportar `build_router` que acepta un `AppState` y MUST devolver el router HTTP del Gateway. `AppState` MUST ser clonable y MUST poder compartirse entre peticiones (servicios inyectados como `Arc<dyn …>` cuando existan). En este change `AppState` MUST construirse sin puertos de dominio (aún no hay casos de uso). El router MUST NOT registrar rutas de negocio, `/health`, `/metrics` ni estáticos. Una petición a una ruta no registrada MUST responder `404` con `Content-Type: application/problem+json` y `code` `not_found` (requirement «404 y 500 genéricos sin detalles internos»).

#### Scenario: ruta inexistente responde 404 (happy)

- **WHEN** se envía `GET` a una ruta que el router no registra (p. ej. `/no-such-route`) sobre el router de `build_router`
- **THEN** la respuesta tiene estado HTTP 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`

#### Scenario: AppState se comparte entre peticiones (happy)

- **WHEN** se clona un `AppState` y se construyen peticiones contra el mismo router
- **THEN** ambas peticiones se atienden sin error de estado
- **AND** no hace falta un servicio de dominio para construir el `AppState`

### Requirement: vortex serve escucha HTTP en server.bind

`vortex serve` MUST dejar de responder `no implementado`. MUST resolver el path de configuración como en `vortex-config` (`--config` gana sobre `VORTEX_CONFIG`), MUST cargar y validar el TOML, y MUST escuchar HTTP (sin TLS) en la dirección `server.bind`. MUST NOT abrir Storage, MUST NOT instalar el subscriber de logging, MUST NOT arrancar sidecars. Una petición HTTP a una ruta no registrada MUST responder 404 con `Content-Type: application/problem+json` y `code` `not_found`.

#### Scenario: vortex serve arranca y responde 404 (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` con `server.bind` en `127.0.0.1` y un puerto libre
- **THEN** el proceso queda en ejecución escuchando en esa dirección
- **AND** `GET /no-such-route` contra ese bind recibe estado HTTP 404
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `not_found`

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

`cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST ejecutarlos. Los tests de `problem+json` MUST asertar estado HTTP, `Content-Type: application/problem+json` y `code`. Los stubs de los demás subcomandos MUST seguir cubiertos.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core`
- **THEN** corren los tests de la tabla
- **AND** los de 404 / `problem+json` / apagado ordenado pasan
- **AND** los de JSON inválido, 500 sin leak, config ausente/inválida y bind ocupado pasan
