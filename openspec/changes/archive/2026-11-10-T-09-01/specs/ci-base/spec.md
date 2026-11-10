# Spec Delta

Ticket padre: T-09-01. Change: T-09-01. HU: HU-00-01, HU-09-01.

## Purpose

Define el pipeline mínimo de GitHub Actions que formatea, lint-ea y testea el workspace Vortex Core en macOS Apple Silicon, más un harness local que verifica ese contrato, para que cada PR falle si el árbol no está limpio.

## ADDED Requirements

### Requirement: Workflow de CI en macos-14

El repositorio MUST contener `.github/workflows/ci.yml`. El workflow MUST dispararse en eventos `pull_request` y MUST definir jobs de chequeo Rust llamados `fmt`, `clippy` y `test` (T-00-02 y siguientes PUEDEN agregar otros jobs). Cada uno de esos tres jobs MUST usar el runner `macos-14`. Cada uno MUST restaurar una caché de Cargo (registry, git y `target`) entre ejecuciones.

#### Scenario: Workflow presente con tres jobs en macos-14

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** el archivo existe
- **AND** declara jobs llamados `fmt`, `clippy` y `test`
- **AND** cada uno de esos jobs declara `runs-on: macos-14`
- **AND** cada uno restaura caché de Cargo

#### Scenario: Falta el workflow o un job

- **WHEN** no existe `.github/workflows/ci.yml`, o falta alguno de los jobs `fmt`, `clippy` o `test`, o alguno no usa `macos-14`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra el archivo, el job o el runner que falta

#### Scenario: Un PR dispara la CI

- **WHEN** se abre o actualiza un pull request contra el repositorio
- **THEN** el workflow se dispara
- **AND** corren los jobs `fmt`, `clippy` y `test`

#### Scenario: El workflow no dispara en pull_request

- **WHEN** `.github/workflows/ci.yml` no declara el trigger `pull_request`
- **THEN** la verificación del contrato de CI termina con código distinto de 0

### Requirement: Jobs fmt, clippy y test sobre vortex-core

Los jobs MUST ejecutar, contra el workspace cuyo manifiesto es `vortex-core/Cargo.toml`, estos comandos (los de [docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) § 5):

- job `fmt`: `cargo fmt --all -- --check`
- job `clippy`: `cargo clippy --workspace -- -D warnings`
- job `test`: `cargo test --workspace`

#### Scenario: Comandos de verificación en el workflow

- **WHEN** se leen los pasos `run` de cada job
- **THEN** `fmt` incluye `cargo fmt --all -- --check`
- **AND** `clippy` incluye `cargo clippy --workspace -- -D warnings`
- **AND** `test` incluye `cargo test --workspace`
- **AND** cada comando se ejecuta con el manifiesto `vortex-core/Cargo.toml` (directorio de trabajo o `--manifest-path`)

#### Scenario: Falta un flag o el manifiesto

- **WHEN** el job `fmt` no pasa `--check`, o `clippy` no pasa `-D warnings`, o `test` no pasa `--workspace`, o el comando no apunta a `vortex-core`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra el job

### Requirement: Árbol limpio pasa; árbol sucio falla

Sobre un checkout que cumple las specs `cargo-workspace` y `rust-toolchain`, los mismos comandos del workflow MUST terminar con código 0. Un árbol con un archivo `.rs` que no cumple rustfmt MUST hacer fallar `cargo fmt --all -- --check`. Un árbol con un warning de Clippy MUST hacer fallar `cargo clippy --workspace -- -D warnings`. Un árbol con un test que paniquea MUST hacer fallar `cargo test --workspace`.

#### Scenario: Workspace actual en verde

- **WHEN** se ejecutan los tres comandos sobre `vortex-core/` del repositorio
- **THEN** los tres terminan con código 0

#### Scenario: Error de formato

- **WHEN** un archivo `.rs` del workspace no cumple rustfmt
- **AND** se ejecuta `cargo fmt --all -- --check` contra ese árbol
- **THEN** el comando termina con código distinto de 0

#### Scenario: Warning de Clippy

- **WHEN** el workspace contiene código que Clippy reporta como warning con el grupo `all`
- **AND** se ejecuta `cargo clippy --workspace -- -D warnings`
- **THEN** el comando termina con código distinto de 0

#### Scenario: Test que falla

- **WHEN** el workspace contiene un test que paniquea
- **AND** se ejecuta `cargo test --workspace`
- **THEN** el comando termina con código distinto de 0

### Requirement: Script de verificación del contrato CI

El repositorio MUST contener `scripts/check-ci.sh`, ejecutable, que comprueba las reglas de este spec sobre el workflow YAML. La raíz MUST ser `CHECK_ROOT` si está definida, o la raíz del repositorio en caso contrario. El script MUST terminar con código 0 cuando el árbol cumple el contrato. MUST NOT exigir que GitHub Actions haya corrido: verifica el archivo y, cuando aplique, los comandos cargo en local.

#### Scenario: Script presente

- **WHEN** se inspecciona `scripts/check-ci.sh`
- **THEN** el archivo existe y es ejecutable

#### Scenario: Contrato válido

- **WHEN** se ejecuta el script sobre el repositorio con `.github/workflows/ci.yml` correcto
- **THEN** termina con código 0

### Requirement: El workflow no incluye motores reales ni lint OpenAPI

`.github/workflows/ci.yml` MUST NOT instalar ni invocar Meilisearch, Ollama, Whisper ni el *lint* Redocly de `docs/openapi.yaml`. Esos chequeos pertenecen a T-09-05, T-09-13, T-09-18 y T-09-03.

#### Scenario: Workflow mínimo

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** no menciona Meilisearch, Ollama, Whisper, Redocly ni `docs/openapi.yaml`

#### Scenario: Extra fuera de alcance

- **WHEN** el workflow declara un paso o job que invoca Meilisearch, Ollama, Whisper o Redocly
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra el extra

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-ci.sh` sobre el repo | happy | Script presente; contrato válido; workflow presente con tres jobs; comandos de verificación; workflow mínimo |
| Fixture sin `ci.yml`, sin job `fmt`, o con `runs-on: ubuntu-latest` | unhappy | Falta el workflow o un job |
| Fixture sin trigger `pull_request` | unhappy | El workflow no dispara en pull_request |
| Fixture sin `--check`, `-D warnings`, `--workspace` o sin path `vortex-core` | unhappy | Falta un flag o el manifiesto |
| Fixture con step Redocly o Meilisearch | unhappy | Extra fuera de alcance |
| Tres comandos § 5 en `vortex-core/` | happy | Workspace actual en verde |
| Copia temporal con `.rs` desformateado | unhappy | Error de formato |
| Copia temporal con warning Clippy | unhappy | Warning de Clippy |
| Copia temporal con test que paniquea | unhappy | Test que falla |

No hay tests de API ni OpenAPI. `scripts/check-toolchain.sh`, `scripts/check-workspace.sh` y `scripts/check-domain-deps.sh` MUST seguir en verde sobre el repo (no se modifican). El escenario «Un PR dispara la CI» se valida en GitHub al abrir el PR de esta rama; en local lo cubre el contrato del YAML (`pull_request`) más el unhappy de formato.
