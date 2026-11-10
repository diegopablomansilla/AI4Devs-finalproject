# cargo-workspace Specification

## Purpose

Define el workspace Cargo de Vortex Core con los crates lib, el binario `bin/vortex` (composition root de configuración), lints heredados y las dependencias del stack fijadas con versión exacta.

Ticket padre: T-00-01. Changes: T-00-01-02, T-00-01-03, T-00-05, T-00-06, T-00-07, T-00-08, T-00-09, T-00-10, T-00-11, T-00-12, T-00-13, T-09-02. HU: HU-00-01, HU-00-02, HU-00-05, HU-00-06, HU-00-09, HU-09-01.

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
| tracing-subscriber | 0.3.23 |
| tracing-appender | 0.2.5 |
| metrics | 0.24.6 |
| metrics-exporter-prometheus | 0.18.3 |
| rcgen | 0.14.10 |
| clap | 4.6.7 |
| toml | 1.2.1 |
| sha2 | 0.10.9 |

El crate `domain` MAY declarar en `[dependencies]` únicamente `thiserror`, `ulid`, `rust_decimal` y `serde`, cada una con `workspace = true`. `domain` MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. `domain` MUST NOT declarar `serde_json` en `[dependencies]`. En particular, `domain` MUST NOT depender de `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort` ni `whisper-rs`. El crate `storage` MAY declarar en `[dependencies]` únicamente `surrealdb`, `thiserror` y `sha2`, cada una con `workspace = true` (`surrealdb` MAY usar `default-features = false` y MUST activar las features `kv-rocksdb` y `kv-mem`). `storage` MAY declarar `tokio` en `[dev-dependencies]` con `workspace = true` (MAY activar features `macros`, `rt` y/o `rt-multi-thread`) y MUST NOT declarar otra entrada ahí. `storage` MUST NOT declarar `serde_json` ni otras crates en `[dependencies]`. El crate `api` MAY declarar en `[dependencies]` `axum`, `serde`, `serde_json`, `thiserror`, `tower-http` y `tracing` (cada una con `workspace = true`; `serde` MAY activar la feature `derive`; `tower-http` MAY activar features `limit`, `timeout`, `compression-gzip`, `compression-br` y `trace`) y la dependencia de path `domain` al crate `crates/domain` del workspace. `api` MAY declarar `tokio` y `tracing-subscriber` en `[dev-dependencies]` con `workspace = true` (`tokio` MAY activar features `macros`, `rt`, `rt-multi-thread`, `net`, `io-util` y/o `time`) y MUST NOT declarar otra entrada ahí. `api` MUST NOT declarar `axum-server` ni otras crates. El crate `supervisor` MAY declarar en `[dependencies]` únicamente la dependencia de path `domain` al crate `crates/domain` del workspace. `supervisor` MUST NOT declarar `[dev-dependencies]` ni otras crates. Los demás crates lib distintos de `domain`, `storage`, `api` y `supervisor` MUST NOT declarar `[dependencies]` ni `[dev-dependencies]`. El crate binario `vortex` MAY declarar dependencias según el requirement «Crate binario vortex con Config».

#### Scenario: Pins exactos

- **WHEN** se lee `[workspace.dependencies]`
- **THEN** cada crate de la tabla aparece con `=` y la versión indicada

#### Scenario: Versión no fijada

- **WHEN** una entrada de `[workspace.dependencies]` no usa `=` o no coincide con la versión de la tabla
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra la crate

#### Scenario: Un lib declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain`, `storage`, `api` ni `supervisor` contiene `[dependencies]` o `[dev-dependencies]`, o `domain` declara una crate fuera de su allowlist (`thiserror`, `ulid`, `rust_decimal`, `serde` en `[dependencies]`; `serde_json` en `[dev-dependencies]`), o `storage` declara una crate fuera de su allowlist (`surrealdb`, `thiserror`, `sha2` en `[dependencies]`; `tokio` en `[dev-dependencies]`), o `api` declara una crate fuera de su allowlist (`axum`, `domain`, `serde`, `serde_json`, `thiserror`, `tower-http`, `tracing` en `[dependencies]`; `tokio`, `tracing-subscriber` en `[dev-dependencies]`), o `supervisor` declara una crate distinta de `domain`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

#### Scenario: domain declara thiserror

- **WHEN** `vortex-core/crates/domain/Cargo.toml` declara `thiserror` en `[dependencies]` con `workspace = true` y ninguna otra dependencia
- **THEN** la verificación del workspace termina con código 0

#### Scenario: domain declara la allowlist de tipos base

- **WHEN** `vortex-core/crates/domain/Cargo.toml` declara `thiserror`, `ulid`, `rust_decimal` y `serde` en `[dependencies]` y `serde_json` en `[dev-dependencies]`, todas con `workspace = true`, y ninguna otra crate
- **THEN** la verificación del workspace termina con código 0

#### Scenario: storage declara la allowlist

- **WHEN** `vortex-core/crates/storage/Cargo.toml` declara `surrealdb`, `thiserror` y `sha2` en `[dependencies]` con `workspace = true` (`surrealdb` con features `kv-rocksdb` y `kv-mem`) y, si declara `[dev-dependencies]`, únicamente `tokio` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: api declara la allowlist

- **WHEN** `vortex-core/crates/api/Cargo.toml` declara `axum`, `domain` (path a `crates/domain`), `serde`, `serde_json`, `thiserror`, `tower-http` y `tracing` en `[dependencies]` (`workspace = true` en las crates.io) y, si declara `[dev-dependencies]`, únicamente `tokio` y/o `tracing-subscriber` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: supervisor declara path domain

