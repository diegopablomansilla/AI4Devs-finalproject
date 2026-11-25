# Proposal

## Ticket padre

- **Ticket:** [T-03-12](../../../docs/7-tickets.md) — Caso de uso de búsqueda por texto
- **Change:** `T-03-12`
- **HU:** [HU-03-03](../../../docs/6-historias.md#hu-03-03--búsqueda-mientras-se-escribe-tolerante-a-errores)
- **Depende de:** T-03-05 (en `develop`: índice `productos` con 4.11.2 + H-03/H-04; `SearchQuery` / `SearchFilters` / `TextSearch`; adaptador que hoy solo envía `q`/`offset`/`limit`)
- **Hace:** en `search`: arma la consulta con `q`, filtros (`categoria_ids`, `marca_id`, `con_stock`, `attr_<clave>`), `sort`, `offset` ≤ 1000 y `limit` ≤ 50; pide `_formatted` para el resaltado.
- **No hace:** facetas (T-03-18) ni respaldo (T-03-30).
- **Listo cuando:** tests con un `TextSearch` falso verifican la traducción de cada filtro.

## Why

HU-03-03 exige resultados en cada tecla con filtros y resaltado. T-03-03 dejó `SearchQuery`/`SearchFilters`; T-03-04/T-03-05 dejaron el adaptador y el índice listos, pero `MeiliTextSearch::search` ignora filtros, `orden` y `_formatted`. Sin el caso de uso que arma esa consulta y sin traducir cada filtro a Meilisearch, T-03-13 no puede exponer `GET /search` sobre un hexágono testeable.

## What Changes

- Caso de uso en el crate `search` (`SearchService`) que construye `SearchQuery` (límites de OpenAPI ya validados en dominio) y llama `TextSearch::search` sobre el uid `productos` (inyectado)
- Traducción de cada filtro de dominio a sintaxis Meilisearch: `categoria_ids`, `marca_id`, `con_stock`, `attr_<clave>` (la clave de `AtributoDef` sin prefijo en `SearchFilters`; el adaptador antepone `attr_`)
- Traducción de `SearchOrden` → `sort` del motor (`relevancia` = sin `sort`; `precio_asc`/`precio_desc`/`stock_desc` sobre `precio` / `stock_disponible`)
- Pedir resaltado en la consulta: `attributesToHighlight` = `nombre`, `descripcion` y mapear `_formatted` → `SearchHit.resaltado` ([4.11.2](../../../docs/4-modelo.md#4112-configuración-del-índice))
- Hidratar `SearchHit` con los campos 4.11.1 del documento (sku, nombre, stock, etc.) para que T-03-13 sea un mapper HTTP delgado
- Tests con `TextSearch` falso (Listo cuando) que registran el `SearchQuery` recibido y verifican cada filtro; unitarios puros de la traducción a filtro/sort/_formatted **sin** sidecar
- Fachada object-safe `SearchGateway` para inyección en T-03-13
- Errores de dominio sin remapeo HTTP (`invalid_input`, `unavailable`, `timeout`)

## Non-goals

- HTTP `GET /api/v1/search`, DTO inglés (`categoryId`, `brandId`, `inStock`, `attr[clave]`, `sort=relevance`), `openapi.yaml`, `assert_conforms`, `problem+json`, *timeout* de gateway 2 s, cancelación Axum, métrica `search_duration_seconds` — **T-03-13**
- Facetas (`facets=true`, `SearchFacets`, distribución) — **T-03-18** (v2)
- Búsqueda de respaldo en SurrealDB / `degraded: true` — **T-03-30** (v2)
- Tests de ranking contra Meilisearch real ("tornilo", "Perno", desempate por stock) — **T-03-14**
- UI Omnibox / resaltado seguro / tarjetas — **T-03-15**, **T-03-16**, **T-03-17**
- Sugerencias, búsqueda por código, sinónimos — **T-03-20**, **T-03-22**, **T-03-24** (v2 salvo by-code)
- Cambiar settings de índice, worker, `build_search_document` o `productos_index_settings` — ya cerrados
- `synonyms` / `attributesToHighlight` como setting de índice
- Spans de `tracing` en `search` (el Gateway instrumenta en T-03-13)
- Relajar allowlists de `search` / `domain`

## Capabilities

### New Capabilities

- (ninguna; el caso de uso y la traducción de consulta viven en el crate `search`, ya cubierto por `vortex-search`)

### Modified Capabilities

- `vortex-search`: el adaptador traduce `SearchFilters` / `SearchOrden` a filtro y `sort` de Meilisearch, pide `_formatted` (`nombre`, `descripcion`) y mapea hits 4.11.1 + resaltado; el crate expone el caso de uso `SearchService` + `SearchGateway` testeable con `TextSearch` falso
- `domain-text-search`: `SearchHit` incorpora campos 4.11.1 y `resaltado` opcional; `SearchFilters` rechaza claves de atributo inválidas o con prefijo `attr_`

## Impact

- `vortex-core/crates/search/` (caso de uso, traductor de consulta, `MeiliTextSearch::search`); `vortex-core/crates/domain/src/text_search.rs` (`SearchHit` / `SearchFilters`)
- Tests de construcción de `SearchHit` existentes en `domain` se actualizan al nuevo shape
- Sin HTTP, sin `openapi.yaml`, sin CI nueva, sin sidecar obligatorio (Listo cuando = falso)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-03-12 (Listo cuando: tests con `TextSearch` falso verifican cada filtro)
- Ámbito: orquestación + traducción pura; cabe en ~2 h. No se divide el ticket
