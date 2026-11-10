# Proposal

## Ticket padre

- **Ticket:** [T-09-01](../../../docs/7-tickets.md) — CI base en GitHub Actions
- **Change:** `T-09-01`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias), [HU-09-01](../../../docs/6-historias.md#hu-09-01--tests-de-integración-contra-motores-reales)
- **Hace:** `.github/workflows/ci.yml` en *runner* `macos-14` con *jobs* `fmt --check`, `clippy -D warnings` y `test --workspace`, con caché de Cargo
- **Listo cuando:** un PR con un error de formato falla la CI y uno limpio pasa

## Why

T-00-01 dejó el workspace `vortex-core/` compilando en local, pero un PR todavía no ejecuta `fmt`, Clippy ni tests. HU-00-01 pide que la integración continua falle si el árbol no está limpio; HU-09-01 y los tickets T-00-02 / T-09-03 / T-09-05 necesitan ese pipeline para colgar jobs. T-09-01 es el paso 02 de la ruta MVP.

## What Changes

- Agregar `.github/workflows/ci.yml` disparado en `pull_request` y en `push` a `develop` y `main`
- Tres jobs en paralelo (`fmt`, `clippy`, `test`) en `macos-14`, cada uno con caché de Cargo
- Los jobs corren los comandos de [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`)
- Harness POSIX `scripts/check-ci.sh` (`CHECK_ROOT` opcional) que verifica el contrato del YAML; TDD con fixtures happy (repo) y unhappy (YAML incompleto, runner incorrecto, extras de motores/OpenAPI, y los mismos comandos cargo sobre copias temporales sucias)

## Non-goals

Este change **no** cubre:

- Job de `scripts/check-domain-deps.sh` ni el Listo cuando «agregar `surrealdb` a `domain` hace fallar la CI» → **T-00-02**
- *Lint* Redocly de `docs/openapi.yaml` → **T-09-03**
- Meilisearch real en CI → **T-09-05**
- Ollama / embeddings en CI → **T-09-13**
- Whisper en CI → **T-09-18**
- Playwright, runners Linux/Windows, caché de `models/`
- Un cuarto job que ejecute `check-ci.sh` en GitHub (el script es harness local, como `check-toolchain.sh`)

## Capabilities

### New Capabilities

- `ci-base`: workflow de GitHub Actions en `macos-14` que corre formato, Clippy y tests del workspace `vortex-core/` con caché de Cargo, y un script que verifica ese contrato

### Modified Capabilities

- Ninguna. `cargo-workspace`, `rust-toolchain` y `domain-deps` no cambian requisitos; CI consume el workspace y la toolchain ya fijados.

## Impact

- **Repo:** `.github/workflows/ci.yml` (nuevo), `scripts/check-ci.sh` (nuevo). No toca crates de `vortex-core/`
- **APIs / OpenAPI:** ninguna
- **CI:** primer workflow del repositorio; desbloquea T-00-02, T-09-03 y T-09-05
- **Ticket:** cierra T-09-01; el Listo cuando en GitHub se confirma en el PR de esta rama
- **BREAKING:** no
