# Spec Delta

Ticket padre: T-09-03. Change: T-09-03. HU: HU-09-02.

## Purpose

Define el lint de Redocly sobre `docs/openapi.yaml` como puerta de CI, para que un error en la especificación ejecutable falle el PR antes de que T-09-04 valide respuestas.

## ADDED Requirements

### Requirement: Configuración Redocly en la raíz

El repositorio MUST contener `redocly.yaml` en la raíz de verificación (`CHECK_ROOT` o la raíz del repo). Ese archivo MUST declarar `docs/openapi.yaml` como API a lint-ear y MUST extender el conjunto de reglas `recommended`. MUST NOT elevar a error las reglas que hoy solo advierten sobre el YAML vigente (`info-license`, `operation-4xx-response`, `no-unused-components`).

#### Scenario: redocly.yaml presente (happy)

- **WHEN** se inspecciona la raíz del repositorio
- **THEN** existe `redocly.yaml`
- **AND** apunta a `docs/openapi.yaml`
- **AND** extiende `recommended`

#### Scenario: falta redocly.yaml (unhappy)

- **WHEN** la raíz de verificación no contiene `redocly.yaml`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra `redocly.yaml`

### Requirement: Lint de docs/openapi.yaml

El comando de lint MUST ejecutarse contra `docs/openapi.yaml` (el mismo path que declara [5.13](../../../../docs/5-contract-api.md#513-especificación-openapi-31)). MUST terminar con código 0 cuando el archivo es una descripción OpenAPI 3.1 válida según Redocly `recommended`, aunque emita warnings. MUST terminar con código distinto de 0 cuando el archivo tiene un **error** de lint (YAML inválido, esquema OpenAPI roto o regla `recommended` en severidad error).

#### Scenario: el contrato vigente pasa el lint (happy)

- **WHEN** se ejecuta el lint Redocly sobre `docs/openapi.yaml` del repositorio
- **THEN** el comando termina con código 0
- **AND** los warnings existentes no cambian el código de salida

#### Scenario: un error de lint hace fallar el comando (unhappy)

- **WHEN** se ejecuta el lint Redocly sobre un `openapi.yaml` con un error estructural (p. ej. falta `info.version` o YAML mal formado)
- **THEN** el comando termina con código distinto de 0
- **AND** la salida nombra el archivo o la regla que falla

### Requirement: Job de CI openapi-lint

`.github/workflows/ci.yml` MUST definir un job llamado `openapi-lint` en el mismo workflow que `fmt`, `clippy`, `test` y `domain-deps`. El job MUST dispararse en `pull_request`. El job MUST usar el runner `macos-14`. El job MUST ejecutarse desde la raíz del repositorio (sin `working-directory: vortex-core`). El job MUST invocar `npx @redocly/cli lint docs/openapi.yaml` (con `--yes` y pin de línea 1.x según el design). El job MUST terminar con código 0 cuando el YAML vigente pasa el lint, y con código distinto de 0 cuando hay un error de lint.

#### Scenario: Job presente en el workflow (happy)

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** declara un job llamado `openapi-lint`
- **AND** ese job declara `runs-on: macos-14`
- **AND** un step ejecuta `npx` con `@redocly/cli` y `lint docs/openapi.yaml`

#### Scenario: Falta el job o no invoca el lint (unhappy)

- **WHEN** `.github/workflows/ci.yml` no declara el job `openapi-lint`, o el job no invoca `@redocly/cli lint docs/openapi.yaml`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra `openapi-lint` o `redocly`

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-ci.sh` sobre el repo (tras el job y `redocly.yaml`) | happy | Job presente en el workflow; redocly.yaml presente |
| `npx --yes @redocly/cli@1 lint docs/openapi.yaml` sobre el repo | happy | el contrato vigente pasa el lint |
| Fixture YAML con `fmt`/`clippy`/`test`/`domain-deps` válidos y sin job `openapi-lint` | unhappy | Falta el job o no invoca el lint |
| Fixture de raíz con workflow completo y sin `redocly.yaml` | unhappy | falta redocly.yaml |
| Fixture `scripts/fixtures/openapi/scenarios/invalid-spec` | unhappy | un error de lint hace fallar el comando |
| `scripts/fixtures/ci/run-script-tests.sh` | happy + unhappy | job presente + falta job + regresiones T-09-01/T-00-02 (Meilisearch extra, no Redocly) |

No hay tests de crate `api` ni validación de cuerpos HTTP (T-09-04). Este change MUST NOT modificar el contenido de `docs/openapi.yaml` ni de [5.13](../../../../docs/5-contract-api.md#513-especificación-openapi-31). El escenario operativo «un PR con un error de lint falla GitHub Actions» se confirma al abrir el PR de esta rama (el job corre el mismo comando) y en cualquier PR posterior que rompa el YAML.
