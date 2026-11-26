# Design

## Context

`develop` ya sirve `GET /manifest.webmanifest` desde `{web}/manifest.webmanifest` con `no-cache` y MIME `application/manifest+json` (T-00-19), y `index.html` con `{{VERSION}}` solo en ese HTML. `vectra-web/` no tiene *manifest* ni PNG. El Gateway **no** tiene ruta `/icons/`; los PNG solo salen por `/static/{version}/{*path}` → `{web}/{path}`. El *manifest* se sirve como bytes crudos: si los `src` llevan `{{VERSION}}`, hay que sustituirlo en `get_manifest`. El stub Node no es jsdom: tests de *manifest* leen archivos y parsean JSON / IHDR. Ver proposal.md para el porqué; spec para campos Lighthouse y Listo cuando.

## Goals / Non-Goals

**Goals:**

- *Manifest* parseable + *link* en `index.html` + PNG 192/512 *any* y *maskable* verificables en Node
- `src` de íconos resolubles en runtime (`/static/<versión>/icons/…`) sin ruta Gateway nueva
- Listo cuando automatizado: criterios *manifest* de instalable (sin Lighthouse CLI)

**Non-Goals:**

- Extraer un *builder* de PNG reutilizable o un *design system* de marca
- Medir la zona segura *maskable* píxel a píxel (solo dimensiones IHDR)
- Registrar Service Worker o crear `sw.js`

## Decisions

### 1. Sustituir `{{VERSION}}` en `get_manifest`, no crear `/icons/`

Los `src` del *manifest* MUST ser `/static/{{VERSION}}/icons/…`. El *link* en HTML queda `/manifest.webmanifest` (no-cache). `get_manifest` lee el archivo como texto y hace el mismo `replace("{{VERSION}}", version)` que `get_index`.

Alternativas descartadas:

- **Ruta `/icons/{*path}`** — duplica el mapa de T-00-19 y no versiona los PNG.
- **`src` relativos `icons/…`** — el navegador pide `/icons/icon-192.png` (404: no hay ruta).
- **Data URI** — Lighthouse/Chrome suelen exigir íconos fetchables; infla el JSON.
- **Versionar el *manifest* bajo `/static/`** — perdería `no-cache` (T-00-19).

Si el archivo no trae el placeholder, el replace es no-op: `manifest_sends_no_cache` sigue verde.

### 2. Cuatro PNG en `vectra-web/icons/`

| Archivo | sizes | purpose |
|---|---|---|
| `icon-192.png` | 192×192 | `any` |
| `icon-512.png` | 512×512 | `any` |
| `icon-192-maskable.png` | 192×192 | `maskable` |
| `icon-512-maskable.png` | 512×512 | `maskable` |

Fondo `#0A0A0A`. En *maskable*, la marca (p. ej. una «V» clara) va en el 80 % central. Se commitean los binarios; no se generan en cada test.

Alternativa descartada: un solo PNG con `purpose: "any maskable"` — Lighthouse pide *maskable* con padding aparte.

Generación: helper mínimo (Node `zlib` o un one-shot local) que escribe PNG RGB; sin npm, sin ImageMagick como dependencia de CI.

### 3. Tests: `manifest.test.mjs` + harness; IHDR sin deps

`scripts/fixtures/vectra-web/manifest.test.mjs` parsea el JSON real y lee IHDR (bytes 16–23, big-endian) de cada PNG. Unhappy path con **objetos in-memory** (JSON string / buffer PNG de 1×1), no mutando `vectra-web/` — mismo patrón que `tokens_low_contrast_pair_fails`.

`check-vectra-web.sh` exige el archivo, los cuatro PNG, el `link rel="manifest"` y `theme-color`. Agrega `manifest.test.mjs` a `node --test`.

`manifest_does_not_register_service_worker` grep de `serviceWorker.register` en `index.html`/`app.js` y ausencia de `vectra-web/sw.js`.

Crate `api`: `manifest_replaces_version` en `static_files.rs` (fixture temporal con `{{VERSION}}` en el JSON).

### 4. `theme-color` en HTML además del *manifest*

El ticket pide colores `#0A0A0A`. El *manifest* cubre `theme_color` / `background_color`; el `<meta name="theme-color">` alinea la UI del navegador (barra) con el mismo valor. Un renglón en `index.html`.

### 5. Listo cuando sin Lighthouse CLI

El ticket dice «Lighthouse marca la app como instalable (junto con T-01-14)». Este change automatiza el subconjunto *manifest*; T-01-14 suma el SW. No se agrega lighthouse ni npm (igual que T-01-02 no metió axe-core).

## Risks / Trade-offs

- **[Sustitución en el Gateway]** → un `replace` extra; T-00-19 decía «plantilla solo en index.html». El delta `vortex-api` lo documenta; tests viejos sin placeholder no cambian.
- **[PNG en git]** → cuatro archivos chicos (~unos KB cada uno si el encoder es decente). Alternativa (generar en CI) añade flakiness visual innecesaria.
- **[macOS: `openspec new change T-01-13`]** → scaffold `t-01-13` y rename a `T-01-13` (igual que T-01-12).
- **[Lighthouse «instalable» rojo hasta T-01-14]** → esperado; el No hace es el SW.

## Migration Plan

N/A (SPA local + un handler existente). Rollback = revertir el change. No hay datos que migrar.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
