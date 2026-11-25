# Proposal

## Ticket padre

- **Ticket:** [T-03-08](../../../docs/7-tickets.md) — Worker: *fan-out* de categoría, marca y atributo
- **Change / rama:** `T-03-08` (CLI OpenSpec: `t-03-08` → carpeta `T-03-08`)
- **HU:** [HU-03-01](../../../docs/6-historias.md#hu-03-01--sincronización-del-índice-de-búsqueda), [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías) (escenario «Renombrar una rama»: la búsqueda refleja el cambio en < 1 s)
- **Depende de:** T-03-07 (worker de productos; cursor; `build_search_document`)
- **Hace:** eventos `categoria` y `marca` → reindexar los productos afectados (rama completa en categorías); `atributo_def` → reindexar los productos que lo usan y, si cambió `facetable`, actualizar `filterableAttributes`
- **No hace:** sinónimos
- **Listo cuando:** test: renombrar "Fijaciones" actualiza `categoria.lvl0` de sus productos en menos de 1 s (escenario de HU-02-01)

## Why

T-03-07 ya consume el outbox y sincroniza eventos `producto`, pero **avanza el cursor sin reindexar** cuando llega `categoria`, `marca` o `atributo_def`. Renombrar "Fijaciones", cambiar el nombre de una marca o marcar un atributo como facetable deja el índice desactualizado: los documentos siguen con `categoria.lvl0` viejo, el nombre de marca anterior o sin `attr_<clave>` / `filterableAttributes`. HU-02-01 y [4.8.1](../../../docs/4-modelo.md#481-outbox) exigen *fan-out* a los productos afectados en < 1 s.

## What Changes

- El Index Sync Worker trata `categoria` / `marca` / `atributo_def` como eventos de *fan-out* (ya no no-op)
- `categoria`: reindexar productos de **toda la rama** (`ProductRepo::list` con filtro de categoría, [4.4.1](../../../docs/4-modelo.md#441-categoria--árbol))
- `marca`: reindexar productos de esa marca (`FiltroProducto.marca`)
- `atributo_def`: aplicar el perfil `productos` (`filterableAttributes` con `attr_<clave>` de defs `facetable` y activas) y reindexar los productos que usan esa definición
- Reutilizar `build_search_document` + *upsert*/*delete* ya definidos para `producto` (fuente de verdad; inactivo → *delete*)
- Mover `productos_index_settings` a `domain` para que `sync` lo llame sin depender de `search` (allowlist T-03-07)
- Tests TDD happy + unhappy; Listo cuando = integración rename "Fijaciones" → `categoria.lvl0` en < 1 s

## Non-goals

- Sinónimos (`sinonimo` → Meilisearch) → **T-03-27** (v2); el worker sigue avanzando el cursor sin mutar `synonyms`
- Reintentos, `outbox_fallo`, métricas, `sync` en `/health` → **T-03-09**
- Purga del outbox → **T-03-10** (v2)
- `vortex reindex` / `swapIndexes` → **T-03-11**
- Wiring en `bin/vortex` / arranque ordenado → **T-00-27**
- `GET /search`, ranking, facetas HTTP, resaltado → **T-03-12+**, **T-03-18** (v2)
- Imagen principal / `imagen_producto` mutando `imagen` → **T-02-28** (v2)
- Ampliar `ProductRepo` con un listado dedicado por `atributo_def` (el worker pagina `list` + `get` en el MVP)
- Cambiar `openapi.yaml`, CI YAML, o el probe T-09-05

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `vortex-sync`: *fan-out* de `categoria` (rama completa), `marca` y `atributo_def` (reindex + `filterableAttributes`); deja de ser no-op
- `domain-text-search`: `productos_index_settings` vive en `domain` (el worker de `sync` no puede depender de `search`)
- `vortex-search`: reexporta `productos_index_settings` desde `domain`; los tests T-03-05 no cambian de contrato

## Impact

- **Repo:** `vortex-core/crates/sync/` (worker); `vortex-core/crates/domain/` (`productos_index_settings`); `vortex-core/crates/search/` (reexport)
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `cargo test -p search`; `cargo test -p sync`; `cargo test --workspace -- --include-ignored meilisearch`; `openspec validate t-03-08`
- **Ticket:** cierra T-03-08 (Listo cuando: rename "Fijaciones" → `categoria.lvl0` < 1 s); desbloquea escenarios de búsqueda de HU-02-01
- **Ámbito:** cabe en ~2 h: el listado de rama/marca ya existe; no se toca ProductRepo; sinónimos / retries / wiring son otros tickets. No se divide el ticket
