# cargo-workspace Specification

## Purpose

Define el workspace Cargo de Vortex Core con los crates lib vacíos, lints heredados y las dependencias del stack fijadas con versión exacta, para que el binario y el código de dominio se agreguen después sobre una base reproducible.

Ticket padre: T-00-01. Change: T-00-01-02. HU: HU-00-01.

## Requirements

### Requirement: Workspace con los crates lib de Vortex Core

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

Cada miembro MUST ser un crate lib con `edition = "2024"` (propia o heredada del workspace), `src/lib.rs` y `[lints] workspace = true`. El workspace MUST NOT incluir `bin/vortex`.

#### Scenario: Los once libs son miembros

- **WHEN** se lee `vortex-core/Cargo.toml`
- **THEN** los once paths anteriores figuran en `members`
- **AND** existe `vortex-core/<path>/src/lib.rs` para cada uno
- **AND** cada `Cargo.toml` de crate declara edición 2024 y hereda los lints del workspace

#### Scenario: Falta un crate lib

- **WHEN** falta cualquiera de esos once paths en `members` o falta su `src/lib.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra el crate que falta

#### Scenario: El binario todavía no es miembro

- **WHEN** `members` incluye `bin/vortex` o existe `vortex-core/bin/vortex/Cargo.toml`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje indica que `bin/vortex` no pertenece a este cambio

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
| tracing | 0.1.44 |
| tracing-appender | 0.2.5 |
| metrics | 0.24.6 |
| metrics-exporter-prometheus | 0.18.3 |
| rcgen | 0.14.10 |
| clap | 4.6.7 |

Ningún crate lib MUST declarar `[dependencies]` ni `[dev-dependencies]` en este cambio. En particular, `domain` MUST NOT depender de `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort` ni `whisper-rs`.

#### Scenario: Pins exactos

- **WHEN** se lee `[workspace.dependencies]`
- **THEN** cada crate de la tabla aparece con `=` y la versión indicada

#### Scenario: Versión no fijada

- **WHEN** una entrada de `[workspace.dependencies]` no usa `=` o no coincide con la versión de la tabla
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra la crate

#### Scenario: Un lib declara dependencias

- **WHEN** el `Cargo.toml` de cualquiera de los once libs contiene `[dependencies]` o `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo test --workspace` y `cargo clippy --workspace -- -D warnings` MUST terminar con código 0. No hace falta ningún test de comportamiento de producto: los libs están vacíos.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro, sobra `bin/vortex`, un pin no es exacto o un lib declara dependencias
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este cambio

