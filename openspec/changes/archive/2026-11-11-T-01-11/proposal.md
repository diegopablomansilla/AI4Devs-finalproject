# Proposal

## Ticket padre

- **Ticket:** [T-01-11](../../../docs/7-tickets.md) — Vista de administración y menú
- **Change:** `T-01-11`
- **HU:** [HU-01-04](../../../docs/6-historias.md#hu-01-04--navegación-operador--administrador)
- **Depende de:** T-01-01 (cerrado; *router* hash `#/` ↔ `#/admin/<sección>` sin recarga) y T-01-03 (cerrado; `vx-field` / `vx-button`)
- **Hace:** layout admin con menú (productos, categorías, marcas, atributos, stock, relaciones, sinónimos, notas técnicas), enlace para volver al operador y ajuste "Nombre de esta terminal"; sin credenciales.
- **No hace:** contenido de cada sección.
- **Listo cuando:** se entra y se vuelve a la vista del operador en un paso.

## Why

HU-01-04 pide una sección de administración separada del mostrador, con menú de catálogo y vuelta al operador en un paso, sin credenciales. T-01-01 dejó un *placeholder* (`<h1>Administración: productos</h1>` + enlace). T-02-07 (categorías), T-02-10 (marcas), T-02-13 (atributos) y T-02-34 (productos) no pueden colgarse de un shell vacío. El ajuste de terminal cierra el circuito de T-01-06 (`localStorage["vectra.terminal"]` → `X-Vectra-Terminal`).

## What Changes

- Layout de `data-view="admin"`: encabezado (volver al operador), navegación de las 8 secciones del ticket, región de contenido **vacía** (`data-region="admin-content"`) y ajuste "Nombre de esta terminal".
- Menú con slugs ASCII: `productos`, `categorias`, `marcas`, `atributos`, `stock`, `relaciones`, `sinonimos`, `notas-tecnicas`. Cada ítem es un `<a href="#/admin/<slug>">` en español; la sección activa se marca (`aria-current="page"`).
- Enlace `<a href="#/">` (texto en español) para volver al operador **en un paso** (Listo cuando). El enlace del operador a `#/admin/productos` se conserva.
- Ajuste de terminal: `vx-field` "Nombre de esta terminal" persiste en `localStorage` clave `vectra.terminal` (misma que T-01-06). Máximo 64 caracteres ([OpenAPI `X-Vectra-Terminal`](../../../docs/openapi.yaml)). Vacío borra la clave. Sin credenciales.
- TDD en `router.test.mjs` (y fixture de terminal si hace falta): happy = menú, navegación de sección, vuelta al operador, persistencia del nombre; unhappy = sin login, nombre > 64, storage que falla, sección no listada, `admin-content` vacío (sin ABM).
- `base.css`: clases de layout admin con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni Omnibox en admin.

## Non-goals

- Contenido / ABM de cada sección — **No hace** del ticket; **T-02-07**, **T-02-10**, **T-02-13**, **T-02-34**, **T-02-38**, **T-04-09**, **T-03-28** (v2), **T-05-28** (v2)
- Estado del sistema / `/health` — **T-01-12**
- Autenticación, usuarios o roles (MVP: red local de confianza; [1.4](../../../docs/1-descripcion-general-del-producto.md#14-usuarios-y-contexto-de-uso))
- Layout operador, Omnibox, keyboard-list, resultados reales — **T-01-08**, **T-01-09**, **T-01-10**, **T-03-15**
- *Manifest* PWA / Service Worker — **T-01-13**, **T-01-14**
- Playwright / e2e — **T-09-10**
- jsdom, `package.json`, job nuevo de CI, OpenAPI, crates Rust
- Truncar el nombre en `api.js` (T-01-06 ya lee la clave; este ticket valida al guardar)

## Capabilities

### New Capabilities

- Ninguna. El layout admin es parte de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: reemplazar el placeholder de administración por shell con menú de 8 secciones, vuelta al operador y ajuste de nombre de terminal; ampliar la suite Node (happy + unhappy). El contenido de `data-region="admin-content"` MUST quedar vacío (sin listas ni formularios de negocio).

## Impact

- **Repo:** `vectra-web/js/router.js` (y módulo auxiliar si el design lo justifica); `vectra-web/css/base.css`; `scripts/fixtures/vectra-web/router.test.mjs` (y/o `admin.test.mjs`); `scripts/check-vectra-web.sh` (comentario / lista de tests)
- **APIs / OpenAPI:** sin paths nuevos; MUST NOT llamar endpoints. El nombre de terminal MUST respetar `maxLength: 64` de `X-Vectra-Terminal`
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-11`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-11 (Listo cuando: se entra y se vuelve a la vista del operador en un paso)
- **Alcance:** cabe en ~2 h (layout + menú + un campo + tests Node); no se divide
