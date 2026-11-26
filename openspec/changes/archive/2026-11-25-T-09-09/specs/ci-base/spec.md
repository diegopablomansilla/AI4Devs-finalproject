# Spec Delta

Ticket padre: T-09-09. Change: T-09-09. HU: HU-09-03.

## MODIFIED Requirements

### Requirement: Script de verificación del contrato CI

El repositorio MUST contener `scripts/check-ci.sh`, ejecutable, que comprueba las reglas de este spec sobre el workflow YAML **y** la presencia del job `domain-deps` que ejecuta `scripts/check-domain-deps.sh` (spec `domain-deps`) **y** la presencia del job `openapi-lint` que ejecuta el lint Redocly de `docs/openapi.yaml` (spec `openapi-lint`) **y** la presencia del job `meilisearch` que levanta Meilisearch y ejecuta los tests ignorados (spec `ci-meilisearch`) **y** la presencia del job `e2e` que ejecuta Playwright contra `e2e/` (spec `e2e-playwright`). La raíz MUST ser `CHECK_ROOT` si está definida, o la raíz del repositorio en caso contrario. El script MUST terminar con código 0 cuando el árbol cumple ese contrato. MUST NOT exigir que GitHub Actions haya corrido: verifica el archivo y, cuando aplique, los comandos cargo en local. MUST NOT tratar la mención de `check-domain-deps`, `redocly`, `docs/openapi.yaml`, Meilisearch, `playwright` ni `e2e` como extra fuera de alcance. MUST exigir `redocly.yaml` y `scripts/meilisearch-version` en la raíz de verificación. MUST seguir rechazando Ollama, Whisper y un job que ejecute `check-ci.sh` en GitHub.

#### Scenario: Script presente

- **WHEN** se inspecciona `scripts/check-ci.sh`
- **THEN** el archivo existe y es ejecutable

#### Scenario: Contrato válido

- **WHEN** se ejecuta el script sobre el repositorio con `.github/workflows/ci.yml` correcto (jobs `fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint`, `meilisearch` y `e2e`), `redocly.yaml` y `scripts/meilisearch-version` presentes
- **THEN** termina con código 0

#### Scenario: Falta el job domain-deps

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy` y `test` pero no el job `domain-deps` (o no invoca `scripts/check-domain-deps.sh`)
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `domain-deps` o `check-domain-deps`

#### Scenario: Falta el job openapi-lint

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy`, `test` y `domain-deps` pero no el job `openapi-lint` (o no invoca `@redocly/cli lint docs/openapi.yaml`)
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `openapi-lint` o `redocly`

#### Scenario: Falta el job meilisearch

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy`, `test`, `domain-deps` y `openapi-lint` pero no el job `meilisearch` (o no invoca `--include-ignored`)
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `meilisearch` o `--include-ignored`

#### Scenario: Falta el job e2e

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint` y `meilisearch` pero no el job `e2e`
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `e2e`

#### Scenario: El job e2e no invoca Playwright

- **WHEN** el job `e2e` existe pero el workflow no menciona Playwright
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra Playwright o `playwright`
