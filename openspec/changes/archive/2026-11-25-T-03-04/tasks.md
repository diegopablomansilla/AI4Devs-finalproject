# Tasks

## 1. Allowlist de `search` (scaffolding)

- [x] 1.1 Agregar `search_deps_allowed` en `scripts/check-workspace.sh` (path `domain`, `meilisearch-sdk`, `tokio`, `serde`, `serde_json`; dev solo `tokio`; rechazar `reqwest`/`surrealdb` directos) y exceptuar `search` del `else` «no debe declarar [dependencies]» — `./scripts/check-workspace.sh` sobre el repo actual sigue saliendo 0 (escenario «Un lib declara dependencias» actualizado)
- [x] 1.2 Extender `scripts/fixtures/workspace/run-script-tests.sh`: happy allowlist de `search`; unhappy `search` con `surrealdb` o `reqwest`; el escenario de lib vacío usa `recommend` en vez de `search` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0 (escenarios «search declara la allowlist Meilisearch», «search declara una dependencia fuera de la allowlist», «Un lib distinto de domain declara dependencias»)
- [x] 1.3 Declarar en `vortex-core/crates/search/Cargo.toml` las deps de la allowlist (`domain` path, `meilisearch-sdk`, `tokio`+`time`, `serde`/`serde_json`; dev `tokio` macros/rt-multi-thread) — `./scripts/check-workspace.sh` sale 0; `cargo metadata -p search` lista `meilisearch-sdk`

## 2. Tests (red) — happy

- [x] 2.1 Agregar tests `#[ignore = "meilisearch"]` `meilisearch_adapter_upserts_then_searches_document` y `meilisearch_adapter_health_is_up` en `vortex-core/crates/search/tests/meilisearch_adapter.rs` — `cargo test -p search -- --include-ignored meilisearch_adapter_upserts` no compila o falla (escenarios «Alta y consulta de un documento», «health up»; **Listo cuando**)
- [x] 2.2 Agregar test `#[ignore = "meilisearch"]` `meilisearch_adapter_deletes_document` — `cargo test -p search -- --include-ignored meilisearch_adapter_deletes` no compila o falla (escenario «Borrado de un documento»; **Listo cuando**)

## 3. Tests (red) — unhappy

- [x] 3.1 Agregar test **no** ignore `meilisearch_adapter_unavailable_when_unreachable` (cliente a URL/puerto cerrado) — `cargo test -p search meilisearch_adapter_unavailable_when_unreachable` no compila o falla (escenario «Motor detenido o inalcanzable»; **Listo cuando**)
- [x] 3.2 Agregar tests de unidad `meilisearch_error_maps_timeout` y `meilisearch_error_maps_invalid_document` — `cargo test -p search meilisearch_error_maps` no compila o falla (escenarios «Timeout de task», «Documento inválido»)

## 4. Implementación (green) — mapper y health

- [x] 4.1 Implementar `map_meili_error` (conexión → `unavailable`, timeout → `timeout`, documento inválido → `invalid_input`) y `MeiliTextSearch::health` — `cargo test -p search meilisearch_error_maps meilisearch_adapter_unavailable_when_unreachable` sale 0 (escenarios unhappy de traducción; Listo cuando inalcanzable)
- [x] 4.2 `health` contra motor real: `ComponentStatus::Up` cuando `/health` está available — `cargo test -p search -- --include-ignored meilisearch_adapter_health_is_up` sale 0 (escenario «health up»)

## 5. Implementación (green) — documentos y *trait*

- [x] 5.1 Implementar `upsert_documents` + `search` (DTO `{ id }`, `wait_for_task` con `tokio::time::timeout` 5 s, `q`/`offset`/`limit`) y `create_index`/`delete_index` para el índice de prueba — `cargo test -p search -- --include-ignored meilisearch_adapter_upserts_then_searches_document` sale 0 (escenario «Alta y consulta»; **Listo cuando**)
- [x] 5.2 Implementar `delete_documents` con espera de *task* — `cargo test -p search -- --include-ignored meilisearch_adapter_deletes_document` sale 0 (escenario «Borrado»; **Listo cuando**)
- [x] 5.3 Completar `TextSearch`: `facet_search` delega en `search`; `apply_settings` vacío es `Ok(())` sin 4.11.2; `swap_indexes` usa `validate_index_swap` + SDK — `cargo test -p search -- --include-ignored meilisearch` sale 0; el crate compila el `impl TextSearch` (escenario «apply_settings vacío no configura 4.11.2»)
- [x] 5.4 Conservar `tests/meilisearch_health.rs` (probe T-09-05) — `git diff -- vortex-core/crates/search/tests/meilisearch_health.rs` vacío

## 6. Cierre (regresión y non-goals)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «La suite por defecto no exige el sidecar»)
- [x] 6.2 `./scripts/check-domain-deps.sh` sale 0 (`domain` sigue sin `meilisearch-sdk`); `./scripts/check-workspace.sh` sale 0
- [x] 6.3 Confirmar non-goals: sin 4.11.2, sin wiring en `bin/vortex`, sin cambios en `.github/workflows/ci.yml` ni `docs/openapi.yaml` — `git diff --name-only` sin esos paths
- [x] 6.4 `openspec validate T-03-04` sin errores