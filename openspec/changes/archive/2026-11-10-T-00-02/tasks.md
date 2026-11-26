# Tasks

## 1. Tests (red)

- [x] 1.1 Crear fixture `scripts/fixtures/ci/scenarios/missing-domain-deps-job/.github/workflows/ci.yml` con tres jobs válidos (`fmt`, `clippy`, `test`, `macos-14`, `pull_request`, comandos § 5, caché) **sin** job `domain-deps` ni `check-domain-deps` — `test -f` del YAML
- [x] 1.2 Extender `scripts/fixtures/ci/run-script-tests.sh` con `assert_fails_with` de esa fixture y needle `domain-deps` — `./scripts/fixtures/ci/run-script-tests.sh` sale ≠ 0 porque `check-ci.sh` aún no exige el job (escenarios «Falta el job domain-deps» y «Falta el job o no invoca el script» en rojo)
- [x] 1.3 `CHECK_ROOT=scripts/fixtures/domain-deps/scenarios/surrealdb-direct ./scripts/check-domain-deps.sh` sale ≠ 0 y el stderr nombra `surrealdb` (escenario «Agregar surrealdb a domain hace fallar el chequeo de CI»)
- [x] 1.4 `./scripts/check-domain-deps.sh` sobre el repo sale 0 (escenario «Domain limpio pasa el job»)

## 2. Harness y job (green)

- [x] 2.1 En `scripts/check-ci.sh`: quitar `check-domain-deps` de la denylist de extras; exigir job `domain-deps` (`^[[:space:]]{2}domain-deps:`) y la cadena `scripts/check-domain-deps.sh` **después** de los extras prohibidos — `./scripts/check-ci.sh` sobre el repo sale ≠ 0 y nombra `domain-deps`; la aserción de 1.2 pasa
- [x] 2.2 Las fixtures T-09-01 (`missing-fmt-job`, `ubuntu-runner`, `no-pr-trigger`, `bad-fmt-command`, `redocly-extra`, `no-workflow`) siguen saliendo ≠ 0 con sus needles al correr `./scripts/fixtures/ci/run-script-tests.sh` (el assert del repo aún falla hasta 2.3)
- [x] 2.3 Agregar job `domain-deps` a `.github/workflows/ci.yml`: `runs-on: macos-14`; `actions/checkout@v4`; `rustup show`; `actions/cache@v4` (registry, git, `vortex-core/target`, key con hash de `vortex-core/Cargo.lock`); **sin** `working-directory: vortex-core`; `run: ./scripts/check-domain-deps.sh` — escenario «Job presente en el workflow»
- [x] 2.4 `./scripts/check-ci.sh` sobre el repo sale 0 (escenarios «Contrato válido» y «Job presente en el workflow»)
- [x] 2.5 `./scripts/fixtures/ci/run-script-tests.sh` sale 0 (happy del repo + unhappy `missing-domain-deps-job` + regresiones T-09-01)

## 3. Cierre (regresión, § 5 y non-goals)

- [x] 3.1 `./scripts/fixtures/domain-deps/run-script-tests.sh` sale 0 — `git diff` vacío en `vortex-core/crates/` y en la lógica de `scripts/check-domain-deps.sh` (solo comentario de ticket si aplica)
- [x] 3.2 `./scripts/check-toolchain.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 3.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 3.4 `.github/workflows/ci.yml` no menciona Meilisearch, Ollama, Whisper, Redocly ni `docs/openapi.yaml`, y no ejecuta `check-ci.sh` (non-goals)
- [x] 3.5 `openspec validate T-00-02` sale 0
