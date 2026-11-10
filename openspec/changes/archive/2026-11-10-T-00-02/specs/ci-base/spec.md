# Spec Delta

Ticket padre: T-00-02. Change: T-00-02. HU: HU-00-01.

## MODIFIED Requirements

### Requirement: Script de verificación del contrato CI

El repositorio MUST contener `scripts/check-ci.sh`, ejecutable, que comprueba las reglas de este spec sobre el workflow YAML **y** la presencia del job `domain-deps` que ejecuta `scripts/check-domain-deps.sh` (spec `domain-deps`). La raíz MUST ser `CHECK_ROOT` si está definida, o la raíz del repositorio en caso contrario. El script MUST terminar con código 0 cuando el árbol cumple ese contrato. MUST NOT exigir que GitHub Actions haya corrido: verifica el archivo y, cuando aplique, los comandos cargo en local. MUST NOT tratar la mención de `check-domain-deps` como extra fuera de alcance. MUST seguir rechazando Meilisearch, Ollama, Whisper, Redocly, `docs/openapi.yaml` y un job que ejecute `check-ci.sh` en GitHub.

#### Scenario: Script presente

- **WHEN** se inspecciona `scripts/check-ci.sh`
- **THEN** el archivo existe y es ejecutable

#### Scenario: Contrato válido

- **WHEN** se ejecuta el script sobre el repositorio con `.github/workflows/ci.yml` correcto (jobs `fmt`, `clippy`, `test` y `domain-deps`)
- **THEN** termina con código 0

#### Scenario: Falta el job domain-deps

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy` y `test` pero no el job `domain-deps` (o no invoca `scripts/check-domain-deps.sh`)
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `domain-deps` o `check-domain-deps`

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-ci.sh` sobre el repo | happy | Script presente; contrato válido |
| Fixture YAML de tres jobs sin `domain-deps` | unhappy | Falta el job domain-deps |
| `scripts/fixtures/ci/run-script-tests.sh` (suite T-09-01) | happy + unhappy | Regresión: workflow, runner, trigger, comandos § 5, extra Redocly/Meilisearch |

Los comandos de [docs/desarrollo-git-openspec.md](../../../../docs/desarrollo-git-openspec.md) § 5 (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`) MUST seguir en verde sobre `vortex-core/`. No hay tests de API.
