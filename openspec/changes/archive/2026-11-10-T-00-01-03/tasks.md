# Tasks

## 1. Harness de verificación (red)

- [x] 1.1 Actualizar `scripts/check-workspace.sh` para exigir member `bin/vortex`, `src/main.rs`, `name = "vortex"`, lints heredados y cero `[dependencies]` / `[dev-dependencies]` en el binario, y quitar la regla que lo prohíbe — verificar que el script nombra T-00-01-03 en su comentario
- [x] 1.2 Fixture sin `bin/vortex` en `members` — `CHECK_ROOT=<tmp> ./scripts/check-workspace.sh` sale ≠ 0 y nombra `bin/vortex` (escenario «Falta el binario»)
- [x] 1.3 Fixture con member y `Cargo.toml` pero sin `src/main.rs` — el script sale ≠ 0 y nombra el binario o `main.rs` (escenario «Falta main.rs»)
- [x] 1.4 Fixture con `[dependencies]` en `bin/vortex/Cargo.toml` — el script sale ≠ 0 y nombra `vortex` (escenario «El binario declara dependencias»)
- [x] 1.5 Fixture sin `[lints] workspace = true` en `bin/vortex/Cargo.toml` — el script sale ≠ 0 y nombra `vortex` (escenario «El binario no hereda lints»)
- [x] 1.6 Correr `./scripts/check-workspace.sh` sobre el repo actual y verificar que falla (red: no existe `vortex-core/bin/vortex`; escenario «Árbol de workspace inválido»)
- [x] 1.7 Repetir fixtures unhappy de T-00-01-02 (falta `crates/domain`, pin `axum` sin `=`, `[dependencies]` en `domain`) y verificar que siguen saliendo ≠ 0

## 2. Crate binario (green)

- [x] 2.1 Añadir `bin/vortex` a `members` en `vortex-core/Cargo.toml` — verificar escenario «El binario es miembro»
- [x] 2.2 Crear `vortex-core/bin/vortex/Cargo.toml` (`name = "vortex"`, `publish = false`, edition heredada, `[lints] workspace = true`, sin dependencias) y `src/main.rs` con `fn main() {}` — verificar escenario «Binario vacío presente»
- [x] 2.3 Repetir 1.2–1.5 y 1.7 y comprobar que las fixtures unhappy siguen fallando

## 3. Compilación y cierre (green)

- [x] 3.1 `cargo metadata --manifest-path vortex-core/Cargo.toml --format-version 1` lista un package `vortex` (escenario «El package vortex forma parte del workspace»)
- [x] 3.2 `cargo build --manifest-path vortex-core/Cargo.toml --workspace` sale 0 (escenario «Build, tests y clippy en verde», parte build)
- [x] 3.3 `cargo test --manifest-path vortex-core/Cargo.toml --workspace` sale 0
- [x] 3.4 `cargo clippy --manifest-path vortex-core/Cargo.toml --workspace -- -D warnings` sale 0
- [x] 3.5 `./scripts/check-workspace.sh` sobre el repo sale 0 (escenarios «El binario es miembro», «Binario vacío presente», «Los once libs son miembros»)
- [x] 3.6 Confirmar non-goals: `bin/vortex/Cargo.toml` no declara `clap` ni crates del workspace; `src/main.rs` no implementa subcomandos
