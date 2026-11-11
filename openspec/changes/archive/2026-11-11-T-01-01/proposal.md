# Proposal

## Ticket padre

- **Ticket:** [T-01-01](../../../docs/7-tickets.md) — Esqueleto de `vectra-web` y *router*
- **Change:** `T-01-01`
- **HU:** [HU-01-04](../../../docs/6-historias.md#hu-01-04--navegación-operador--administrador)
- **Depende de:** T-00-19 (cerrado en `develop`; el Gateway ya sirve `index.html` con `{{VERSION}}`, `/static/<versión>/…` y `/media`)
- **Hace:** `vectra-web/index.html` (`lang="es"`), `css/`, `js/app.js` (ES Modules, sin *build*), `js/router.js` con rutas *hash* (`#/` operador, `#/admin/<sección>`) y carpeta `js/components/`
- **No hace:** estilos ni componentes
- **Listo cuando:** navegar entre `#/` y `#/admin/productos` cambia la vista sin recargar

## Why

El Gateway ya entrega estáticos desde `paths.web` (T-00-19), pero el repositorio no tiene `vectra-web/`: no hay documento HTML, ni ES Modules, ni *router*. HU-01-04 pide separar Operador y Administrador por navegación (sin credenciales). T-01-02 (*tokens*), T-01-06 (`api.js`), T-01-11 (menú admin) y T-01-13 (*manifest*) necesitan este esqueleto y las rutas *hash* para colgarse encima.

## What Changes

- Crear `vectra-web/` en la raíz del repo según [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio): `index.html` con `lang="es"`, carpetas `css/` y `js/components/`, `js/app.js` y `js/router.js`
- `index.html` carga `js/app.js` como ES Module vía `/static/{{VERSION}}/js/app.js` (mapa URL → disco de T-00-19); sin *bundler* ni `package.json`
- *Router* *hash*: `#/` renderiza la vista operador; `#/admin/<sección>` (al menos `productos`) la vista administración; el cambio de *hash* actualiza el *outlet* **sin** recargar el documento
- Vistas *placeholder* (HTML semántico, sin *design tokens* ni Web Components): distinguibles por `data-view` y un enlace `<a>` para ir y volver
- TDD: happy = `#/` ↔ `#/admin/productos` cambia la vista y no llama `location.reload`; unhappy = *hash* desconocido o mal formado muestra vista «no encontrada» sin recargar
- Harness `scripts/check-vectra-web.sh` (+ tests `node --test` sin dependencias npm) que certifica el árbol, `lang="es"` y el *router*

## Non-goals

Este change **no** cubre:

- *Design tokens*, `css/tokens.css` / `css/base.css`, contraste, 44 px — **No hace** del ticket; **T-01-02**
- Web Components (`vx-button`, `vx-field`, `vx-omnibox`, `vx-product-card`, modal/panel/toast) — **T-01-03** … **T-01-05**, **T-01-08**
- Cliente HTTP `api.js` y mensajes de error — **T-01-06**, **T-01-07**
- Layout de administración, menú de secciones y «Nombre de esta terminal» — **T-01-11**
- *Manifest* PWA, Service Worker, aviso sin conexión — **T-01-13**, **T-01-14**, **T-01-15**
- Autenticación / roles (el MVP no pide credenciales; [1.4](../../../docs/1-descripcion-general-del-producto.md#14-usuarios-y-contexto-de-uso))
- Fallback SPA `/*` → `index.html` en el Gateway (el *router* es *hash*; T-00-19 ya lo dejó fuera)
- Playwright / suite e2e — **T-09-10** (aprox.); este ticket no agrega `package.json`
- Job nuevo en `.github/workflows/ci.yml` ni cambios a `ci-base` / `check-ci.sh` (el harness corre en local; Node ya está en el runner, se engancha cuando un ticket de CI lo pida)
- Cambiar `docs/openapi.yaml`, crates Rust, `paths.web` o handlers de estáticos
- `sw.js` de producción (T-01-14); el Gateway ya sabe servirlo cuando exista

## Capabilities

### New Capabilities

- `vectra-web`: esqueleto de la SPA vanilla (documento `lang="es"`, ES Modules sin *build*, *router* *hash* Operador/Administrador, *outlet* sin recarga, vista no encontrada) y su harness de tests

### Modified Capabilities

- (ninguna — el Gateway de `vortex-api` ya sirve `{web}`; este change solo crea el contenido de `vectra-web/`)

## Impact

- **Repo:** `vectra-web/` (`index.html`, `css/`, `js/app.js`, `js/router.js`, `js/components/`), `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/`
- **APIs / OpenAPI:** sin paths nuevos; la SPA sigue fuera del contrato ([5.12](../../../docs/5-contract-api.md#512-fuera-de-este-contrato))
- **CI / verificación:** `./scripts/check-vectra-web.sh` (estructura + `node --test` del *router*); [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` como regresión (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `openspec validate T-01-01` (o `t-01-01` si el CLI exige minúsculas)
- **Ticket:** cierra T-01-01 (Listo cuando: navegar `#/` ↔ `#/admin/productos` cambia la vista sin recargar)
- **BREAKING:** no (árbol nuevo; el Gateway no cambia)
