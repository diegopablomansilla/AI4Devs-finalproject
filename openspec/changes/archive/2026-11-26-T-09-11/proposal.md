# Proposal

## Ticket padre

- **Ticket:** [T-09-11](../../../docs/7-tickets.md) — e2e H1: recomendaciones y alta de producto
- **Change:** `T-09-11`
- **HU:** [HU-09-03](../../../docs/6-historias.md#hu-09-03--tests-de-extremo-a-extremo-de-la-spa) (flujos H1 restantes: panel de recomendaciones y alta; búsqueda ya en T-09-10)
- **Depende de:** T-09-10 (cerrado en `develop`: e2e búsqueda + helper de índice); T-04-08 (cerrado: panel `vx-side-panel` con tres bloques y sustitutos destacados)
- **Hace:** panel de recomendaciones con sustitutos destacados y alta de producto que aparece en la búsqueda
- **No hace:** — (el ticket no declara exclusiones; los non-goals salen de otros tickets del backlog y del recableo MVP § 7.5.1)
- **Listo cuando:** pasa en CI
- **Recableo MVP (§ 7.5.1):** HU-09-03 en el curso cubre búsqueda, recomendaciones y alta (sin facetas; T-09-10 ya cubrió búsqueda)

## Why

HU-09-03 pide verificar en navegador real los flujos H1 como los ve el operador. T-09-10 cubrió typo, stock visible y teclado, y **prohibió** abrir el panel y navegar a admin. El panel (T-04-08) y el alta (T-02-35) ya existen, pero solo en `node --test` con DOM stub. Sin Playwright sobre `seed/` (grafo T-04-11 + ficha admin), un cambio de UI, outbox o indexación puede romper lo que el operador ve al elegir un resultado o dar de alta un SKU, y el job `e2e` no lo detecta.

## What Changes

- Spec Playwright `e2e/tests/recommendations.spec.js` sobre la fixture de T-09-09: el operador busca un SKU semilla con grafo, abre el resultado (Enter o clic) y ve el `vx-side-panel` con «Similares», «Sustitutos» y «Se usa junto con»
- Sustitutos **destacados** (HU-04-06): el dataset no tiene `stock_inicial` `"0"`; el spec deja en 0 el stock de `PER-ACERO-01` (ajuste de ledger, no `seed/`) y reabre el panel: bloque Sustitutos con `data-highlight-substitutes` / «Destacados» y tarjeta `PER-ACERO-02` (stock 12)
- Con stock > 0 el mismo origen **no** destaca Sustitutos
- Spec Playwright `e2e/tests/product-create.spec.js`: administrador abre `#/admin/productos`, «Nuevo producto», completa Datos (SKU único, nombre, tipo, categoría, precio) y Guardar; el SKU aparece en el listado y, tras esperar el índice, en la búsqueda del operador
- Unhappy: panel de un SKU sin aristas muestra los tres títulos sin tarjetas compactas; alta con SKU duplicado no crea el producto; formulario incompleto no navega al listado
- El job CI `e2e` de T-09-09 ya ejecuta `npx playwright test`; **no** se cambia el YAML ni `check-ci.sh`
- TDD: red = specs que fallan hasta locators/helpers; green = suite Playwright en verde en el job

## Non-goals

Este change **no** cubre:

- Filtros facetados / `vx-chip` de categoría, marca, atributo o «con stock» → **T-03-18**, **T-03-19** (v2; recableo)
- Agregar al carrito desde resultados o recomendaciones → **T-08-06** (v2)
- e2e Modo IA / voz / visión+POS → **T-09-17**, **T-09-21**, **T-09-24** (v2)
- Alta de relaciones desde la ficha → **T-04-10** (v2); el grafo de este e2e es el de `seed/relaciones.jsonl` (T-04-11)
- Formulario de movimientos de stock como escenario propio → **T-02-38** (el ajuste a 0 es *setup* del highlight, no el Listo cuando de HU-02-07)
- Edición con `version_conflict`, baja/reactivación, pestaña Relaciones → **T-02-37**, **T-02-22**, **T-04-09**
- *Benchmarks* de latencia del panel (HU-04-05 «Latencia») → **T-09-07** (v2)
- Degradación Meilisearch / 503 del panel → **T-09-12** (v2); no se mockea HTTP
- Cambiar `seed/` (no hay SKU con stock 0 ni se agrega uno)
- Cambiar `vectra-web/` salvo un bug que impida el Listo cuando
- Job nuevo de CI, Ollama, Whisper, Firefox/WebKit, Lighthouse, `vectra-web/package.json`
- Cambios a `docs/openapi.yaml` o crates Rust

## Capabilities

### New Capabilities

- Ninguna. Los escenarios cuelgan del *setup* `e2e-playwright` (T-09-09 / T-09-10 lo dejaron para T-09-11).

### Modified Capabilities

- `e2e-playwright`: exigir specs H1 de panel de recomendaciones (tres bloques, sustitutos destacados con stock 0, no destacado con stock) y alta de producto visible en búsqueda; unhappy de panel vacío / SKU duplicado / ficha incompleta; la suite de búsqueda de T-09-10 MUST seguir pasando

## Impact

- **Repo:** `e2e/tests/recommendations.spec.js` y `e2e/tests/product-create.spec.js` (nuevos); MAY helpers en `e2e/` (abrir panel, ajustar stock a 0, esperar índice de un SKU nuevo, llenar `vx-field`). MUST NOT exigir cambios en `e2e/tests/search.spec.js` ni `smoke.spec.js`. MUST NOT tocar `.github/workflows/ci.yml` ni `scripts/check-ci.sh`
- **APIs / OpenAPI:** ninguna. Consumo de `GET /api/v1/search`, `GET /api/v1/products/{id}/recommendations`, `POST /api/v1/products`, `POST /api/v1/products/{id}/stock-movements` ya existentes
- **CI / verificación:** el job `e2e` existente corre todos los `*.spec.js`. Comandos [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sin cambio. Local: `npx playwright test` en `e2e/` con `VORTEX_BIN` / `MEILI_BIN`. `openspec validate T-09-11` (o `t-09-11` si el CLI exige minúsculas)
- **Ticket:** cierra T-09-11; cierra el recorte MVP de HU-09-03. Listo cuando = el job `e2e` del PR pasa
- **Alcance ~2 h:** dos specs + helpers de wait/stock. No se divide el backlog
