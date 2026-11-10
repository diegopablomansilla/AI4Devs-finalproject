# Spec Delta

Ticket padre: T-00-07. Change: T-00-07. HU: HU-00-06.

## MODIFIED Requirements

### Requirement: Crate binario vortex con Config

`vortex-core/bin/vortex` MUST ser un crate binario con `package.name = "vortex"`, `edition = "2024"` (propia o heredada del workspace), `[lints] workspace = true` y `src/main.rs` que define `fn main`. MAY declarar en `[dependencies]` únicamente `toml`, `serde`, `thiserror`, `tracing`, `tracing-subscriber`, `tracing-appender` y `clap`, cada una con `workspace = true` (`serde` MAY activar la feature `derive`; `tracing-subscriber` MAY activar features `json` y `env-filter`; `clap` MAY activar la feature `derive`). MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. MUST NOT declarar `metrics` ni otras crates, ni `serde_json` en `[dependencies]`. El binario MUST exponer la CLI de subcomandos (capability `vortex-cli`), MUST poder cargar y validar el TOML de configuración (capability `vortex-config`) y MUST exponer la inicialización de logging (capability `vortex-logging`). MUST NOT cargar el TOML ni instalar el subscriber global solo por invocar un subcomando stub.

#### Scenario: Binario con Config presente

- **WHEN** se inspecciona `vortex-core/bin/vortex`
- **THEN** `Cargo.toml` declara `name = "vortex"`, edición 2024 y hereda los lints del workspace
- **AND** existe `src/main.rs` y contiene `fn main`
- **AND** `Cargo.toml` declara `toml`, `serde` y `thiserror` con `workspace = true`
- **AND** las únicas otras entradas permitidas en `[dependencies]` son `tracing`, `tracing-subscriber`, `tracing-appender` y `clap` (todas `workspace = true`)
- **AND** si declara `[dev-dependencies]`, la única crate permitida es `serde_json` con `workspace = true`

#### Scenario: Falta main.rs

- **WHEN** no existe `vortex-core/bin/vortex/src/main.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje indica que falta el binario o `main.rs`

#### Scenario: El binario declara una dependencia fuera de la allowlist

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `metrics` u otra crate distinta de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap}` en `[dependencies]`, o declara en `[dev-dependencies]` una crate distinta de `serde_json`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: El binario no hereda lints

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` no tiene `[lints] workspace = true`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` MUST terminar con código 0. `cargo test --workspace` MUST ejecutar los tests unitarios de `domain` de la taxonomía de errores y de los tipos base (`Id`, `Decimal`, `Timestamp`) y los tests de carga/validación de configuración, de logging y de CLI del package `vortex`. El grafo de paquetes MUST incluir el package `vortex`. Los crates lib distintos de `domain` MAY seguir sin lógica de producto.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto, un lib distinto de `domain` declara dependencias, `domain` declara una crate fuera de su allowlist, o `bin/vortex` declara una crate fuera de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap}` en `[dependencies]` o distinta de `serde_json` en `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este change
