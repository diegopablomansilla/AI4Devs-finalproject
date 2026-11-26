# Proposal

## Ticket padre

- **Ticket:** [T-01-04](../../../docs/7-tickets.md) — Componentes `vx-modal`, `vx-side-panel`, `vx-toast`
- **Change:** `T-01-04`
- **HU:** [HU-01-02](../../../docs/6-historias.md#hu-01-02--design-system-oscuro-glassmorphism)
- **Depende de:** T-01-02 (en `develop`; T-01-03 ya aportó `shared.js`, Shadow DOM y `dev/components.html`)
- **Hace:** modal y panel lateral con trampa de foco, cierre con Escape y evento `vx-closed`; `vx-toast` con variantes información/error y `aria-live`.
- **No hace:** lógica de negocio.
- **Listo cuando:** en la página de muestra, al cerrar un panel el foco vuelve al elemento que lo abrió.

## Why

HU-01-02 pide panel lateral, modal y aviso en el catálogo de Web Components. T-01-07 (`showError` → `vx-toast`), T-01-08 (hueco del panel en el operador) y T-04-08 (recomendaciones en `vx-side-panel`) necesitan superposiciones accesibles — trampa de foco, Escape, `vx-closed` y restauración de foco — antes de montar flujos de negocio.

## What Changes

- `vx-modal.js` y `vx-side-panel.js`: Shadow DOM + `adoptedStyleSheets`; abiertos con trampa de foco; Escape y control de cierre emiten `vx-closed`; al cerrar, el foco vuelve al elemento que los abrió
- `vx-toast.js`: variantes `info` / `error` con `aria-live` (polite / assertive)
- Extender `index.js`, `dev/components.html` y el stub DOM para Tab/Escape/`document.activeElement`
- TDD: happy = trap, Escape, `vx-closed`, restore de foco, `aria-live`; unhappy = overlay ya cerrado, Tab que escapa, toast sin `aria-live`, variante desconocida, módulo ausente
- Extender `scripts/check-vectra-web.sh` y `scripts/fixtures/vectra-web/components.test.mjs`

## Non-goals

- Lógica de negocio, mensajes por `ErrorCode`, `showError` — **T-01-07**
- `vx-product-card` — **T-01-05**
- Omnibox, layout operador (hueco del panel, foco al Omnibox al cerrar) — **T-01-08**
- Panel de recomendaciones (Similares / Sustitutos / Se usa junto con) — **T-04-08**
- Vista admin, `api.js`, PWA, Playwright, jsdom, `package.json`, CI job nuevo, OpenAPI, crates Rust

## Capabilities

### New Capabilities

- Ninguna. Los overlays son el mismo design system `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `vx-modal`, `vx-side-panel` y `vx-toast`; ampliar el catálogo `dev/components.html` y la suite de componentes con happy/unhappy de foco y `aria-live`

## Impact

- **Repo:** `vectra-web/js/components/{overlay,vx-modal,vx-side-panel,vx-toast,index}.js`, `vectra-web/dev/components.html`, `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/`
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-04`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
