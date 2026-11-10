# Tasks

## 1. Tests (red) — contrato CI

- [x] 1.1 Crear fixture `scripts/fixtures/ci/scenarios/missing-openapi-job/.github/workflows/ci.yml` con cuatro jobs válidos (`fmt`, `clippy`, `test`, `domain-deps`, `macos-14`, `pull_request`, comandos § 5, caché) **sin** job `openapi-lint` ni `@redocly/cli` — `test -f` del YAML (escenario «Falta el job o no invoca el lint»)
- [x] 1.2 Crear fixture `scripts/fixtures/ci/scenarios/meilisearch-extra/.github/workflows/ci.yml` (esqueleto de `redocly-extra` + `domain-deps`, step que invoca Meilisearch) — `test -f` del YAML (escenario «Extra de sidecar fuera de alcance»)
- [x] 1.3 Crear fixture `scripts/fixtures/ci/scenarios/missing-redocly-yaml/.github/workflows/ci.yml` con los cinco jobs válidos **incluyendo** `openapi-lint`, **sin** `redocly.yaml` en esa raíz — `test -f` del YAML (escenario «falta redocly.yaml»)
- [x] 1.4 Extender `scripts/fixtures/ci/run-script-tests.sh`: `assert_fails_with` `missing-openapi-job` needle `openapi-lint`; `meilisearch-extra` needle `Meilisearch` o `meilisearch`; `missing-redocly-yaml` needle `redocly.yaml`; **quitar** `assert_fails_with` de `redocly-extra` — `./scripts/fixtures/ci/run-script-tests.sh` sale ≠ 0 porque `check-ci.sh` aún no exige el job ni deja pasar Redocly (escenarios «Falta el job openapi-lint», «falta redocly.yaml» y «Extra de sidecar fuera de alcance» en rojo)

## 2. Tests (red) — oráculo de lint

- [x] 2.1 Crear `scripts/fixtures/openapi/scenarios/invalid-spec/openapi.yaml` con OpenAPI 3.1 estructuralmente inválido (p. ej. sin `info.version`) — `test -f` del YAML (escenario «un error de lint hace fallar el comando»)
- [x] 2.2 Crear `scripts/fixtures/openapi/run-lint-oracles.sh`: happy = `npx --yes @redocly/cli@1 lint docs/openapi.yaml` sobre el repo sale 0; unhappy = el mismo comando contra la fixture 2.1 sale ≠ 0 y el stderr/stdout nombra el archivo o la regla — `./scripts/fixtures/openapi/run-lint-oracles.sh` sale 0 en el happy (el YAML vigente ya pasa) y el unhappy pasa en cuanto existe la fixture (escenarios «el contrato vigente pasa el lint» y «un error de lint hace fallar el comando»)
- [x] 2.3 `npx --yes @redocly/cli@1 lint docs/openapi.yaml` sobre el repo sale 0 (warnings permitidos; no se toca `docs/openapi.yaml`)

## 3. Harness y job (green)

- [x] 3.1 En `scripts/check-ci.sh`: quitar `Redocly`, `redocly` y `docs/openapi.yaml` de la denylist de extras; exigir job `openapi-lint` (`^[[:space:]]{2}openapi-lint:`), las cadenas `@redocly/cli` y `lint docs/openapi.yaml`, y archivo `redocly.yaml` **después** de `domain-deps` y de los extras que siguen (Meilisearch, Ollama, Whisper, `check-ci.sh`) — `./scripts/check-ci.sh` sobre el repo sale ≠ 0 y nombra `openapi-lint` o `redocly.yaml`; las aserciones de 1.4 sobre `missing-openapi-job` pasan
- [x] 3.2 Agregar `redocly.yaml` en la raíz: `extends: recommended` y `apis.vortex@v1.root: docs/openapi.yaml`; no subir a error `info-license`, `operation-4xx-response` ni `no-unused-components` — `test -f redocly.yaml` (escenario «redocly.yaml presente»)
- [x] 3.3 Agregar job `openapi-lint` a `.github/workflows/ci.yml`: `runs-on: macos-14`; `actions/checkout@v4`; **sin** `working-directory: vortex-core`; **sin** caché Cargo; `run: npx --yes @redocly/cli@1 lint docs/openapi.yaml` — escenario «Job presente en el workflow»
- [x] 3.4 `./scripts/check-ci.sh` sobre el repo sale 0 (escenarios «Contrato válido», «Job presente en el workflow», «Workflow sin sidecars»)
- [x] 3.5 `./scripts/fixtures/ci/run-script-tests.sh` sale 0 (happy del repo + unhappy `missing-openapi-job`, `missing-redocly-yaml`, `meilisearch-extra` + regresiones T-09-01/T-00-02)
- [x] 3.6 `./scripts/fixtures/openapi/run-lint-oracles.sh` sale 0 (happy del YAML vigente + unhappy `invalid-spec` / Listo cuando)

## 4. Cierre (regresión, § 5 y non-goals)

- [x] 4.1 `git diff` vacío en `vortex-core/`, `docs/openapi.yaml` y `docs/5-contract-api.md`; no hay `package.json` ni job que ejecute `scripts/check-ci.sh` (non-goals T-09-04, PAPI-01 ya aplicado, codegen)
- [x] 4.2 `./scripts/check-toolchain.sh`, `./scripts/check-workspace.sh` y `./scripts/check-domain-deps.sh` salen 0
- [x] 4.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.4 `.github/workflows/ci.yml` no menciona Meilisearch, Ollama ni Whisper (non-goals T-09-05, T-09-13, T-09-18)
- [x] 4.5 `openspec validate T-09-03` (o `t-09-03` si el CLI exige minúsculas) sale 0
