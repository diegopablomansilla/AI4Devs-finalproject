# Tasks

## 1. Tests (red) — helper y GET /health

- [x] 1.1 Crear `vortex-core/crates/api/tests/support/mod.rs` que declare `pub mod openapi` y `vortex-core/crates/api/tests/support/openapi.rs` con `pub fn conforms(...)` y `pub fn assert_conforms(...)` que aún `todo!()` o no existan símbolos — `test -f vortex-core/crates/api/tests/support/openapi.rs`
- [x] 1.2 En `crates/api/tests/health.rs`: `mod support;` y llamar `support::openapi::assert_conforms("GET", "/health", status, &ctype, &json)` en `get_health_ok_returns_200` y `get_health_starting_returns_503` — `cargo test -p api --test health -- --exact get_health_ok_returns_200` no compila o falla (escenario «GET /health 200 conforme»)
- [x] 1.3 Crear `crates/api/tests/openapi_conformance.rs`: `mod support;`; test que arma un JSON tipo `Health` **sin** `version` y aserta que `conforms("GET", "/health", 200, "application/json", &body)` es `Err` y el mensaje nombra `version` o `required`; test que `conforms("POST", "/health", 200, "application/json", &json!({}))` es `Err` y nombra `POST` o `/health` — `cargo test -p api --test openapi_conformance` no compila o falla (escenarios «campo required omitido hace fallar» y «operación o content type desconocidos»)

## 2. Tests (red) — allowlist

- [x] 2.1 En `scripts/fixtures/workspace/run-script-tests.sh`: happy `api con allowlist openapi` (Cargo.toml de `api` con allowlist prometheus **más** `jsonschema` y `serde_yaml` en `[dev-dependencies]`, `workspace = true`, `jsonschema` con `default-features = false`); unhappy `api con jsonschema en dependencies` (`jsonschema` bajo `[dependencies]`) — `test` del bloque en el script (escenarios «api declara la allowlist openapi» y «api declara jsonschema en dependencies»)
- [x] 2.2 `./scripts/fixtures/workspace/run-script-tests.sh` sale ≠ 0 porque `check-workspace.sh` aún rechaza `jsonschema` / `serde_yaml` en `api` o no falla el unhappy de `[dependencies]` (escenarios de allowlist en rojo)

## 3. Helper y allowlist (green)

- [x] 3.1 En `vortex-core/Cargo.toml` `[workspace.dependencies]`: `jsonschema = "=0.61.1"` y `serde_yaml = "=0.9.34"`; en `scripts/check-workspace.sh`: `check_pin jsonschema 0.61.1` y `check_pin serde_yaml 0.9.34` — `./scripts/check-workspace.sh` nombra el pin o pasa el check_pin cuando el manifiesto ya los tiene (escenario «Pins exactos»)
- [x] 3.2 En `api_deps_allowed`: aceptar `jsonschema` y `serde_yaml` **solo** en `[dev-dependencies]` (`workspace = true`; si hay `jsonschema`, exigir `default-features = false`); rechazarlas en `[dependencies]` — `./scripts/check-workspace.sh` sobre una copia con esas dev-deps sale 0; la fixture 2.1 unhappy nombra `api` (escenarios «api declara la allowlist openapi» y «api declara jsonschema en dependencies»)
- [x] 3.3 En `crates/api/Cargo.toml` `[dev-dependencies]`: `jsonschema = { workspace = true, default-features = false }` y `serde_yaml = { workspace = true }` — `./scripts/check-workspace.sh` sobre el repo sale 0
- [x] 3.4 Implementar loader + inliner de `$ref` + lookup (método, path exacto o `{param}`, status, MIME sin parámetros) + `jsonschema::draft202012` en `tests/support/openapi.rs`; `conforms` / `assert_conforms` según design — `cargo test -p api --test openapi_conformance` sale 0 (unhappy de required omitido y operación desconocida; escenario «el YAML vigente se carga»)
- [x] 3.5 Completar el cableado de 1.2 — `cargo test -p api --test health` sale 0 (escenarios «GET /health 200 conforme» y «GET /health 503 conforme» / Listo cuando)
- [x] 3.6 `./scripts/fixtures/workspace/run-script-tests.sh` sale 0 (happy openapi + unhappy jsonschema en dependencies + regresiones T-00-11/T-00-12/T-00-14)

## 4. Cierre (regresión, § 5 y non-goals)

- [x] 4.1 `git diff` vacío en `docs/openapi.yaml`, `docs/5-contract-api.md` y `vortex-core/crates/api/src/` (non-goals: sin codegen, sin mutar contrato ni handler); `crates/api/Cargo.toml` no declara `jsonschema` ni `serde_yaml` en `[dependencies]`
- [x] 4.2 `./scripts/check-toolchain.sh`, `./scripts/check-workspace.sh` y `./scripts/check-domain-deps.sh` salen 0
- [x] 4.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.4 `.github/workflows/ci.yml` no agrega jobs nuevos ni menciona codegen (non-goals T-09-05 / PAPI-04)
- [x] 4.5 `openspec validate T-09-04` (o `t-09-04` si el CLI exige minúsculas) sale 0
