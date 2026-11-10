# cargo-workspace Specification

## Purpose

Define el workspace Cargo de Vortex Core con los crates lib, el binario `bin/vortex` (composition root de configuración), lints heredados y las dependencias del stack fijadas con versión exacta.

Ticket padre: T-00-01. Changes: T-00-01-02, T-00-01-03, T-00-05. HU: HU-00-01, HU-00-06.

## Requirements

### Requirement: Workspace con los crates de Vortex Core

El repositorio MUST contener `vortex-core/Cargo.toml` como workspace Cargo (`resolver = "3"`) cuyos `members` sean exactamente estos paths, y ningún otro:

- `crates/api`
- `crates/domain`
- `crates/catalog`
- `crates/search`
- `crates/recommend`
- `crates/ai`
- `crates/speech`
- `crates/vision`
- `crates/sync`
- `crates/storage`
- `crates/supervisor`
- `bin/vortex`

Cada path `crates/*` MUST ser un crate lib con `edition = "2024"` (propia o heredada del workspace), `src/lib.rs` y `[lints] workspace = true`. El path `bin/vortex` MUST ser el crate binario del workspace.

#### Scenario: Los once libs son miembros

- **WHEN** se lee `vortex-core/Cargo.toml`
- **THEN** los once paths `crates/*` de la lista figuran en `members`
- **AND** existe `vortex-core/<path>/src/lib.rs` para cada uno
- **AND** cada `Cargo.toml` de crate lib declara edición 2024 y hereda los lints del workspace

#### Scenario: Falta un crate lib

- **WHEN** falta cualquiera de esos once paths `crates/*` en `members` o falta su `src/lib.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra el crate que falta

#### Scenario: El binario es miembro

- **WHEN** se lee `vortex-core/Cargo.toml`
- **THEN** `members` incluye `bin/vortex`
- **AND** existe `vortex-core/bin/vortex/Cargo.toml`

#### Scenario: Falta el binario

- **WHEN** `members` no incluye `bin/vortex` o no existe `vortex-core/bin/vortex/Cargo.toml`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `bin/vortex`

### Requirement: Lints de workspace

`vortex-core/Cargo.toml` MUST declarar `[workspace.lints.clippy]` con el grupo `all` en nivel `warn` y `[workspace.lints.rust]` con `unsafe_code` en nivel `forbid`. Cada crate lib MUST heredarlas con `[lints] workspace = true`.

#### Scenario: Lints presentes y heredados

- **WHEN** se inspeccionan el workspace y los once `Cargo.toml`
- **THEN** el workspace fija `clippy` grupo `all` en `warn` y `unsafe_code` en `forbid`
- **AND** cada crate contiene `workspace = true` bajo `[lints]`

#### Scenario: Un crate no hereda lints

- **WHEN** el `Cargo.toml` de un lib no tiene `[lints] workspace = true`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

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
| tracing-appender | 0.2.5 |
| metrics | 0.24.6 |
| metrics-exporter-prometheus | 0.18.3 |
| rcgen | 0.14.10 |
| clap | 4.6.7 |
| toml | 1.2.1 |

El crate `domain` MAY declarar en `[dependencies]` únicamente `thiserror`, `ulid`, `rust_decimal` y `serde`, cada una con `workspace = true`. `domain` MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. `domain` MUST NOT declarar `serde_json` en `[dependencies]`. En particular, `domain` MUST NOT depender de `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort` ni `whisper-rs`. Los demás crates lib MUST NOT declarar `[dependencies]` ni `[dev-dependencies]`. El crate binario `vortex` MAY declarar `toml`, `serde` y `thiserror` según el requirement «Crate binario vortex con Config».

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

`vortex-core/bin/vortex` MUST ser un crate binario con `package.name = "vortex"`, `edition = "2024"` (propia o heredada del workspace), `[lints] workspace = true` y `src/main.rs` que define `fn main`. MAY declarar en `[dependencies]` únicamente `toml`, `serde` y `thiserror`, cada una con `workspace = true` (`serde` MAY activar la feature `derive`). MUST NOT declarar `clap`, otras crates, ni `[dev-dependencies]`. El binario MUST cargar y validar el TOML de configuración al arrancar (capability `vortex-config`).

#### Scenario: Binario con Config presente

- **WHEN** se inspecciona `vortex-core/bin/vortex`
- **THEN** `Cargo.toml` declara `name = "vortex"`, edición 2024 y hereda los lints del workspace
- **AND** existe `src/main.rs` y contiene `fn main`
- **AND** `Cargo.toml` declara `toml`, `serde` y `thiserror` con `workspace = true` y ninguna otra crate en `[dependencies]`
- **AND** no declara `[dev-dependencies]`

#### Scenario: Falta main.rs

- **WHEN** no existe `vortex-core/bin/vortex/src/main.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje indica que falta el binario o `main.rs`

#### Scenario: El binario declara una dependencia fuera de la allowlist

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `clap` u otra crate distinta de `toml`, `serde` y `thiserror`, o declara `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: El binario no hereda lints

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` no tiene `[lints] workspace = true`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` MUST terminar con código 0. `cargo test --workspace` MUST ejecutar los tests unitarios de `domain` de la taxonomía de errores y de los tipos base (`Id`, `Decimal`, `Timestamp`) y los tests de carga/validación de configuración del package `vortex`. El grafo de paquetes MUST incluir el package `vortex`. Los crates lib distintos de `domain` MAY seguir sin lógica de producto.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto, un lib distinto de `domain` declara dependencias, `domain` declara una crate fuera de su allowlist, o `bin/vortex` declara una crate fuera de `{toml, serde, thiserror}`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este change
