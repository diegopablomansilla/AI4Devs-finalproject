# Spec Delta

Ticket padre: T-00-01. Change: T-00-01-03. HU: HU-00-01.

## REMOVED Requirements

### Requirement: Workspace con los crates lib de Vortex Core

**Reason:** T-00-01-03 agrega el member `bin/vortex`. La lista exacta de once libs y el escenario «El binario todavía no es miembro» dejan de ser válidos.

**Migration:** Usar el requisito «Workspace con los crates de Vortex Core», que conserva los once libs y exige `bin/vortex`.

## MODIFIED Requirements

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo test --workspace` y `cargo clippy --workspace -- -D warnings` MUST terminar con código 0. No hace falta ningún test de comportamiento de producto: los libs y el binario están vacíos. El grafo de paquetes MUST incluir el package `vortex`.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto o un crate declara dependencias
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este cambio

## ADDED Requirements

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

### Requirement: Crate binario vortex vacío

`vortex-core/bin/vortex` MUST ser un crate binario con `package.name = "vortex"`, `edition = "2024"` (propia o heredada del workspace), `[lints] workspace = true` y `src/main.rs` que define `fn main` sin lógica de producto. MUST NOT declarar `[dependencies]` ni `[dev-dependencies]`.

#### Scenario: Binario vacío presente

- **WHEN** se inspecciona `vortex-core/bin/vortex`
- **THEN** `Cargo.toml` declara `name = "vortex"`, edición 2024 y hereda los lints del workspace
- **AND** existe `src/main.rs` y contiene `fn main`
- **AND** `Cargo.toml` no declara `[dependencies]` ni `[dev-dependencies]`

#### Scenario: Falta main.rs

- **WHEN** no existe `vortex-core/bin/vortex/src/main.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje indica que falta el binario o `main.rs`

#### Scenario: El binario declara dependencias

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` contiene `[dependencies]` o `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: El binario no hereda lints

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` no tiene `[lints] workspace = true`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

## Tests previstos

| Test | Tipo | Escenarios |
|---|---|---|
| `scripts/check-workspace.sh` sobre el repo | happy | El binario es miembro; binario vacío presente; once libs siguen |
| Fixture sin member `bin/vortex` | unhappy | Falta el binario |
| Fixture sin `src/main.rs` | unhappy | Falta main.rs |
| Fixture con `[dependencies]` en `bin/vortex` | unhappy | El binario declara dependencias |
| Fixture sin `[lints] workspace = true` en `bin/vortex` | unhappy | El binario no hereda lints |
| `cargo metadata --manifest-path vortex-core/Cargo.toml` | happy | El package vortex forma parte del workspace |
| `cargo build/test/clippy --workspace` en `vortex-core` | happy | Build, tests y clippy en verde |

No hay tests de API ni de dominio: este ZZ no agrega comportamiento de producto. Las fixtures unhappy de T-00-01-02 (falta un lib, pin sin `=`, lib con dependencias) MUST seguir fallando.
