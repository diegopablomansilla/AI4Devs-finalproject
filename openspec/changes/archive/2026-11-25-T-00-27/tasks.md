# Tasks

## 1. Tests (red) — sonda y Listo cuando

- [x] 1.1 Agregar `wait_meilisearch_returns_true_when_probe_is_up` y `wait_meilisearch_returns_false_when_probe_stays_down` en `vortex-core/bin/vortex/src/` — `cargo test -p vortex wait_meilisearch` no compila o falla (escenarios sonda `up` / timeout corto con sonda `down`)
- [x] 1.2 Agregar `vortex_serve_starts_degraded_when_meilisearch_down` en `vortex-core/bin/vortex/tests/serve.rs` — `cargo test -p vortex vortex_serve_starts_degraded_when_meilisearch_down` no compila o falla (Listo cuando: URL inalcanzable → API arriba, HTTP 200, `status` `degraded`, `meilisearch: down`, `storage: up`, `sync: up`)

## 2. Implementación (green) — espera inyectable

- [x] 2.1 Implementar `wait_meilisearch_up` (timeout + intervalo; default 20 s) y leer `VORTEX_BOOT_MEILI_TIMEOUT_MS` solo si está set — `cargo test -p vortex wait_meilisearch` sale 0

## 3. Tests (red) — happy con sidecar y harness existente

- [x] 3.1 Agregar `vortex_serve_health_ok_when_meilisearch_up` `#[ignore = "meilisearch"]` en `vortex-core/bin/vortex/tests/serve.rs` — `cargo test -p vortex -- --include-ignored vortex_serve_health_ok_when_meilisearch_up` no compila o falla (`status` `ok`, `meilisearch: up`, `sync: up`)
- [x] 3.2 Relajar `wait_for_health_ok_storage_up` a HTTP 200 + `storage` `up` + `status` `ok` o `degraded`; `spawn_serve_production` setea `VORTEX_BOOT_MEILI_TIMEOUT_MS=0`; TOML de tests usa URL inalcanzable — `cargo test -p vortex vortex_serve_health_ok_with_storage_up vortex_serve_get_health_returns_json` no asume Meilisearch en 7700 (pueden seguir en rojo hasta el cableado)

## 4. Implementación (green) — allowlist y composition root

- [x] 4.1 Ampliar `vortex_deps_allowed` y el delta `cargo-workspace` (`search`, `sync`, `domain`; `tokio` feature `time` si hace falta) y fixtures de `scripts/check-workspace.sh` — `./scripts/check-workspace.sh` no rechaza esas paths en `bin/vortex`
- [x] 4.2 Declarar path `search`, `sync` y `domain` en `vortex-core/bin/vortex/Cargo.toml` — `cargo build -p vortex` resuelve las crates
- [x] 4.3 En `serve.rs`: tras Storage OK, `MeiliTextSearch` + `boot.meilisearch` (starting → wait → up/down) + spawn `MeiliProductWorker` uid `productos` + `PrometheusSyncTelemetry` (y `set_sync_stats`) + `sync = up` + `boot.sync` + abort del worker en shutdown; **después** bind HTTP — `cargo test -p vortex vortex_serve_starts_degraded_when_meilisearch_down wait_meilisearch vortex_serve_health_ok_with_storage_up vortex_serve_get_health_returns_json vortex_serve_boot_logs_phase_durations` sale 0 (unhappy + regresión; sin sidecar)

## 5. Cierre (regresión y non-goals)

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 5.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 5.3 Confirmar non-goals: sin spawn del binario Meilisearch, sin modelos IA, sin `openapi.yaml` ni CI YAML, sin `vortex reindex` — `git diff --name-only` sin esos paths de código
- [x] 5.4 `openspec validate T-00-27` sin errores