# Proposal

## Ticket padre

- **Ticket:** [T-00-01](../../../docs/7-tickets.md) — Workspace Cargo y crates vacíos
- **Change:** `T-00-01-03`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias)
- **Hace (ticket, recorte de este ZZ):** miembro `bin/vortex` (`fn main() {}`) en el workspace de [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio)
- **Listo cuando (ticket, este ZZ cierra el ticket):** `cargo build --workspace`, `cargo test --workspace` y `cargo clippy --workspace -- -D warnings` terminan sin errores sobre **todo** el workspace, incluido el binario

## Why

T-00-01-02 dejó once libs vacíos y un `scripts/check-workspace.sh` que **rechaza** `bin/vortex`. El ticket T-00-01 y HU-00-01 todavía no se cierran: falta el crate de composición. Este ZZ (~0,5 h) agrega el binario vacío y hace que build, test y clippy cubran el workspace completo.

## What Changes

- Agregar el member `bin/vortex` a `vortex-core/Cargo.toml` (paquete `vortex`, `src/main.rs` con `fn main() {}`)
- El binario hereda edición 2024 y lints del workspace; sin `[dependencies]` ni `[dev-dependencies]` (la CLI y el composition root son tickets posteriores)
- Invertir en `scripts/check-workspace.sh` la regla «`bin/vortex` no pertenece a este cambio»: ahora es miembro obligatorio
- Cerrar el **Listo cuando** del ticket: `cargo build/test/clippy --workspace` incluye el binario

## Non-goals

Este ZZ **no** cubre:

- Toolchain, rustfmt y `.gitignore` → **T-00-01-01** (ya en `develop`)
- Workspace, lints, pins `=` y libs vacíos → **T-00-01-02** (ya en `develop`)
- Código funcional, CLI (`clap`), `serve` u otros subcomandos (ticket **No hace**; CLI → **T-00-07**)
- CI de GitHub Actions → **T-09-01**
- Script `cargo tree` de `domain` → **T-00-02**
- Crate `pos` (PC-03, T-08-01) ni `vectra-web`

## Capabilities

### New Capabilities

- Ninguna. El binario es un member más del workspace ya especificado.

### Modified Capabilities

- `cargo-workspace`: el workspace MUST incluir `bin/vortex` (paquete `vortex`, `fn main() {}`); se elimina el requisito de T-00-01-02 que prohibía ese member. `cargo build/test/clippy --workspace` MUST pasar sobre libs **y** binario.

## Impact

- **Repo:** `vortex-core/Cargo.toml` (un member más), `vortex-core/bin/vortex/`, `scripts/check-workspace.sh`, `vortex-core/Cargo.lock` si el lockfile cambia
- **APIs / OpenAPI:** ninguna
- **Build:** primer binario del workspace; sigue sin compilar motores porque el binario no declara dependencias
- **Ticket:** cierra T-00-01; desbloquea T-09-01, T-00-03, T-00-04, T-00-05
- **BREAKING** (specs internas): `check-workspace.sh` deja de fallar si existe `bin/vortex` y pasa a fallar si falta
