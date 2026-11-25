# Proposal

## Ticket padre

- **Ticket:** [T-03-05](../../../docs/7-tickets.md) — Configuración del índice `productos`
- **Change:** `T-03-05`
- **HU:** [HU-03-04](../../../docs/6-historias.md#hu-03-04--ranking-de-relevancia), [HU-03-07](../../../docs/6-historias.md#hu-03-07--filtros-facetados)
- **Depende de:** T-03-04 (cerrado en `develop`: adaptador `MeiliTextSearch`; `apply_settings` vacío es no-op)
- **Hace:** aplica [4.11.2](../../../docs/4-modelo.md#4112-configuración-del-índice) más los agregados de los hallazgos H-03 y H-04: `marca_id`, `categoria_ids` y `tipo` como filtrables; `filterableAttributes` con los `attr_<clave>` de las `AtributoDef` con `facetable = true`. Actualiza 4.11 en [4-modelo.md](../../../docs/4-modelo.md)
- **No hace:** sinónimos (T-03-27)
- **Listo cuando:** test de integración lee la configuración aplicada y la compara con la esperada

## Why

T-03-04 dejó `apply_settings` como no-op: el índice no tiene ranking 4.11.2, typo desactivado en códigos ni atributos filtrables. HU-03-04 (desempate por stock, peso de `nombre` vs `descripcion`) y los filtros de T-03-12 (`marca_id`, `categoria_ids`, `con_stock`, `attr_*`) no pueden cumplirse. H-03 y H-04 piden `marca_id`, `categoria_ids` y `tipo` filtrables; 4.11 aún no los documenta.

## What Changes

- Rellenar `domain::IndexSettings` con las listas 4.11.2 (searchable, ranking, filterable, sortable, typo-disable); `Default`/vacío sigue siendo no-op. El resaltado (`attributesToHighlight`) queda para T-03-12
- En `search`, un constructor `productos_index_settings(claves_facetables)` que arma esas listas más H-03/H-04 (`marca_id`, `categoria_ids`, `tipo`) y `attr_<clave>` por cada clave facetable; **sin** `synonyms`
- `MeiliTextSearch::apply_settings` envía esas listas al motor (espera de *task*) cuando el payload no está vacío
- Test de integración `#[ignore = "meilisearch"]` que aplica, lee GET settings y compara con lo esperado (Listo cuando)
- Actualizar [4.11](../../../docs/4-modelo.md#411-modelo-derivado-en-meilisearch) con `marca_id`, `categoria_ids` y `tipo` filtrables; `synonyms` queda explícitamente fuera (T-03-27)

## Non-goals

Este change **no** cubre:

- Diccionario `synonyms` / tabla `sinonimo` → **T-03-27** (**No hace** del ticket; v2)
- Construcción del documento 4.11.1 (`build_search_document`, campos `sku`/`nombre`/`marca_id` en el JSON) → **T-03-06**
- Worker Index Sync, *fan-out* de `atributo_def`, `vortex reindex` → **T-03-07**, **T-03-08**, **T-03-11**
- Traducir `SearchFilters` / `SearchOrden` a sintaxis Meilisearch, `GET /search` → **T-03-12**, **T-03-13**
- Facetas HTTP/UI (`facets=true`, chips) → **T-03-18**, **T-03-19** (v2; HU-03-07 MVP solo deja los filtrables listos)
- Sugerencias / *facet search* sobre `tipo` → **T-03-20** (v2; este ticket solo marca `tipo` filtrable, H-04)
- Wiring en `bin/vortex` / arranque ordenado / crear el índice real `productos` al servir → **T-00-27**
- Consultar SurrealDB / `AttributeDefRepo` para listar claves facetables (el constructor recibe las claves; T-03-08 / T-00-27 las proveerán)
- Cambiar el puerto `TextSearch`, `openapi.yaml`, CI YAML, ni el probe T-09-05

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `vortex-search`: `apply_settings` aplica 4.11.2 + H-03/H-04 cuando `IndexSettings` no está vacío; el constructor de perfil `productos` arma las listas; test de integración lee y compara la config
- `domain-text-search`: `IndexSettings` deja de ser un placeholder vacío: transporta las listas de settings y sigue pudiendo construirse vacío (no-op)

## Impact

- **Repo:** `vortex-core/crates/domain/src/text_search.rs` (`IndexSettings`); `vortex-core/crates/search/` (constructor + `apply_settings` real + tests); `docs/4-modelo.md` § 4.11. No toca `openapi.yaml`, CI, `bin/vortex`, ni `storage`/`catalog`
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `cargo test -p search`; `cargo test --workspace -- --include-ignored meilisearch`; `openspec validate T-03-05`
- **Ticket:** cierra T-03-05 (Listo cuando: GET settings vs esperado); desbloquea T-03-07, T-03-12 y T-03-20 (este último v2)
- **Ámbito:** cabe en ~2 h (tipo + builder + apply + un ignore de GET settings + doc 4.11). No se divide el ticket
