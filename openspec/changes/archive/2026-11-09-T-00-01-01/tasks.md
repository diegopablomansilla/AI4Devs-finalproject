# Tasks

## 1. Harness de verificación (red)

- [x] 1.1 Crear `scripts/check-toolchain.sh` (POSIX, `CHECK_ROOT` opcional, default raíz del repo) que exige canal `1.99.0`, componentes `rustfmt` y `clippy`, `edition = "2024"` y patrones `models/`, `target/` (no solo `/target/`) y `data/` — verificar que el archivo existe y es ejecutable (`test -x scripts/check-toolchain.sh`)
- [x] 1.2 Fixture sin canal: árbol temporal con `rustfmt.toml` válido y `.gitignore` completo pero sin `channel = "1.99.0"` — `CHECK_ROOT=<tmp> ./scripts/check-toolchain.sh` sale ≠ 0 y el stderr nombra el canal (escenario «Falta el canal fijado»)
- [x] 1.3 Fixture sin edición: árbol temporal con toolchain válida y `.gitignore` completo pero sin `edition = "2024"` — el script sale ≠ 0 y nombra la edición rustfmt (escenario «Falta la edición de rustfmt»)
- [x] 1.4 Fixture de ignore incompleto: tres árboles (falta `models/`, queda `/target/` de raíz, falta `data/`) — el script sale ≠ 0 y nombra el patrón que falta (escenario «Ignore incompleto»)
- [x] 1.5 Correr `./scripts/check-toolchain.sh` sobre el repo actual y verificar que falla (red: faltan TOML y/o ignore incompleto; escenario «Árbol inválido»)

## 2. Toolchain y rustfmt (green)

- [x] 2.1 Añadir `rust-toolchain.toml` en la raíz con `channel = "1.99.0"` y `components = ["rustfmt", "clippy"]` — verificar escenario «Canal fijado en la raíz» (`grep` de canal y componentes)
- [x] 2.2 Añadir `rustfmt.toml` en la raíz con `edition = "2024"` y `newline_style = "Unix"` — verificar escenario «rustfmt con edición 2024»
- [x] 2.3 Repetir 1.2–1.4 y comprobar que las fixtures unhappy siguen fallando (no se rompió el camino no feliz)

## 3. Gitignore y cierre happy (green)

- [x] 3.1 Ampliar `.gitignore`: agregar `models/` y `data/`; reemplazar `/target/` por `target/` — no borrar las entradas de curso (macOS, `.env`, `.cursor/`)
- [x] 3.2 Escenario «Artefactos locales no se versionan»: `git check-ignore -q models/ejemplo.bin vortex-core/target/debug/libdomain.rlib data/db/LOCK` sale 0; `git check-ignore docs/7-tickets.md openspec/config.yaml` sale ≠ 0
- [x] 3.3 `./scripts/check-toolchain.sh` sobre el repo sale 0 (escenarios «Árbol válido», «Canal fijado», «rustfmt con edición 2024»)
- [x] 3.4 Confirmar non-goals: no existen `vortex-core/Cargo.toml` ni `bin/vortex`; `find vortex-core -name Cargo.toml` no lista nada (o no hay `vortex-core/`)
