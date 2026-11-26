# Proposal

## Ticket padre

- **Ticket:** [T-00-19](../../../docs/7-tickets.md) — Estáticos de la SPA y `/media`
- **Change:** `T-00-19`
- **HU:** [HU-01-01](../../../docs/6-historias.md#hu-01-01--aplicación-instalable-pwa)
- **Depende de:** T-00-10 (cerrado en `develop`; el Gateway ya tiene `problem+json`, middlewares, `/health` y `/metrics`)
- **Hace:** `index.html` servido desde `paths.web` reemplazando `{{VERSION}}`; activos bajo `/static/<versión>/…` con `Cache-Control: max-age=31536000, immutable`; `index.html`, `sw.js` y el *manifest* con `no-cache`; `/media/*` desde `<datos>/media` con caché larga (nombres por SHA-256)
- **No hace:** contenido de la SPA (Fase 1)
- **Listo cuando:** tests verifican las cabeceras de caché de cada tipo de recurso

## Why

El Gateway ya escucha HTTP y aplica `Cache-Control: no-store` solo bajo `/api/*`, pero no sirve la SPA ni `/media`. HU-01-01 (PWA instalable) y T-01-01 (esqueleto de `vectra-web`) necesitan que el Nodo Maestro entregue `index.html` con versión inyectada, activos versionados con caché inmutable, el *app shell* (`sw.js`, *manifest*) con `no-cache` y archivos de `<datos>/media` con caché larga. Sin estas rutas y cabeceras, T-00-20 no puede exponer un proceso que sirva la UI y T-01-01 no tiene de dónde cargar el HTML.

## What Changes

- `AppState` recibe raíces de estáticos: `paths.web` y `<datos>/media`, más la cadena de versión (la misma que `/health`, `CARGO_PKG_VERSION` en `vortex serve`)
- `build_router` registra GET de document root, *app shell*, `/static/<versión>/…` y `/media/*`; el 404 de archivo ausente o path inválido sigue siendo `application/problem+json` / `not_found`
- `GET /` (y `GET /index.html`) lee `{web}/index.html`, sustituye `{{VERSION}}` y envía `Cache-Control: no-cache` y `Content-Type: text/html`
- `GET /sw.js` y `GET /manifest.webmanifest` se sirven desde `paths.web` con `Cache-Control: no-cache`
- `GET /static/<versión>/{*path}` mapea a `{web}/{path}` solo si `<versión>` coincide con la versión inyectada; `Cache-Control: max-age=31536000, immutable`
- `GET /media/{*path}` mapea a `{datos}/media/{path}` con la misma caché larga (nombres content-addressed; este change no calcula SHA-256)
- `vortex serve` cablea las raíces desde `Config` (`paths.web`, `paths.datos.join("media")`)
- TDD: happy = cabeceras de cada tipo y sustitución de `{{VERSION}}`; unhappy = archivo ausente, versión de `/static` distinta, *path traversal*, método distinto de GET
- **BREAKING** (router): `GET /`, `GET /index.html`, `GET /sw.js`, `GET /manifest.webmanifest`, `GET /static/…` y `GET /media/…` dejan de ser 404 genéricos cuando el archivo existe

## Non-goals

Este change **no** cubre:

- Contenido de la SPA (`vectra-web/`, *router*, componentes, *tokens*) — **No hace** del ticket; **T-01-01** y Fase 1
- `manifest.webmanifest` real (nombre, íconos, `display`) — **T-01-13**
- Service Worker (*app shell*, precache, aviso sin conexión) — **T-01-14** / T-01-15
- Alta de imágenes, hashing SHA-256 al guardar, o limpieza de huérfanos — **T-02-28** … **T-02-33** (v2; HU-02-09)
- Fallback SPA a `index.html` en rutas desconocidas (el *router* es *hash*: `#/`, `#/admin/…`) — T-01-01
- TLS, `vectra.local`, listado de directorios, CORS
- Arranque ordenado (logs → Storage → API) — **T-00-20** (depende de este change)
- Cambiar `docs/openapi.yaml` (SPA y `/media` están fuera del contrato, [5.12](../../../docs/5-contract-api.md#512-fuera-de-este-contrato))
- Nuevas crates en `[workspace.dependencies]` ni `tokio` en `[dependencies]` de `api` (sigue en *dev-dependencies*)

## Capabilities

### New Capabilities

- (ninguna — el comportamiento vive en el API Gateway ya especificado)

### Modified Capabilities

- `vortex-api`: `build_router` sirve estáticos de `paths.web` y `/media` desde `<datos>/media` con las políticas de `Cache-Control` del ticket; `AppState` lleva las raíces y la versión; `vortex serve` las cablea desde `Config`. Deja de exigir que el router no registre estáticos

## Impact

- **Repo:** `vortex-core/crates/api/` (`AppState`, handlers de estáticos, tests de integración), `vortex-core/bin/vortex/` (`serve` cablea `paths.web` y `paths.datos/media`; test de binario)
- **APIs / OpenAPI:** sin paths nuevos en `docs/openapi.yaml`; estáticos y `/media` siguen fuera del contrato
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `openspec validate T-00-19` (o `t-00-19` si el CLI exige minúsculas)
- **Ticket:** cierra T-00-19 (Listo cuando: tests de cabeceras de caché por tipo de recurso)
- **BREAKING** (router): las rutas de document root, *app shell*, `/static/…` y `/media/…` pasan a servirse cuando el archivo existe; si no, siguen `404 not_found`
