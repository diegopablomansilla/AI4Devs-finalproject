# Tasks

## 1. Tests (red) — agregador en domain

- [x] 1.1 Agregar `vortex-core/crates/domain/src/health.rs` (mod tests) o `tests/health.rs` con `aggregate_all_up_is_ok`: seis componentes `up` → agregado `ok` — `cargo test -p domain -- aggregate_all_up` no compila o falla (escenario «todos up es ok»)
- [x] 1.2 Agregar `aggregate_storage_down_is_down`: `storage` `down`, resto `up` → `down` — el test no compila o falla (escenario «storage down es down»)
- [x] 1.3 Agregar `aggregate_any_starting_is_starting`: `storage` `up`, `meilisearch` `starting`, resto `up` → `starting` — el test no compila o falla (escenario «algún starting es starting»)
- [x] 1.4 Agregar `aggregate_any_down_or_degraded_is_degraded`: `storage` `up`, nadie `starting`, `meilisearch` `down` → `degraded` — el test no compila o falla (escenario «algún down o degraded es degraded»)
- [x] 1.5 Agregar `aggregate_disabled_does_not_degrade`: `storage` `up`, el resto `disabled` → `ok` — el test no compila o falla (escenario «disabled no degrada»)
- [x] 1.6 Agregar `aggregate_storage_down_wins_over_starting`: `storage` `down` y `meilisearch` `starting` → `down` (no `starting`) — el test no compila o falla (escenario «storage down gana sobre starting»)

## 2. Tests (red) — HealthRegistry en supervisor

- [x] 2.1 Agregar `registry_new_is_ok_with_disabled_components`: registro con `version` `0.1.0` → seis `disabled`, agregado `ok`, `uptimeSeconds` ≥ 0 — `cargo test -p supervisor -- registry_new` no compila o falla (escenario «registro nuevo es ok con disabled»)
- [x] 2.2 Agregar `registry_set_meilisearch_down_is_degraded`: `set(meilisearch, down)` → agregado `degraded` — el test no compila o falla (escenario «publicar meilisearch down degrada»)
- [x] 2.3 Agregar `registry_set_storage_starting_is_starting`: `set(storage, starting)` → agregado `starting` — el test no compila o falla (escenario «publicar starting se refleja»)

## 3. Tests (red) — GET /health

- [x] 3.1 Agregar `vortex-core/crates/api/tests/health.rs` con `get_health_ok_returns_200`: snapshot agregado `ok` → HTTP 200, `Content-Type: application/json`, `status` `ok`, `version` no vacía, `uptimeSeconds` ≥ 0, seis componentes — `cargo test -p api --test health -- get_health_ok` no compila o falla (escenario «GET /health ok es 200»)
- [x] 3.2 Agregar `get_health_degraded_returns_200`: agregado `degraded` → 200 y `status` `degraded` — el test no compila o falla (escenario «GET /health degraded es 200»)
- [x] 3.3 Agregar `get_health_starting_returns_503`: agregado `starting` → 503, `application/json`, `status` `starting`, no `problem+json` ni `code` de problema — el test no compila o falla (escenario «GET /health starting es 503» / Listo cuando)
- [x] 3.4 Agregar `get_health_down_returns_503`: `storage` `down` → 503, `status` `down`, `application/json` — el test no compila o falla (escenario «GET /health down es 503»)
- [x] 3.5 Agregar `get_health_is_not_problem_json` y `post_health_is_not_200_health`: GET no lleva `Cache-Control: no-store`; POST `/health` no es 200 Health — `cargo test -p api --test health -- not_problem post_health` no compila o falla (escenarios «GET /health no lleva no-store», «POST /health no es Health 200»)
- [x] 3.6 Agregar `vortex_serve_get_health_returns_json` en `bin/vortex/tests/serve.rs`: `vortex serve` + `GET /health` → JSON con `status`, `version`, `uptimeSeconds` y seis componentes, sin `code` de problema — `cargo test -p vortex -- vortex_serve_get_health` no compila o falla (escenario «vortex serve GET /health responde Health»)

## 4. Tests (red) — allowlist

- [x] 4.1 Fixture temporal de `crates/supervisor/Cargo.toml` con `clap` en `[dependencies]` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `supervisor` (escenario «supervisor declara una dependencia fuera de la allowlist»; **no** usar `domain`, pasará a estar permitido)
- [x] 4.2 Fixture temporal de `crates/supervisor/Cargo.toml` con path `domain` — hoy el script sale ≠ 0 (escenario «supervisor declara path domain» aún en rojo)
- [x] 4.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (supervisor todavía sin deps; escenario previo a este change)

## 5. Harness (green) — allowlist supervisor y vortex

- [x] 5.1 En `scripts/check-workspace.sh`: permitir en `supervisor` path `domain` en `[dependencies]`; permitir en `vortex` path `supervisor`; `clap` en supervisor sigue prohibido — el fixture 4.2 sale 0 y el 4.1 sigue ≠ 0 nombrando `supervisor` (escenarios «supervisor declara path domain», «supervisor declara una dependencia fuera de la allowlist»)
- [x] 5.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy `supervisor` con `domain`; unhappy `supervisor` con `clap`; happy `vortex` con `supervisor` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 5.3 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin las deps nuevas en los `Cargo.toml` reales)

## 6. Implementación (green) — domain y supervisor

- [x] 6.1 Implementar tipos, `HealthReport`, `HealthSnapshot::all_disabled` y `aggregate` en `domain`; exportar desde `lib.rs` — `cargo test -p domain -- aggregate_` sale 0 (escenarios 1.1–1.6 / Listo cuando cuatro estados)
- [x] 6.2 Declarar en `vortex-core/crates/supervisor/Cargo.toml` path `domain`; implementar `HealthRegistry` (`new`, `set`, `HealthReport`) — `./scripts/check-workspace.sh` sobre el repo sale 0 y `cargo test -p supervisor -- registry_` sale 0 (escenarios 2.1–2.3)
- [x] 6.3 Confirmar que `domain` no declara `supervisor` ni HTTP — `./scripts/check-domain-deps.sh` sale 0 (escenario «el puerto no conoce HTTP»)

## 7. Implementación (green) — GET /health y vortex serve

- [x] 7.1 `AppState::with_health(Arc<dyn HealthReport>)`; `AppState::new()` usa snapshot `all_disabled`; DTO JSON camelCase; `build_router` registra `GET /health` (200 `ok`/`degraded`, 503 `starting`/`down`, no `problem+json`) — `cargo test -p api --test health` sale 0 (escenarios 3.1–3.5 / Listo cuando 503 `starting`)
- [x] 7.2 En `bin/vortex/Cargo.toml` path `supervisor`; `serve` cablea `HealthRegistry::new(env!("CARGO_PKG_VERSION"))` — `cargo test -p vortex -- vortex_serve_get_health` sale 0 y los tests previos de `serve` siguen en verde (escenario 3.6)
- [x] 7.3 Confirmar que no hay `/metrics`, rutas de negocio, estáticos, probes de sidecars ni cambios en `docs/openapi.yaml` — `rg` / `git diff` (non-goals T-00-14, T-00-19, T-00-20, T-00-25)

## 8. Cierre (regresión y § 5)

- [x] 8.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde» / tablas de tests de `health-report` y `vortex-api`)
- [x] 8.2 `./scripts/check-workspace.sh`, `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`supervisor` depende de `domain`, `api` no depende de `supervisor`)
- [x] 8.3 `openspec validate T-00-13` (o `t-00-13` si el CLI exige minúsculas) sale 0
