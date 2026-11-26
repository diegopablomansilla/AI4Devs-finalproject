# Design

## Context

T-00-01 está en `develop`: `vortex-core/` con once libs y `bin/vortex` vacíos, `rust-toolchain.toml` (`1.99.0`, rustfmt, clippy) y `rustfmt.toml` en la raíz, `vortex-core/Cargo.lock` presente. No existe `.github/workflows/`. `scripts/check-domain-deps.sh` ya está en el árbol (T-00-02-01) pero **no** se engancha a CI en este change (T-00-02). Ver proposal.md para motivación y `specs/ci-base/spec.md` para el contrato.

Constraints: runner macOS Apple Silicon ([7.2](../../../docs/7-tickets.md)); comandos de verificación evolutivos en [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; workspace Cargo **no** está en la raíz.

## Goals / Non-Goals

**Goals:**

- Un YAML que T-00-02, T-09-03 y T-09-05 puedan extender con jobs nuevos sin reescribir fmt/clippy/test
- Misma toolchain que el portátil (`rust-toolchain.toml`), sin un segundo pin en el workflow
- TDD del contrato YAML y de los comandos cargo **sin** `act` (runners macos de `act` son frágiles y no son el DoD)

**Non-Goals (diseño):**

- Cuarto job que corra `check-ci.sh` en GitHub (el script es harness local, como `check-toolchain.sh`)
- `dtolnay/rust-toolchain` ni `Swatinem/rust-cache` (terceros); se elige la acción oficial de caché
- Duplicar `rustfmt.toml` / `rust-toolchain.toml` dentro de `vortex-core/`

## Decisions

### 1. Tres jobs en paralelo, no un job con tres steps

**Elección:** `jobs.fmt`, `jobs.clippy`, `jobs.test`, cada uno `runs-on: macos-14`.

**Por qué:** el ticket nombra *jobs*. El Listo cuando («un PR con error de formato falla la CI») queda como check independiente. T-00-02 / T-09-03 / T-09-05 agregan jobs, no steps mezclados con fmt.

**Alternativa:** un job `ci` secuencial (un rustup, una caché, menos minutos de runner). Se descartó: mezcla señales y contradice el wording del ticket. Con crates vacíos el costo extra de tres runners es bajo.

### 2. `working-directory: vortex-core`

**Elección:** `defaults.run.working-directory: vortex-core` a nivel de cada job (o en cada `run` de cargo).

**Por qué:** el manifiesto es `vortex-core/Cargo.toml`. rustup y rustfmt caminan hacia la raíz y encuentran `rust-toolchain.toml` / `rustfmt.toml` (decisión T-00-01-01).

**Alternativa:** `--manifest-path vortex-core/Cargo.toml` desde la raíz. Equivalente; se prefiere cwd para no repetir el path en cada comando.

### 3. Toolchain vía rustup + `rust-toolchain.toml`

**Elección:** tras `actions/checkout@v4`, un step `rustup show` (o `rustup component add rustfmt clippy` si hiciera falta) deja que rustup instale `1.99.0` y los componentes del archivo de raíz. El YAML **no** pinea otra versión.

**Por qué:** una sola fuente de verdad. Si el runner no puede instalar 1.99.0, el job falla de forma visible (riesgo ya anotado en T-00-01-01).

**Alternativa:** `dtolnay/rust-toolchain@1.99.0`. Se descartó: duplica el pin y suma una acción de terceros.

### 4. Caché con `actions/cache@v4`

**Elección:** acción oficial. Paths: `~/.cargo/registry`, `~/.cargo/git`, `vortex-core/target`. Key: `macos-14` + nombre del job + hash de `vortex-core/Cargo.lock`. Restore-keys parciales por OS + job.

**Por qué:** el ticket pide caché de Cargo sin nombrar marketplace. Separar la key por job evita mezclar artefactos de clippy y test.

**Alternativa:** `Swatinem/rust-cache@v2` (`workspaces: vortex-core`). Plan B si el miss duele; no es el default.

### 5. Triggers `pull_request` + `push` a `develop` y `main`

**Elección:** ambos. El spec exige `pull_request` (Listo cuando). `push` a las ramas de integración corre la misma suite post-merge.

**Alternativa:** solo `pull_request`. Se descartó: un push directo a `develop` no se vería (el flujo del repo no lo usa, pero el default es barato).

### 6. Harness POSIX `scripts/check-ci.sh`, no `act`

**Elección:** mismo patrón que `check-toolchain.sh` / `check-workspace.sh`: `CHECK_ROOT`, grep del YAML, exit 0/≠0. Fixtures temporales para YAML inválido. Para el Listo cuando de formato: copia del workspace a un tmp, ensuciar un `.rs`, correr `cargo fmt --all -- --check` (expect ≠ 0). Igual para clippy (binding sin usar) y test (`panic!`).

**Por qué:** TDD local alineado al resto de Fase 0. `act` no reproduce `macos-14` de forma fiable.

**Alternativa:** crate de tests que parsee YAML. Se descartó: no hay crate de infra, y un parser YAML sería alcance extra.

El script no es producto de runtime (no lo invoca `vortex`).

## Risks / Trade-offs

- **[Tres runners macos vs minutos]** → crates vacíos; si más adelante duele, T-09-xx puede serializar sin cambiar el contrato de tres jobs.
- **[Caché oficial vs Swatinem]** → keys más toscas; mitigación: documentar Swatinem como plan B, no cambiarlo en este ticket.
- **[Chequeo YAML por grep]** → un comentario puede falsear positivos. Mitigación: el archivo real es corto y las fixtures son mínimas.
- **[Listo cuando real es GitHub]** → el harness cubre contrato + `cargo fmt --check` local; el PR de `T-09-01` es la prueba operativa.
- **[`openspec new change` rechaza `T-09-01`]** → scaffold en `t-09-01` y rename a `T-09-01` (igual que T-00-01-01). Nota operativa, no bloquea.

## Migration Plan

1. Red: `check-ci.sh` + fixtures; el repo actual falla (no hay workflow).
2. Green: escribir `ci.yml`; script y comandos § 5 en verde.
3. Rollback: borrar `.github/workflows/ci.yml` y `scripts/check-ci.sh`. No hay estado persistido en GitHub más que runs históricos.

## Open Questions

Ninguna que cambie specs, enfoque o tasks. Defaults cerrados en la exploración: jobs paralelos, `actions/cache@v4`, rustup, trigger PR + push.
