# Proposal

## Ticket padre

- **Ticket:** [T-01-10](../../../docs/7-tickets.md) — Navegación por teclado de listas
- **Change:** `T-01-10`
- **HU:** [HU-01-03](../../../docs/6-historias.md#hu-01-03--componente-omnibox) (escenario Operación por teclado)
- **Depende de:** T-01-08 (cerrado en `develop`; layout operador + `vx-omnibox` con Escape y foco) y T-01-05 (cerrado; `vx-product-card`)
- **Hace:** `js/keyboard-list.js` con flechas, Enter (abrir) y Escape (volver al Omnibox) usando `aria-activedescendant`; reutilizable en resultados y sugerencias.
- **No hace:** datos reales.
- **Listo cuando:** con resultados de ejemplo se recorre, abre y cierra sin mouse.

## Why

HU-01-03 pide recorrer resultados con flechas, abrir con Enter y volver/limpiar con Escape. Hoy el Omnibox y las tarjetas existen, pero no hay un helper de lista: T-03-15 (resultados) y T-03-21 (sugerencias, v2) no pueden compartir `aria-activedescendant` si cada uno inventa el teclado. Este ticket deja el primitive **antes** de cablear `/search`.

## What Changes

- Módulo ES `vectra-web/js/keyboard-list.js` (`bindKeyboardList`): lista `role="listbox"` + opciones `role="option"`, `aria-activedescendant`, flechas (ciclo), Enter → `onOpen`, Escape → foco al Omnibox. Reutilizable: `getItems()` no asume `vx-product-card`.
- Teclas también desde el input del Omnibox (flechas/Enter) **sin** capturar Escape ahí (T-01-08 sigue vaciando la consulta).
- Catálogo `dev/components.html`: lista de ejemplo (`vx-product-card`) + Omnibox; Enter abre el panel de muestra; Escape en la lista vuelve al Omnibox. Listo cuando humano: recorrer, abrir y cerrar sin mouse.
- TDD en `keyboard-list.test.mjs`: happy = recorrido, `aria-activedescendant`, Enter abre, Escape restaura foco (Listo cuando); unhappy = lista vacía, Enter sin selección, sin Omnibox, sin `fetch`.
- `scripts/check-vectra-web.sh`: exige el módulo y corre el fixture. MUST NOT pintar datos en `data-region="results"` del *router* (sigue vacío para T-03-15). MUST NOT llamar a `/search` ni importar `api.js`.

## Non-goals

- Resultados reales / search-as-you-type — **T-03-15**
- Tarjeta completa con highlight — **T-03-16**, **T-03-17**
- Sugerencias bajo el Omnibox — **T-03-21** (v2; reutilizará este módulo)
- Panel de recomendaciones (Similares / Sustitutos / Se usa junto con) — **T-04-08**
- Ejecutar modos voz/IA/imagen — **T-05-20**, **T-06-09**, **T-07-11**
- Playwright / e2e de teclado en búsqueda — **T-09-10**
- jsdom, `package.json`, job nuevo de CI, OpenAPI, crates Rust

## Capabilities

### New Capabilities

- Ninguna. El helper es parte de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `keyboard-list.js` (listbox + `aria-activedescendant`, flechas/Enter/Escape, reutilizable); demo en `dev/components.html`; ampliar la suite Node (happy + unhappy). El *router* operador no cambia: `data-region="results"` sigue vacío.

## Impact

- **Repo:** `vectra-web/js/keyboard-list.js`; `vectra-web/dev/components.html`; `scripts/fixtures/vectra-web/keyboard-list.test.mjs`; `scripts/check-vectra-web.sh` (y `dom-stub.mjs` solo si el stub no alcanza para `keydown` / `aria-*`)
- **APIs / OpenAPI:** sin paths nuevos; MUST NOT llamar endpoints
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-10`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-10 (Listo cuando: con resultados de ejemplo se recorre, abre y cierra sin mouse)
- **Alcance:** cabe en ~2 h (un módulo + demo + tests Node); no se divide
