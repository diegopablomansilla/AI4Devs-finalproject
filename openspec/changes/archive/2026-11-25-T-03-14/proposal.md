# Proposal

## Ticket padre

- **Ticket:** [T-03-14](../../../docs/7-tickets.md) — Tests de ranking y tolerancia a errores
- **Change:** `T-03-14`
- **HU:** [HU-03-03](../../../docs/6-historias.md#hu-03-03--búsqueda-mientras-se-escribe-tolerante-a-errores), [HU-03-04](../../../docs/6-historias.md#hu-03-04--ranking-de-relevancia)
- **Depende de:** T-03-13 (en `develop`: `GET /api/v1/search` + `SearchGateway`); T-02-41 (dataset de desarrollo con nombres y stocks de las HU)
- **Hace:** tests de integración con Meilisearch real y datos fijos: "tornilo" encuentra "Tornillo autoperforante 10 x 1"; "Perno" ordena "Perno de Acero" antes que "Arandela para Perno"; ante igual relevancia, primero stock 40 y luego stock 3.
- **No hace:** *benchmarks* de latencia (T-09-07).
- **Listo cuando:** los tres escenarios pasan en CI.

## Why

HU-03-03 y HU-03-04 prometen typo ("tornilo" → tornillo) y ranking (`nombre`/`tipo` antes que coincidencia secundaria; desempate `stock_disponible:desc`). T-03-05 configuró el índice 4.11.2 y T-03-12/T-03-13 ya consultan el motor, pero ningún test contra Meilisearch real cierra esos escenarios Gherkin. Sin esa red, un cambio de settings o de versión del sidecar puede romper el ranking y CI no lo vería.

## What Changes

- Tests de integración `#[ignore = "meilisearch"]` en el crate `search` contra un índice efímero (no `productos`) con documentos fijos alineados a T-02-41
- Tres escenarios Listo cuando: typo `"tornilo"`; orden `"Perno"`; desempate stock 40 antes que 3
- Unhappy path del contrato 4.11.2: typo en `sku`/`codigos` no matchea; consulta sin hits devuelve página vacía (no `DomainError`)
- El job CI `meilisearch` de T-09-05 ya ejecuta `--include-ignored meilisearch`; no se cambia el workflow

## Non-goals

- *Benchmarks* de latencia, p95 &lt; 100 ms, catálogo de 10 k SKUs — **T-09-07**, **T-09-08** (HU-03-03 escenario Latencia)
- UI Omnibox / `AbortController` / resaltado seguro / tarjetas — **T-03-15**, **T-03-16**, **T-03-17**
- Facetas, sugerencias, búsqueda por código, sinónimos, respaldo SurrealDB — **T-03-18**, **T-03-20**, **T-03-22**, **T-03-24**, **T-03-30** (v2 salvo by-code)
- Reabrir settings de índice, worker, `build_search_document`, handler HTTP o `openapi.yaml` — ya cerrados en T-03-05 / T-03-09 / T-03-06 / T-03-13
- Sembrar el dataset completo (~200 SKUs) ni `vortex seed` / SurrealDB en estos tests
- Tests HTTP `GET /api/v1/search` con sidecar (el mapper ya está cubierto con fake en T-03-13)
- Cambiar el job CI `meilisearch` o el pin de versión (T-09-05)

## Capabilities

### New Capabilities

- (ninguna; el ranking observable es comportamiento del adaptador + perfil `productos`, ya cubierto por `vortex-search`)

### Modified Capabilities

- `vortex-search`: exigir tests de integración contra Meilisearch real que demuestran typo, ranking de atributo y desempate por stock con los fixtures de HU-03-03 / HU-03-04, más unhappy path de typo desactivado en códigos y consulta sin hits

## Impact

- `vortex-core/crates/search/tests/` (nuevo archivo de ranking; reusa `MeiliTextSearch` + `productos_index_settings`)
- `openspec/specs/vortex-search`
- Sin cambios de API, OpenAPI, allowlist, CI yaml ni seed
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test --workspace` (ignore omitidos); `cargo test --workspace -- --include-ignored meilisearch`; `openspec validate T-03-14`
- Ticket: cierra T-03-14 (Listo cuando: los tres escenarios en el job `meilisearch`)
- Ámbito: solo tests + delta de spec; cabe en ~2 h. No se divide el ticket
