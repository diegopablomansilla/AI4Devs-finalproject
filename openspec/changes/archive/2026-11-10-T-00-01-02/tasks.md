# Tasks

## 1. Harness de verificación (red)

- [x] 1.1 Crear `scripts/check-workspace.sh` (POSIX, `CHECK_ROOT` opcional, default raíz del repo) que exige los once members, ausencia de `bin/vortex`, lints de workspace heredados, pins `=` de la tabla de la spec y cero `[dependencies]` / `[dev-dependencies]` en los libs — verificar `test -x scripts/check-workspace.sh`
- [x] 1.2 Fixture sin `crates/domain` en `members` — `CHECK_ROOT=<tmp> ./scripts/check-workspace.sh` sale ≠ 0 y nombra `domain` (escenario «Falta un crate lib»)
- [x] 1.3 Fixture con member `bin/vortex` y su `Cargo.toml` — el script sale ≠ 0 y nombra `bin/vortex` (escenario «El binario todavía no es miembro»)
- [x] 1.4 Fixture con `axum = "0.8.9"` sin `=` — el script sale ≠ 0 y nombra `axum` (escenario «Versión no fijada»)
- [x] 1.5 Fixture con `[dependencies]` y `surrealdb` en `crates/domain/Cargo.toml` — el script sale ≠ 0 y nombra `domain` (escenario «Un lib declara dependencias»)
- [x] 1.6 Correr `./scripts/check-workspace.sh` sobre el repo actual y verificar que falla (red: no existe `vortex-core/`; escenario «Árbol de workspace inválido»)

## 2. Workspace y crates vacíos (green)

- [x] 2.1 Añadir `vortex-core/Cargo.toml` con `resolver = "3"`, los once members, `[workspace.package] edition = "2024"`, los lints de la spec y `[workspace.dependencies]` con los pins `=` — verificar escenario «Pins exactos» y «Lints presentes y heredados» sobre el manifiesto
- [x] 2.2 Crear los once `crates/<nombre>/Cargo.toml` (`publish = false`, edition heredada, `[lints] workspace = true`, sin dependencias) y `src/lib.rs` vacío de lógica — verificar escenario «Los once libs son miembros»
- [x] 2.3 Generar `vortex-core/Cargo.lock` con `cargo generate-lockfile --manifest-path vortex-core/Cargo.toml` y verificar que el archivo existe
- [x] 2.4 Repetir 1.2–1.5 y comprobar que las fixtures unhappy siguen fallando

## 3. Compilación y cierre (green)

- [x] 3.1 `cargo build --manifest-path vortex-core/Cargo.toml --workspace` sale 0 (escenario «Build, tests y clippy en verde», parte build)
- [x] 3.2 `cargo test --manifest-path vortex-core/Cargo.toml --workspace` sale 0
- [x] 3.3 `cargo clippy --manifest-path vortex-core/Cargo.toml --workspace -- -D warnings` sale 0
- [x] 3.4 `./scripts/check-workspace.sh` sobre el repo sale 0 (escenarios «Once libs», «Pins exactos», «Lints presentes»)
- [x] 3.5 Confirmar non-goals: no existe `vortex-core/bin/vortex` y `cargo metadata` no lista un package `vortex`
