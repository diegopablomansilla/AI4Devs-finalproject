# Tasks

## 1. Tests (red)

- [x] 1.1 Confirmar que `scripts/check-ci.sh` y `.github/workflows/ci.yml` no existen — escenario «Script presente» / «Falta el workflow o un job» en rojo (`test ! -e`)
- [x] 1.2 Escribir `scripts/check-ci.sh` POSIX (`CHECK_ROOT`, `fail`) que exige archivo, jobs `fmt`/`clippy`/`test`, `macos-14`, `pull_request`, comandos § 5 sobre `vortex-core`, caché de Cargo, y ausencia de Meilisearch/Ollama/Whisper/Redocly/`docs/openapi.yaml` — `test -x scripts/check-ci.sh` (escenario «Script presente»)
- [x] 1.3 `./scripts/check-ci.sh` sobre el repo actual sale ≠ 0 y nombra el workflow o un job (escenario «Falta el workflow o un job»)
- [x] 1.4 Fixture `CHECK_ROOT` sin `.github/workflows/ci.yml` — el script sale ≠ 0 y nombra el archivo
- [x] 1.5 Fixture con `ci.yml` sin job `fmt` (o `clippy`/`test`) — el script sale ≠ 0 y nombra el job (escenario «Falta el workflow o un job»)
- [x] 1.6 Fixture con `runs-on: ubuntu-latest` en un job — el script sale ≠ 0 y nombra el runner
- [x] 1.7 Fixture sin trigger `pull_request` — el script sale ≠ 0 (escenario «El workflow no dispara en pull_request»)
- [x] 1.8 Fixture sin `--check`, o sin `-D warnings`, o sin `--workspace`, o sin path `vortex-core` — el script sale ≠ 0 y nombra el job (escenario «Falta un flag o el manifiesto»)
- [x] 1.9 Fixture con un step Redocly o Meilisearch — el script sale ≠ 0 y nombra el extra (escenario «Extra fuera de alcance»)
- [x] 1.10 `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` del repo salen 0 (escenario «Workspace actual en verde»; ya cumple T-00-01)
- [x] 1.11 Copia temporal del workspace con un `.rs` desformateado — `cargo fmt --all -- --check` sale ≠ 0 (escenario «Error de formato»)
- [x] 1.12 Copia temporal con un warning Clippy (p. ej. binding sin usar) — `cargo clippy --workspace -- -D warnings` sale ≠ 0 (escenario «Warning de Clippy»)
- [x] 1.13 Copia temporal con `#[test] fn boom() { panic!() }` — `cargo test --workspace` sale ≠ 0 (escenario «Test que falla»)

## 2. Workflow (green)

- [x] 2.1 Crear `.github/workflows/ci.yml`: `on.pull_request` y `on.push` a `develop`/`main`; jobs `fmt`, `clippy`, `test` en `macos-14`; `actions/checkout@v4`; `rustup show`; `actions/cache@v4` (registry/git/`vortex-core/target`, key por job + hash de `vortex-core/Cargo.lock`); `defaults.run.working-directory: vortex-core`; comandos § 5 — escenarios «Workflow presente con tres jobs en macos-14», «Comandos de verificación en el workflow», «Workflow mínimo»
- [x] 2.2 `./scripts/check-ci.sh` sobre el repo sale 0 (escenarios «Contrato válido», «Workflow mínimo»)
- [x] 2.3 Repetir 1.4–1.9 y comprobar que las fixtures unhappy siguen saliendo ≠ 0
- [x] 2.4 Repetir 1.10–1.13: árbol limpio 0; copias sucias ≠ 0

## 3. Cierre (non-goals y harnesses previos)

- [x] 3.1 `./scripts/check-toolchain.sh`, `./scripts/check-workspace.sh` y `./scripts/check-domain-deps.sh` sobre el repo salen 0 (`git diff` vacío en esos scripts y en `vortex-core/crates/`)
- [x] 3.2 El YAML no declara job de `check-domain-deps.sh` (T-00-02), ni Redocly (T-09-03), ni Meilisearch/Ollama/Whisper — escenario «Workflow mínimo»
- [x] 3.3 No hay cuarto job que ejecute `check-ci.sh` en GitHub (non-goal de diseño)
- [x] 3.4 `openspec validate T-09-01` sale 0
