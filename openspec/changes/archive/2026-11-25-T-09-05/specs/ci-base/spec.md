# Spec Delta

Ticket padre: T-09-05. Change: T-09-05. HU: HU-09-01.

## ADDED Requirements

### Requirement: El workflow no incluye motores v2

`.github/workflows/ci.yml` MUST NOT instalar ni invocar Ollama ni Whisper. Esos chequeos pertenecen a T-09-13 y T-09-18. Meilisearch MUST figurar en el workflow (capability `ci-meilisearch`). El *lint* Redocly de `docs/openapi.yaml` MUST figurar en el workflow (capability `openapi-lint`).

#### Scenario: Workflow con Meilisearch y sin motores v2 (happy)

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** menciona Meilisearch
- **AND** no menciona Ollama ni Whisper
- **AND** sí menciona Redocly y `docs/openapi.yaml` (job `openapi-lint`)

#### Scenario: Extra de sidecar v2 fuera de alcance (unhappy)

- **WHEN** el workflow declara un paso o job que invoca Ollama o Whisper
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra el extra

## MODIFIED Requirements

### Requirement: Script de verificación del contrato CI

El repositorio MUST contener `scripts/check-ci.sh`, ejecutable, que comprueba las reglas de este spec sobre el workflow YAML **y** la presencia del job `domain-deps` que ejecuta `scripts/check-domain-deps.sh` (spec `domain-deps`) **y** la presencia del job `openapi-lint` que ejecuta el lint Redocly de `docs/openapi.yaml` (spec `openapi-lint`) **y** la presencia del job `meilisearch` que levanta Meilisearch y ejecuta los tests ignorados (spec `ci-meilisearch`). La raíz MUST ser `CHECK_ROOT` si está definida, o la raíz del repositorio en caso contrario. El script MUST terminar con código 0 cuando el árbol cumple ese contrato. MUST NOT exigir que GitHub Actions haya corrido: verifica el archivo y, cuando aplique, los comandos cargo en local. MUST NOT tratar la mención de `check-domain-deps`, `redocly`, `docs/openapi.yaml` ni Meilisearch como extra fuera de alcance. MUST exigir `redocly.yaml` y `scripts/meilisearch-version` en la raíz de verificación. MUST seguir rechazando Ollama, Whisper y un job que ejecute `check-ci.sh` en GitHub.

#### Scenario: Script presente

- **WHEN** se inspecciona `scripts/check-ci.sh`
- **THEN** el archivo existe y es ejecutable

#### Scenario: Contrato válido

- **WHEN** se ejecuta el script sobre el repositorio con `.github/workflows/ci.yml` correcto (jobs `fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint` y `meilisearch`), `redocly.yaml` y `scripts/meilisearch-version` presentes
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

## REMOVED Requirements

### Requirement: El workflow no incluye motores reales

**Reason:** T-09-05 agrega Meilisearch real al workflow. La prohibición que queda es solo la de motores v2 (Ollama, Whisper), ahora en el requirement «El workflow no incluye motores v2».

**Migration:** el job `meilisearch` y `scripts/meilisearch-version` satisfacen `ci-meilisearch`. `scripts/check-ci.sh` deja de listar `Meilisearch`/`meilisearch` en la denylist; el extra fuera de alcance se prueba con Ollama/Whisper. No hay migración de datos.

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-ci.sh` sobre el repo | happy | Script presente; contrato válido |
| Fixture YAML de cinco jobs (`fmt`/`clippy`/`test`/`domain-deps`/`openapi-lint`) sin `meilisearch` | unhappy | Falta el job meilisearch |
| Fixture con step Ollama (reemplaza el rol de `meilisearch-extra`) | unhappy | Extra de sidecar v2 fuera de alcance |
| `scripts/fixtures/ci/run-script-tests.sh` | happy + unhappy | Regresión: workflow, runner, trigger, comandos § 5, domain-deps, openapi-lint |

Los comandos de [docs/desarrollo-git-openspec.md](../../../../docs/desarrollo-git-openspec.md) § 5 MUST seguir en verde sobre `vortex-core/`. No hay tests de API.
