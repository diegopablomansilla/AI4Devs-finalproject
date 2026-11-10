# health-report Specification

## Purpose

Define el puerto de salud, los estados por componente y la agregación in-memory (sin probes de sidecars) para que `GET /health` y tickets posteriores publiquen un snapshot estable.

Ticket padre: T-00-13. HU: HU-00-05.

## Requirements

### Requirement: Puerto HealthReport y snapshot de componentes

El crate `domain` MUST exponer un puerto `HealthReport` que devuelve un snapshot de salud. El snapshot MUST incluir: un estado agregado (`ok`, `degraded`, `starting`, `down`); una `version` no vacía; `uptimeSeconds` entero ≥ 0; y exactamente estos componentes, cada uno con un estado de componente (`up`, `starting`, `degraded`, `down`, `disabled`): `storage`, `meilisearch`, `llm`, `speech`, `textEmbeddings`, `sync`. El crate `domain` MUST NOT conocer HTTP, `problem+json` ni URLs de sidecars. El puerto MUST NOT sondear Meilisearch, Ollama ni Storage: solo expone el estado ya publicado.

#### Scenario: snapshot incluye los seis componentes (happy)

- **WHEN** se pide un snapshot a un `HealthReport`
- **THEN** el snapshot tiene estado agregado, `version` no vacía y `uptimeSeconds` ≥ 0
- **AND** `components` incluye `storage`, `meilisearch`, `llm`, `speech`, `textEmbeddings` y `sync`
- **AND** cada componente tiene un estado de componente de la lista permitida

#### Scenario: el puerto no conoce HTTP (happy)

- **WHEN** se usa el snapshot de `HealthReport` desde `domain`
- **THEN** el snapshot no incluye status HTTP, `Content-Type` ni `problem+json`

### Requirement: Agregación de los cuatro estados

El estado agregado MUST calcularse con esta prioridad, sobre los seis componentes:

1. Si `storage` está `down`, el agregado MUST ser `down`
2. Si no, si algún componente está `starting`, el agregado MUST ser `starting`
3. Si no, si algún componente está `down` o `degraded`, el agregado MUST ser `degraded`
4. Si no, el agregado MUST ser `ok`

Un componente en `disabled` o `up` MUST NOT por sí solo producir `down`, `starting` ni `degraded`. Este change MUST NOT consultar sidecars para obtener esos estados.

#### Scenario: todos up es ok (happy)

- **WHEN** los seis componentes están `up`
- **THEN** el estado agregado es `ok`

#### Scenario: storage down es down (unhappy)

- **WHEN** `storage` está `down` y el resto está `up`
- **THEN** el estado agregado es `down`

#### Scenario: algún starting es starting (unhappy)

- **WHEN** `storage` está `up` y `meilisearch` está `starting` y el resto está `up`
- **THEN** el estado agregado es `starting`

#### Scenario: algún down o degraded es degraded (unhappy)

- **WHEN** `storage` no está `down` y ningún componente está `starting`
- **AND** `meilisearch` está `down` o `llm` está `degraded`
- **THEN** el estado agregado es `degraded`

#### Scenario: disabled no degrada (happy)

- **WHEN** `storage` está `up` y `meilisearch`, `llm`, `speech`, `textEmbeddings` y `sync` están `disabled`
- **THEN** el estado agregado es `ok`

#### Scenario: storage down gana sobre starting (unhappy)

- **WHEN** `storage` está `down` y `meilisearch` está `starting`
- **THEN** el estado agregado es `down`
- **AND** el agregado no es `starting`

### Requirement: HealthRegistry publica estado sin probes

El crate `supervisor` MUST exponer un registro de salud que implementa `HealthReport`. Al crearse MUST inicializar los seis componentes en `disabled`, MUST guardar una `version` no vacía y MUST medir `uptimeSeconds` desde esa creación. MUST permitir publicar el estado de un componente conocido (`set`). MUST NOT abrir conexiones, lanzar procesos ni hacer HTTP a sidecars. Un componente desconocido MUST NOT poder publicarse (el conjunto de nombres es cerrado).

#### Scenario: registro nuevo es ok con disabled (happy)

- **WHEN** se crea un registro con `version` `0.1.0`
- **THEN** el snapshot tiene `version` `0.1.0` y `uptimeSeconds` ≥ 0
- **AND** los seis componentes están `disabled`
- **AND** el estado agregado es `ok`

#### Scenario: publicar meilisearch down degrada (unhappy)

- **WHEN** un registro nuevo recibe `meilisearch` = `down`
- **THEN** el snapshot tiene `meilisearch` en `down`
- **AND** el estado agregado es `degraded`

#### Scenario: publicar starting se refleja (unhappy)

- **WHEN** un registro nuevo recibe `storage` = `starting`
- **THEN** el snapshot tiene `storage` en `starting`
- **AND** el estado agregado es `starting`

#### Scenario: el registro no sondea sidecars (happy)

- **WHEN** se crea un registro y se pide un snapshot sin haber publicado nada
- **THEN** el snapshot se obtiene sin error
- **AND** no se requiere un sidecar ni Storage abiertos

### Requirement: Tests del agregador y del registro

Antes de archive, la suite MUST incluir estos tests (nombres estables; crate / tipo). Happy y unhappy path son obligatorios.

| Test | Crate | Tipo |
|---|---|---|
| `aggregate_all_up_is_ok` | `domain` | unitario |
| `aggregate_storage_down_is_down` | `domain` | unitario |
| `aggregate_any_starting_is_starting` | `domain` | unitario |
| `aggregate_any_down_or_degraded_is_degraded` | `domain` | unitario |
| `aggregate_disabled_does_not_degrade` | `domain` | unitario |
| `aggregate_storage_down_wins_over_starting` | `domain` | unitario |
| `registry_new_is_ok_with_disabled_components` | `supervisor` | crate |
| `registry_set_meilisearch_down_is_degraded` | `supervisor` | crate |
| `registry_set_storage_starting_is_starting` | `supervisor` | crate |

`cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST ejecutarlos.

#### Scenario: la suite del agregador cubre los cuatro estados (happy)

- **WHEN** se ejecuta `cargo test --workspace` sobre `vortex-core`
- **THEN** corren los tests de la tabla
- **AND** los de `ok`, `down`, `starting` y `degraded` pasan
- **AND** los de `disabled` y prioridad `storage` `down` sobre `starting` pasan
