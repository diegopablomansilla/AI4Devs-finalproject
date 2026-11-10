# Spec Delta

Ticket padre: T-00-12. Change: T-00-12. HU: HU-00-09.

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
| sha2 | 0.10.9 |

El crate `domain` MAY declarar en `[dependencies]` únicamente `thiserror`, `ulid`, `rust_decimal` y `serde`, cada una con `workspace = true`. `domain` MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. `domain` MUST NOT declarar `serde_json` en `[dependencies]`. En particular, `domain` MUST NOT depender de `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort` ni `whisper-rs`. El crate `storage` MAY declarar en `[dependencies]` únicamente `surrealdb`, `thiserror` y `sha2`, cada una con `workspace = true` (`surrealdb` MAY usar `default-features = false` y MUST activar las features `kv-rocksdb` y `kv-mem`). `storage` MAY declarar `tokio` en `[dev-dependencies]` con `workspace = true` (MAY activar features `macros`, `rt` y/o `rt-multi-thread`) y MUST NOT declarar otra entrada ahí. `storage` MUST NOT declarar `serde_json` ni otras crates en `[dependencies]`. El crate `api` MAY declarar en `[dependencies]` `axum`, `serde`, `serde_json`, `thiserror`, `tower-http` y `tracing` (cada una con `workspace = true`; `serde` MAY activar la feature `derive`; `tower-http` MAY activar features `limit`, `timeout`, `compression-gzip`, `compression-br` y `trace`) y la dependencia de path `domain` al crate `crates/domain` del workspace. `api` MAY declarar `tokio` y `tracing-subscriber` en `[dev-dependencies]` con `workspace = true` (`tokio` MAY activar features `macros`, `rt`, `rt-multi-thread`, `net`, `io-util` y/o `time`) y MUST NOT declarar otra entrada ahí. `api` MUST NOT declarar `axum-server` ni otras crates. Los demás crates lib distintos de `domain`, `storage` y `api` MUST NOT declarar `[dependencies]` ni `[dev-dependencies]`. El crate binario `vortex` MAY declarar dependencias según el requirement «Crate binario vortex con Config».

#### Scenario: Pins exactos

- **WHEN** se lee `[workspace.dependencies]`
- **THEN** cada crate de la tabla aparece con `=` y la versión indicada

#### Scenario: Versión no fijada

- **WHEN** una entrada de `[workspace.dependencies]` no usa `=` o no coincide con la versión de la tabla
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra la crate

#### Scenario: Un lib declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain`, `storage` ni `api` contiene `[dependencies]` o `[dev-dependencies]`, o `domain` declara una crate fuera de su allowlist (`thiserror`, `ulid`, `rust_decimal`, `serde` en `[dependencies]`; `serde_json` en `[dev-dependencies]`), o `storage` declara una crate fuera de su allowlist (`surrealdb`, `thiserror`, `sha2` en `[dependencies]`; `tokio` en `[dev-dependencies]`), o `api` declara una crate fuera de su allowlist (`axum`, `domain`, `serde`, `serde_json`, `thiserror`, `tower-http`, `tracing` en `[dependencies]`; `tokio`, `tracing-subscriber` en `[dev-dependencies]`)
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

#### Scenario: Un lib distinto de domain declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain`, `storage` ni `api` (p. ej. `catalog`) contiene `[dependencies]` o `[dev-dependencies]`
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

#### Scenario: domain declara una dependencia distinta de thiserror

- **WHEN** `crates/domain/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de la allowlist (por ejemplo `clap` o `surrealdb`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `domain`

### Requirement: El workspace lib compila, testea y pasa Clippy

Sobre `vortex-core`, `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` MUST terminar con código 0. `cargo test --workspace` MUST ejecutar los tests unitarios de `domain` de la taxonomía de errores y de los tipos base (`Id`, `Decimal`, `Timestamp`), los tests de carga/validación de configuración, de logging y de CLI del package `vortex`, los tests de `vortex serve` y apagado ordenado del package `vortex`, los tests del router HTTP, de `problem+json` y de middlewares transversales (cuerpo, timeout, cabeceras, span) del package `api`, los tests de apertura de SurrealDB embebido del package `storage`, los tests del motor de migraciones con checksum del package `storage` y los tests del harness `storage::testing` del package `storage`. El grafo de paquetes MUST incluir el package `vortex`. Los crates lib distintos de `domain`, `storage` y `api` MAY seguir sin lógica de producto.

#### Scenario: Build, tests y clippy en verde

- **WHEN** se ejecutan esos tres comandos con el manifiesto `vortex-core/Cargo.toml`
- **THEN** los tres terminan con código 0

#### Scenario: El package vortex forma parte del workspace

- **WHEN** se consulta `cargo metadata` del manifiesto `vortex-core/Cargo.toml`
- **THEN** existe un package de nombre `vortex`

#### Scenario: Árbol de workspace inválido

- **WHEN** falta un miembro (incluido `bin/vortex`), un pin no es exacto, un lib distinto de `domain`, `storage` y `api` declara dependencias, `domain`, `storage` o `api` declara una crate fuera de su allowlist, o `bin/vortex` declara una crate fuera de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap, api, tokio, axum}` en `[dependencies]` o distinta de `serde_json` en `[dev-dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** no se considera cumplido el criterio de este change
