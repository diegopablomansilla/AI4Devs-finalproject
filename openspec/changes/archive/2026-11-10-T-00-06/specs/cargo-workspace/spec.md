# Spec Delta

Ticket padre: T-00-06. Change: T-00-06. HU: HU-00-09.

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
| serde_json | 1.0.154 |
| tracing | 0.1.44 |
| tracing-subscriber | 0.3.23 |
| tracing-appender | 0.2.5 |
| metrics | 0.24.6 |
| metrics-exporter-prometheus | 0.18.3 |
| rcgen | 0.14.10 |
| clap | 4.6.7 |
| toml | 1.2.1 |

El crate `domain` MAY declarar en `[dependencies]` únicamente `thiserror`, `ulid`, `rust_decimal` y `serde`, cada una con `workspace = true`. `domain` MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. `domain` MUST NOT declarar `serde_json` en `[dependencies]`. En particular, `domain` MUST NOT depender de `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort` ni `whisper-rs`. Los demás crates lib MUST NOT declarar `[dependencies]` ni `[dev-dependencies]`. El crate binario `vortex` MAY declarar dependencias según el requirement «Crate binario vortex con Config».

#### Scenario: Pins exactos

- **WHEN** se lee `[workspace.dependencies]`
- **THEN** cada crate de la tabla aparece con `=` y la versión indicada

#### Scenario: Versión no fijada

- **WHEN** una entrada de `[workspace.dependencies]` no usa `=` o no coincide con la versión de la tabla
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra la crate

#### Scenario: Un lib declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain` contiene `[dependencies]` o `[dev-dependencies]`, o `domain` declara una crate fuera de su allowlist (`thiserror`, `ulid`, `rust_decimal`, `serde` en `[dependencies]`; `serde_json` en `[dev-dependencies]`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

#### Scenario: domain declara thiserror

- **WHEN** `vortex-core/crates/domain/Cargo.toml` declara `thiserror` en `[dependencies]` con `workspace = true` y ninguna otra dependencia
- **THEN** la verificación del workspace termina con código 0

#### Scenario: domain declara la allowlist de tipos base

- **WHEN** `vortex-core/crates/domain/Cargo.toml` declara `thiserror`, `ulid`, `rust_decimal` y `serde` en `[dependencies]` y `serde_json` en `[dev-dependencies]`, todas con `workspace = true`, y ninguna otra crate
- **THEN** la verificación del workspace termina con código 0

#### Scenario: Un lib distinto de domain declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain` contiene `[dependencies]` o `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

#### Scenario: domain declara una dependencia distinta de thiserror

- **WHEN** `crates/domain/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de la allowlist (por ejemplo `clap` o `surrealdb`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `domain`

### Requirement: Crate binario vortex con Config

`vortex-core/bin/vortex` MUST ser un crate binario con `package.name = "vortex"`, `edition = "2024"` (propia o heredada del workspace), `[lints] workspace = true` y `src/main.rs` que define `fn main`. MAY declarar en `[dependencies]` únicamente `toml`, `serde`, `thiserror`, `tracing`, `tracing-subscriber` y `tracing-appender`, cada una con `workspace = true` (`serde` MAY activar la feature `derive`; `tracing-subscriber` MAY activar features `json` y `env-filter`). MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. MUST NOT declarar `clap`, `metrics` ni otras crates, ni `serde_json` en `[dependencies]`. El binario MUST cargar y validar el TOML de configuración al arrancar (capability `vortex-config`) y MUST exponer la inicialización de logging (capability `vortex-logging`).

#### Scenario: Binario con Config presente

- **WHEN** se inspecciona `vortex-core/bin/vortex`
- **THEN** `Cargo.toml` declara `name = "vortex"`, edición 2024 y hereda los lints del workspace
- **AND** existe `src/main.rs` y contiene `fn main`
- **AND** `Cargo.toml` declara `toml`, `serde` y `thiserror` con `workspace = true`
- **AND** las únicas otras entradas permitidas en `[dependencies]` son `tracing`, `tracing-subscriber` y `tracing-appender` (todas `workspace = true`)
- **AND** si declara `[dev-dependencies]`, la única crate permitida es `serde_json` con `workspace = true`

#### Scenario: Falta main.rs

- **WHEN** no existe `vortex-core/bin/vortex/src/main.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje indica que falta el binario o `main.rs`

#### Scenario: El binario declara una dependencia fuera de la allowlist

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `clap`, `metrics` u otra crate distinta de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender}` en `[dependencies]`, o declara en `[dev-dependencies]` una crate distinta de `serde_json`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: El binario no hereda lints

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` no tiene `[lints] workspace = true`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` MUST terminar con código 0. `cargo test --workspace` MUST ejecutar los tests unitarios de `domain` de la taxonomía de errores y de los tipos base (`Id`, `Decimal`, `Timestamp`) y los tests de carga/validación de configuración y de logging del package `vortex`. El grafo de paquetes MUST incluir el package `vortex`. Los crates lib distintos de `domain` MAY seguir sin lógica de producto.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto, un lib distinto de `domain` declara dependencias, `domain` declara una crate fuera de su allowlist, o `bin/vortex` declara una crate fuera de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender}` en `[dependencies]` o distinta de `serde_json` en `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este change

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-workspace.sh` sobre el repo (pin `tracing-subscriber` + allowlist ampliada de `vortex`) | happy | Pins exactos; Binario con Config presente |
| Fixture `bin/vortex` con `clap` o `metrics` (crate fuera de allowlist) | unhappy | El binario declara una dependencia fuera de la allowlist |
| Fixture de un lib que no es `domain` (p. ej. `api`) con `[dependencies]` | unhappy | Un lib distinto de domain declara dependencias |
| En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` | happy | Build, tests y clippy en verde ([docs/desarrollo-git-openspec.md](../../../../docs/desarrollo-git-openspec.md) § 5) |
