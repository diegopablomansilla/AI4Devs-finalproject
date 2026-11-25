# Tasks

## 1. Tests (red) — orquestador de maestros en `catalog`

- [x] 1.1 Agregar `seed_import_masters_creates_categoria_marca_atributo` y `seed_import_masters_does_not_persist_productos` en `vortex-core/crates/catalog/src/seed.rs` (directorio temp con `fijaciones`/`pernos`, `fixser`, `medida`; gateways in-memory; `productos.jsonl` presente) — `cargo test -p catalog seed_import_masters_creates_categoria_marca_atributo seed_import_masters_does_not_persist_productos` falla (API inexistente; escenarios happy de primera carga y No hace productos)
- [x] 1.2 Agregar `seed_import_masters_second_run_does_not_duplicate` y `seed_import_masters_second_run_updates_nombre` (misma base in-memory, segundo JSONL con `nombre` distinto en `fixser`) — `cargo test -p catalog seed_import_masters_second_run` falla (Listo cuando + update por slug)
- [x] 1.3 Agregar `seed_import_masters_rejects_malformed_categoria_jsonl_without_writes`, `seed_import_masters_rejects_unknown_padre_without_writes` y `seed_import_masters_rejects_missing_marcas_file_without_writes` — `cargo test -p catalog seed_import_masters_rejects_` falla (escenarios unhappy: cero escrituras, `archivo`/`linea`/`campo`)

## 2. Implementación (green) — `import_masters`

- [x] 2.1 Ampliar parsers de `categorias.jsonl` / `marcas.jsonl` / `atributos.jsonl` para conservar `nombre` y campos de `AtributoDef`; implementar `import_masters` (parseo de los tres archivos, issues, return sin tocar gateways si hay errores) — pasan 1.3; `cargo test -p catalog seed_import_masters_rejects_` verde
- [x] 2.2 Completar upsert: orden topológico de categorías, `create`/`rename`/`move_category`/`reactivate` y equivalentes de marca/`AtributoDef` por slug o `clave`; ignorar `productos.jsonl` — pasan 1.1 y 1.2; `cargo test -p catalog seed_import_masters_` verde (Listo cuando)
- [x] 2.3 Conservar verdes `validate_dir` y dataset T-02-40/T-02-41 — `cargo test -p catalog seed_valid_minimal_directory_has_no_issues seed_rejects_non_numeric_precio_with_file_line_field seed_repo_seed_dir_has_no_issues` verde

## 3. Tests (red) — CLI `vortex seed`

- [x] 3.1 Quitar `seed` del loop `vortex_top_level_stubs_print_unimplemented` y agregar `vortex_seed_without_config_is_error` y `vortex_seed_missing_config_file_is_not_unimplemented` en `vortex-core/bin/vortex/tests/cli.rs` — `cargo test -p vortex --test cli vortex_seed_without_config_is_error vortex_seed_missing_config_file_is_not_unimplemented` falla (stderr aún tiene `no implementado`; escenarios unhappy de `vortex-cli` / `vortex-seed`)
- [x] 3.2 Agregar `vortex_seed_imports_masters_exit_0`, `vortex_seed_twice_does_not_duplicate` y `vortex_seed_locked_db_is_error` (TOML + `datos/` temp como `tests/serve.rs`; DIR con tres JSONL; lock = `Db::open` vivo) — `cargo test -p vortex --test seed` falla (comando stub; happy CLI + Listo cuando + base bloqueada)

## 4. Implementación (green) — despacho CLI y README

- [x] 4.1 Despachar `Commands::Seed` en `bin/vortex`: `resolve_config_path`, `Config`, `Db::open` + `migrate`, gateways de storage, `import_masters`, exit 0/1; DIR posicional default `seed` — pasan 3.1 y 3.2; `cargo test -p vortex --test cli --test seed` verde; stubs `reindex`/`backup`/… siguen con `no implementado`
- [x] 4.2 Actualizar `seed/README.md`: `vortex seed [DIR]`, `--config` / `VORTEX_CONFIG`, servidor detenido, solo maestros en este ticket — escenario README de `seed-dataset`; el README menciona `vortex seed` y que no importa productos

## 5. Verificación de cierre

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 5.2 `openspec validate T-02-42` verde
