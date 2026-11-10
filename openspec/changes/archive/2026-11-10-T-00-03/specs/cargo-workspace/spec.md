# Spec Delta

Ticket padre: T-00-03. Change: T-00-03. HU: HU-00-01.

## MODIFIED Requirements

### Requirement: Dependencias del stack con versión exacta

`[workspace.dependencies]` MUST fijar con operador `=` (sin `^` ni `~`) estas crates y versiones:

| Crate | Versión |
|---|---|
| tokio | 1.54.1 |
| axum | 0.8.9 |
| tower-http | 0.7.1 |
| axum-server | 0.9.0 |
| surrealdb | 2.7.0 |
| meilisearch-sdk | 0.33.0 |
| reqwest | 0.13.5 |
| ort | 2.0.0-rc.15 |
| tokenizers | 0.23.2 |
| whisper-rs | 0.16.0 |
| rust_decimal | 1.43.0 |
| ulid | 3.0.0 |
| thiserror | 2.0.21 |
| serde | 1.0.229 |
| tracing | 0.1.44 |
| tracing-appender | 0.2.5 |
| metrics | 0.24.6 |
| metrics-exporter-prometheus | 0.18.3 |
| rcgen | 0.14.10 |
| clap | 4.6.7 |

El crate `domain` MAY declarar `thiserror` en `[dependencies]` con `workspace = true`. `domain` MUST NOT declarar otra entrada en `[dependencies]` ni `[dev-dependencies]`. En particular, `domain` MUST NOT depender de `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort` ni `whisper-rs`. Los demás crates lib MUST NOT declarar `[dependencies]` ni `[dev-dependencies]`.

#### Scenario: Pins exactos

- **WHEN** se lee `[workspace.dependencies]`
- **THEN** cada crate de la tabla aparece con `=` y la versión indicada

#### Scenario: Versión no fijada

- **WHEN** una entrada de `[workspace.dependencies]` no usa `=` o no coincide con la versión de la tabla
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra la crate

#### Scenario: Un lib declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain` contiene `[dependencies]` o `[dev-dependencies]`, o `domain` declara una crate distinta de `thiserror`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

#### Scenario: domain declara thiserror

- **WHEN** `vortex-core/crates/domain/Cargo.toml` declara `thiserror` en `[dependencies]` con `workspace = true` y ninguna otra dependencia
- **THEN** la verificación del workspace termina con código 0

#### Scenario: Un lib distinto de domain declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain` contiene `[dependencies]` o `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

#### Scenario: domain declara una dependencia distinta de thiserror

- **WHEN** `crates/domain/Cargo.toml` declara en `[dependencies]` una crate que no es `thiserror` (por ejemplo `ulid` o `surrealdb`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `domain`

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` MUST terminar con código 0. `cargo test --workspace` MUST ejecutar los tests unitarios de `domain` de la taxonomía de errores. El grafo de paquetes MUST incluir el package `vortex`. Los crates lib distintos de `domain` MAY seguir sin lógica de producto.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto, un lib distinto de `domain` declara dependencias, o `domain` declara una crate que no es `thiserror`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este change

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-workspace.sh` sobre el repo (con `thiserror` en `domain`) | happy | domain declara thiserror; Pins exactos |
| Fixture `domain` con `ulid` (u otra crate) en `[dependencies]` | unhappy | domain declara una dependencia distinta de thiserror |
| Fixture de un lib que no es `domain` (p. ej. `api`) con `[dependencies]` | unhappy | Un lib distinto de domain declara dependencias |
| En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` | happy | Build, tests y clippy en verde ([docs/desarrollo-git-openspec.md](../../../../docs/desarrollo-git-openspec.md) § 5) |
| `scripts/check-domain-deps.sh` sobre el repo | happy | `thiserror` no viola la denylist de motores |
