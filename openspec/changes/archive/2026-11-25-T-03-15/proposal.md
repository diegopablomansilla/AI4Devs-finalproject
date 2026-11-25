# Proposal

## Ticket padre

- **Ticket:** [T-03-15](../../../docs/7-tickets.md) — UI búsqueda mientras se escribe
- **Change:** `T-03-15`
- **HU:** [HU-03-03](../../../docs/6-historias.md#hu-03-03--búsqueda-mientras-se-escribe-tolerante-a-errores) (escenario Cancelación; tolerancia/ranking ya cerrados en T-03-14)
- **Depende de:** T-03-13 (en `develop`: `GET /api/v1/search` + `AbortController` en el servidor); T-01-10 (en `develop`: `vx-omnibox` emite `vx-query`, `bindKeyboardList`, región `data-region="results"` vacía)
- **Hace:** en cada `input` del Omnibox en modo texto, cancela la petición anterior con `AbortController`, llama a `/search` y renderiza la lista; solo se pinta la respuesta de la última consulta.
- **No hace:** tarjeta completa (T-03-17).
- **Listo cuando:** escribiendo rápido "perno m8", la pestaña de red muestra las peticiones anteriores canceladas.

## Why

HU-03-03 pide resultados en cada tecla y que solo se vean los de la última consulta. El endpoint y la cancelación en Axum ya existen (T-03-13); el Omnibox emite `vx-query` sin ejecutar nada (T-01-09) y `data-region="results"` sigue vacío (T-01-08 / T-01-10). Sin el cliente search-as-you-type, el operador no puede buscar desde Vectra y T-03-16 / T-03-17 no tienen lista que resaltar ni tarjetas que completar.

## What Changes

- Módulo ES `vectra-web/js/search.js` (`mountSearch`): escucha `vx-query` del Omnibox; si `mode === "text"` y la consulta no está vacía, aborta la petición anterior (`AbortController`) y llama `GET /api/v1/search?q=…` vía `request` (reenvía `signal`, ya cubierto en T-01-06).
- Sin *debounce*: una petición por evento de consulta (cada `input` / `vx-query` en modo texto).
- Pinta `data-region="results"` con una lista accesible (`role="listbox"` + `bindKeyboardList` de T-01-10): cada hit muestra **nombre y SKU** como texto (`textContent`). MUST NOT interpolar `highlight` como HTML (T-03-16) ni armar la `vx-product-card` completa (T-03-17).
- Solo se pinta la respuesta cuya `signal` no fue abortada (Listo cuando: peticiones anteriores `canceled` en red; tests Node con `AbortSignal`).
- `startRouter` monta la búsqueda en la vista operador (espejo de `mountProducts` en admin). Administración y 404 MUST NOT llamar a `/search`.
- TDD Node (`node --test`, sin jsdom ni npm): happy = consulta texto → GET con `q`, abort de la anterior, solo última lista; unhappy = abort no pinta / no toast, modo IA/voz/imagen no llama, error `problem+json` → `showError`, hits vacíos, falta el módulo.
- `scripts/check-vectra-web.sh`: exige `search.js` y corre `search.test.mjs`.

## Non-goals

- Tarjeta completa (`vx-product-card` con marca, precio, stock, ubicación, imagen, estado sin stock) — **T-03-17**
- Resaltado seguro (`<mark>` vía `highlight.js`) — **T-03-16**
- Facetas / filtros en la UI — **T-03-18**, **T-03-19** (v2)
- Sugerencias bajo el Omnibox — **T-03-20**, **T-03-21** (v2)
- Búsqueda por código (`GET /search/by-code/{code}`) — **T-03-22**, **T-03-23**
- UI modo reducido / `degraded: true` — **T-03-30**, **T-03-31** (v2)
- Ejecutar modos voz, IA o imagen — **T-05-20**, **T-06-09**, **T-07-11** (v2)
- Abrir ficha de producto / panel de recomendaciones al elegir un hit — **T-02-35** (admin), **T-04-08**
- Playwright / e2e de búsqueda — **T-09-09**, **T-09-10**
- Cambios a `docs/openapi.yaml`, crates Rust, ranking Meilisearch, timeout de ruta
- jsdom, `package.json`, dependencias npm, job nuevo de CI
- *Debounce* o umbral de teclas: el ticket pide cada `input`

## Capabilities

### New Capabilities

- Ninguna. La búsqueda mientras se escribe es comportamiento de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: search-as-you-type en la vista operador (`AbortController` + `GET /api/v1/search` + lista en `data-region="results"`; solo la última respuesta se pinta); ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/search.js` (nuevo); `vectra-web/js/router.js` y `vectra-web/js/app.js` (montar en operador); `scripts/fixtures/vectra-web/search.test.mjs` (nuevo); `scripts/check-vectra-web.sh`; `vectra-web/css/base.css` solo si hace falta layout mínimo de la lista con *tokens* `--vx-*` existentes
- **APIs / OpenAPI:** consumo de `GET /api/v1/search` (`q` únicamente; defaults de `limit`/`sort` del contrato). MUST NOT agregar paths ni DTOs. MUST NOT enviar `facets=true`
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-03-15`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando humano: escribir rápido "perno m8" y ver canceladas las peticiones anteriores en la pestaña de red
- **Ticket:** cierra T-03-15
- **Alcance:** cabe en ~2 h (un módulo + mount en router + tests Node); no se divide
