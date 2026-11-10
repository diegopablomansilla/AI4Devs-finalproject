# Spec Delta

Ticket padre: T-00-02. Change: T-00-02. HU: HU-00-01.

## ADDED Requirements

### Requirement: Job de CI que verifica el árbol de domain

`.github/workflows/ci.yml` MUST definir un job llamado `domain-deps` en el mismo workflow que `fmt`, `clippy` y `test`. El job MUST dispararse en `pull_request`. El job MUST usar el runner `macos-14`. El job MUST ejecutar `scripts/check-domain-deps.sh` sobre el checkout (raíz del repositorio). El job MUST restaurar una caché de Cargo (registry, git y `vortex-core/target`). El job MUST terminar con código 0 cuando el árbol `normal,build` de `domain` cumple las reglas de esta spec, y con código distinto de 0 cuando las viola (incluido si el árbol incluye `surrealdb`).

#### Scenario: Job presente en el workflow

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** declara un job llamado `domain-deps`
- **AND** ese job declara `runs-on: macos-14`
- **AND** un step ejecuta `scripts/check-domain-deps.sh`

#### Scenario: Domain limpio pasa el job

- **WHEN** el crate `domain` del repositorio no incluye SDKs de motores ni crates del workspace en su árbol `normal,build`
- **AND** se ejecuta `scripts/check-domain-deps.sh` (el mismo comando que el job de CI)
- **THEN** el comando termina con código 0

#### Scenario: Agregar surrealdb a domain hace fallar el chequeo de CI

- **WHEN** el árbol `normal,build` de `domain` incluye el package `surrealdb`
- **AND** se ejecuta `scripts/check-domain-deps.sh` (el mismo comando que el job de CI)
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `surrealdb`

#### Scenario: Falta el job o no invoca el script

- **WHEN** `.github/workflows/ci.yml` no declara el job `domain-deps`, o el job no ejecuta `scripts/check-domain-deps.sh`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra `domain-deps` o `check-domain-deps`

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-ci.sh` sobre el repo (tras el job) | happy | Job presente en el workflow |
| `scripts/check-domain-deps.sh` sobre el repo | happy | Domain limpio pasa el job |
| Fixture YAML con `fmt`/`clippy`/`test` válidos y sin job `domain-deps` | unhappy | Falta el job o no invoca el script |
| Fixture `scripts/fixtures/domain-deps/scenarios/surrealdb-direct` | unhappy | Agregar surrealdb a domain hace fallar el chequeo de CI |
| `scripts/fixtures/domain-deps/run-script-tests.sh` (suite T-00-02-01) | happy + unhappy | Regresión: script, SDKs, transitiva, workspace, `dev-dependencies` |

No hay tests de API ni OpenAPI. No se agrega `surrealdb` al crate `domain` del árbol git: la violación se prueba con la fixture ya existente. El escenario operativo «un PR con `surrealdb` en `domain` falla GitHub Actions» se confirma al abrir el PR de esta rama (el job corre el script) y en cualquier PR posterior que viole la regla.
