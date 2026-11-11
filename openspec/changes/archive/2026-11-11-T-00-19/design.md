# Design

## Context

En `develop`, `api::build_router` registra `/health`, `/metrics` y fallback `404 problem+json`. El middleware de T-00-12 pone `Cache-Control: no-store` **solo** en `/api` y `/api/*`; el resto de paths no lleva política de caché. `AppState` solo tiene `HealthReport`. `vortex serve` construye `AppState::with_health` y no pasa `paths.web` ni `paths.datos`. `tokio` en `api` es solo `[dev-dependencies]`; `tower-http` no activa `fs`. No existe `vectra-web/` (T-01-01). Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: HTTP `localhost` (MVP); sin contenido de SPA; ~2 h; sin crates nuevas en `[workspace.dependencies]`.

## Goals / Non-Goals

**Goals:**

- Handlers GET de document root, *app shell*, `/static/<versión>/` y `/media/` con las cabeceras del ticket
- `AppState` con raíces + `version`; `vortex serve` las cablea desde `Config`
- 404 `not_found` (mismo `ApiError`) si falta el archivo, la versión no coincide o el path sale de la raíz
- Tests de crate (cabeceras, *traversal*) y un test de binario (`paths.web`)

**Non-Goals (diseño):**

- `tower-http` feature `fs` / `ServeDir` (el 404 de tower no es `problem+json` y no sustituye `{{VERSION}}`)
- `tokio` en `[dependencies]` de `api` (lectura de archivos locales con `std::fs`; el Nodo Maestro sirve disco local)
- `mime_guess` u otra crate para MIME (mapa mínimo por extensión)
- Fallback SPA (`/*` → `index.html`); el *router* de T-01-01 es *hash*
- Crear `vectra-web/` ni un `index.html` de producción

## Decisions

### 1. Handlers propios, no ServeDir

**Elección:** módulo `api` (`static_files` o equivalente) con handlers Axum `GET`. Leen con `std::fs::read` / `read_to_string` (archivos chicos, disco local). Respuesta: `StatusCode` + cabeceras + `Bytes`/`String`, o `ApiError::route_not_found`. `{{VERSION}}` se reemplaza con `str::replace` en el HTML.

**Por qué:** un único 404 `problem+json`; control exacto de `Cache-Control`; sustitución de versión; *path traversal* explícito. ServeDir devolvería 404 de tower y no toca `{{VERSION}}`.

**Alternativa:** `tower-http` `ServeDir` + `set_status` fallback. Se descartó: dos formas de 404 y allowlist de features. **Alternativa:** `tokio::fs` en `[dependencies]` de `api`. Se descartó: el ticket cabe sin mover tokio; el I/O es local y acotado.

### 2. Mapa URL → disco (sin directorio `static/<versión>/` en disco)

**Elección:**

```
GET /                      → {web}/index.html     (sustituye {{VERSION}})
GET /index.html            → igual
GET /sw.js                 → {web}/sw.js
GET /manifest.webmanifest  → {web}/manifest.webmanifest
GET /static/{ver}/{*path}  → {web}/{path}         si ver == AppState.version
GET /media/{*path}         → {media}/{path}
```

T-01-01 deja `css/` y `js/` en la raíz de `vectra-web/`, no bajo `static/<versión>/`. El prefijo `/static/<versión>/` es solo de URL (caché inmutable). `index.html` referenciará `/static/{{VERSION}}/js/app.js` cuando exista (Fase 1).

**Por qué:** el Hace pide esa URL; el layout de T-01-01 no versiona carpetas. Una versión distinta → 404, así un HTML viejo no pincha JS nuevo con `immutable`.

**Alternativa:** servir `{web}/static/{ver}/…` tal cual. Se descartó: obliga a T-01-01 a copiar assets por versión sin *build*. **Alternativa:** ignorar `{ver}` y servir cualquier prefijo. Se descartó: rompe el contrato de caché inmutable.

### 3. Join seguro contra path traversal

**Elección:** función `safe_join(root, rel) -> Option<PathBuf>`:

1. Rechazar `rel` vacío, absoluto, o con componente `..` / `.` superfluo / NUL
2. `root.join(rel)` y `canonicalize` de `root` (debe existir como dir) y del candidato si el archivo existe
3. El resultado canonicalizado MUST tener prefijo `root` canonicalizado (comparar `starts_with` sobre componentes, no string crudo)
4. Si el archivo no existe: 404, sin seguir symlinks fuera (canonicalize falla o el prefijo no coincide)

