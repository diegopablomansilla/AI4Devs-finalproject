# Proposal

## Ticket padre

- **Ticket:** [T-01-06](../../../docs/7-tickets.md) — Cliente HTTP `api.js`
- **Change:** `T-01-06`
- **HU:** [HU-01-05](../../../docs/6-historias.md#hu-01-05--estado-del-sistema-y-errores-visibles)
- **Depende de:** T-01-01 (en `develop`; el esqueleto ES Module ya existe)
- **Hace:** `request(method, path, { body, signal })` con JSON; `problem+json` → `ApiError { status, code, title, errors }`; error de red → `code: "network"`; cabecera `X-Vectra-Terminal` con el nombre de terminal guardado en `localStorage`.
- **No hace:** SSE (T-05-20) ni *multipart* (T-02-39).
- **Listo cuando:** pruebas manuales contra un endpoint inexistente devuelven `ApiError` con `code: "not_found"`.

## Why

La SPA aún no tiene un cliente HTTP: T-01-07 (`errors.js` / `vx-toast`), T-01-12 (`GET /health`) y todos los ABM/búsqueda posteriores necesitan un único `request()` que hable JSON, traduzca RFC 9457 a `ApiError` y envíe `X-Vectra-Terminal`. Sin eso, cada vista reinventaría `fetch` y el mapeo de `code`.

## What Changes

- `vectra-web/js/api.js`: ES Module con `request(method, path, { body, signal })` y clase `ApiError { status, code, title, errors }`
- Cuerpo JSON (`Content-Type: application/json`); respuestas 2xx se parsean como JSON
- Respuesta no OK con `application/problem+json` (esquema `Problem` de `docs/openapi.yaml`) → rechaza con `ApiError`
- Fallo de red (`fetch` rechaza) → `ApiError` con `code: "network"`
- Si hay nombre de terminal en `localStorage`, cada petición lleva `X-Vectra-Terminal`
- TDD: happy = JSON 2xx, cabecera de terminal, `signal` de `AbortController`; unhappy = 404 `not_found`, `invalid_input` con `errors[]`, red, cuerpo de error no JSON, abort no se disfraza de `network`
- Extender `scripts/check-vectra-web.sh` y `scripts/fixtures/vectra-web/` (tests `node --test` con `fetch` inyectado; sin `package.json`)

## Non-goals

- SSE / `ReadableStream` (`js/sse.js`) — **T-05-20** (v2; el ticket cita T-05-20)
- *Multipart* / subida de imágenes — **T-02-30** … **T-02-39** (HU-02-09, v2; el ticket cita T-02-39)
- Mensajes en español por `ErrorCode` y `showError` con `vx-toast` — **T-01-07**
- Polling de `/health` y capacidades — **T-01-12**
- UI "Nombre de esta terminal" — **T-01-11** (este change solo *lee* la clave de `localStorage`)
- Autenticación, cookies, CORS, base URL configurable (mismo origen; MVP en `localhost`)
- Timeouts de cliente distintos del `signal` que pasa el caller (los timeouts de [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway) son del Gateway)
- Playwright, jsdom, `package.json`, job nuevo de CI, cambios a `docs/openapi.yaml` o crates Rust
- Cablear `request()` en vistas de negocio (búsqueda T-03-15, ABM T-02-*)

## Capabilities

### New Capabilities

- Ninguna. El cliente HTTP es parte de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `js/api.js` (`request` + `ApiError`); ampliar el árbol exigido y la suite con happy/unhappy de JSON, `problem+json`, red, abort y cabecera de terminal

## Impact

- **Repo:** `vectra-web/js/api.js`, `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/` (p. ej. `api.test.mjs`)
- **APIs / OpenAPI:** sin paths nuevos; el cliente consume el esquema `Problem` ya definido ([5.4](../../../docs/5-contract-api.md#54-errores), `docs/openapi.yaml`)
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-06`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-06 (Listo cuando: petición a ruta inexistente → `ApiError` con `code: "not_found"`; automatizado con `fetch` simulado más comprobación manual contra el Gateway)
