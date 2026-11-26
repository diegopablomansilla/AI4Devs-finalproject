# Tasks

## 1. Tests (red) — orquestador de relaciones en `catalog`

- [x] 1.1 Agregar `seed_import_relations_creates_complement_edge` y `seed_import_relations_similar_creates_bidirectional_pair` en `vortex-core/crates/catalog/src/seed.rs` (fixture con dos SKUs + fake del puerto `create`; asertar arista `complementa` unidireccional y par `similar_a`; resumen `importados`/`omitidas`/`rechazados`) — `cargo test -p catalog seed_import_relations_creates_complement_edge seed_import_relations_similar_creates_bidirectional_pair` falla (API inexistente; escenarios happy de primera carga)
- [x] 1.2 Agregar `seed_import_relations_second_run_omits_existing`, `seed_import_relations_dev_dataset_seeds_panel_edges` y `seed_import_relations_missing_file_is_ok` — `cargo test -p catalog seed_import_relations_second_run_omits_existing seed_import_relations_dev_dataset_seeds_panel_edges seed_import_relations_missing_file_is_ok` falla (idempotencia / Hace; Listo cuando dataset: `PER-M8X40-ZN`→`ARA-PERNO-01`; archivo ausente = no-op)
- [x] 1.3 Agregar `seed_import_relations_rejects_unknown_sku_only`, `seed_import_relations_rejects_malformed_line_only`, `seed_import_relations_rejects_self_relation_only` y `seed_import_relations_does_not_persist_synonyms_notes_images` — `cargo test -p catalog seed_import_relations_rejects_ seed_import_relations_does_not_persist_synonyms_notes_images` falla (unhappy: issue `archivo`/`linea`/`campo`; la arista válida queda; la inválida no; No hace sinónimos/notas/imágenes)

## 2. Implementación (green) — `import_relations`

- [x] 2.1 Exponer puerto fino de alta (`create(NewRelacion)`) y `SeedRelationSummary`; parsear `relaciones.jsonl`; devolver issues sin llamar a `create` si hay parseo inválido, SKU ausente o `origen == destino`, o si falta el JSONL (no-op) — pasan 1.3 y `seed_import_relations_missing_file_is_ok`; `cargo test -p catalog seed_import_relations_rejects_ seed_import_relations_missing_file_is_ok` verde
- [x] 2.2 Completar `import_relations`: mapa SKU→id vía `ProductGateway::list` (activo true y false); un `create` por línea válida; `duplicate_relation` → `omitidas`; `prioridad` default 1; `bidireccional` `None` si se omite; dataset `seed/` del repo — pasan 1.1 y 1.2; `cargo test -p catalog seed_import_relations_` verde (Listo cuando in-memory)
- [x] 2.3 Reexportar `import_relations` / `SeedRelationSummary` desde `catalog`; conservar verdes de maestros, productos y `validate_dir` — `cargo test -p catalog seed_import_masters_ seed_import_products_ seed_valid_minimal_directory_has_no_issues seed_repo_seed_dir_has_no_issues` verde

## 3. Tests (red) — CLI `vortex seed` con relaciones

- [x] 3.1 Extender el fixture mínimo de `vortex-core/bin/vortex/tests/seed.rs` con `relaciones.jsonl` opcional (Perno→Arandela `complementa`) y agregar `vortex_seed_imports_relations_exit_0` y `vortex_seed_twice_does_not_duplicate_relations` (reabrir `Db`; `RelacionRepo::list_for_product` del SKU origen; contar aristas) — `cargo test -p vortex --test seed vortex_seed_imports_relations_exit_0 vortex_seed_twice_does_not_duplicate_relations` falla (aún no persiste relaciones; escenarios happy CLI + omitir existentes)
- [x] 3.2 Agregar `vortex_seed_dev_dataset_persists_panel_relations` (DIR = `seed/` del repo, `PER-M8X40-ZN` tiene complementario `ARA-PERNO-01`, exit 0, `rechazados` 0) y `vortex_seed_invalid_relation_sku_rejected_alone` (copia del dataset + `destino` desconocido: exit ≠ 0, issue `relaciones.jsonl`/`destino`, aristas válidas persistidas, la inválida no) — `cargo test -p vortex --test seed vortex_seed_dev_dataset_persists_panel_relations vortex_seed_invalid_relation_sku_rejected_alone` falla (Listo cuando CLI happy + unhappy)

## 4. Implementación (green) — despacho CLI y README

- [x] 4.1 Tras `import_products`, construir `RelationService`/`RelacionRepo`, adaptar al puerto de `catalog` y llamar `import_relations`; imprimir issues y resumen `importados`/`omitidas`/`rechazados`; exit 0 iff productos y relaciones tienen `rechazados == 0`; si maestros fallan, no importar relaciones — pasan 3.1 y 3.2; `cargo test -p vortex --test seed --test cli` verde; lock/config unhappy de T-02-42 siguen ≠ 0; fixture sin `relaciones.jsonl` sigue exit 0
- [x] 4.2 Actualizar `seed/README.md`: importación MVP = maestros, productos **y** relaciones; imágenes/sinónimos/notas siguen pendientes — escenario README de `seed-dataset`; el README no afirma que las relaciones quedan para un ticket posterior

## 5. Verificación de cierre

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 5.2 `openspec validate T-04-11` verde
- [ ] 5.3 Listo cuando humano: `vortex seed` del `seed/` del repo, `vortex serve`, abrir un resultado de `PER-M8X40-ZN` y comprobar que el panel muestra el complementario `ARA-PERNO-01` (T-04-08; no automatizar Playwright)