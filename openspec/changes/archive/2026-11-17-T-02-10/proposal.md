# Proposal

## Ticket padre

- **Ticket:** [T-02-10](../../../docs/7-tickets.md) — UI de marcas
- **Change:** `T-02-10`
- **HU:** [HU-02-02](../../../docs/6-historias.md#hu-02-02--marcas)
- **Depende de:** T-02-09 (endpoints `/api/v1/brands`, en `develop`); T-01-11 (shell admin con menú `#/admin/marcas` y `data-region="admin-content"`, en `develop`)
- **Hace:** listado, alta, edición y baja/reactivación.
- **No hace:** logos de marca.
- **Listo cuando:** escenarios de HU-02-02 ejecutados a mano.

## Why

HU-02-02 pide que el administrador cree "Fixser", la edite y la dé de baja lógica sin perder referencias. T-02-09 ya expone `GET`/`POST /api/v1/brands` y `PATCH`/`DELETE /api/v1/brands/{brandId}`; T-01-11 dejó `#/admin/marcas` con un título placeholder. Sin este change no hay ABM en la SPA: el Listo cuando (alta y baja lógica) no se puede ejecutar a mano.

## What Changes

- Pintar el listado plano de marcas en `data-region="admin-content"` cuando el hash es `#/admin/marcas`, hidratado desde `{ items }` de `GET /api/v1/brands?includeInactive=true`.
- Acciones: alta (`POST { name }`), edición de nombre (`PATCH { name }`), baja (`DELETE`) y reactivación (`PATCH { active: true }`). Listado vacío ofrece crear.
- Errores de API (`duplicate_slug`, `invalid_input`, `not_found`, `unavailable`, `network`) mostrados por `code` vía `showError` / `messageFor` (T-01-07); sin interpretar `title`.
- TDD Node (`node --test`, sin jsdom ni npm): happy = listado, alta "Fixser", rename, baja/reactivación; unhappy = slug duplicado, name vacío, red, sin logos, secciones admin que no son categorías ni marcas siguen sin ABM.
- `base.css`: clases del listado con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni UI de logos.

## Non-goals

- Logos de marca (campo, archivo, `<input type="file">`, `<img>` de marca) — **No hace** del ticket
- Campo `slug` editable; filtro `q` del contrato — el servidor genera el slug desde `name` (T-02-09); el listado admin usa `includeInactive`, no el prefijo `q`
- Conservación de referencias en `producto` — **T-02-14** / **T-02-18** (la UI no consulta occupancy; T-02-09 ya permite la baja)
- Reflejo en búsqueda / worker *fan-out* — **T-03-08**
- ABM de atributos, productos, stock — **T-02-13**, **T-02-34**, **T-02-38**
- Seed de marcas — **T-02-42**
- Endpoints nuevos o cambios a `docs/openapi.yaml` — contrato cerrado en **T-02-09**
- Playwright / e2e — **T-09-10** / **T-09-11**
- jsdom, `package.json`, dependencias npm, job nuevo de CI, crates Rust
- Autenticación o roles (MVP: red local de confianza)
- Extraer un *framework* de vistas admin (T-02-13 reutilizará el patrón)

## Capabilities

### New Capabilities

- Ninguna. La UI de marcas es contenido de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: montar el ABM de marcas en `#/admin/marcas` (listado plano, alta, edición, baja/reactivación, errores por `code`, sin logos); exceptuar esa sección del «admin-content vacío» de T-01-11 (junto a `categorias`); ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/admin/brands.js` (módulo nuevo); `vectra-web/js/router.js` y `vectra-web/js/app.js` (montar la sección); `vectra-web/css/base.css`; `scripts/fixtures/vectra-web/brands.test.mjs`; `scripts/check-vectra-web.sh` (incluir el test); ajuste de `router.test.mjs` para que el ABM vacío siga valiendo en secciones que no son `categorias` ni `marcas`
- **APIs / OpenAPI:** consumo de `/api/v1/brands` según [docs/openapi.yaml](../../../docs/openapi.yaml); MUST NOT agregar paths ni DTOs
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-02-10`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando del ticket: ejecución manual de HU-02-02 (alta "Fixser" y baja lógica; las referencias en producto se validan cuando exista T-02-14)
- **Ticket:** cierra T-02-10
- **Alcance:** cabe en ~2 h (espejo plano de T-02-07 sin árbol ni mover); no se divide
