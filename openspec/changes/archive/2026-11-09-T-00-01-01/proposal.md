# Proposal

## Ticket padre

- **Ticket:** [T-00-01](../../../docs/7-tickets.md) — Workspace Cargo y crates vacíos
- **Change:** `T-00-01-01`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias)
- **Hace (ticket, recorte de este ZZ):** `rust-toolchain.toml`, `rustfmt.toml` y `.gitignore` (`models/`, `target/`, datos locales)
- **Listo cuando (ticket, recorte de este ZZ):** el canal de Rust está fijado, `rustfmt` tiene convención de proyecto y Git ignora artefactos de build, modelos y datos locales. El `Listo cuando` completo del ticket (`cargo build/test/clippy --workspace`) queda para `T-00-01-02` y `T-00-01-03`

## Why

Vortex Core todavía no tiene toolchain Rust versionada ni higiene de repositorio para un workspace Cargo. Sin canal fijado, `rustfmt` de proyecto y exclusiones de Git, el workspace de T-00-01-02 no sería reproducible ni CI-ready. Este ZZ es el primer slice de T-00-01 (~0,5 h) para dejar esa base antes de crear crates.

## What Changes

- Agregar `rust-toolchain.toml` en la raíz del repositorio: canal **estable fijado** (`1.99.0`, el estable actual del Nodo Maestro) y componentes `rustfmt` y `clippy`
- Agregar `rustfmt.toml` en la raíz: edición **2024** y convención mínima de formato
- Ampliar `.gitignore` existente: `models/`, `target/` en cualquier nivel (hoy solo `/target/` en raíz) y directorios de datos locales de desarrollo
- Agregar un script de verificación (happy + unhappy path) que falle si falta canal, edición o entradas de ignore; no introduce workspace Cargo

## Non-goals

Este ZZ **no** cubre:

- `vortex-core/Cargo.toml`, `[workspace.lints]`, `[workspace.dependencies]` ni crates lib vacíos → **T-00-01-02**
- Miembro `bin/vortex` ni el `Listo cuando` del ticket sobre todo el workspace (`cargo build/test/clippy --workspace`) → **T-00-01-03**
- Código funcional de Vortex Core (ticket **No hace**)
- CI de GitHub Actions → **T-09-01**
- Verificación automática de dependencias de `domain` → **T-00-02**
- Versiones de crates de aplicación (`tokio`, `axum`, `surrealdb`, …) → **T-00-01-02**

## Capabilities

### New Capabilities

- `rust-toolchain`: contrato de toolchain Rust fijada, formato con `rustfmt` y exclusiones Git para artefactos de build, modelos y datos locales

### Modified Capabilities

- Ninguna (inventario de specs vacío)

## Impact

- **Repo:** raíz (`rust-toolchain.toml`, `rustfmt.toml`, `.gitignore`); script de chequeo en `scripts/`
- **APIs / OpenAPI:** ninguna
- **Cargo / crates:** ninguno en este ZZ
- **Siguientes specs:** T-00-01-02 asume estos archivos ya presentes; T-09-01 podrá usar el mismo canal y `rustfmt --check`
- **Riesgo:** bajo; no hay código de producto. El pin `1.99.0` se documenta para que T-09-01 use el mismo canal
