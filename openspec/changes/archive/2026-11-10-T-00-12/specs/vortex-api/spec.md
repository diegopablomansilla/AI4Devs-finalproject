# Spec Delta

Ticket padre: T-00-12. Change: T-00-12. HU: HU-00-09.

## ADDED Requirements

### Requirement: Límite de cuerpo por defecto 256 KiB

El Gateway MUST rechazar un cuerpo de petición cuyo tamaño supere **256 KiB** (262144 bytes). La respuesta MUST tener estado HTTP `413`, `Content-Type: application/problem+json`, `code` `payload_too_large`, `title` "El cuerpo es demasiado grande" y `type` `https://vectra.local/problems/payload-too-large`. `instance`, cuando la petición tiene path, MUST ser esa ruta. El JSON MUST NOT incluir `errors`. Un cuerpo de **exactamente** 256 KiB MUST NOT producir `413` por este límite. Este change MUST NOT aplicar límites distintos por ruta (search, imágenes, speech, etc.): esos valores viven en [5.10](../../../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway) y los fija cada ticket de endpoint.

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

Cada petición atendida por `build_router` MUST abrir un span de `tracing` que registre al menos el método HTTP, la ruta (path o ruta coincidente), el estado de la respuesta y la duración. El Gateway MUST NOT instalar el subscriber global de logging en este change (`vortex serve` sigue sin cablear T-00-06). Si hay un subscriber activo (p. ej. en un test), el span MUST ser observable.

#### Scenario: span incluye método, ruta, estado y duración (happy)

- **WHEN** se envía `GET /no-such-route` con un subscriber de tracing de test instalado
- **THEN** existe un span de la petición cuyo método es `GET`
- **AND** el span registra la ruta `/no-such-route` (o equivalente)
- **AND** el span registra el estado 404
- **AND** el span registra una duración

## MODIFIED Requirements

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

`cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST ejecutarlos. Los tests de `problem+json` MUST asertar estado HTTP, `Content-Type: application/problem+json` y `code`. Los de middleware MUST cubrir cuerpo excedido, timeout y cabeceras (`Cache-Control`, `Content-Encoding`). Los stubs de los demás subcomandos MUST seguir cubiertos.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core`
- **THEN** corren los tests de la tabla
- **AND** los de 404 / `problem+json` / apagado ordenado pasan
- **AND** los de JSON inválido, 500 sin leak, config ausente/inválida y bind ocupado pasan
- **AND** los de cuerpo excedido, timeout, cabeceras y span HTTP pasan
