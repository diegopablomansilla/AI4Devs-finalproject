# Proposal

## Ticket padre

- **Ticket:** [T-01-02](../../../docs/7-tickets.md) — *Design tokens* oscuros Glassmorphism
- **Change:** `T-01-02`
- **HU:** [HU-01-02](../../../docs/6-historias.md#hu-01-02--design-system-oscuro-glassmorphism)
- **Depende de:** T-01-01 (cerrado en `develop`; existe `vectra-web/` con *router* *hash* y `css/` vacío)
- **Hace:** `css/tokens.css` (fondo `#0A0A0A`, superficies translúcidas con `backdrop-filter`, colores de texto, acento, error, éxito, anillo de foco, espaciados, radios, tamaño táctil mínimo de 44 px) y `css/base.css`; alternativa sólida con `prefers-reduced-transparency`
- **No hace:** componentes
- **Listo cuando:** todos los pares texto/superficie de los *tokens* dan contraste ≥ 4,5:1 medido con axe DevTools

## Why

El esqueleto de Vectra ya navega entre operador y administración, pero `css/` está vacío: no hay tema oscuro, ni *Glassmorphism*, ni contrato de contraste o de 44 px. HU-01-02 y el DoD de UI ([6.1](../../../docs/6-historias.md)) exigen WCAG AA sobre `#0A0A0A` y objetivos táctiles. T-01-03…T-01-05 necesitan variables CSS compartidas (y más adelante `adoptedStyleSheets`) antes de construir Web Components.

## What Changes

- Agregar `vectra-web/css/tokens.css` con custom properties `--vx-*`: fondo `#0A0A0A`, superficie translúcida + blur, texto, acento, error, éxito, anillo de foco, escala de espaciado y radios, `--vx-touch-min: 44px`
- Agregar `vectra-web/css/base.css` que aplica los *tokens* al documento (fondo, color de texto, foco visible, mínimo táctil en controles nativos) y la clase de superficie `.vx-surface` con `backdrop-filter`
- En `@media (prefers-reduced-transparency: reduce)`, las superficies pasan a color sólido y **sin** `backdrop-filter`
- `index.html` enlaza ambas hojas vía `/static/{{VERSION}}/css/…` (mismo mapa URL de T-00-19 / T-01-01); sin *bundler* ni `package.json`
- TDD: happy = *tokens* presentes, pares texto/superficie ≥ 4,5:1 (WCAG 2.1, superficie *glass* compositeda sobre el fondo), *media query* de transparencia reducida; unhappy = archivo o *token* ausente, fondo distinto de `#0A0A0A`, `--vx-touch-min` menor que 44 px, contraste insuficiente, *glass* sin fallback sólido
- Extender `scripts/check-vectra-web.sh` con tests `node --test` de *tokens* (sin npm, sin axe en CI; axe DevTools queda como verificación humana del Listo cuando sobre la SPA ya estilizada)

## Non-goals

Este change **no** cubre:

- Web Components (`vx-button`, `vx-field`, `vx-chip`, modal, panel, toast, `vx-product-card`) — **No hace** del ticket; **T-01-03**, **T-01-04**, **T-01-05**
- Página de muestra `vectra-web/dev/components.html` — **T-01-03** (Listo cuando de ese ticket)
- Omnibox, vista admin con menú, `api.js`, errores por `code` — **T-01-06** … **T-01-11**
- *Manifest* PWA, Service Worker, aviso sin conexión — **T-01-13**, **T-01-14**, **T-01-15**
- Tema claro, *dark/light* toggle o `prefers-color-scheme` (el producto es oscuro nativo)
- `adoptedStyleSheets` en Shadow DOM (lo usan los componentes en T-01-03; aquí solo hojas globales)
- Playwright, axe-core npm, jsdom o `package.json`
- Job nuevo en `.github/workflows/ci.yml` (el harness local se engancha cuando un ticket de CI lo pida)
- Cambiar `docs/openapi.yaml`, crates Rust, *router* *hash* o el Gateway de estáticos

## Capabilities

### New Capabilities

- (ninguna — los *tokens* viven en la SPA ya especificada)

### Modified Capabilities

- `vectra-web`: el árbol `css/` deja de estar vacío; `index.html` carga *design tokens* y base; el tema oscuro Glassmorphism, el contraste AA de los pares texto/superficie y el fallback `prefers-reduced-transparency` pasan a ser requisitos; la suite de `check-vectra-web.sh` incorpora tests de *tokens* (happy + unhappy)

## Impact

- **Repo:** `vectra-web/css/tokens.css`, `vectra-web/css/base.css`, `vectra-web/index.html` (`<link>` versionados), `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/` (tests de contraste / *tokens*)
- **APIs / OpenAPI:** sin cambios; la SPA sigue fuera del contrato ([5.12](../../../docs/5-contract-api.md#512-fuera-de-este-contrato))
- **CI / verificación:** `./scripts/check-vectra-web.sh` (árbol + *router* de T-01-01 + *tokens*); [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` como regresión (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `openspec validate T-01-02` (o `t-01-02` si el CLI exige minúsculas)
- **Ticket:** cierra T-01-02 (Listo cuando: pares texto/superficie ≥ 4,5:1; axe DevTools en la SPA servida)
- **BREAKING:** no (hojas nuevas; el *router* no cambia de contrato)
