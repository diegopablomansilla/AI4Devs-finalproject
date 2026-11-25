# Tasks

## 1. Tests (red) — `IndexSettings` en domain

- [x] 1.1 Agregar test `index_settings_constructs_with_lists` en `vortex-core/crates/domain/` (listas `searchable_attributes` / `filterable_attributes` no vacías; payload no vacío) — `cargo test -p domain index_settings_constructs_with_lists` no compila o falla (escenario «IndexSettings con listas»)
- [x] 1.2 Conservar `index_settings_constructs_empty` — el test existente sigue compilando contra el placeholder; al verde de 1.3 MUST seguir pasando (escenario «IndexSettings vacío»)

## 2. Implementación (green) — payload `IndexSettings`

- [x] 2.1 Ampliar `domain::IndexSettings` a listas (`searchable_attributes`, `ranking_rules`, `filterable_attributes`, `sortable_attributes`, `typo_disable_on_attributes`); `Default` + `is_empty()`; quitar `Copy`; dummy y `apply_settings(..., IndexSettings)` usan `IndexSettings::default()` — `cargo test -p domain index_settings` sale 0 (escenarios vacío y con listas)
- [x] 2.2 `./scripts/check-domain-deps.sh` sale 0 — `domain` sigue sin `meilisearch-sdk` (escenario «check-domain-deps sigue verde»)

## 3. Tests (red) — perfil `productos` (happy + unhappy)

- [x] 3.1 Agregar tests de unidad `productos_index_settings_includes_4_11_2_h03_h04` y `productos_index_settings_omits_synonyms` en `vortex-core/crates/search/` — `cargo test -p search productos_index_settings_includes productos_index_settings_omits` no compila o falla (escenarios «Perfil productos con un atributo facetable», «Perfil sin claves facetables» / omite synonyms)
- [x] 3.2 Agregar test de unidad `productos_index_settings_rejects_invalid_clave` — `cargo test -p search productos_index_settings_rejects_invalid_clave` no compila o falla (escenario «Clave facetable inválida»)

## 4. Implementación (green) — constructor del perfil

- [x] 4.1 Implementar `search::productos_index_settings` (constantes 4.11.2 + `marca_id` / `categoria_ids` / `tipo` + `attr_<clave>`; validar patrón `AtributoDef.clave`; rechazar prefijo `attr_`; cero claves válido; sin `synonyms`) y reexportarlo — `cargo test -p search productos_index_settings` sale 0 (escenarios del perfil; happy + unhappy)

## 5. Tests (red) — `apply_settings` contra el motor

- [x] 5.1 Agregar test `#[ignore = "meilisearch"]` `meilisearch_apply_productos_settings_matches_expected` (índice efímero ≠ `productos`, aplica perfil con `medida`, GET settings y compara) — `cargo test -p search -- --include-ignored meilisearch_apply_productos_settings_matches_expected` no compila o falla (escenario «apply_settings aplica el perfil productos y GET coincide»; **Listo cuando**)
- [x] 5.2 Agregar test **no** ignore `meilisearch_apply_settings_unavailable_when_unreachable` (payload no vacío, URL/puerto cerrado) — `cargo test -p search meilisearch_apply_settings_unavailable_when_unreachable` no compila o falla (escenario «apply_settings con motor inalcanzable»)

## 6. Implementación (green) — PATCH settings

- [x] 6.1 `MeiliTextSearch::apply_settings`: vacío → `Ok(())` sin HTTP; no vacío → `set_settings` (sin `synonyms` ni `attributesToHighlight`) + espera de *task* — `cargo test -p search meilisearch_apply_settings_unavailable_when_unreachable` sale 0; `cargo test -p search -- --include-ignored meilisearch_apply_productos_settings_matches_expected` sale 0 (Listo cuando + unhappy inalcanzable)
- [x] 6.2 Conservar `tests/meilisearch_health.rs` (probe T-09-05) y los tests T-03-04 — `git diff -- vortex-core/crates/search/tests/meilisearch_health.rs` vacío; `cargo test -p search -- --include-ignored meilisearch_adapter` sale 0 si el sidecar está arriba

## 7. Documentación 4.11

- [x] 7.1 Actualizar [docs/4-modelo.md](../../../docs/4-modelo.md) § 4.11: ejemplo 4.11.1 con `marca_id` y `categoria_ids`; 4.11.2 `filterableAttributes` incluye `marca_id`, `categoria_ids`, `tipo` y `attr_*`; `synonyms` remitido a T-03-27; resaltado remitido a la consulta (T-03-12) — el diff de 4.11 nombra esos campos (H-03, H-04)

## 8. Cierre (regresión y non-goals)

- [x] 8.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «La suite por defecto no exige el sidecar»)
- [x] 8.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 8.3 Confirmar non-goals: sin `synonyms` en PATCH, sin `build_search_document`, sin wiring en `bin/vortex`, sin `AttributeDefRepo`, sin cambios en `.github/workflows/ci.yml` ni `docs/openapi.yaml` — `git diff --name-only` sin esos paths de código
- [x] 8.4 `openspec validate T-03-05` sin errores