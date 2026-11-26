# Design

## Context

En `develop`, `api::build_router` sirve `{web}/index.html` (sustituye `{{VERSION}}`), `/static/<versión>/{*path}` → `{web}/{path}` y `/media`. No existe `vectra-web/` (T-00-19 lo dejó explícitamente fuera). ADR-06: frontend vanilla, ES Modules, sin *build*. `config.example.toml` pone `paths.web = "/opt/vectra-web"` (instalación; no se toca). Ver proposal.md para motivación y `specs/vectra-web/spec.md` para el contrato.

Constraints: ~2 h; sin estilos ni componentes; Node 22 disponible en local y en `macos-14` (T-09-03 ya usa `npx`); sin `package.json`; Playwright es e2e posterior.

## Goals / Non-Goals

**Goals:**

- Árbol `vectra-web/` cargable por el Gateway cuando `paths.web` apunta a ese directorio
- *Router* *hash* con *outlet* y vistas *placeholder* distinguibles
- Tests sin npm: harness de árbol + `node --test` con un `window`/`document` mínimo

**Non-Goals (diseño):**

- History API (`pushState`) o fallback `/*` → `index.html` en Axum
- jsdom, Playwright, Deno, o un job nuevo de GitHub Actions
- Cambiar `paths.web` del example a la ruta del clone
- Import maps, TypeScript o CSS *layers*

## Decisions

### 1. Hash router, no History API

**Elección:** `hashchange` + parseo de `location.hash`. Rutas: `#/`, `#/admin/<slug>`. El documento sigue siendo `GET /`; el Gateway no necesita SPA fallback (T-00-19).

**Por qué:** el ticket nombra rutas *hash*. Un *reload* de documento no ocurre al cambiar solo el fragmento. Encaja con caché inmutable de `/static/<versión>/…`.

**Alternativa:** `pushState` (`/admin/productos`). Se descartó: exigiría que Axum sirva `index.html` en paths desconocidos y choca con 404 `problem+json`. **Alternativa:** query `?view=admin`. Se descartó: no es el **Hace**.

### 2. Mapa URL de assets = contrato T-00-19

**Elección:**

```
vectra-web/index.html          GET /
vectra-web/js/app.js           GET /static/{{VERSION}}/js/app.js
vectra-web/js/router.js        GET /static/{{VERSION}}/js/router.js
vectra-web/css/                (vacío; T-01-02)
vectra-web/js/components/      (vacío; T-01-03+)
```

`index.html`: `<script type="module" src="/static/{{VERSION}}/js/app.js"></script>`. `app.js` hace `import { startRouter } from "./router.js"` (URL relativa al módulo, resuelve bajo el mismo prefijo `/static/<versión>/js/`).

**Por qué:** el *design* de T-00-19 ya documentó que T-01-01 debe usar `/static/{{VERSION}}/…`. Sin eso, un HTML cacheado pincharía JS nuevo.

**Alternativa:** `src="js/app.js"` relativo al documento. Se descartó: no pasa por el prefijo versionado; `Cache-Control: no-cache` del HTML no versiona el JS. **Alternativa:** copiar assets a `vectra-web/static/<versión>/`. Se descartó: no hay *build* (ADR-06).

### 3. API del router inyectable para tests sin jsdom

**Elección:** `js/router.js` exporta `parseHash(hash)` y `startRouter({ window, document, outlet })`. `parseHash` devuelve `{ view: "operator" | "admin" | "not-found", section?: string }`. `startRouter` pinta el *outlet* (`innerHTML` o `replaceChildren` de un *template* de string) y escucha `hashchange`. En el navegador, `app.js` llama `startRouter` con `window` / `document.getElementById("app")`. En tests, un stub:

- `location.hash` get/set
- `addEventListener("hashchange", …)` que el stub dispara al setear el hash
- `location.reload = () => { reloadCount++ }`
- `outlet` con `innerHTML` + `querySelector`

Node 22: `node --test scripts/fixtures/vectra-web/router.test.mjs` importa el módulo por `file:` URL. Sin jsdom.

**Por qué:** ADR-06 y el proposal prohiben `package.json`. El Listo cuando es «cambia la vista sin recargar»: asertar `data-view` y `reloadCount === 0` lo cubre. jsdom sería una dependencia.

