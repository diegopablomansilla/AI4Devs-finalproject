# Proposal

## Ticket padre

- **Ticket:** [T-00-01](../../../docs/7-tickets.md) — Workspace Cargo y crates vacíos
- **Change:** `T-00-01-02`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias)
- **Hace (ticket, recorte de este ZZ):** `vortex-core/Cargo.toml` (workspace), lints de Clippy en `[workspace.lints]`, `[workspace.dependencies]` con versiones fijadas, y los crates lib de [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio) / [3.2](../../../docs/3-componentes.md#32-mapa-de-crates-y-regla-de-dependencias): `api`, `domain`, `catalog`, `search`, `recommend`, `ai`, `speech`, `vision`, `sync`, `storage`, `supervisor`
- **Listo cuando (ticket, recorte de este ZZ):** `cargo build`, `cargo test` y `cargo clippy -- -D warnings` sobre esos libs terminan sin errores. El `Listo cuando` del ticket sobre **todo** el workspace, incluido `bin/vortex`, queda para `T-00-01-03`

## Why

T-00-01-01 dejó el canal Rust y la higiene de Git, sin crates. El siguiente slice de T-00-01 necesita un workspace Cargo reproducible: miembros lib vacíos, lints heredados y la tabla de dependencias del stack ya fijada, para que T-00-01-03 solo agregue el binario y los tickets siguientes consuman versiones únicas.

## What Changes

- Crear `vortex-core/Cargo.toml` con `resolver = "3"`, edición 2024 y los 11 crates lib como `members` (sin `bin/vortex`)
- Declarar `[workspace.lints]` (Clippy + `unsafe_code`) y hacer que cada lib las herede
- Declarar `[workspace.dependencies]` con versión **exacta** (`=`) para el stack de [7.2](../../../docs/7-tickets.md): tokio, axum 0.8, tower-http, axum-server, surrealdb 2.x, meilisearch-sdk, reqwest, ort, tokenizers, whisper-rs, rust_decimal, ulid, thiserror, serde, tracing, tracing-appender, metrics, metrics-exporter-prometheus, rcgen, clap
- Un `Cargo.toml` + `src/lib.rs` vacío por crate, sin dependencias de otros crates ni de motores
- `Cargo.lock` del workspace (sin descargar motores: ningún lib depende de ellos todavía)
- Script de verificación (happy + unhappy) de miembros, ausencia de `bin/vortex` y pins `=`

## Non-goals

Este ZZ **no** cubre:

- Miembro `bin/vortex` ni `fn main` → **T-00-01-03** (cierra el Listo cuando del ticket)
- Toolchain, rustfmt y `.gitignore` → **T-00-01-01** (ya en `develop`)
- Código funcional de dominio, API o adaptadores (ticket **No hace**)
- CI de GitHub Actions → **T-09-01**
- Script que falle si `domain` depende de SurrealDB, Meilisearch, ort o whisper → **T-00-02**
- Crate `pos` (PC-03, T-08-01) ni `vectra-web`

## Capabilities

### New Capabilities

- `cargo-workspace`: workspace `vortex-core` con libs vacíos, lints heredados y dependencias del stack fijadas con `=`

### Modified Capabilities

- Ninguna. `rust-toolchain` (T-00-01-01) no cambia de requisitos

## Impact

- **Repo:** `vortex-core/**`, `Cargo.lock` bajo `vortex-core/`, `scripts/check-workspace.sh`
- **APIs / OpenAPI:** ninguna
- **Build:** primer `cargo build/test/clippy` del repo; no compila surrealdb, ort ni whisper porque ningún lib los declara en `[dependencies]`
- **Siguiente spec:** T-00-01-03 agrega `bin/vortex` al mismo workspace
- **Supuesto de versiones:** pins al estable de crates.io en 2026-11-09, con dos excepciones documentadas en design (`surrealdb` se queda en 2.7.0; `ort` solo tiene 2.0.0-rc)
