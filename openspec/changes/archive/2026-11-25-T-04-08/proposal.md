# Proposal

## Ticket padre

- **Ticket:** [T-04-08](../../../docs/7-tickets.md) — UI panel de recomendaciones
- **Change:** `T-04-08`
- **HU:** [HU-04-05](../../../docs/6-historias.md#hu-04-05--panel-de-recomendaciones) (Tres bloques, Productos inactivos; **no** Latencia), [HU-04-06](../../../docs/6-historias.md#hu-04-06--sustitutos-destacados-sin-stock)
- **Depende de:** T-04-07 (en `develop`: `GET /api/v1/products/{productId}/recommendations`); T-03-17 (tarjetas `full` en resultados); T-01-04 (`vx-side-panel` con trampa de foco, Escape, `vx-closed`)
- **Hace:** al abrir un resultado, `vx-side-panel` con "Similares", "Sustitutos" (destacado si `highlightSubstitutes`) y "Se usa junto con", ordenados por prioridad; elegir un producto del panel lo abre; al cerrar, el foco vuelve al Omnibox.
- **No hace:** agregar al carrito (T-08-06).
- **Listo cuando:** escenarios de HU-04-05 y HU-04-06 verificados a mano.

## Why

HU-04-05 y HU-04-06 piden que, al seleccionar un producto en los resultados, el operador vea similares, sustitutos y complementarios en un panel lateral, con sustitutos destacados si el origen no tiene stock. T-04-07 ya expone `GET /api/v1/products/{productId}/recommendations`; `vx-side-panel` y `vx-product-card` `compact` existen, pero `search.js` deja `onOpen` vacío y la spec de T-03-17 prohíbe abrir el panel. Sin este cableado, el grafo no llega a la terminal.

## What Changes

- Al activar un resultado (Enter en el listbox de T-01-10 **o** clic en la tarjeta), abrir el `vx-side-panel` de `data-region="side-panel"` y llamar `GET /api/v1/products/{productId}/recommendations` con el `id` del hit.
- Pintar tres bloques en español: **Similares**, **Sustitutos**, **Se usa junto con**, cada uno con `vx-product-card` `compact` en el orden del array de la API (prioridad ya resuelta en T-04-06).
- Si `highlightSubstitutes === true`, destacar visualmente el bloque Sustitutos (HU-04-06). Si es `false`, el bloque se muestra sin destacar.
- Elegir un producto del panel (Enter o clic) abre **sus** recomendaciones (misma petición, mismo panel; aborta la ronda anterior).
- Al cerrar el panel (Escape o control de cierre), el foco vuelve al Omnibox (T-01-08 ya escucha `vx-closed`; no reimplementar el overlay).
- Invertir el non-goal de T-03-17: activar un hit **sí** abre el panel.
- TDD Node (`node --test`, sin jsdom ni npm): happy = tres bloques, orden, destaque, abrir desde el panel, foco al cerrar; unhappy = 404/503/network, abort, hit sin `id`, `highlightSubstitutes: false`.
- `scripts/check-vectra-web.sh` cubre el módulo nuevo y la suite.

## Non-goals

- Agregar al carrito desde resultados o recomendaciones — **No hace** del ticket; **T-08-06** (v2)
- UI de relaciones en la ficha / carga rápida / `vortex seed` de aristas — **T-04-09**, **T-04-10** (v2), **T-04-11**
- Latencia p95 < 20 ms del escenario HU-04-05 «Latencia» — [6.3.1](../../../docs/6-historias.md#631-alcance-por-historia) lo deja para *benchmarks* v2 (**T-09-07**)
- Playwright / e2e de recomendaciones — **T-09-09**, **T-09-11**
- Reabrir consulta 4.6.2, filtros `activo`/`stock > 0`, endpoint, DTO u OpenAPI — **T-04-06**, **T-04-07** (ya en `develop`; la UI pinta lo que llega)
- Caché HTTP de recomendaciones — **No hace** de T-04-07
- Imagen principal hidratada (`imageUrl` sigue `null`) — **T-02-28** (v2)
- Cambiar `vx-side-panel` para lógica de negocio (el componente MUST seguir sin recomendaciones)
- Tokens `--vx-*` nuevos, Web Component nuevo, `package.json`, jsdom, npm
- Precache del Service Worker de módulos de búsqueda (T-03-15 tampoco precacheó `search.js`)
- Sugerencias del Omnibox abriendo el panel — **T-03-21** (v2)

## Capabilities

### New Capabilities

- (ninguna; el panel es comportamiento de la SPA `vectra-web`)

### Modified Capabilities

- `vectra-web`: al abrir un resultado, `vx-side-panel` consume `GET /api/v1/products/{productId}/recommendations`, pinta los tres bloques (Sustitutos destacado si `highlightSubstitutes`), permite abrir un recomendado, restaura el foco al Omnibox al cerrar; la búsqueda deja de prohibir esa apertura; tests Node happy + unhappy.

## Impact

- **Repo:** `vectra-web/js/recommendations.js` (nuevo); `vectra-web/js/search.js` (`onOpen` + `data-product-id` + clic); `scripts/fixtures/vectra-web/recommendations.test.mjs` (nuevo) y extensión de `search.test.mjs`; `scripts/check-vectra-web.sh`; MAY `vectra-web/css/base.css` con *tokens* existentes para el destaque de Sustitutos.
- **APIs / OpenAPI:** solo consume `GET /api/v1/products/{productId}/recommendations` (`operationId` `getRecommendations`, schema `Recommendations`). MUST NOT agregar paths ni DTOs. MUST NOT filtrar de nuevo inactivos ni `stock = 0` (ya lo hace el backend).
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-04-08`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando humano: escenarios HU-04-05 / HU-04-06 a mano (con relaciones creadas vía API; el seed T-04-11 llega después).
- **Ticket:** cierra T-04-08
- **Alcance:** cabe en ~2 h (cablear `onOpen`, pintar tres bloques compactos, destaque, abort, tests Node). No se divide.
