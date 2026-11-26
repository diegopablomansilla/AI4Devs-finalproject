# Proposal

## Ticket padre

- **Ticket:** [T-03-23](../../../docs/7-tickets.md) — UI búsqueda por código
- **Change:** `T-03-23`
- **HU:** [HU-03-08](../../../docs/6-historias.md#hu-03-08--búsqueda-por-código) (escenarios OEM compartido y Coincidencia exacta; el endpoint ya cerró T-03-22)
- **Depende de:** T-03-22 (en `develop`: `GET /api/v1/search/by-code/{code}`); T-03-15 (en `develop`: Omnibox search-as-you-type + `AbortController`); T-03-17 (en `develop`: resultados como `vx-product-card` completa)
- **Hace:** si la consulta parece un código (sin espacios, con letras y dígitos o guiones), llama a `by-code` en paralelo con `/search` y fija las coincidencias exactas arriba con la etiqueta "Código exacto".
- **No hace:** lector de código de barras.
- **Listo cuando:** escribir un OEM del dataset muestra ambas piezas primero.

## Why

HU-03-08 pide encontrar el repuesto exacto por SKU, fabricante u OEM, incluido un OEM compartido entre dos piezas. T-03-22 ya consulta SurrealDB; el Omnibox (T-03-15/T-03-17) solo llama a Meilisearch, que tolera prefijos y typos (H-05). Sin el cliente paralelo, el operador no ve las coincidencias exactas fijas arriba. Ahora, porque el endpoint y las tarjetas de resultado ya están en `develop`.

## What Changes

- `vectra-web/js/search.js`: en `vx-query` modo texto, si el texto recortado **parece un código**, dispara en paralelo `GET /api/v1/search/by-code/{code}` y `GET /api/v1/search?q=…` con el mismo `AbortController`.
- Heurística (ticket): sin espacios; solo letras, dígitos o guiones; longitud 1..=64 (tope del path `code` en `openapi.yaml`). Consultas con espacios (`"perno m8"`) siguen yendo solo a `/search`.
- Fusiona resultados: `items` de `by-code` primero, etiquetados **"Código exacto"**; después los `hits` de `/search` cuyo `id` no está ya en el grupo exacto. Cada pieza sigue siendo `vx-product-card` `full` (T-03-17).
- TDD Node (`node --test`, sin jsdom ni npm): happy = OEM compartido pinta ambas tarjetas primero + etiqueta; unhappy = no parece código, abort de ambas peticiones, `by-code` 400/503 no oculta `/search`, consulta no exacta no pinnea.
- `scripts/check-vectra-web.sh`: la suite `search.test.mjs` cubre los tests nuevos.

## Non-goals

- Lector de código de barras — **No hace** del ticket
- Cambios al endpoint, dominio, storage o ranking Meilisearch — **T-03-22** (ya en `develop`)
- Coincidencia parcial / typo en `by-code` — **No hace** de T-03-22
- Facetas / filtros — **T-03-18**, **T-03-19** (v2)
- Sugerencias — **T-03-20**, **T-03-21** (v2)
- UI modo reducido / `degraded: true` — **T-03-30**, **T-03-31** (v2)
- Abrir ficha / panel de recomendaciones al elegir un hit — **T-04-08**
- Playwright / e2e de búsqueda — **T-09-09**, **T-09-10**
- Cambios a `docs/openapi.yaml` o crates Rust
- jsdom, `package.json`, dependencias npm, job nuevo de CI
- Web Component nuevo o *tokens* `--vx-*` nuevos

## Capabilities

### New Capabilities

- Ninguna. La UI de búsqueda por código es comportamiento de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: si la consulta parece un código, `mountSearch` llama a `GET /api/v1/search/by-code/{code}` en paralelo con `/search`, pinnea las coincidencias exactas arriba con la etiqueta «Código exacto» y las omite del resto de hits; ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/search.js`; `scripts/fixtures/vectra-web/search.test.mjs`; `scripts/check-vectra-web.sh` (comentario). MAY tocar `vectra-web/css/base.css` solo con *tokens* `--vx-*` existentes para el grupo «Código exacto».
- **APIs / OpenAPI:** consumo de `GET /api/v1/search/by-code/{code}` (`operationId` `findProductsByCode`, DTO `{ items: ProductSummary[] }`) ya declarado. MUST NOT agregar paths ni DTOs. MUST NOT enviar `facets` en `/search`.
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-03-23`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando humano: escribir `OEM-PER-8840` (OEM compartido del seed) y ver ambas piezas primero con «Código exacto».
- **Ticket:** cierra T-03-23
- **Alcance:** cabe en ~2 h (heurística + `Promise.allSettled` + pin + tests Node); no se divide
