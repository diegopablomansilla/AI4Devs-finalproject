# Proposal

## Ticket padre

- **Ticket:** [T-03-04](../../../docs/7-tickets.md) — Adaptador Meilisearch base
- **Change:** `T-03-04`
- **HU:** [HU-03-01](../../../docs/6-historias.md#hu-03-01--sincronización-del-índice-de-búsqueda)
- **Depende de:** T-03-03, T-09-05 (cerrados en `develop`: puerto `TextSearch` en `domain`; job CI `meilisearch` con `--include-ignored`)
- **Hace:** implementación en `search` con `meilisearch-sdk`, espera de *tasks* con *timeout*, `health()` y traducción de errores (conexión → `Unavailable`, *timeout* → `Timeout`, documento inválido → `InvalidInput`)
- **No hace:** configuración del índice (T-03-05)
- **Listo cuando:** tests de integración de alta, borrado y consulta de documentos, y del error con Meilisearch detenido

## Why

HU-03-01 y [3.3](../../../docs/3-componentes.md#33-puertos-y-adaptadores) exigen un adaptador Meilisearch detrás del puerto `TextSearch`, con `health()` y errores de dominio tipados. T-03-03 dejó el *trait* sin implementación; T-09-05 dejó el motor en CI y un probe TCP. Sin el adaptador, T-03-05 (settings 4.11.2), T-03-07 (worker) y T-03-12 (caso de uso) no pueden hablar con Meilisearch.

## What Changes

- Implementar `TextSearch` en el crate `search` con `meilisearch-sdk` `=0.33.0` (pin de workspace): alta (`upsert_documents`), borrado (`delete_documents`), consulta (`search`), `health()`, espera de *tasks* con *timeout*, y el resto de métodos del *trait* como envoltorio del SDK (sin 4.11.2)
- Traducir errores del SDK: conexión / motor inalcanzable → `DomainError` `unavailable`; espera de *task* agotada → `timeout`; documento rechazado por el motor → `invalid_input`
- Abrir la allowlist de `cargo-workspace` para que `search` pueda declarar `domain`, `meilisearch-sdk`, `tokio` y `serde`/`serde_json`
- Tests de integración `#[ignore = "meilisearch"]` contra el motor real (alta, consulta, borrado, `health` *up*) y unhappy de motor caído / inalcanzable → `unavailable` (Listo cuando)
- TDD: mapper unitario de Timeout / InvalidInput; happy path contra Meilisearch de T-09-05; `cargo test --workspace` sin `--include-ignored` sigue verde sin el sidecar

## Non-goals

Este change **no** cubre:

- Configuración del índice `productos` (`searchableAttributes`, `filterableAttributes`, `rankingRules`, `attr_*`) → **T-03-05** (**No hace** del ticket)
- `build_search_document` / campos 4.11.1 (`sku`, `nombre`, `marca_id`, …) → **T-03-06**
- Worker Index Sync, *fan-out*, reindex `swapIndexes` de producto → **T-03-07**, **T-03-08**, **T-03-11**
- Caso de uso de búsqueda, traducción de filtros HTTP, `GET /search` → **T-03-12**, **T-03-13**
- Wiring en `bin/vortex` / supervisor / arranque ordenado del sidecar → **T-00-27**
- Facetas HTTP/UI, sinónimos, respaldo SurrealDB → **T-03-18**, **T-03-27**, **T-03-30** (v2)
- Cambiar el puerto `TextSearch`, `DomainError`, o relajar `scripts/check-domain-deps.sh`
- Quitar el probe TCP de T-09-05 (`meilisearch_health_is_available`)
- `openapi.yaml`, UI, métricas del worker (T-03-09)

## Capabilities

### New Capabilities

- `vortex-search`: adaptador Meilisearch en el crate `search` que implementa `TextSearch` (espera de *tasks* con *timeout*, `health()`, traducción de errores; tests de alta/borrado/consulta y motor inalcanzable)

### Modified Capabilities

- `cargo-workspace`: el crate `search` MAY declarar `[dependencies]` / `[dev-dependencies]` según una allowlist (path `domain`, `meilisearch-sdk`, `tokio`, `serde`, `serde_json`); los demás libs vacíos siguen prohibidos

## Impact

- **Repo:** `vortex-core/crates/search/` (adaptador + tests); `scripts/check-workspace.sh` y fixtures `scripts/fixtures/workspace/`; `openspec/specs/cargo-workspace/spec.md` vía archive. No toca `domain` salvo reutilizar el puerto; no toca `docs/openapi.yaml` ni CI YAML (el job `meilisearch` de T-09-05 ya corre `--include-ignored meilisearch`)
- **APIs / OpenAPI:** ninguna (el adaptador no es HTTP)
- **CI / verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p search`; `cargo test --workspace -- --include-ignored meilisearch`; `./scripts/check-workspace.sh`; `./scripts/check-domain-deps.sh`; `openspec validate T-03-04`
- **Ticket:** cierra T-03-04 (Listo cuando: integración de alta, borrado, consulta y error con motor detenido); desbloquea T-03-05
- **Ámbito:** cabe en ~2 h (adaptador del *trait* + mapper + tests ignore + allowlist). No se divide el ticket
