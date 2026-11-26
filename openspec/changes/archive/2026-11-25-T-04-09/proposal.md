# Proposal

## Ticket padre

- **Ticket:** [T-04-09](../../../docs/7-tickets.md) — UI relaciones en la ficha
- **Change:** `T-04-09`
- **HU:** [HU-04-03](../../../docs/6-historias.md#hu-04-03--gestionar-las-relaciones-de-un-producto) (Listado; Edición y eliminación)
- **Depende de:** T-04-05 (en `develop`: `GET /api/v1/products/{productId}/relations`, `PATCH`/`DELETE /api/v1/relations/{relationType}/{relationId}`); T-02-35 (ficha con pestañas; Relaciones visible y no operable)
- **Hace:** pestaña "Relaciones" agrupada por tipo, con prioridad, nota e indicador bidireccional; edición en línea de prioridad y nota; eliminar.
- **No hace:** alta (T-04-10).
- **Listo cuando:** escenarios de HU-04-03 verificados a mano.

## Why

HU-04-03 pide que el administrador vea, edite y elimine las relaciones de un producto desde su ficha. T-04-05 ya expone el contrato HTTP; T-02-35/T-02-38 dejan la pestaña **Relaciones** visible y `aria-disabled`. Sin este cableado, el grafo solo se gestiona por API y el panel de recomendaciones de T-04-08 no se puede curar desde la SPA.

## What Changes

- En la **ficha de edición**, la pestaña **Relaciones** pasa a operable (mismo patrón que Stock en T-02-38): activarla selecciona la pestaña, no cambia el hash (`#/admin/productos`) y carga `GET /api/v1/products/{productId}/relations` (dirección saliente por defecto).
- Pintar las aristas **agrupadas por tipo** en español (**Similares**, **Sustitutos**, **Se usa junto con**), cada fila con destino (SKU/nombre), prioridad, nota e indicador bidireccional (`bidirectionalGroup` no nulo).
- Edición en línea de prioridad y nota → `PATCH /api/v1/relations/{relationType}/{relationId}` con `RelationUpdate`. Tras 200, la lista refleja el cambio de inmediato (re-GET o ítems del 200).
- Eliminar → `DELETE` del mismo path; tras 204, la fila desaparece. El grupo bidireccional lo resuelve el backend.
- En la ficha de **alta**, Relaciones sigue no operable (no hay `productId`).
- TDD Node (`node --test`, sin jsdom ni npm): happy = listado agrupado, PATCH, DELETE y lista vacía; unhappy = 404/400/503/network, prioridad inválida, no hay alta, pestaña deshabilitada en alta.
- `scripts/check-vectra-web.sh` cubre los tests nuevos. Invertir en spec el “Relaciones no operable” de T-02-35/T-02-38 **solo en edición**.

## Non-goals

- Alta / buscador de relaciones en la pestaña — **No hace** del ticket; **T-04-10** (v2)
- `vortex seed` de `relaciones.jsonl` — **T-04-11**
- Sección de menú `#/admin/relaciones` (shell vacío de T-01-11) — no es esta ficha
- Crear, reabrir o cambiar endpoints/DTOs de relaciones — **T-04-05** (ya en `develop`; la UI consume el contrato)
- Panel operador de recomendaciones, carrito, latencia p95 — **T-04-08**, **T-08-06** (v2), **T-09-07** (v2)
- Playwright / e2e — **T-09-09**, **T-09-11**
- Pestaña Imágenes — **T-02-39** (v2)
- Tokens `--vx-*` nuevos, Web Component nuevo, `package.json`, jsdom, npm
- Confirmación modal de borrado, edición de tipo/destino, listado de aristas entrantes (`direction=incoming`)

## Capabilities

### New Capabilities

- (ninguna; la pestaña es comportamiento de la SPA `vectra-web`)

### Modified Capabilities

- `vectra-web`: pestaña Relaciones operable en la ficha de edición; listado agrupado por tipo con prioridad, nota e indicador bidireccional; PATCH/DELETE en línea; tests Node happy + unhappy. La pestaña deja de estar prohibida en edición (T-02-35/T-02-38).

## Impact

- **Repo:** `vectra-web/js/admin/product-sheet.js` (panel Relaciones); `vectra-web/js/admin/products.js` (tab `relaciones` + GET/PATCH/DELETE); `scripts/fixtures/vectra-web/product-sheet.test.mjs`; MAY `vectra-web/css/base.css` con *tokens* existentes; `scripts/check-vectra-web.sh` si hace falta un test file extra.
- **APIs / OpenAPI:** solo consume `listProductRelations`, `updateRelation`, `deleteRelation`. MUST NOT agregar paths ni DTOs. MUST NOT llamar `POST /api/v1/relations`.
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-04-09`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando humano: escenarios HU-04-03 a mano (el seed T-04-11 llega después; relaciones vía API).
- **Ticket:** cierra T-04-09
- **Alcance:** cabe en ~2 h (habilitar tab, pintar grupos, PATCH/DELETE, tests Node). No se divide.
