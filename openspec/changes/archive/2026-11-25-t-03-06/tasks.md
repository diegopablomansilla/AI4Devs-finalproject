# Tasks

## 1. Tests (red) — snapshot Listo cuando

- [x] 1.1 Agregar `build_search_document_snapshot_4_4_4` en `domain` — falla hasta existir builder y campos 4.11.1

## 2. Tests (red) — happy path

- [x] 2.1 `build_search_document_formats_ubicacion`, `build_search_document_atributos_texto_only_buscable`, `build_search_document_attr_keys_only_facetable`, `build_search_document_imagen_is_null`, `build_search_document_categoria_ids_ancestors_and_self`, `build_search_document_con_stock_false_when_zero`, `build_search_document_without_marca`

## 3. Tests (red) — unhappy path

- [x] 3.1 `build_search_document_rejects_marca_mismatch`, `build_search_document_rejects_categoria_mismatch`, `build_search_document_rejects_missing_atributo_def`

## 4. Implementación (green) — SearchDocument + builder

- [x] 4.1 Módulo `search_document.rs`: ampliar `SearchDocument`, `build_search_document`, serialización JSON; export en `lib.rs`; `try_new` compatible

## 5. Refactor y regresión

- [x] 5.1 `cargo fmt`, `clippy -D warnings`, `cargo test --workspace`, `check-domain-deps.sh`

## 6. Cierre

- [x] 6.1 Non-goals: sin cambios en `search/meilisearch.rs` upsert body, sin worker, sin openapi
- [x] 6.2 `openspec validate t-03-06`