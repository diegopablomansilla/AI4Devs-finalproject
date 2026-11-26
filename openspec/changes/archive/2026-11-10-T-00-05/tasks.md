# Tasks

## 1. Tests (red) — parse y validación de `Config`

- [x] 1.1 Agregar tests unitarios `config_parses_example_toml` y `config_allows_omitted_meilisearch_api_key` en `vortex-core/bin/vortex` (`src/config.rs`) — `cargo test -p vortex` no compila o falla (escenarios «Parse de TOML válido», «El example del repo es válido», «api_key de Meilisearch omitida»)
- [x] 1.2 Agregar test unhappy `config_rejects_missing_required_key` (falta `paths.datos` o sección `negocio`) — `cargo test -p vortex` sigue en rojo (escenario «Clave o sección faltante»: mensaje nombra la clave)
- [x] 1.3 Agregar test unhappy `config_rejects_invalid_values` (`bind` sin puerto, `moneda = "peso"`, `hora = "25:00"`, `retencion = 0`, `nivel = "verbose"`, url sin `http`) — sigue en rojo (escenario «Valor inválido»: cada caso nombra la clave)
- [x] 1.4 Agregar tests unhappy `config_rejects_unknown_key` (`server.port` o `[foo]`) y `config_rejects_malformed_toml` — sigue en rojo (escenarios «Clave desconocida», «TOML mal formado»)

## 2. Tests (red) — proceso `vortex` (exit 0 / ≠ 0)

- [x] 2.1 Agregar `vortex-core/bin/vortex/tests/config_cli.rs` con `vortex_exits_nonzero_without_path` y `vortex_exits_nonzero_missing_file` usando `env!("CARGO_BIN_EXE_vortex")` — `cargo test -p vortex --test config_cli` falla (hoy `main` sale 0; escenarios «Sin path de configuración», «Archivo inexistente»)
- [x] 2.2 Agregar `vortex_exits_nonzero_invalid_config` (TOML temporal con `negocio.moneda = "peso"`) — el test falla: exit 0 o stderr sin clave+motivo (escenario «Configuración inválida aborta con clave y motivo» / Listo cuando)

## 3. Tests (red) — allowlist de `check-workspace.sh` para `vortex`

- [x] 3.1 Fixture temporal (`CHECK_ROOT`) de `bin/vortex/Cargo.toml` con `[dependencies] clap.workspace = true` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `vortex` (escenario «El binario declara una dependencia fuera de la allowlist»; hoy ya falla por cualquier deps)
- [x] 3.2 Fixture temporal de un lib que no es `domain` (p. ej. `api`) con `[dependencies]` — el script sale ≠ 0 y nombra ese crate (escenario «Un lib distinto de domain declara dependencias»)
- [x] 3.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (vortex todavía sin deps; escenario «Pins exactos» aún sin `toml`)

## 4. Harness (green) — pin `toml` y allowlist de `vortex`

- [x] 4.1 En `vortex-core/Cargo.toml` agregar `toml = "=1.2.1"` a `[workspace.dependencies]` y `check_pin toml 1.2.1` en `scripts/check-workspace.sh` — escenario «Pins exactos» incluye `toml`
- [x] 4.2 En `scripts/check-workspace.sh`: permitir en `bin/vortex` el subconjunto `{toml, serde, thiserror}` en `[dependencies]` (`workspace = true`; `serde` MAY `features = ["derive"]`); rechazar `[dev-dependencies]`, `clap` y cualquier otra crate; dejar `domain` como está — una fixture temporal con esa allowlist sale 0 (escenario «Binario con Config presente»)
- [x] 4.3 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy del repo; unhappy de `vortex` usa `clap` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 4.4 Re-correr 3.1 y 3.2: `clap` en `vortex` y deps en `api` siguen ≠ 0 con el needle del crate
- [x] 4.5 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin las deps nuevas en el `Cargo.toml` real de `vortex`)

## 5. Implementación (green) — `Config`, example y `main`

- [x] 5.1 Declarar en `vortex-core/bin/vortex/Cargo.toml` `toml`, `serde` (`features = ["derive"]`) y `thiserror` (`workspace = true`) y actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «Binario con Config presente»)
- [x] 5.2 Implementar `config.rs` (`Config` + secciones, `deny_unknown_fields`, `ConfigError { key, reason }`, validación de la tabla) — `cargo test -p vortex -- config_` sale 0 (escenarios de parse/validación 1.1–1.4)
- [x] 5.3 Agregar `vortex-core/config.example.toml` (bind `127.0.0.1:8080`, hostname `localhost`, moneda `ARS`, Meilisearch `http://127.0.0.1:7700`, Ollama `http://127.0.0.1:11434`, retenciones 14, `logs.nivel = "info"`) — `config_parses_example_toml` pasa (escenario «El example del repo es válido»)
- [x] 5.4 En `main.rs`: resolver `--config` / `--config=` / `VORTEX_CONFIG` (flag gana); cargar; stderr + exit 1 si falla; exit 0 si el TOML es válido — `cargo test -p vortex --test config_cli` pone 2.1–2.2 en verde
- [x] 5.5 Agregar tests de binario `vortex_exits_zero_with_config_flag`, `vortex_exits_zero_with_env_path` y `vortex_config_flag_overrides_env` — `cargo test -p vortex` sale 0 (escenarios «`--config` con archivo válido», «`VORTEX_CONFIG` con archivo válido», «`--config` gana sobre `VORTEX_CONFIG`», «Configuración válida termina 0»)

## 6. Cierre (regresión, § 5 y non-goals)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde»)
- [x] 6.2 `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`toml` no está en `domain`)
- [x] 6.3 `git diff` vacío en `vortex-core/crates/`, `docs/openapi.yaml` y `.github/workflows/ci.yml`; `bin/vortex` no declara `clap`; no hay recarga en caliente ni `watch` (non-goals: T-00-07, T-00-11, No hace)
- [x] 6.4 `openspec validate T-00-05` sale 0
