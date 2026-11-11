# Proposal

## Ticket padre

- **Ticket:** [T-01-08](../../../docs/7-tickets.md) — Vista del operador y `vx-omnibox`
- **Change:** `T-01-08`
- **HU:** [HU-01-03](../../../docs/6-historias.md#hu-01-03--componente-omnibox)
- **Depende de:** T-01-03 (cerrado en `develop`; kit de Web Components). En `develop` también están T-01-04 (`vx-side-panel` / `vx-closed`) y el *router* de T-01-01.
- **Hace:** layout del operador (Omnibox arriba, área de resultados, lugar para el panel lateral); `vx-omnibox` con foco al cargar y al recibir `vx-closed`; Escape limpia la consulta.
- **No hace:** modos (T-01-09) ni búsqueda real (T-03-15).
- **Listo cuando:** al cargar y al cerrar un panel el foco queda en el Omnibox.

## Why

HU-01-03 pide una única barra de búsqueda siempre visible y con foco, para que el operador empiece a escribir apenas el cliente pide algo. Hoy `#/` pinta un `<h1>` de placeholder: no hay Omnibox, no hay regiones de resultados ni panel, y el foco no vuelve al mostrador al cerrar un `vx-side-panel`. T-01-09, T-01-10 y T-03-15 necesitan este *shell* antes de modos, teclado de listas y `/search`.

## What Changes

- `vectra-web/js/components/vx-omnibox.js`: custom element de la barra (input de texto, Shadow DOM, `adoptedStyleSheets`, `--vx-touch-min`); foco al conectar y al recibir `vx-closed`; Escape vacía la consulta
- Vista operador del *router*: Omnibox arriba, `data-region="results"` y `data-region="side-panel"` con un `vx-side-panel` cerrado (lugar para T-04-08); se conserva el enlace a `#/admin/productos`
- Estilos de layout operador en `base.css` (sin tokens nuevos)
- `index.js` registra `vx-omnibox`; `dev/components.html` lo muestra
- TDD: happy = layout, foco al montar, foco tras `vx-closed` / cierre de panel, Escape limpia; unhappy = Escape con consulta vacía, `vx-closed` con Omnibox desconectado, falta el módulo, Escape con panel abierto no vacía la consulta
- Extender `scripts/check-vectra-web.sh` y `scripts/fixtures/vectra-web/` (`omnibox.test.mjs`; *stub* DOM si hace falta para `document` / burbujeo)

## Non-goals

- Selector de modos texto/voz/IA, prefijo `?`, evento `vx-query { mode, text }` — **T-01-09**
- Llamadas a `/search`, *search-as-you-type*, tarjetas de resultado reales — **T-03-15** / **T-03-17**
- Navegación por flechas / Enter / `aria-activedescendant` — **T-01-10**
- Layout y menú de administración — **T-01-11**
- Polling `/health` y deshabilitar modos — **T-01-12**
- Contenido de recomendaciones en el panel — **T-04-08**
- Playwright, jsdom, `package.json`, job nuevo de CI, OpenAPI, crates Rust

## Capabilities

### New Capabilities

- Ninguna. El Omnibox y el layout del operador son parte de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `vx-omnibox` y el layout de la vista operador (`#/`); ampliar el árbol exigido, el catálogo y la suite con foco al cargar / `vx-closed` y Escape (happy + unhappy)

## Impact

- **Repo:** `vectra-web/js/components/{vx-omnibox,index}.js`, `vectra-web/js/router.js`, `vectra-web/css/base.css`, `vectra-web/dev/components.html`, `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/` (`omnibox.test.mjs`, posiblemente `dom-stub.mjs` y `router.test.mjs`)
- **APIs / OpenAPI:** sin paths nuevos; MUST NOT llamar a `/search` ni a otros endpoints
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-08`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-08 (Listo cuando: al cargar y al cerrar un panel el foco queda en el Omnibox)
