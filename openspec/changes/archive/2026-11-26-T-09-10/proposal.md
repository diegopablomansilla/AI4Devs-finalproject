# Proposal

## Ticket padre

- **Ticket:** [T-09-10](../../../docs/7-tickets.md) — e2e H1: búsqueda y filtros
- **Change:** `T-09-10`
- **HU:** [HU-09-03](../../../docs/6-historias.md#hu-09-03--tests-de-extremo-a-extremo-de-la-spa) (flujo H1 de búsqueda; recomendaciones y alta quedan en T-09-11)
- **Depende de:** T-09-09 (cerrado en `develop`: Playwright, fixture, smoke, job `e2e`); T-03-15 (cerrado: Omnibox → `GET /api/v1/search` + lista; tarjetas y teclado ya en T-03-17 / T-01-10)
- **Hace:** búsqueda con error tipográfico, tarjetas con stock y navegación por teclado
- **No hace:** filtros por atributo y stock (v2, con T-03-19)
- **Listo cuando:** pasa en CI
- **Recableo MVP (§ 7.5.1):** e2e de búsqueda **sin facetas**; depende de T-03-15 en vez de T-03-19

## Why

HU-09-03 pide verificar en navegador real los flujos H1 como los ve el operador. T-09-09 dejó el *setup* (fixture + smoke de carga) y **no** escenarios de negocio. La tolerancia a typos, las tarjetas con stock y el listbox por teclado ya existen (T-03-14 / T-03-17 / T-01-10) pero solo en tests de crate o `node --test` con DOM stub. Sin un spec Playwright sobre el dataset `seed/`, un cambio de UI o de indexación puede romper lo que el operador ve y el job `e2e` no lo detecta.

## What Changes

- Spec Playwright `e2e/tests/search.spec.js` sobre la fixture de T-09-09: el operador escribe `"tornilo"` en el Omnibox y aparece el producto semilla `"Tornillo autoperforante 10 x 1"` (SKU `TOR-AUTO-10X1`, HU-03-03)
- Cada resultado visible es un `vx-product-card` `full` con stock observable (el tornillo semilla tiene `stock_inicial` `"40"`; el dataset **no** tiene stock 0 — no se altera `seed/`)
- Navegación por teclado del listbox: `ArrowDown` desde el Omnibox activa la primera tarjeta y actualiza `aria-activedescendant`; Escape restaura el foco al Omnibox **sin** abrir el panel de recomendaciones (T-09-11)
- Helper de espera: el índice Meilisearch puede no estar listo al primer `GET /health` 200; el spec MUST esperar hits de `"tornilo"` (API o UI) y MUST NOT marcar skipped
- Unhappy: consulta sin coincidencias deja la lista vacía; typo en SKU no encuentra el producto (`typoTolerance.disableOnAttributes`); vaciar el Omnibox limpia las tarjetas; **no** hay barra de filtros facetados
- El job CI `e2e` de T-09-09 ya ejecuta `npx playwright test`; **no** se cambia el YAML ni `check-ci.sh`
- TDD: red = specs que fallan (o timeout) hasta el helper de índice + aserciones; green = suite Playwright en verde en el job

## Non-goals

Este change **no** cubre:

- Filtros por atributo, marca, categoría o «con stock» (facetas / `vx-chip`) → **T-03-18**, **T-03-19** (v2; recableo del ticket)
- Panel de recomendaciones y alta de producto en e2e → **T-09-11**
- e2e Modo IA / voz / visión+POS → **T-09-17**, **T-09-21**, **T-09-24** (v2)
- Búsqueda por código como escenario propio → **T-03-23** (ya en UI; no es el Hace de este ticket)
- *Benchmarks* de latencia p95 / 10 k SKUs → **T-09-07**, **T-09-08** (v2)
- Degradación Meilisearch → **T-09-12** (v2)
- Cambiar `seed/` (no hay SKU con `stock_inicial` `"0"`; no se agrega uno aquí)
- Cambiar `vectra-web/` (comportamiento ya cerrado en T-03-15 / T-03-17 / T-01-10) salvo un bug que impida el Listo cuando
- Job nuevo de CI, Ollama, Whisper, Firefox/WebKit, Lighthouse, `vectra-web/package.json`
- Cambios a `docs/openapi.yaml` o crates Rust

## Capabilities

### New Capabilities

- Ninguna. Los escenarios cuelgan del *setup* `e2e-playwright` (T-09-09 lo dejó para T-09-10+).

### Modified Capabilities

- `e2e-playwright`: exigir specs H1 de búsqueda con typo, tarjetas con stock y teclado sobre la fixture existente; unhappy de sin resultados / typo en SKU / lista limpia / ausencia de filtros facetados; MUST NOT abrir recomendaciones ni crear productos (T-09-11)

## Impact

- **Repo:** `e2e/tests/search.spec.js` (nuevo); MAY un helper `e2e/search-ready.js` (espera de índice). MUST NOT tocar `e2e/tests/smoke.spec.js` (sigue sin tipear). MUST NOT tocar `.github/workflows/ci.yml` ni `scripts/check-ci.sh` salvo que un bug del harness impida el índice (entonces un wait acotado en el helper, no un job nuevo)
- **APIs / OpenAPI:** ninguna. Consumo de `GET /api/v1/search?q=tornilo` ya existente
- **CI / verificación:** el job `e2e` existente corre todos los `*.spec.js`. Comandos [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sin cambio. Local: `npx playwright test` en `e2e/` con `VORTEX_BIN` / `MEILI_BIN`. `openspec validate T-09-10`
- **Ticket:** cierra T-09-10; desbloquea T-09-11. Listo cuando = el job `e2e` del PR pasa
- **Alcance ~2 h:** un spec + helper de wait + delta de spec. No se divide el backlog
