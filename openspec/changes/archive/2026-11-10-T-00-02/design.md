# Design

## Context

En `develop`: `scripts/check-domain-deps.sh` y sus fixtures (T-00-02-01) están verdes; `.github/workflows/ci.yml` tiene tres jobs (`fmt`, `clippy`, `test`) en `macos-14` (T-09-01). `scripts/check-ci.sh` **prohíbe** la cadena `check-domain-deps` en el YAML a propósito, para que este ticket la enganche. El crate `domain` sigue vacío. Ver proposal.md para motivación y los deltas en `specs/` para el contrato.

Constraints: runner `macos-14` ([7.2](../../../docs/7-tickets.md)); el script vive en la raíz y usa `vortex-core/Cargo.toml`; no ensuciar `vortex-core/crates/domain` para el unhappy path.

## Goals / Non-Goals

**Goals:**

- Un job de CI independiente que corre exactamente `scripts/check-domain-deps.sh`
- Un solo harness local (`check-ci.sh`) que falle si falta ese job
- TDD sin `act` y sin agregar `surrealdb` al crate real

**Non-Goals (diseño):**

- Reescribir el oráculo `cargo tree` ni la denylist
- Parser YAML; se sigue usando grep POSIX como T-09-01
- Cuarto *step* mezclado dentro de `test` (el ticket pide un *job*)
- Job que ejecute `check-ci.sh` en GitHub

## Decisions

### 1. Job `domain-deps` en paralelo, no un step de `test`

**Elección:** `jobs.domain-deps` con `runs-on: macos-14`, en el mismo workflow (`pull_request` + `push` a `develop`/`main`).

**Por qué:** T-09-01 diseñó el YAML para que T-00-02 agregue **jobs**, no steps. El Listo cuando («un PR con `surrealdb` en `domain` falla la CI») queda como check independiente en GitHub. El nombre coincide con la capability.

**Alternativa:** un step `run: ../scripts/check-domain-deps.sh` al final de `test`. Se descartó: mezcla señales (un fallo de tests vs. un acoplamiento hexagonal) y contradice el wording del ticket.

### 2. El job corre desde la raíz del repo

**Elección:** **no** usar `defaults.run.working-directory: vortex-core` en este job. Steps: `actions/checkout@v4`, `rustup show` (misma toolchain que T-09-01), `actions/cache@v4` (registry, git, `vortex-core/target`, key `macos-14-domain-deps-` + hash de `vortex-core/Cargo.lock`), `run: ./scripts/check-domain-deps.sh`.

**Por qué:** el script resuelve `CHECK_ROOT` / raíz y apunta a `vortex-core/Cargo.toml`. `cargo tree` necesita `cargo` (rustup) y el índice de crates. Caché alineada a los otros jobs.

**Alternativa:** `working-directory: vortex-core` y `../scripts/check-domain-deps.sh`. Equivalente; se prefiere la raíz para no invertir el path del **Hace**.

### 3. Extender `check-ci.sh`, no un script nuevo

**Elección:** quitar `check-domain-deps` de la denylist de extras; exigir job `domain-deps` (`^[[:space:]]{2}domain-deps:`) y la cadena `scripts/check-domain-deps.sh`. Mantener `>= 3` caches y `macos-14` para no romper el orden de error de las fixtures T-09-01; el job nuevo se comprueba **después** de fmt/clippy/test y de los extras prohibidos (Redocly, etc.).

**Por qué:** un solo comando local cubre el contrato CI. Si se exigiera `runs-on`/`cache` `>= 4` *antes* del nombre del job, la fixture de tres jobs fallaría con «macos-14» en vez de nombrar `domain-deps`.

**Alternativa:** `scripts/check-domain-deps-ci.sh`. Se descartó: duplica grep de workflow y alarga el apply.

Orden de chequeos (relevante para TDD):

1. Archivo, `pull_request`, jobs `fmt`/`clippy`/`test`, no ubuntu, `macos-14` `>= 3`, cache `>= 3`, `vortex-core`, comandos § 5
2. Extras prohibidos **sin** `check-domain-deps` (sigue `check-ci.sh`, motores, OpenAPI)
3. Job `domain-deps` + invocación de `scripts/check-domain-deps.sh`

Así `redocly-extra` sigue fallando por Redocly, no por el job nuevo.

### 4. Unhappy de `surrealdb` = fixture existente, no el crate git

**Elección:** no modificar `vortex-core/crates/domain`. El escenario «agregar surrealdb hace fallar el chequeo de CI» se prueba con `scripts/fixtures/domain-deps/scenarios/surrealdb-direct` (ya sale ≠ 0 y nombra `surrealdb`). El YAML se prueba con una fixture nueva de tres jobs válidos sin `domain-deps`.

**Por qué:** el Listo cuando en GitHub es «el job corre el mismo comando»; ensuciar el crate real rompería `check-workspace.sh` y el PR de esta rama. T-00-02-01 ya cubrió el oráculo.

**Alternativa:** un commit temporal con `surrealdb` en `domain` y revert. Se descartó: no entra en el árbol de `T-00-02` y pelea con T-00-01.

### 5. No tocar la lógica de `check-domain-deps.sh`

**Elección:** a lo sumo actualizar el comentario de cabecera (`T-00-02` en vez de `T-00-02-01`). Denylist, aristas `normal,build` y `CHECK_ROOT` quedan.

**Por qué:** el **Hace** del script ya está entregado. Este change cierra el job y el Listo cuando.

## Risks / Trade-offs

- **[Grep del YAML]** → un comentario `domain-deps:` podría falsear. Mitigación: el archivo real es corto; el patrón ancla indentación de job.
- **[Listo cuando real es GitHub]** → el harness cubre contrato + el mismo comando sobre fixture `surrealdb`; el PR de `T-00-02` es la prueba operativa de que el job corre.
- **[`cargo tree` en CI sin lock de domain]** → `domain` vacío; el job igual necesita rustup. Mitigación: misma instalación que T-09-01.
- **[Cuatro runners macos]** → costo extra bajo con crates vacíos; misma decisión que T-09-01 (jobs en paralelo).

## Migration Plan

1. Red: `check-ci.sh` exige el job `domain-deps` → el repo actual (tres jobs) sale ≠ 0.
2. Fixture YAML sin el job: sale ≠ 0 y nombra `domain-deps`.
3. Green: agregar el job al workflow; `check-ci.sh` sobre el repo sale 0.
4. Confirmar `./scripts/check-domain-deps.sh`, `./scripts/fixtures/domain-deps/run-script-tests.sh` y `./scripts/fixtures/ci/run-script-tests.sh` en verde.
5. Rollback: revertir el job y la denylist de `check-ci.sh`. No hay datos.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre del job (`domain-deps`) queda fijado aquí.
