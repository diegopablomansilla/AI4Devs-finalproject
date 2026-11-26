# Spec Delta

Ticket padre: T-01-13. Change: T-01-13. HU: HU-01-01. El Gateway ya sirve `GET /manifest.webmanifest`; este delta solo sustituye `{{VERSION}}` en ese cuerpo para que los `src` de íconos bajo `/static/<versión>/` resuelvan.

## MODIFIED Requirements

### Requirement: Estáticos de la SPA y /media

`AppState` MUST incluir una raíz `web` (directorio de `paths.web`), una raíz `media` (`<datos>/media`) y una cadena `version` no vacía. El constructor por defecto (`AppState::new` / `with_health`) MAY usar raíces inexistentes y `version` igual a `CARGO_PKG_VERSION` del crate `api`. `vortex serve` MUST inyectar `paths.web`, `paths.datos` unido a `media`, y la misma `version` que publica `/health`.

El router de `build_router` MUST registrar estos GET (y MUST NOT exigir que los archivos existan al construir el router):

| Ruta | Archivo en disco | `Cache-Control` | Cuerpo |
|---|---|---|---|
| `/` y `/index.html` | `{web}/index.html` | `no-cache` | HTML con cada `{{VERSION}}` reemplazado por `version` |
| `/sw.js` | `{web}/sw.js` | `no-cache` | bytes del archivo |
| `/manifest.webmanifest` | `{web}/manifest.webmanifest` | `no-cache` | texto con cada `{{VERSION}}` reemplazado por `version` |
| `/static/{version}/{*path}` | `{web}/{path}` | `max-age=31536000, immutable` | bytes del archivo, solo si `{version}` es exactamente `AppState.version` y `{path}` es un archivo regular bajo `{web}` |
| `/media/{*path}` | `{media}/{path}` | `max-age=31536000, immutable` | bytes del archivo, solo si `{path}` es un archivo regular bajo `{media}` |

`Content-Type` de `/` e `/index.html` MUST ser `text/html` (MAY incluir `charset=utf-8`). Las demás rutas MUST enviar un `Content-Type` coherente con el recurso (p. ej. `text/javascript` o `application/javascript` para `.js`; `application/manifest+json` para el *manifest*; tipo de imagen o `application/octet-stream` para `/media`). Estas respuestas MUST NOT enviar `Cache-Control: no-store` (el middleware de `/api/*` no aplica). `{{VERSION}}` MUST sustituirse de forma literal, todas las ocurrencias, en `index.html` **y** en `manifest.webmanifest`; MUST NOT interpretarse como plantilla fuera de esos dos archivos.

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
