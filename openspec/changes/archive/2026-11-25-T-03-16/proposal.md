# Proposal

## Ticket padre

- **Ticket:** [T-03-16](../../../docs/7-tickets.md) — UI resaltado seguro
- **Change:** `T-03-16`
- **HU:** [HU-03-05](../../../docs/6-historias.md#hu-03-05--tarjetas-de-resultado-con-stock-precio-y-ubicación) (escenario Resaltado seguro)
- **Depende de:** T-03-15 (en `develop`: search-as-you-type con lista nombre/SKU)
- **Hace:** `js/highlight.js`: escapa todo el HTML y luego restaura solo `<mark>` y `</mark>`; se usa en nombre y descripción.
- **No hace:** otros formatos.
- **Listo cuando:** test en navegador con `<img onerror>` dentro del resaltado: se muestra como texto.

## Why

[5.7](../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación) define `highlight` con `<mark>…</mark>` desde Meilisearch. T-03-15 pintó la lista con `textContent` (frontera temporal). HU-03-05 exige fragmento resaltado sin XSS. Sin `highlight.js`, T-03-17 no puede proyectar coincidencias en el slot `name` de `vx-product-card`.

## What Changes

- `vectra-web/js/highlight.js`: `sanitizeHighlight` + `appendSanitizedHtml` (solo `<mark>` restaurado; resto escapado).
- `vectra-web/js/search.js`: nombre y `highlight.description` vía helper; SKU sigue texto plano.
- Tests `highlight.test.mjs` + ampliar `search.test.mjs`; harness `check-vectra-web.sh`.
- MODIFIED delta `vectra-web`: relajar prohibición de T-03-15 donde corresponda al resaltado seguro.

## Non-goals

- Tarjeta completa — **T-03-17**
- `<em>`, atributos en `<mark>`, innerHTML crudo — ticket No hace
- OpenAPI / Rust / Playwright

## Capabilities

### Modified Capabilities

- `vectra-web`: resaltado seguro en resultados de búsqueda.

## Impact

- **Repo:** `highlight.js`, `search.js`, tests, harness
- **Verificación:** `./scripts/check-vectra-web.sh`; suite Rust §5