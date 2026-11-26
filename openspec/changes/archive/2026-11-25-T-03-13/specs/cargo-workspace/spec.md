# Spec Delta

Ticket: T-03-13. HU: HU-03-03, HU-03-04.

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
| jsonschema | 0.61.1 |
| serde_yaml | 0.9.34 |

El crate `domain` MAY declarar en `[dependencies]` únicamente `thiserror`, `ulid`, `rust_decimal` y `serde`, cada una con `workspace = true`. `domain` MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true` y MUST NOT declarar otra entrada ahí. `domain` MUST NOT declarar `serde_json` en `[dependencies]`. En particular, `domain` MUST NOT depender de `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort` ni `whisper-rs`. El crate `storage` MAY declarar en `[dependencies]` `surrealdb`, `thiserror`, `sha2` y `tokio`, cada una con `workspace = true` (`surrealdb` MAY usar `default-features = false` y MUST activar las features `kv-rocksdb` y `kv-mem`; `tokio` MAY activar la feature `sync`), y la dependencia de path `domain` al crate `crates/domain` del workspace. `storage` MAY declarar `tokio` en `[dev-dependencies]` con `workspace = true` (MAY activar features `macros`, `rt` y/o `rt-multi-thread`) y MUST NOT declarar otra entrada ahí. `storage` MUST NOT declarar `serde_json` ni otras crates distintas de esa allowlist en `[dependencies]`. El crate `api` MAY declarar en `[dependencies]` `axum`, `serde`, `serde_json`, `thiserror`, `tower-http`, `tracing`, `metrics`, `metrics-exporter-prometheus` y `tokio` (cada una con `workspace = true`; `serde` MAY activar la feature `derive`; `tower-http` MAY activar features `limit`, `timeout`, `compression-gzip`, `compression-br` y `trace`; `tokio` MAY activar la feature `time`) y las dependencias de path `domain`, `catalog` y `search` a los crates `crates/domain`, `crates/catalog` y `crates/search` del workspace. `api` MAY declarar `tokio`, `tracing-subscriber`, `jsonschema` y `serde_yaml` en `[dev-dependencies]` con `workspace = true` (`tokio` MAY activar features `macros`, `rt`, `rt-multi-thread`, `net`, `io-util` y/o `time`; `jsonschema` MUST usar `default-features = false`). MUST NOT declarar `jsonschema` ni `serde_yaml` en `[dependencies]`. `api` MUST NOT declarar `axum-server` ni otras crates. El crate `supervisor` MAY declarar en `[dependencies]` únicamente la dependencia de path `domain` al crate `crates/domain` del workspace. `supervisor` MUST NOT declarar `[dev-dependencies]` ni otras crates. El crate `catalog` MAY declarar en `[dependencies]` la dependencia de path `domain` al crate `crates/domain` del workspace y `serde` y `serde_json` (cada una con `workspace = true`; `serde` MAY activar la feature `derive`). `catalog` MAY declarar `tokio` en `[dev-dependencies]` con `workspace = true` (MAY activar features `macros`, `rt` y/o `rt-multi-thread`) y MUST NOT declarar otra entrada ahí. `catalog` MUST NOT declarar `surrealdb` ni el crate `storage`. El crate `search` MAY declarar en `[dependencies]` la dependencia de path `domain` al crate `crates/domain` del workspace y `meilisearch-sdk`, `tokio`, `serde` y `serde_json` (cada una con `workspace = true`; `serde` MAY activar la feature `derive`; `tokio` MAY activar features `time`, `rt` y/o `macros`). `search` MAY declarar `tokio` en `[dev-dependencies]` con `workspace = true` (MAY activar features `macros`, `rt` y/o `rt-multi-thread`) y MUST NOT declarar otra entrada ahí. `search` MUST NOT declarar `surrealdb` ni `reqwest` de forma directa. El crate `sync` MAY declarar en `[dependencies]` las dependencias de path `domain` y `storage` a los crates `crates/domain` y `crates/storage` del workspace, y `tokio` con `workspace = true` (MAY activar features `macros`, `rt`, `rt-multi-thread`, `time` y/o `sync`). `sync` MAY declarar `search` (path a `crates/search`) y `tokio` en `[dev-dependencies]` (`workspace = true` en `tokio`; MAY activar features `macros`, `rt` y/o `rt-multi-thread`) y MUST NOT declarar otra entrada ahí. `sync` MUST NOT declarar `meilisearch-sdk` ni `surrealdb` de forma directa. Los demás crates lib distintos de `domain`, `storage`, `api`, `supervisor`, `catalog`, `search` y `sync` MUST NOT declarar `[dependencies]` ni `[dev-dependencies]`. El crate binario `vortex` MAY declarar dependencias según el requirement «Crate binario vortex con Config».

#### Scenario: Pins exactos

- **WHEN** se lee `[workspace.dependencies]`
- **THEN** cada crate de la tabla aparece con `=` y la versión indicada

#### Scenario: Versión no fijada

- **WHEN** una entrada de `[workspace.dependencies]` no usa `=` o no coincide con la versión de la tabla
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra la crate

#### Scenario: Un lib declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain`, `storage`, `api`, `supervisor`, `catalog`, `search` ni `sync` contiene `[dependencies]` o `[dev-dependencies]`, o `domain` declara una crate fuera de su allowlist (`thiserror`, `ulid`, `rust_decimal`, `serde` en `[dependencies]`; `serde_json` en `[dev-dependencies]`), o `storage` declara una crate fuera de su allowlist (`surrealdb`, `thiserror`, `sha2`, `tokio`, `domain` en `[dependencies]`; `tokio` en `[dev-dependencies]`), o `api` declara una crate fuera de su allowlist (`axum`, `domain`, `catalog`, `search`, `serde`, `serde_json`, `thiserror`, `tower-http`, `tracing`, `metrics`, `metrics-exporter-prometheus` en `[dependencies]`; `tokio`, `tracing-subscriber`, `jsonschema`, `serde_yaml` en `[dev-dependencies]`), o `supervisor` declara una crate distinta de `domain`, o `catalog` declara una crate fuera de su allowlist (`domain`, `serde`, `serde_json` en `[dependencies]`; `tokio` en `[dev-dependencies]`), o `search` declara una crate fuera de su allowlist (`domain`, `meilisearch-sdk`, `tokio`, `serde`, `serde_json` en `[dependencies]`; `tokio` en `[dev-dependencies]`), o `sync` declara una crate fuera de su allowlist (`domain`, `storage`, `tokio` en `[dependencies]`; `search`, `tokio` en `[dev-dependencies]`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra ese crate

#### Scenario: domain declara thiserror

- **WHEN** `vortex-core/crates/domain/Cargo.toml` declara `thiserror` en `[dependencies]` con `workspace = true` y ninguna otra dependencia
- **THEN** la verificación del workspace termina con código 0

#### Scenario: domain declara la allowlist de tipos base

- **WHEN** `vortex-core/crates/domain/Cargo.toml` declara `thiserror`, `ulid`, `rust_decimal` y `serde` en `[dependencies]` y `serde_json` en `[dev-dependencies]`, todas con `workspace = true`, y ninguna otra crate
- **THEN** la verificación del workspace termina con código 0

#### Scenario: storage declara la allowlist

- **WHEN** `vortex-core/crates/storage/Cargo.toml` declara `surrealdb`, `thiserror`, `sha2`, `tokio` (feature `sync`) y `domain` (path a `crates/domain`) en `[dependencies]` (`workspace = true` en las crates.io; `surrealdb` con features `kv-rocksdb` y `kv-mem`) y, si declara `[dev-dependencies]`, únicamente `tokio` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: api declara la allowlist

- **WHEN** `vortex-core/crates/api/Cargo.toml` declara `axum`, `domain` (path a `crates/domain`), `serde`, `serde_json`, `thiserror`, `tower-http` y `tracing` en `[dependencies]` (`workspace = true` en las crates.io) y, si declara `[dev-dependencies]`, únicamente `tokio` y/o `tracing-subscriber` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: api declara la allowlist prometheus

- **WHEN** `vortex-core/crates/api/Cargo.toml` declara `axum`, `domain` (path a `crates/domain`), `serde`, `serde_json`, `thiserror`, `tower-http`, `tracing`, `metrics` y `metrics-exporter-prometheus` en `[dependencies]` (`workspace = true` en las crates.io) y, si declara `[dev-dependencies]`, únicamente `tokio` y/o `tracing-subscriber` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: api declara la allowlist openapi

- **WHEN** `vortex-core/crates/api/Cargo.toml` declara `axum`, `domain` (path a `crates/domain`), `serde`, `serde_json`, `thiserror`, `tower-http`, `tracing`, `metrics` y `metrics-exporter-prometheus` en `[dependencies]` (`workspace = true` en las crates.io) y, si declara `[dev-dependencies]`, `tokio`, `tracing-subscriber`, `jsonschema` y `serde_yaml` con `workspace = true` (`jsonschema` con `default-features = false`)
- **THEN** la verificación del workspace termina con código 0

#### Scenario: api declara path catalog

- **WHEN** `vortex-core/crates/api/Cargo.toml` declara `catalog` (path a `crates/catalog`) en `[dependencies]` junto al resto de la allowlist de `api`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: api declara path search

- **WHEN** `vortex-core/crates/api/Cargo.toml` declara `search` (path a `crates/search`) en `[dependencies]` junto al resto de la allowlist de `api`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: supervisor declara path domain

- **WHEN** `vortex-core/crates/supervisor/Cargo.toml` declara `domain` (path a `crates/domain`) en `[dependencies]` y ninguna otra crate
- **THEN** la verificación del workspace termina con código 0

#### Scenario: catalog declara path domain

- **WHEN** `vortex-core/crates/catalog/Cargo.toml` declara `domain` (path a `crates/domain`) en `[dependencies]` y, si declara `[dev-dependencies]`, únicamente `tokio` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: catalog declara serde y serde_json

- **WHEN** `vortex-core/crates/catalog/Cargo.toml` declara `domain` (path a `crates/domain`), `serde` y `serde_json` en `[dependencies]` (`workspace = true` en las crates.io; `serde` MAY activar `derive`) y, si declara `[dev-dependencies]`, únicamente `tokio` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: search declara la allowlist Meilisearch

- **WHEN** `vortex-core/crates/search/Cargo.toml` declara `domain` (path a `crates/domain`), `meilisearch-sdk`, `tokio`, `serde` y `serde_json` en `[dependencies]` (`workspace = true` en las crates.io; `serde` MAY activar `derive`; `tokio` MAY activar `time`) y, si declara `[dev-dependencies]`, únicamente `tokio` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: sync declara la allowlist del worker

- **WHEN** `vortex-core/crates/sync/Cargo.toml` declara `domain` (path a `crates/domain`), `storage` (path a `crates/storage`) y `tokio` en `[dependencies]` (`workspace = true` en `tokio`) y, si declara `[dev-dependencies]`, únicamente `search` (path a `crates/search`) y/o `tokio` con `workspace = true`
- **THEN** la verificación del workspace termina con código 0

#### Scenario: Un lib distinto de domain declara dependencias

- **WHEN** el `Cargo.toml` de un lib que no es `domain`, `storage`, `api`, `supervisor`, `catalog`, `search` ni `sync` (p. ej. `recommend`) contiene `[dependencies]` o `[dev-dependencies]`
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

#### Scenario: api declara jsonschema en dependencies

- **WHEN** `crates/api/Cargo.toml` declara `jsonschema` o `serde_yaml` en `[dependencies]`
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `api`

#### Scenario: supervisor declara una dependencia fuera de la allowlist

- **WHEN** `crates/supervisor/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate distinta de `domain` (por ejemplo `clap`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `supervisor`

#### Scenario: catalog declara una dependencia fuera de la allowlist

- **WHEN** `crates/catalog/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de su allowlist (por ejemplo `surrealdb` o `clap`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `catalog`

#### Scenario: search declara una dependencia fuera de la allowlist

- **WHEN** `crates/search/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de su allowlist (por ejemplo `surrealdb`, `reqwest` o `clap`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `search`

#### Scenario: sync declara una dependencia fuera de la allowlist

- **WHEN** `crates/sync/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de su allowlist (por ejemplo `meilisearch-sdk` o `surrealdb` directos)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `sync`

#### Scenario: domain declara una dependencia distinta de thiserror

- **WHEN** `crates/domain/Cargo.toml` declara en `[dependencies]` o `[dev-dependencies]` una crate fuera de la allowlist (por ejemplo `clap` o `surrealdb`)
- **THEN** la verificación del workspace termina con código distinto de 0
- **AND** el mensaje nombra `domain`