**Alternativa:** Playwright. Se descartó: e2e (T-09), ~2 h, `package.json`. **Alternativa:** tests solo de `parseHash` sin DOM. Se descartó: no prueba el «sin recargar». **Alternativa:** Deno. Se descartó: no está en CI ni en el stack.

### 4. Slug de sección genérico; menú en T-01-11

**Elección:** `#/admin/<sección>` acepta cualquier slug `[a-z0-9-]+` (no una allowlist de productos/categorías/…). La vista admin es un *placeholder* con `data-section`. T-01-11 agrega el menú y el contenido. *Unhappy:* sin sección, segmento extra, mayúsculas, caracteres fuera del slug → `not-found`.

**Por qué:** el ticket pide el patrón `#/admin/<sección>` y el Listo cuando nombra `productos`. Fijar el menú completo aquí pisa T-01-11. Una allowlist ahora obligaría a tocar el *router* en cada ABM.

**Alternativa:** allowlist (productos, categorias, …). Se descartó: alcance de T-01-11 y slugs en español vs inglés (`categorías`). **Alternativa:** `#/admin` sin sección = productos. Se descartó: el Listo cuando usa `#/admin/productos`; silenciar el unhappy path.

### 5. Placeholders semánticos, no CSS

**Elección:** cada vista es un `<main data-view="…">` (admin suma `data-section`) con un `<h1>` en español y los `<a href>` de la spec. `css/` y `js/components/` se versionan con `.gitkeep`. `index.html` no enlaza hojas de estilo.

**Por qué:** **No hace** estilos ni componentes. `data-view` es el oráculo de los tests. Los `<a>` cumplen navegación por teclado mínima del DoD de UI sin *tokens* (44 px / contraste → T-01-02).

**Alternativa:** `hidden` + tres bloques en el HTML. Equivale; se prefiere pintar desde el *router* para que una ruta desconocida no deje HTML de operador en el DOM. **Alternativa:** Web Component `<vx-app>`. Se descartó: T-01-03.

### 6. Harness `check-vectra-web.sh`, sin job de CI en este ticket

**Elección:** `scripts/check-vectra-web.sh` falla si falta el árbol / `lang="es"` / el `src` versionado, y después corre `node --test` sobre `scripts/fixtures/vectra-web/`. No se modifica `.github/workflows/ci.yml` ni `check-ci.sh`.

**Por qué:** agregar un job implica fixtures de `ci-base` y ~el mismo tamaño que T-00-02; el ticket es el esqueleto SPA (~2 h). Node está en el runner: un ticket de CI posterior puede una línea `run: ./scripts/check-vectra-web.sh`.

**Alternativa:** job `vectra-web` ahora. Se descartó: mezcla `ci-base` / oráculos de `check-ci.sh`. **Alternativa:** solo inspección grep del HTML. Se descartó: no cubre el Listo cuando (sin recarga).

## Risks / Trade-offs

- **[Tests sin navegador real]** → El stub no ejecuta el parser HTML del motor. Mitigación: el harness lee `index.html` de verdad; el *router* se importa como el navegador lo hará. Playwright llega en e2e.
- **[ESM en Node vs browser]** → `import` relativo funciona en ambos si el test usa URL `file:`. Mitigación: no usar `import.meta.resolve` ni `node:` dentro de `vectra-web/js/`.
- **[paths.web del example sigue en `/opt/vectra-web`]** → Un `vortex serve` con el TOML de ejemplo no sirve el clone. Mitigación: fuera de alcance (T-00-05); en apply se documenta en un comentario de `index.html` o no (evitar docs no pedidos). El test de binario de T-00-19 ya usa un dir temporal.
- **[innerHTML en placeholders]** → XSS futuro si se interpola búsqueda. Mitigación: strings fijas en este ticket; T-03-16 cubre resaltado seguro.
- **[Hash con `location.hash` incluye `#`]** → `parseHash` acepta con o sin `#` y normaliza. Tests cubren vacío, `#` y `#/`.

## Migration Plan

1. Red: harness y `node --test` no existen o fallan; no hay `vectra-web/`.
2. Green: árbol + *router* + `app.js`; `check-vectra-web.sh` sale 0.
3. Cierre: § 5 en `vortex-core/` (regresión); `openspec validate T-01-01`.
4. Rollback: borrar `vectra-web/` y el harness. Sin datos de usuario ni contrato OpenAPI. El Gateway sigue sirviendo 404 si `{web}` está vacío.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. Los slugs de menú (categorías vs `categorias`) los define T-01-11.
