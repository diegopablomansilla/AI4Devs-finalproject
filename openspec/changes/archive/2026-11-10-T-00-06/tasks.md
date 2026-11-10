# Tasks

## 1. Tests (red) — JSON, filtro y span

- [x] 1.1 Agregar `vortex-core/bin/vortex/src/logging.rs` con test `logging_writes_json_line_in_temp_dir` y `mod logging` en `main.rs` — `cargo test -p vortex -- logging_writes_json` no compila o falla (escenario «Evento info se persiste como JSON»)
- [x] 1.2 Agregar test unhappy `logging_filters_debug_when_level_is_info` — `cargo test -p vortex -- logging_filters_debug` sigue en rojo (escenario «Evento debug se filtra con nivel info»)
- [x] 1.3 Agregar test `logging_json_includes_span_name` — `cargo test -p vortex -- logging_json_includes_span` sigue en rojo (escenario «Evento dentro de un span incluye el span»)

## 2. Tests (red) — retención, consola y I/O

- [x] 2.1 Agregar test `logging_retention_keeps_at_most_n_files` (temp dir con más de N archivos `vortex.log*`, `logs.retencion = N`) — `cargo test -p vortex -- logging_retention` falla (escenario «Retención recorta archivos de más» / Listo cuando)
- [x] 2.2 Agregar tests `logging_pretty_console_in_development` (`VORTEX_ENV=development`) y `logging_no_pretty_console_in_production` (`VORTEX_ENV=production`) — siguen en rojo (escenarios «Consola pretty en desarrollo», «Sin pretty si VORTEX_ENV es production»)
- [x] 2.3 Agregar tests unhappy `logging_fails_when_datos_is_a_file` y `logging_fails_when_logs_dir_not_writable` — fallan: no hay error con clave+motivo (escenarios «datos no es un directorio», «directorio de logs no escribible»)

## 3. Tests (red) — allowlist de `check-workspace.sh`

- [x] 3.1 Fixture temporal (`CHECK_ROOT`) de `bin/vortex/Cargo.toml` con `[dependencies] clap.workspace = true` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `vortex` (escenario «El binario declara una dependencia fuera de la allowlist»)
- [x] 3.2 Fixture temporal de un lib que no es `domain` (p. ej. `api`) con `[dependencies]` — el script sale ≠ 0 y nombra ese crate (escenario «Un lib distinto de domain declara dependencias»)
- [x] 3.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (vortex todavía sin tracing*; escenario «Pins exactos» aún sin `tracing-subscriber`)

## 4. Harness (green) — pin `tracing-subscriber` y allowlist

- [x] 4.1 En `vortex-core/Cargo.toml` agregar `tracing-subscriber = "=0.3.23"` a `[workspace.dependencies]` y `check_pin tracing-subscriber 0.3.23` en `scripts/check-workspace.sh` — escenario «Pins exactos» incluye `tracing-subscriber`
- [x] 4.2 En `scripts/check-workspace.sh`: permitir en `bin/vortex` `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender}` en `[dependencies]` (`workspace = true`; `tracing-subscriber` MAY `features = ["json", "env-filter"]`) y `serde_json` en `[dev-dependencies]` (`workspace = true`); rechazar `clap`, `metrics` y cualquier otra crate — una fixture temporal con esa allowlist sale 0 (escenario «Binario con Config presente»)
- [x] 4.3 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy del repo; unhappy de `vortex` usa `clap` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 4.4 Re-correr 3.1 y 3.2: `clap` en `vortex` y deps en `api` siguen ≠ 0 con el needle del crate
- [x] 4.5 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin las deps nuevas en el `Cargo.toml` real de `vortex`)

## 5. Implementación (green) — módulo `logging`

- [x] 5.1 Declarar en `vortex-core/bin/vortex/Cargo.toml` `tracing`, `tracing-subscriber` (features `json`, `env-filter`) y `tracing-appender` (`workspace = true`) más `serde_json` en `[dev-dependencies]`, y actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «Binario con Config presente»)
- [x] 5.2 Implementar `logging.rs` (`retain_log_files`, subscriber JSON con rotación diaria y `max_log_files`, filtro por `logs.nivel`, capa pretty según `VORTEX_ENV`, `LoggingError { key, reason }`, `with_default` en tests, sin `set_global_default` en `main`) — `cargo test -p vortex -- logging_` sale 0 (escenarios 1.1–2.3 / Listo cuando)
- [x] 5.3 Confirmar que `vortex-core/bin/vortex/src/main.rs` no llama a `init_logging` — `cargo test -p vortex --test config_cli` sigue en verde (non-goal T-00-20; example.toml no escribe en `/var/lib/vortex/datos`)

## 6. Cierre (regresión, § 5 y non-goals)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde»)
- [x] 6.2 `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`tracing-subscriber` no está en `domain`)
- [x] 6.3 `git diff` vacío en `vortex-core/crates/`, `docs/openapi.yaml` y `.github/workflows/ci.yml`; `bin/vortex` no declara `clap` ni `metrics`; `main` no instala el subscriber global (non-goals: T-00-07, T-00-14, T-00-12, T-00-20)
- [x] 6.4 `openspec validate T-00-06` sale 0