- **WHEN** `vortex-core/crates/supervisor/Cargo.toml` declara `domain` (path a `crates/domain`) en `[dependencies]` y ninguna otra crate
- **THEN** la verificación del workspace termina con código 0

#### Scenario: Un lib distinto de domain declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain`, `storage`, `api` ni `supervisor` (p. ej. `catalog`) contiene `[dependencies]` o `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

#### Scenario: storage declara una dependencia fuera de la allowlist

- **WHEN** `crates/storage/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de la allowlist (por ejemplo `axum`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `storage`

#### Scenario: api declara una dependencia fuera de la allowlist

- **WHEN** `crates/api/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de la allowlist (por ejemplo `clap`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `api`

#### Scenario: supervisor declara una dependencia fuera de la allowlist

- **WHEN** `crates/supervisor/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate distinta de `domain` (por ejemplo `clap`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `supervisor`

#### Scenario: domain declara una dependencia distinta de thiserror

- **WHEN** `crates/domain/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de la allowlist (por ejemplo `clap` o `surrealdb`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `domain`

### Requirement: Crate binario vortex con Config

`vortex-core/bin/vortex` MUST ser un crate binario con `package.name = "vortex"`, `edition = "2024"` (propia o heredada del workspace), `[lints] workspace = true` y `src/main.rs` que define `fn main`. MAY declarar en `[dependencies]` únicamente `toml`, `serde`, `thiserror`, `tracing`, `tracing-subscriber`, `tracing-appender`, `clap`, `api`, `supervisor`, `tokio` y `axum`. Las crates de crates.io MUST usar `workspace = true` (`serde` MAY activar la feature `derive`; `tracing-subscriber` MAY activar features `json` y `env-filter`; `clap` MAY activar la feature `derive`; `tokio` MAY activar features `macros`, `rt-multi-thread`, `net` y/o `signal`). `api` MUST ser dependencia de path al crate `crates/api` del workspace. `supervisor` MUST ser dependencia de path al crate `crates/supervisor` del workspace cuando se declara. MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. MUST NOT declarar `metrics` ni otras crates, ni `serde_json` en `[dependencies]`. El binario MUST exponer la CLI de subcomandos (capability `vortex-cli`), MUST poder cargar y validar el TOML de configuración (capability `vortex-config`), MUST exponer la inicialización de logging (capability `vortex-logging`) y MUST poder servir HTTP con `vortex serve` (capability `vortex-api`). MUST NOT cargar el TOML ni instalar el subscriber global solo por invocar un subcomando stub.

#### Scenario: Binario con Config presente

- **WHEN** se inspecciona `vortex-core/bin/vortex`
- **THEN** `Cargo.toml` declara `name = "vortex"`, edición 2024 y hereda los lints del workspace
- **AND** existe `src/main.rs` y contiene `fn main`
- **AND** `Cargo.toml` declara `toml`, `serde` y `thiserror` con `workspace = true`
- **AND** las únicas otras entradas permitidas en `[dependencies]` son `tracing`, `tracing-subscriber`, `tracing-appender`, `clap`, `api`, `supervisor`, `tokio` y `axum`
- **AND** si declara `[dev-dependencies]`, la única crate permitida es `serde_json` con `workspace = true`

#### Scenario: Falta main.rs

- **WHEN** no existe `vortex-core/bin/vortex/src/main.rs`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje indica que falta el binario o `main.rs`

#### Scenario: El binario declara una dependencia fuera de la allowlist

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `metrics` u otra crate distinta de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap, api, supervisor, tokio, axum}` en `[dependencies]`, o declara en `[dev-dependencies]` una crate distinta de `serde_json`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: El binario no hereda lints

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` no tiene `[lints] workspace = true`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `vortex`

#### Scenario: vortex declara path supervisor

- **WHEN** `vortex-core/bin/vortex/Cargo.toml` declara `supervisor` (path a `crates/supervisor`) en `[dependencies]` junto al resto de la allowlist
- **THEN** la verificación del workspace termina con código 0

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` MUST terminar con código 0. `cargo test --workspace` MUST ejecutar los tests unitarios de `domain` de la taxonomía de errores, de los tipos base (`Id`, `Decimal`, `Timestamp`) y de la agregación de salud, los tests del registro de salud del package `supervisor`, los tests de carga/validación de configuración, de logging y de CLI del package `vortex`, los tests de `vortex serve`, apagado ordenado y `GET /health` del package `vortex`, los tests del router HTTP, de `problem+json`, de middlewares transversales (cuerpo, timeout, cabeceras, span) y de `GET /health` del package `api`, los tests de apertura de SurrealDB embebido del package `storage`, los tests del motor de migraciones con checksum del package `storage` y los tests del harness `storage::testing` del package `storage`. El grafo de paquetes MUST incluir el package `vortex`. Los crates lib distintos de `domain`, `storage`, `api` y `supervisor` MAY seguir sin lógica de producto.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto, un lib distinto de `domain`, `storage`, `api` y `supervisor` declara dependencias, `domain`, `storage`, `api` o `supervisor` declara una crate fuera de su allowlist, o `bin/vortex` declara una crate fuera de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap, api, supervisor, tokio, axum}` en `[dependencies]` o distinta de `serde_json` en `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este change
