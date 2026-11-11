# Tasks

## 1. Tests (red) — arranque feliz

- [x] 1.1 En `vortex-core/bin/vortex/tests/serve.rs`, agregar `vortex_serve_health_ok_with_storage_up`: TOML con `paths.datos` en un directorio temporal único y `server.bind` libre; spawn `vortex serve`; esperar hasta 15 s `GET /health` → 200, `application/json`, `status` `ok`, `components.storage.status` `up`, sin `code` de problema — `cargo test -p vortex -- vortex_serve_health_ok_with_storage_up` no compila o falla (escenario «vortex serve espera health ok con storage up» / Listo cuando)
- [x] 1.2 Agregar `vortex_serve_boot_logs_phase_durations`: el mismo arranque; leer `<datos>/logs/vortex.log*`; asertar JSON con spans o eventos `boot.config`, `boot.logging`, `boot.storage`, `boot.health` y `boot.api` y una duración en cada uno — `cargo test -p vortex -- vortex_serve_boot_logs` no compila o falla (escenario «las fases de boot registran duración»)
- [x] 1.3 Extender `vortex_serve_get_health_returns_json` para exigir `status` `ok` y `components.storage.status` `up` (además de los seis componentes) — el test sigue en rojo hasta el secuenciador (escenario «vortex serve GET /health responde Health»)

## 2. Tests (red) — caminos no felices

- [x] 2.1 Agregar `vortex_serve_inaccessible_storage_exits`: `paths.datos` directorio escribible; `<datos>/db` archivo regular; `server.bind` libre; `vortex serve` termina ≠ 0, el puerto no acepta TCP, stderr o log menciona Storage / abrir / base — `cargo test -p vortex -- vortex_serve_inaccessible_storage` no compila o falla (escenario «base inaccesible no expone la API» / Listo cuando)
- [x] 2.2 Agregar `vortex_serve_unwritable_datos_exits`: `paths.datos` es un archivo regular; `vortex serve` termina ≠ 0, no escucha HTTP, stderr nombra `paths.datos` o `logs` — `cargo test -p vortex -- vortex_serve_unwritable_datos` no compila o falla (escenarios «logs no escribibles no expone la API», «serve con datos archivo no escucha HTTP»)

## 3. Tests (red) — allowlist

- [x] 3.1 Fixture temporal de `bin/vortex/Cargo.toml` con path `storage` (más la allowlist actual) — hoy `./scripts/check-workspace.sh` sale ≠ 0 (escenario «vortex declara path storage» aún en rojo)
- [x] 3.2 Fixture temporal con `metrics` en `vortex` — el script sigue saliendo ≠ 0 y nombra `vortex` (escenario «El binario declara una dependencia fuera de la allowlist»)
- [x] 3.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (todavía sin `storage` en el `Cargo.toml` real)

## 4. Harness (green) — allowlist vortex + storage

- [x] 4.1 En `scripts/check-workspace.sh` (`vortex_deps_allowed`): permitir `storage` con `path` a `crates/storage`; `metrics` sigue prohibido — el fixture 3.1 sale 0 y el 3.2 sigue ≠ 0 nombrando `vortex` (escenarios «vortex declara path storage», «dependencia fuera de la allowlist»)
- [x] 4.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh` si hace falta un happy explícito `vortex con path storage` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 4.3 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin la dep nueva en el `Cargo.toml` real)

## 5. Implementación (green) — secuenciador de boot

- [x] 5.1 En `bin/vortex/Cargo.toml` declarar `storage = { path = "../../crates/storage" }` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «vortex declara path storage»)
- [x] 5.2 En `tests/serve.rs`, hacer que `write_config` / helpers usen un `paths.datos` temporal único (no `/tmp/datos`) y subir a 15 s los waits que abren RocksDB — los tests previos de `serve` que hoy pasan con `datos` compartido siguen compilando
- [x] 5.3 En `serve.rs`: tras `Config`, `init_logging` (retener `LoggingGuard`); crear `HealthRegistry`; `set(storage, starting)`; `Db::open(<datos>/db)` + `migrate`; `set(storage, up)`; retener `Db`; entonces bind HTTP; spans `boot.config` / `boot.logging` / `boot.storage` / `boot.health` / `boot.api`; errores de logs o Storage → stderr + exit ≠ 0 sin listener — `cargo test -p vortex -- vortex_serve_health_ok_with_storage_up vortex_serve_boot_logs vortex_serve_inaccessible_storage vortex_serve_unwritable_datos` sale 0 (escenarios 1.1–2.2 / Listo cuando)
- [x] 5.4 Quitar `#![allow(dead_code)]` de `logging.rs` si `init_logging` queda usado; `VORTEX_ENV=production` en los tests de binario que leen JSON de log — `cargo test -p vortex --test serve` sale 0 y `cargo test -p vortex -- logging_` sigue en verde (escenario «serve escribe JSON de boot bajo datos/logs»; non-goal: stubs no instalan logging)
- [x] 5.5 Confirmar que no se inyectó `Db` en `AppState`, no hay sidecars, no cambió `docs/openapi.yaml` y no hay crate nueva en `[workspace.dependencies]` — `git diff --stat` / `rg` (non-goals T-00-27, T-00-16, OpenAPI)

## 6. Cierre (regresión y § 5)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «la suite acordada cubre happy y unhappy» / «Build, tests y clippy en verde»)
- [x] 6.2 `./scripts/check-workspace.sh` y `./scripts/check-domain-deps.sh` salen 0 (`vortex` depende de `storage`; `domain` y `api` no)
- [x] 6.3 `openspec validate T-00-20` (o `t-00-20` si el CLI exige minúsculas) sale 0
