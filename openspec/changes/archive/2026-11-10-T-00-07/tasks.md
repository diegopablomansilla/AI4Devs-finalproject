# Tasks

## 1. Tests (red) — help y `--config` global

- [x] 1.1 Agregar `vortex-core/bin/vortex/tests/cli.rs` con `vortex_help_lists_all_subcommands` y `vortex_short_help_lists_subcommands` — `cargo test -p vortex --test cli -- help_lists` no compila o falla (escenarios «vortex --help lista los ocho subcomandos», «vortex -h también lista los subcomandos» / Listo cuando)
- [x] 1.2 Agregar test `vortex_service_help_lists_install_uninstall` — `cargo test -p vortex --test cli -- service_help` sigue en rojo (escenario «vortex service --help lista install y uninstall»)
- [x] 1.3 Agregar tests `vortex_help_mentions_config_flag` y `vortex_serve_unimplemented_ignores_missing_config` — siguen en rojo (escenarios «--config aparece en la ayuda raíz», «stub ignora un --config inexistente»)

## 2. Tests (red) — stubs `no implementado`

- [x] 2.1 Agregar test `vortex_serve_prints_unimplemented` — `cargo test -p vortex --test cli -- serve_prints` falla: no sale código 1 con `no implementado` en stderr (escenario «vortex serve no implementado»)
- [x] 2.2 Agregar test `vortex_top_level_stubs_print_unimplemented` (`reindex`, `backup`, `cert`, `models`, `seed`, `verify`) — sigue en rojo (escenario «el resto de subcomandos de primer nivel son stub»)
- [x] 2.3 Agregar test `vortex_service_install_uninstall_print_unimplemented` — sigue en rojo (escenario «service install y uninstall son stub»)

## 3. Tests (red) — argv inválido (unhappy)

- [x] 3.1 Agregar test `vortex_without_subcommand_is_error` — `cargo test -p vortex --test cli -- without_subcommand` falla (escenario «sin subcomando»: ≠ 0 y stderr sin `no implementado`)
- [x] 3.2 Agregar tests `vortex_unknown_subcommand_is_error` y `vortex_service_without_action_is_error` — siguen en rojo (escenarios «subcomando desconocido», «service sin acción»)

## 4. Tests (red) — allowlist de `check-workspace.sh`

- [x] 4.1 Fixture temporal (`CHECK_ROOT`) de `bin/vortex/Cargo.toml` con `[dependencies] metrics.workspace = true` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `vortex` (escenario «El binario declara una dependencia fuera de la allowlist»)
- [x] 4.2 Fixture temporal de un lib que no es `domain` (p. ej. `api`) con `[dependencies]` — el script sale ≠ 0 y nombra ese crate (escenario «Un lib distinto de domain declara dependencias»)
- [x] 4.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (vortex todavía sin `clap` en `Cargo.toml`; escenario «Binario con Config presente» previo a este change)

## 5. Harness (green) — allowlist `clap`

- [x] 5.1 En `scripts/check-workspace.sh`: permitir en `bin/vortex` `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap}` en `[dependencies]` (`workspace = true`; `clap` MAY `features = ["derive"]`) y `serde_json` en `[dev-dependencies]`; rechazar `metrics` y cualquier otra crate — una fixture temporal con esa allowlist (incluye `clap`) sale 0 (escenario «Binario con Config presente»)
- [x] 5.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy del repo; unhappy de `vortex` usa `metrics` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 5.3 Re-correr 4.1 y 4.2: `metrics` en `vortex` y deps en `api` siguen ≠ 0 con el needle del crate
- [x] 5.4 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin `clap` en el `Cargo.toml` real de `vortex`)

## 6. Implementación (green) — `cli` + despacho

- [x] 6.1 Declarar en `vortex-core/bin/vortex/Cargo.toml` `clap = { workspace = true, features = ["derive"] }` y actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «Binario con Config presente»)
- [x] 6.2 Implementar `src/cli.rs` (`Parser`, subcomandos del ticket, `--config` global) y despachar en `main`: stubs `eprintln!("no implementado"); exit(1)`; quitar `parse_config_flag` y la carga de `Config` en `run` — `cargo test -p vortex --test cli` sale 0 (escenarios 1.1–3.2 / Listo cuando)
- [x] 6.3 Reescribir `tests/config_cli.rs` para que no espere exit 0 ni error de TOML sin subcomando (el esquema sigue en unitarios de `config.rs`) — `cargo test -p vortex --test config_cli` sale 0 (escenarios modificados de `vortex-config`: stub no exige archivo; config válida no arranca un stub como éxito)
- [x] 6.4 Confirmar que `main` no llama a `Config::from_path` ni a `init_logging` — `rg "from_path|init_logging" vortex-core/bin/vortex/src/main.rs` no muestra esas llamadas (non-goals T-00-10 / T-00-20)

## 7. Cierre (regresión, § 5 y non-goals)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde»)
- [x] 7.2 `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`clap` no está en `domain`)
- [x] 7.3 `git diff` vacío en `vortex-core/crates/`, `docs/openapi.yaml` y `.github/workflows/ci.yml`; `bin/vortex` no declara `metrics`; ningún stub arranca HTTP/Storage/`launchd` (non-goals: T-00-10, T-00-14, T-00-08, T-00-30)
- [x] 7.4 `openspec validate T-00-07` (o `t-00-07` si el CLI exige minúsculas) sale 0