Tests plantan `secret.txt` **fuera** de `{web}` / `{media}` y piden `../secret.txt` y `%2e%2e`.

**Por qué:** el unhappy path de HU/ticket es no filtrar el disco del Nodo Maestro. `Path::join("..")` de Rust no bloquea `..`.

**Alternativa:** `normalize` sin canonicalize. Se descartó: symlinks. **Alternativa:** denylist de `..` solo en el string. Se descartó: encodings (`%2e%2e`); Axum decodifica el path, se aserta también el caso decodificado.

### 4. AppState: raíces opcionales en el default

**Elección:** `AppStateInner` suma `web_root: PathBuf`, `media_root: PathBuf`, `version: String`. `new()` / `with_health()` usan paths vacíos o un dir que no existe y `env!("CARGO_PKG_VERSION")`. Builder `with_static_roots(self, web, media) -> Self` (o `with_paths`) para tests y para `serve`. `vortex serve`: `AppState::with_health(health).with_static_roots(web, datos.join("media"))`.

**Por qué:** los tests actuales de `/health` y 404 no arman fixtures; `GET /` sin `index.html` sigue 404 `not_found` (handler registrado que no encuentra archivo, o el mismo `ApiError`). No se rompe `AppState::new()`.

**Alternativa:** exigir raíces en `new()`. Se descartó: reescribir todos los tests del crate. **Alternativa:** `Option<PathBuf>`. Equivale; `PathBuf` vacío es más simple al join (falla → 404).

### 5. Fixtures temporales, no vectra-web/

**Elección:** tests de `crates/api/tests/static_files.rs` crean un dir con `std::env::temp_dir()` (mismo patrón que `bin/vortex/tests/serve.rs`): `index.html` con `{{VERSION}}`, `sw.js`, `manifest.webmanifest`, `app.js`, y `{datos}/media/productos/01TEST/3f2a9c.jpg`. El test de binario apunta `paths.web` / `paths.datos` del TOML a esos dirs.

**Por qué:** **No hace** contenido de la SPA. T-01-01 crea `vectra-web/`.

**Alternativa:** un `vectra-web/index.html` mínimo en el repo. Se descartó: pisa el alcance de T-01-01.

### 6. MIME mínimo sin crate nueva

**Elección:** mapa por extensión: `.html` → `text/html; charset=utf-8`; `.js` → `text/javascript`; `.webmanifest` → `application/manifest+json`; `.css` → `text/css`; `.jpg`/`.jpeg` → `image/jpeg`; `.png` → `image/png`; `.webp` → `image/webp`; default `application/octet-stream`.

**Por qué:** no hay `mime_guess` en `[workspace.dependencies]`. El Listo cuando es cabeceras de **caché**, no MIME exhaustivo.

## Risks / Trade-offs

- **[std::fs bloquea el worker Tokio]** → Aceptado en MVP: HTML/JS/imágenes locales, Nodo Maestro un proceso. Si clippy `disallowed_methods` no aplica, no spawn_blocking. Reevaluar si T-02-29 sirve imágenes grandes (v2).
- **[GET / pasa de fallback 404 a handler]** → Si falta `index.html`, sigue 404 `not_found`; tests de `/no-such-route` no cambian. POST `/` deja de ser fallback y puede ser 405; el test `post_index_is_not_200_html` lo fija.
- **[canonicalize en macOS (APFS case-insensitive)]** → Comparar paths canonicalizados; fixtures con nombres distintos (`secret.txt` vs `app.js`).
- **[CompressionLayer sobre HTML/JS]** → Las cabeceras de caché se setean en el handler **antes** de compression; el test raw HTTP/1.1 sin `Accept-Encoding` ve el `Cache-Control` sin gzip. No asertar ausencia de `Content-Encoding` si el cliente no pide compresión.
- **[T-01-01 debe usar `/static/{{VERSION}}/…` en el HTML]** → Este change documenta el mapa; T-01-01 consume el placeholder. Sin eso, el *router* hash igual carga `/` y los JS colgarían de URLs sin versionar (fuera de alcance).

## Migration Plan

1. Red: `static_files.rs` y el test de `serve` no compilan o fallan; `GET /` con fixture sigue 404.
2. Green: `AppState` + handlers + `safe_join` + cableado en `serve`.
3. Cierre: § 5 en `vortex-core/`; `openspec validate T-00-19`.
4. Rollback: revertir handlers, campos de `AppState` y tests. Sin datos de usuario ni contrato OpenAPI.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre del *manifest* queda `manifest.webmanifest` (T-01-13); no se sirve `manifest.json`.
