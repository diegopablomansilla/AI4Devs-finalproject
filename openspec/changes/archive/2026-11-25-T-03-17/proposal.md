# Proposal

## Ticket padre

- **Ticket:** [T-03-17](../../../docs/7-tickets.md) — UI tarjetas de resultado completas
- **Change:** `T-03-17`
- **HU:** [HU-03-05](../../../docs/6-historias.md#hu-03-05--tarjetas-de-resultado-con-stock-precio-y-ubicación) (escenarios Tarjeta de resultado y Sin stock; Resaltado seguro ya cerrado en T-03-16)
- **Depende de:** T-03-16 (en `develop`: lista nombre/SKU + `highlight.js`); T-01-05 (en `develop`: `vx-product-card` full/compact, slot `name`, estado sin stock)
- **Hace:** resultados con `vx-product-card` completa (nombre resaltado, SKU, marca, precio, stock, ubicación, imagen) y estado visual sin stock.
- **No hace:** panel de recomendaciones (T-04-08).
- **Listo cuando:** escenarios de HU-03-05 verificados a mano.

## Why

HU-03-05 pide que el operador vea stock, precio, ubicación y foto en cada resultado sin abrir la ficha. T-03-15/T-03-16 ya buscan y pintan nombre + SKU (con `<mark>` seguro). `vx-product-card` (T-01-05) ya tiene la variante `full` y el marcador «Sin stock», pero `search.js` aún renderiza filas sueltas y la spec **prohíbe** la tarjeta. Sin este change, HU-03-05 queda incompleta y T-04-08 no tiene tarjetas reutilizables en resultados.

## What Changes

- `vectra-web/js/search.js`: cada hit de `GET /api/v1/search` se pinta como `vx-product-card` `variant="full"` en el `role="listbox"` existente (`bindKeyboardList` de T-01-10).
- Mapeo `SearchHit` → atributos: `sku`, `name`, `brand`, `price`, `stock`, `location`, `image-url`; nombre resaltado en el slot `name` vía `appendSanitizedHtml` (T-03-16). Stock `0` → la tarjeta expone `data-out-of-stock` y el texto «Sin stock» (comportamiento ya en T-01-05).
- Imagen MVP: `imageUrl` suele ser `null` (HU-03-05 parcial / recableo sin foto principal); la tarjeta muestra el *placeholder* «Sin imagen». Si `imageUrl` viene, se pasa a `image-url`.
- `highlight.description` deja de pintarse: la tarjeta no tiene ese campo (ticket Hace no lo lista).
- TDD Node (`node --test`, sin jsdom ni npm): happy = tarjeta completa + mark + sin stock + placeholder; unhappy = HTML peligroso en highlight, hit malformado no lanza, campos de texto no interpretan HTML.
- Extender `dom-stub.mjs` lo mínimo para `querySelector("vx-product-card")`, `createElement` de custom elements registrados y `connectedCallback` al insertar.
- Delta MODIFIED `vectra-web`: retirar la prohibición de T-03-16 sobre `vx-product-card` en resultados.

## Non-goals

- Panel de recomendaciones (Similares / Sustitutos / Se usa junto con) — **T-04-08**
- Abrir ficha de producto al activar un hit — **T-02-35** (admin) / T-04-08
- Variante `compact` en esta lista — **T-04-08**
- Facetas / filtros — **T-03-18**, **T-03-19** (v2)
- Sugerencias — **T-03-20**, **T-03-21** (v2)
- Búsqueda por código — **T-03-22**, **T-03-23**
- UI modo reducido — **T-03-31** (v2)
- Subida o almacenamiento de fotos — **T-02-28** … **T-02-39** (HU-02-09, v2)
- Cambiar el contrato de `vx-product-card` (ya cerrado en T-01-05) salvo lo mínimo para proyectar el slot
- Playwright / e2e — **T-09-09**, **T-09-10**
- Cambios a `docs/openapi.yaml`, crates Rust, ranking, timeout de ruta
- jsdom, `package.json`, dependencias npm, job nuevo de CI

## Capabilities

### New Capabilities

- Ninguna. Las tarjetas de resultado son comportamiento de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: pintar cada hit de búsqueda como `vx-product-card` completa (campos de `SearchHit`, nombre resaltado seguro, estado sin stock, imagen o *placeholder*); ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/search.js`; `scripts/fixtures/vectra-web/search.test.mjs`; `scripts/fixtures/vectra-web/dom-stub.mjs`; `scripts/check-vectra-web.sh` (comentario / tests existentes). MAY tocar `vectra-web/css/base.css` solo con *tokens* `--vx-*` existentes para el listado de tarjetas.
- **APIs / OpenAPI:** consumo del `SearchHit` ya existente (`brand`, `price`, `stock`, `location`, `imageUrl`, `highlight`). MUST NOT agregar paths ni DTOs.
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-03-17`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando humano: escenarios de HU-03-05 (tarjeta completa + sin stock; resaltado seguro ya cubierto).
- **Ticket:** cierra T-03-17
- **Alcance:** cabe en ~2 h (mapeo a componente existente + tests + stub); no se divide
