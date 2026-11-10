# ci-base Specification

## Purpose

Define el pipeline mínimo de GitHub Actions que formatea, lint-ea y testea el workspace Vortex Core en macOS Apple Silicon, más un harness local que verifica ese contrato, para que cada PR falle si el árbol no está limpio.

## Requirements

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

Los jobs MUST ejecutar, contra el workspace cuyo manifiesto es `vortex-core/Cargo.toml`, estos comandos (los de [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5):

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

El repositorio MUST contener `scripts/check-ci.sh`, ejecutable, que comprueba las reglas de este spec sobre el workflow YAML **y** la presencia del job `domain-deps` que ejecuta `scripts/check-domain-deps.sh` (spec `domain-deps`) **y** la presencia del job `openapi-lint` que ejecuta el lint Redocly de `docs/openapi.yaml` (spec `openapi-lint`). La raíz MUST ser `CHECK_ROOT` si está definida, o la raíz del repositorio en caso contrario. El script MUST terminar con código 0 cuando el árbol cumple ese contrato. MUST NOT exigir que GitHub Actions haya corrido: verifica el archivo y, cuando aplique, los comandos cargo en local. MUST NOT tratar la mención de `check-domain-deps`, `redocly` ni `docs/openapi.yaml` como extra fuera de alcance. MUST exigir `redocly.yaml` en la raíz de verificación. MUST seguir rechazando Meilisearch, Ollama, Whisper y un job que ejecute `check-ci.sh` en GitHub.

#### Scenario: Script presente

- **WHEN** se inspecciona `scripts/check-ci.sh`
- **THEN** el archivo existe y es ejecutable

#### Scenario: Contrato válido

- **WHEN** se ejecuta el script sobre el repositorio con `.github/workflows/ci.yml` correcto (jobs `fmt`, `clippy`, `test`, `domain-deps` y `openapi-lint`) y `redocly.yaml` presente
- **THEN** termina con código 0

#### Scenario: Falta el job domain-deps

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy` y `test` pero no el job `domain-deps` (o no invoca `scripts/check-domain-deps.sh`)
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `domain-deps` o `check-domain-deps`

#### Scenario: Falta el job openapi-lint

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy`, `test` y `domain-deps` pero no el job `openapi-lint` (o no invoca `@redocly/cli lint docs/openapi.yaml`)
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `openapi-lint` o `redocly`

### Requirement: El workflow no incluye motores reales

`.github/workflows/ci.yml` MUST NOT instalar ni invocar Meilisearch, Ollama ni Whisper. Esos chequeos pertenecen a T-09-05, T-09-13 y T-09-18. El *lint* Redocly de `docs/openapi.yaml` MUST figurar en el workflow (capability `openapi-lint`).

#### Scenario: Workflow sin sidecars

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** no menciona Meilisearch, Ollama ni Whisper
- **AND** sí menciona Redocly y `docs/openapi.yaml` (job `openapi-lint`)

#### Scenario: Extra de sidecar fuera de alcance

- **WHEN** el workflow declara un paso o job que invoca Meilisearch, Ollama o Whisper
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra el extra
