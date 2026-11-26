# Proposal

## Ticket padre

- **Ticket:** [T-01-05](../../../docs/7-tickets.md) — Componente `vx-product-card`
- **Change:** `T-01-05`
- **HU:** [HU-01-02](../../../docs/6-historias.md#hu-01-02--design-system-oscuro-glassmorphism) (catálogo); contrato de campos alineado a [HU-03-05](../../../docs/6-historias.md#hu-03-05--tarjetas-de-resultado-con-stock-precio-y-ubicación)
- **Depende de:** T-01-02 (en `develop`; T-01-03/T-01-04 ya aportaron `shared.js`, Shadow DOM, `adoptedStyleSheets` y `dev/components.html`)
- **Hace:** tarjeta con nombre, SKU, marca, precio, stock, ubicación, imagen y *slot* para el nombre resaltado; estado visual "sin stock"; variantes compacta (recomendaciones) y completa (resultados).
- **No hace:** sanitización del resaltado (T-03-16).
- **Listo cuando:** la página de muestra renderiza ambas variantes con datos de ejemplo.

## Why

HU-01-02 pide la tarjeta de producto en el catálogo de Web Components. T-03-17 (resultados), T-04-08 (recomendaciones compactas) y T-01-10 (lista por teclado) necesitan un primitive presentacional con los campos de `SearchHit` — nombre, SKU, marca, precio, stock, ubicación, imagen y estado sin stock — antes de cablear búsqueda o el panel lateral.

## What Changes

- `vx-product-card.js`: custom element con Shadow DOM + `adoptedStyleSheets`; atributos alineados a `SearchHit` (`sku`, `name`, `brand`, `price`, `stock`, `location`, `image-url`); variantes `full` (default, resultados) y `compact` (recomendaciones); estado visual sin stock; *slot* `name` para el resaltado de T-03-16
- Extender `index.js` y `dev/components.html` con ambas variantes y datos de ejemplo (criterio **Listo cuando**)
- TDD: happy = campos de la variante completa, compacta oculta ubicación, *slot* de nombre, marcador sin stock, placeholder sin imagen; unhappy = HTML en el atributo `name` no se interpreta, variante desconocida cae a `full`, falta el módulo, stock 0 sin marcador visual
- Extender `scripts/check-vectra-web.sh` y `scripts/fixtures/vectra-web/components.test.mjs`

## Non-goals

- Sanitización del resaltado (`highlight.js`, solo `<mark>`) — **T-03-16**
- Pintar resultados reales de `/search` — **T-03-17**
- Navegación por teclado de listas (`aria-activedescendant`) — **T-01-10**
- Panel de recomendaciones (Similares / Sustitutos / Se usa junto con) — **T-04-08**
- Subida o almacenamiento de fotos de producto — **T-02-28** … **T-02-39** (HU-02-09, v2)
- Omnibox, `api.js`, mensajes por `ErrorCode`, vista admin, PWA, Playwright, jsdom, `package.json`, CI job nuevo, OpenAPI, crates Rust

## Capabilities

### New Capabilities

- Ninguna. La tarjeta es el mismo design system `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `vx-product-card` (variantes `full`/`compact`, *slot* de nombre, estado sin stock); ampliar el catálogo `dev/components.html` y la suite de componentes con happy/unhappy de campos, variante inválida e HTML en el nombre

## Impact

- **Repo:** `vectra-web/js/components/{vx-product-card,index}.js`, `vectra-web/dev/components.html`, `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/components.test.mjs`
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-05`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
