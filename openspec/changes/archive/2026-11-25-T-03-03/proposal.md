# Proposal

## Ticket padre

- **Ticket:** [T-03-03](../../../docs/7-tickets.md) — Puerto `TextSearch`
- **Change:** `T-03-03`
- **HU:** [HU-03-01](../../../docs/6-historias.md#hu-03-01--sincronización-del-índice-de-búsqueda)
- **Depende de:** T-00-03 (cerrado en `develop`; `DomainError` con `Unavailable` / `Timeout` / `InvalidInput`)
- **Hace:** *trait* en `domain` con `upsert_documents`, `delete_documents`, `search`, `facet_search`, `apply_settings`, `create_index`, `swap_indexes`, `delete_index` y `health`; tipos `SearchQuery`, `SearchFilters` y `SearchPage`
- **No hace:** implementación
- **Listo cuando:** compila y el crate `domain` sigue sin SDKs (T-00-02)

## Why

HU-03-01 y [3.3](../../../docs/3-componentes.md#33-puertos-y-adaptadores) exigen que Meilisearch quede detrás de un puerto hexagonal: el dominio no puede conocer `meilisearch-sdk`. T-03-04 (adaptador), T-03-07 (worker) y T-03-12 (caso de uso) no pueden inyectar ni testear un doble sin el *trait* y los tipos de consulta/página. Hay que publicarlos ahora, sin implementación.

## What Changes

- Exponer en `domain` el puerto `TextSearch` con los nueve métodos del ticket; errores `DomainError` (sin HTTP)
- Tipos de dominio `SearchQuery`, `SearchFilters` y `SearchPage` (nombres en español en los campos, alineados a [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) y a los filtros que T-03-12 traducirá)
- Tipos auxiliares mínimos para firmar el *trait* (`IndexUid`, documento por `id`, `IndexSettings` opaco, hit de página) **sin** el documento 4.11.1 (T-03-06) ni la config 4.11.2 (T-03-05)
- Dummy de tests en `domain` que implementa el puerto (mismo molde que `ProductRepo`)
- TDD: happy = construcción válida + dummy que satisface el *trait*; unhappy = query/filtros/índice inválidos y dummy que devuelve `Unavailable` / `Timeout` / `InvalidInput`
- Verificar **Listo cuando:** `cargo test -p domain` y `scripts/check-domain-deps.sh` en verde (`meilisearch-sdk` ausente del árbol de `domain`)

## Non-goals

Este change **no** cubre:

- Implementación con `meilisearch-sdk`, espera de *tasks* ni traducción conexión/`timeout`/documento inválido → **T-03-04** (**No hace** del ticket)
- Configuración del índice `productos` (`searchableAttributes`, `filterableAttributes`, `attr_*`) → **T-03-05**
- `build_search_document` / campos 4.11.1 (`marca_id`, `categoria_ids`, `imagen`) → **T-03-06**
- Worker Index Sync, fan-out, reindex `swapIndexes` real → **T-03-07**, **T-03-08**, **T-03-11**
- Caso de uso de búsqueda, `GET /search`, ranking, UI → **T-03-12**, **T-03-13**, **T-03-14**, **T-03-15**
- Uso de facetas en HTTP/UI (el método `facet_search` sí se declara) → **T-03-18** / **T-03-19** (v2)
- Búsqueda de respaldo en SurrealDB → **T-03-30** (v2)
- Crate `search` (sigue placeholder + probe T-09-05), `openapi.yaml`, `tracing`, UI
- Nuevas variantes o `code` de `DomainError` — **T-00-03** (solo se reutilizan)
- Relajar la allowlist de `domain` ni `scripts/check-domain-deps.sh`

## Capabilities

### New Capabilities

- `domain-text-search`: puerto `TextSearch` y tipos `SearchQuery` / `SearchFilters` / `SearchPage` en el crate `domain`, compilables sin SDKs ni adaptador Meilisearch

### Modified Capabilities

- (ninguna; `domain-deps` ya prohíbe `meilisearch-sdk` y no cambia de requisitos; este ticket solo debe seguir cumpliéndola)

## Impact

- **Repo:** `vortex-core/crates/domain/` (módulo nuevo, reexports en `lib.rs`). No toca `crates/search`, `meilisearch-sdk`, `docs/openapi.yaml` ni CI
- **APIs / OpenAPI:** ninguna (el puerto no es HTTP; T-03-13 mapeará a `SearchResponse`)
- **CI / verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `scripts/check-domain-deps.sh`; `openspec validate T-03-03`
- **Ticket:** cierra T-03-03 (Listo cuando: compila y `domain` sin SDKs); desbloquea T-03-04
- **Ámbito:** cabe en ~2 h (puerto + tipos + tests unitarios). No se divide el ticket
