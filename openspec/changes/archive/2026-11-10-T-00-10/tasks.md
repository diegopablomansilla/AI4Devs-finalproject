# Tasks

## 1. Tests (red) — router 404

- [x] 1.1 Agregar `vortex-core/crates/api/tests/router.rs` con `build_router_unknown_path_returns_404` (`GET /no-such-route` → 404) — `cargo test -p api --test router` no compila o falla (escenario «ruta inexistente responde 404» / test de la tabla)
- [x] 1.2 En el mismo archivo, cubrir que `AppState` se clona y atiende dos peticiones — sigue en rojo (escenario «AppState se comparte entre peticiones»)

## 2. Tests (red) — `vortex serve` unhappy

- [x] 2.1 Reemplazar `vortex_serve_prints_unimplemented` por `vortex_serve_without_config_fails` en tests de binario — `cargo test -p vortex --test serve -- without_config` (o `--test cli`) falla: hoy imprime `no implementado` (escenario «vortex serve sin path de configuración»)
- [x] 2.2 Reemplazar `vortex_serve_unimplemented_ignores_missing_config` por `vortex_serve_missing_config_file_fails` — sigue en rojo: no debe contener `no implementado` (escenario «vortex serve con archivo inexistente»; stub de `reindex` sí conserva el mensaje)
- [x] 2.3 Agregar `vortex_serve_invalid_config_fails` (TOML con `negocio.moneda = "peso"`) — sigue en rojo (escenario «vortex serve con configuración inválida»)
- [x] 2.4 Agregar `vortex_serve_bind_in_use_fails` — sigue en rojo (escenario «vortex serve con bind ocupado»)

## 3. Tests (red) — `vortex serve` happy y apagado

- [x] 3.1 Agregar `vortex_serve_starts_and_returns_404` (TOML temporal, puerto libre, `GET /no-such-route` → 404) — `cargo test -p vortex --test serve -- starts_and_returns` falla (escenario «vortex serve arranca y responde 404» / Listo cuando)
- [x] 3.2 Agregar `vortex_serve_exits_clean_on_sigterm` (unix) — sigue en rojo: exit 0 y puerto libre (escenario «SIGTERM termina limpio» / Listo cuando)
- [x] 3.3 Agregar `vortex_serve_exits_clean_on_sigint` (unix) — sigue en rojo (escenario «SIGINT termina limpio»)

## 4. Tests (red) — allowlist

- [x] 4.1 Fixture temporal de `crates/api/Cargo.toml` con `serde` (o `clap`) en `[dependencies]` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `api` (escenario «api declara una dependencia fuera de la allowlist»)
- [x] 4.2 Fixture temporal de `crates/catalog/Cargo.toml` con `[dependencies]` — el script sale ≠ 0 y nombra `catalog` (escenario «Un lib distinto de domain, storage y api declara dependencias»)
- [x] 4.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (api todavía sin `axum`; escenario previo a este change)

## 5. Harness (green) — allowlist `api` / `vortex`

- [x] 5.1 En `scripts/check-workspace.sh`: permitir `axum` en `api` y `tokio` en sus `[dev-dependencies]`; en `vortex` permitir path `api`, `tokio` y `axum` además de la allowlist actual; `metrics` sigue prohibido — fixture temporal con esa allowlist sale 0 (escenarios «api declara la allowlist», «Binario con Config presente»)
- [x] 5.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy `api` con `axum`; unhappy `api` con `serde`/`clap`; unhappy de lib sin allowlist usa `catalog` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 5.3 Re-correr 4.1 y 4.2: `serde` en `api` y deps en `catalog` siguen ≠ 0 con el needle del crate
- [x] 5.4 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin `axum` en el `Cargo.toml` real de `api`)

## 6. Implementación (green) — crate `api`

- [x] 6.1 Declarar en `vortex-core/crates/api/Cargo.toml` `axum = { workspace = true }` y `tokio` en `[dev-dependencies]` con features de test; actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «api declara la allowlist»)
- [x] 6.2 Implementar `AppState` (`Clone` + `Arc`) y `build_router` sin rutas de negocio — `cargo test -p api --test router` sale 0 (escenarios 1.1–1.2)

## 7. Implementación (green) — `vortex serve`

- [x] 7.1 En `bin/vortex/Cargo.toml` declarar path `api`, `tokio` (features `macros`/`rt-multi-thread`/`net`/`signal`) y `axum`; `./scripts/check-workspace.sh` sale 0 (escenario «Binario con Config presente»)
- [x] 7.2 Implementar resolución `--config` / `VORTEX_CONFIG` y despachar `serve` (no stub): config inválida o ausente → stderr con clave/motivo, exit ≠ 0, sin `no implementado` — `cargo test -p vortex --test serve -- without_config missing_config invalid_config` sale 0 (escenarios 2.1–2.3)
- [x] 7.3 Bind HTTP + `axum::serve` + `build_router`; bind ocupado falla sin colgarse — tests `starts_and_returns_404` y `bind_in_use_fails` salen 0 (escenarios 3.1 y 2.4 / Listo cuando)
- [x] 7.4 `with_graceful_shutdown` para SIGTERM e SIGINT; exit 0 y puerto libre — `cargo test -p vortex --test serve -- exits_clean` sale 0 (escenarios 3.2–3.3 / Listo cuando)
- [x] 7.5 Confirmar que `reindex`/`backup`/… y `service install|uninstall` siguen `no implementado` / exit 1, y que un `--config` inexistente con `reindex` no valida el TOML — `cargo test -p vortex --test cli` sale 0 (escenarios modificados de `vortex-cli` / `vortex-config`)
- [x] 7.6 Confirmar que `serve` no llama a `init_logging` ni abre Storage — `rg "init_logging|storage::" vortex-core/bin/vortex/src` no muestra esas llamadas en el camino `serve` (non-goals T-00-20)

## 8. Cierre (regresión, § 5 y non-goals)

- [x] 8.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde» / tabla de tests de `vortex-api`)
- [x] 8.2 `./scripts/check-workspace.sh`, `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`api` no está en `domain`)
- [x] 8.3 `git diff` vacío en `docs/openapi.yaml`; `api` no declara `tower-http` ni `axum-server`; no hay rutas `/health`, `/metrics` ni `/api/v1`; 404 no exige `problem+json` (non-goals: T-00-11, T-00-12, T-00-13, T-00-14, T-00-16, T-00-19)
- [x] 8.4 `openspec validate T-00-10` (o `t-00-10` si el CLI exige minúsculas) sale 0
