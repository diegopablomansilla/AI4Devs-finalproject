# Tasks

## 1. Allowlist de `catalog` (`serde` / `serde_json`)

- [x] 1.1 En `scripts/check-workspace.sh`, `catalog_deps_allowed` acepta `serde` y `serde_json` (`workspace = true`) en `[dependencies]` además de `domain`; `surrealdb` / `clap` siguen vetados — `./scripts/check-workspace.sh` verde con el `Cargo.toml` actual (solo `domain`)
- [x] 1.2 En `scripts/fixtures/workspace/run-script-tests.sh` agregar happy `catalog con serde y serde_json` y conservar unhappy `catalog con surrealdb` — `./scripts/fixtures/workspace/run-script-tests.sh` verde (escenarios `catalog declara serde y serde_json` y `catalog declara una dependencia fuera de la allowlist`)

## 2. Tests (red) — validador

- [x] 2.1 Agregar `seed_valid_minimal_directory_has_no_issues` y `seed_rejects_non_numeric_precio_with_file_line_field` (precio `"barato"` → `archivo` `productos.jsonl`, `linea` 1, `campo` `precio`) — `cargo test -p catalog seed_valid_minimal_directory_has_no_issues seed_rejects_non_numeric_precio_with_file_line_field` falla (API inexistente; Listo cuando / HU-02-11)
- [x] 2.2 Agregar `seed_rejects_malformed_jsonl_line`, `seed_rejects_unknown_categoria_slug`, `seed_rejects_duplicate_sku` y `seed_rejects_unknown_atributo_clave` — `cargo test -p catalog seed_rejects_` falla (escenarios unhappy de `seed-dataset`)

## 3. Implementación (green) — validador

- [x] 3.1 Declarar `serde` (feature `derive`) y `serde_json` en `vortex-core/crates/catalog/Cargo.toml` (`workspace = true`) y módulo `seed` exportado — `./scripts/check-workspace.sh` verde; `cargo test -p catalog seed_rejects_non_numeric_precio_with_file_line_field` sigue en rojo hasta 3.2
- [x] 3.2 Implementar `validate_dir`: JSONL, `Decimal` para `precio`, referencias slug/clave/SKU, issues `{ archivo, linea, campo, mensaje }`, sin I/O SurrealDB — pasan 2.1 y 2.2; `vortex seed` sigue imprimiendo `no implementado` (`cargo test -p vortex --test cli`)

## 4. Tests + árbol `seed/` del repositorio

- [x] 4.1 Agregar `seed_repo_seed_dir_has_no_issues` que valida `<repo>/seed` — `cargo test -p catalog seed_repo_seed_dir_has_no_issues` falla (directorio ausente o incompleto)
- [x] 4.2 Crear `seed/README.md` (esquema completo, archivos opcionales v2) y JSONL mínimos (`categorias`, `marcas`, `atributos`, `productos` con `PER-M8X40-ZN` / `precio` `"120.50"`) — pasa 4.1 (escenarios happy `seed/` tiene README y JSONL maestros, ejemplo mínimo, `seed/` del repositorio)

## 5. Verificación de cierre

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 5.2 `./scripts/check-workspace.sh` verde
- [x] 5.3 `openspec validate T-02-40` verde