# ci-meilisearch Specification

## Purpose

Define la puerta de CI que levanta Meilisearch con versión fijada y ejecuta los tests de integración ignorados contra el motor real, para que HU-09-01 y T-03-04 no dependan de un sidecar local no reproducido en GitHub.

## Requirements

### Requirement: Versión fijada de Meilisearch

El repositorio MUST contener `scripts/meilisearch-version` con exactamente una línea de versión (tag GitHub `vMAJOR.MINOR.PATCH`, hoy `v1.58.0`) y sin manifiesto de `vortex models`. Esa versión MUST ser la misma que usa el job de CI para descargar el binario. MUST NOT leer ni escribir `models/manifest.toml` (T-00-24, v2).

#### Scenario: Pin presente y alineado con el workflow (happy)

- **WHEN** se inspecciona `scripts/meilisearch-version` y `.github/workflows/ci.yml`
- **THEN** el archivo de pin existe y declara un tag `v` seguido de SemVer
- **AND** el workflow menciona ese mismo tag al descargar Meilisearch

#### Scenario: Falta el pin o el workflow usa otra versión (unhappy)

- **WHEN** no existe `scripts/meilisearch-version`, o el workflow no menciona el tag que declara el pin
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra `meilisearch-version` o el tag esperado

### Requirement: Job de CI meilisearch

`.github/workflows/ci.yml` MUST definir un job llamado `meilisearch` en el mismo workflow que `fmt`, `clippy`, `test`, `domain-deps` y `openapi-lint`. El job MUST dispararse en `pull_request`. El job MUST usar el runner `macos-14`. El job MUST instalar el binario oficial `meilisearch-macos-apple-silicon` de la versión fijada, escucharlo en `127.0.0.1:7700`, esperar a que `GET /health` responda con éxito, y entonces ejecutar contra `vortex-core/` el comando `cargo test --workspace -- --include-ignored meilisearch`. El job `test` MUST seguir ejecutando `cargo test --workspace` **sin** `--include-ignored`.

#### Scenario: Job presente con motor y filtro ignore (happy)

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** declara un job llamado `meilisearch`
- **AND** ese job declara `runs-on: macos-14`
- **AND** descarga o invoca `meilisearch-macos-apple-silicon` con el tag del pin
- **AND** un step ejecuta `cargo test --workspace` con `--include-ignored` y el filtro `meilisearch`

#### Scenario: Falta el job meilisearch (unhappy)

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy`, `test`, `domain-deps` y `openapi-lint` pero no el job `meilisearch`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra `meilisearch`

#### Scenario: El job no incluye los tests ignorados (unhappy)

- **WHEN** el job `meilisearch` existe pero no ejecuta `cargo test` con `--include-ignored`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra `--include-ignored`

### Requirement: Probe de integración contra Meilisearch real

El workspace MUST incluir al menos un test de integración cuyo nombre contiene `meilisearch` y que está marcado `#[ignore = "meilisearch"]`. Ese test MUST hacer `GET {MEILISEARCH_URL}/health` (por defecto `http://127.0.0.1:7700`, alineado a `vortex-core/config.example.toml`) y MUST terminar en éxito solo si la respuesta HTTP es 200 y el cuerpo JSON tiene `status` igual a `available`. MUST NOT usar `meilisearch-sdk` ni implementar el puerto `TextSearch`. `cargo test --workspace` sin `--include-ignored` MUST omitir ese test y terminar con código 0 aunque el motor no esté levantado.

#### Scenario: Probe con Meilisearch disponible (happy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test --workspace -- --include-ignored meilisearch`
- **THEN** el test ignorado marcado `meilisearch` corre
- **AND** termina con código 0

#### Scenario: Probe con Meilisearch caído (unhappy)

- **WHEN** no hay proceso escuchando en la URL configurada
- **AND** se ejecuta el test ignorado con `--include-ignored`
- **THEN** el test falla (no se omite)
- **AND** el mensaje indica fallo de conexión o de `/health`

#### Scenario: La suite por defecto no exige el motor (happy de aislamiento)

- **WHEN** Meilisearch no está levantado
- **AND** se ejecuta `cargo test --workspace` **sin** `--include-ignored`
- **THEN** el comando termina con código 0
- **AND** el probe figura como ignorado
