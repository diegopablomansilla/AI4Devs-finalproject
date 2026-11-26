# Proposal

## Ticket padre

- **Ticket:** [T-01-03](../../../docs/7-tickets.md) — Componentes `vx-button`, `vx-field`, `vx-chip`
- **Change:** `T-01-03`
- **HU:** [HU-01-02](../../../docs/6-historias.md#hu-01-02--design-system-oscuro-glassmorphism)
- **Depende de:** T-01-02 (cerrado en `develop`; *tokens* y `base.css`)
- **Hace:** Web Components con Shadow DOM y hojas compartidas (`adoptedStyleSheets`); `vx-field` con etiqueta, ayuda y mensaje de error por campo; roles ARIA y estados deshabilitado/cargando.
- **No hace:** componentes de superposición (T-01-04).
- **Listo cuando:** página de muestra `vectra-web/dev/components.html` operable por teclado y con objetivos de 44 px.

## Why

HU-01-02 pide un catálogo de Web Components reutilizables (botón, campo, chip). T-01-02 dejó el contrato `--vx-*` en hojas globales; T-01-08 (Omnibox), T-01-11 (admin) y T-01-07 (`vx-field` + errores por campo) necesitan controles con Shadow DOM, accesibilidad y 44 px antes de montar vistas.

## What Changes

- `vectra-web/js/components/shared.js`: carga `tokens.css` + `base.css` en `CSSStyleSheet` compartidas vía `fetch` + `adoptedStyleSheets`
- `vx-button.js`, `vx-field.js`, `vx-chip.js` + `index.js` que registra los tres custom elements
- `vectra-web/dev/components.html`: catálogo con ejemplos (variantes, error, loading, chip seleccionable)
- `app.js` importa `components/index.js` para registro en la SPA
- TDD: happy = shadow + adopted styles, ARIA, eventos, `--vx-touch-min`; unhappy = módulo ausente, loading sin `aria-busy`, error sin `aria-invalid`, chip disabled que togglea
- Extender `scripts/check-vectra-web.sh` y `scripts/fixtures/vectra-web/components.test.mjs`

## Non-goals

- `vx-modal`, `vx-side-panel`, `vx-toast` — **T-01-04**
- `vx-product-card` — **T-01-05**
- Omnibox, layout operador/admin — **T-01-08**, **T-01-11**
- `api.js`, `errors.js` — **T-01-06**, **T-01-07**
- Playwright, jsdom, `package.json`, CI job nuevo, OpenAPI, crates Rust

## Capabilities

### Modified Capabilities

- `vectra-web`: primeros Web Components del design system, página `dev/components.html`, suite de componentes en `check-vectra-web.sh`

## Impact

- **Repo:** `vectra-web/js/components/*`, `vectra-web/dev/components.html`, `vectra-web/js/app.js`, `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/`
- **Verificación:** `./scripts/check-vectra-web.sh`; regresión `cargo test --workspace` en `vortex-core/`
