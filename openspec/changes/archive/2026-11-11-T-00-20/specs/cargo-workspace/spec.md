# Spec Delta

Ticket padre: T-00-20. Change: T-00-20. HU: HU-00-06.

## MODIFIED Requirements

### Requirement: Crate binario vortex con Config

`vortex-core/bin/vortex` MUST ser un crate binario con `package.name = "vortex"`, `edition = "2024"` (propia o heredada del workspace), `[lints] workspace = true` y `src/main.rs` que define `fn main`. MAY declarar en `[dependencies]` únicamente `toml`, `serde`, `thiserror`, `tracing`, `tracing-subscriber`, `tracing-appender`, `clap`, `api`, `supervisor`, `storage`, `tokio` y `axum`. Las crates de crates.io MUST usar `workspace = true` (`serde` MAY activar la feature `derive`; `tracing-subscriber` MAY activar features `json` y `env-filter`; `clap` MAY activar la feature `derive`; `tokio` MAY activar features `macros`, `rt-multi-thread`, `net` y/o `signal`). `api` MUST ser dependencia de path al crate `crates/api` del workspace. `supervisor` MUST ser dependencia de path al crate `crates/supervisor` del workspace cuando se declara. `storage` MUST ser dependencia de path al crate `crates/storage` del workspace cuando se declara. MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. MUST NOT declarar `metrics` ni otras crates, ni `serde_json` en `[dependencies]`. El binario MUST exponer la CLI de subcomandos (capability `vortex-cli`), MUST poder cargar y validar el TOML de configuración (capability `vortex-config`), MUST exponer la inicialización de logging (capability `vortex-logging`) y MUST poder servir HTTP con `vortex serve` (capability `vortex-api`). MUST NOT cargar el TOML ni instalar el subscriber global solo por invocar un subcomando stub.

#### Scenario: Binario con Config presente

- **WHEN** se inspecciona `vortex-core/bin/vortex`
- **THEN** `Cargo.toml` declara `name = "vortex"`, edición 2024 y hereda los lints del workspace
- **AND** existe `src/main.rs` y contiene `fn main`
- **AND** `Cargo.toml` declara `toml`, `serde` y `thiserror` con `workspace = true`
- **AND** las únicas otras entradas permitidas en `[dependencies]` son `tracing`, `tracing-subscriber`, `tracing-appender`, `clap`, `api`, `supervisor`, `storage`, `tokio` y `axum`
- **AND** si declara `[dev-dependencies]`, la única crate permitida es `serde_json` con `workspace = true`

#### Scenario: Falta main.rs

- **WHEN** no existe `vortex-core/bin/vortex/src/main.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje indica que falta el binario o `main.rs`

#### Scenario: El binario declara una dependencia fuera de la allowlist

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `metrics` u otra crate distinta de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap, api, supervisor, storage, tokio, axum}` en `[dependencies]`, o declara en `[dev-dependencies]` una crate distinta de `serde_json`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: El binario no hereda lints

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` no tiene `[lints] workspace = true`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: vortex declara path supervisor

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `supervisor` (path a `crates/supervisor`) en `[dependencies]` junto al resto de la allowlist
- **THEN** la verificación del workspace termina con código 0

#### Scenario: vortex declara path storage

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `storage` (path a `crates/storage`) en `[dependencies]` junto al resto de la allowlist
- **THEN** la verificación del workspace termina con código 0

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` MUST terminar con código 0. `cargo test --workspace` MUST ejecutar los tests unitarios de `domain` de la taxonomía de errores, de los tipos base (`Id`, `Decimal`, `Timestamp`) y de la agregación de salud, los tests del registro de salud del package `supervisor`, los tests de carga/validación de configuración, de logging y de CLI del package `vortex`, los tests de `vortex serve`, apagado ordenado, arranque ordenado (espera `/health` `ok` con Storage `up`, base inaccesible, `datos` no escribible), `GET /health` y `GET /metrics` del package `vortex`, los tests del router HTTP, de `problem+json`, de middlewares transversales (cuerpo, timeout, cabeceras, span), de `GET /health` y de `GET /metrics` (histograma de petición previa) del package `api`, los tests de apertura de SurrealDB embebido del package `storage`, los tests del motor de migraciones con checksum del package `storage` y los tests del harness `storage::testing` del package `storage`. El grafo de paquetes MUST incluir el package `vortex`. Los crates lib distintos de `domain`, `storage`, `api` y `supervisor` MAY seguir sin lógica de producto.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto, un lib distinto de `domain`, `storage`, `api` y `supervisor` declara dependencias, `domain`, `storage`, `api` o `supervisor` declara una crate fuera de su allowlist, o `bin/vortex` declara una crate fuera de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap, api, supervisor, storage, tokio, axum}` en `[dependencies]` o distinta de `serde_json` en `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este change
