# Tasks

## 1. Tests (red) — contenido y unhappy

- [x] 1.1 Agregar `seed_dev_dataset_has_expected_sku_count` y `seed_dev_dataset_has_rubro_attributes_oem_and_typo_fixtures` en `vortex-core/crates/catalog/src/seed.rs` (leen `seed/` del repo: 180–220 SKUs; claves `medida`/`largo`/`grado`/`material`/`rosca` usadas; OEM compartido; nombres tornillo/arandela/bujía; fixtures `"Tornillo autoperforante 10 x 1"`, `"Perno de Acero"`, `"Arandela para Perno"`; `stock_inicial` `"40"` y `"3"`; SKU `PER-M8X40-ZN`) — `cargo test -p catalog seed_dev_dataset_has_expected_sku_count seed_dev_dataset_has_rubro_attributes_oem_and_typo_fixtures` falla (ejemplo mínimo de T-02-40; escenarios happy de conteo, atributos, OEM y ranking)
- [x] 1.2 Agregar `seed_dev_dataset_has_relations_synonyms_notes_and_example_images` (existen `relaciones.jsonl` con los tres `tipo`, `sinonimos.jsonl` mutuo+unidireccional, 2–3 notas, `imagenes/<sku>/` con `.png`/`.jpg`/`.jpeg`/`.webp`) — `cargo test -p catalog seed_dev_dataset_has_relations_synonyms_notes_and_example_images` falla (archivos opcionales ausentes)
- [x] 1.3 Agregar `seed_dev_copy_rejects_non_numeric_precio`, `seed_dev_copy_rejects_unknown_relacion_sku` y `seed_dev_copy_rejects_unknown_imagen_sku` (copian `seed/` a temp, mutan, `validate_dir`; el `seed/` original sigue sin issues) — `cargo test -p catalog seed_dev_copy_rejects_` rojo o verde según exista el árbol; MUST quedar verde al cerrar el ticket (escenarios unhappy)

## 2. Dataset (green) — maestros y ~200 productos

- [x] 2.1 Ampliar `seed/categorias.jsonl`, `seed/marcas.jsonl` y `seed/atributos.jsonl` (árbol ferretería + repuestos; `fixser` y 2–3 marcas; claves `material` y `rosca` además de `medida`/`largo`/`grado`) — las referencias de 2.2 resuelven slug/clave; `cargo test -p catalog seed_rejects_unknown_categoria_slug seed_rejects_unknown_atributo_clave` sigue verde (directorio mínimo en temp, no el repo)
- [x] 2.2 Ampliar `seed/productos.jsonl` por familias combinatorias + fixtures nominales (180–220 SKUs únicos, conservar `PER-M8X40-ZN`, OEM compartido, nombres exactos de T-03-14, stocks `"40"` y `"3"`) — pasan 1.1; `cargo test -p catalog seed_repo_seed_dir_has_no_issues` verde (Listo cuando)

## 3. Dataset (green) — relaciones, sinónimos, notas, imágenes y README

- [x] 3.1 Crear `seed/relaciones.jsonl` (al menos `similar_a`, `sustituible_por`, `complementa` entre SKUs existentes), `seed/sinonimos.jsonl` (mutuo p. ej. bulón/perno; unidireccional p. ej. tarugo→taco) y `seed/notas.jsonl` (2 o 3 notas) — `validate_dir` sobre `seed/` sigue vacío
- [x] 3.2 Crear `seed/imagenes/PER-M8X40-ZN/principal.png` (PNG liviano válido; MAY repetir en 1–2 fixtures) — 1.2 verde; `cargo test -p catalog seed_dev_dataset_has_relations_synonyms_notes_and_example_images` verde
- [x] 3.3 Actualizar `seed/README.md`: quitar que el dataset «llegará» en T-02-41; documentar que `seed/` es el dataset de desarrollo (~200 SKUs) y que los archivos opcionales están poblados — escenario README del árbol; `grep` no encuentra la frase de aplazamiento

## 4. Tests unhappy en verde y regresiones T-02-40

- [x] 4.1 Completar copias temporales de 1.3 si hacía falta (crear `relaciones.jsonl` / `imagenes/` en la copia) — `cargo test -p catalog seed_dev_copy_rejects_` verde; `seed_repo_seed_dir_has_no_issues` verde (el repo no se muta)
- [x] 4.2 Conservar verdes `seed_valid_minimal_directory_has_no_issues` y `seed_rejects_non_numeric_precio_with_file_line_field` — `cargo test -p catalog seed_valid_minimal_directory_has_no_issues seed_rejects_non_numeric_precio_with_file_line_field seed_rejects_malformed_jsonl_line seed_rejects_unknown_categoria_slug seed_rejects_duplicate_sku seed_rejects_unknown_atributo_clave` verde (unhappy de T-02-40; directorio mínimo MAY omitir opcionales)

## 5. Verificación de cierre

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 5.2 `openspec validate T-02-41` verde
- [x] 5.3 `vortex seed` sigue imprimiendo `no implementado` — `cargo test -p vortex --test cli` (stub T-00-07 / T-02-42)