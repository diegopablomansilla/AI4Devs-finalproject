# Design

## Context

T-00-01-01 y T-00-01-02 están en `develop`: toolchain `1.99.0`, workspace `vortex-core/` con once libs vacíos, lints heredados y pins `=`. `scripts/check-workspace.sh` **falla** si aparece `bin/vortex`. No existe `vortex-core/bin/`. Ver proposal.md para el recorte; la estructura objetivo está en [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio) y [3.4.13](../../../docs/3-componentes.md#3413-plataforma-binvortex).

## Goals / Non-Goals

**Goals:**

- Un member binario que T-00-07 pueda llenar con `clap` sin mover de sitio el crate
- El mismo harness de T-00-01-02 (`check-workspace.sh`) como fuente de verdad del árbol, ahora con el binario obligatorio
- `cargo clippy --workspace -- -D warnings` en verde con `fn main` vacío

**Non-Goals (diseño):**

- Features de `clap`, Tokio o Axum en el binario (T-00-07, T-00-10)
- Dependencias a crates del workspace (el composition root se cablea cuando existan implementaciones)
- Nuevo script de verificación paralelo al de workspace

## Decisions

### 1. Paquete `vortex` en `vortex-core/bin/vortex/`

**Elección:** member `"bin/vortex"`, `package.name = "vortex"`, `src/main.rs`. Cargo infiere el binario `vortex`. `publish = false`, `edition.workspace = true`, `[lints] workspace = true`.

**Por qué:** [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio) coloca el binario ahí. T-00-01-02 ya reservó el nombre de package `vortex` (task 3.5: metadata no lo listaba aún). La CLI de T-00-07 es `vortex serve`, no `vortex-core serve`.

**Alternativa:** `crates/vortex` o `name = "vortex-core"`. Se descartó: contradice 2.8 y el nombre del proceso.

### 2. Invertir `scripts/check-workspace.sh`, no crear otro script

**Elección:** el mismo POSIX script (`CHECK_ROOT` opcional) pasa a **exigir** `bin/vortex`, `src/main.rs`, `name = "vortex"`, lints heredados y cero dependencias en el binario. Sigue validando los once libs y los pins.

**Por qué:** dejar la regla de T-00-01-02 haría fallar el script en verde. Dos scripts (uno que prohíbe el binario y otro que lo exige) se contradicen. El patrón de fixtures temporales se reutiliza.

**Alternativa:** `scripts/check-vortex-bin.sh`. Se descartó: duplica el árbol de members y deja un chequeo obsoleto.

### 3. `fn main() {}` sin dependencias

**Elección:** `src/main.rs` con `fn main() {}` y un comentario de módulo, igual que los `lib.rs` placeholder. Sin `[dependencies]`.

**Por qué:** el recorte de HU-00-01 es exactamente eso. `clap` y subcomandos son T-00-07; cablear `api`/`storage` ahora no aporta comportamiento y podría arrastrar el grafo antes de tiempo.

**Alternativa:** stub `clap` con "no implementado". Se descartó: es el **Hace** de T-00-07.

### 4. Clippy del workspace incluye el binario

**Elección:** no hay `default-members` especial ni se excluye el binario de `--workspace`. El Listo cuando del ticket es sobre todo el workspace.

**Por qué:** un `fn main` vacío no dispara `clippy::all` en 1.99. Si el comentario del `main.rs` generara warning, se ajusta el comentario, no se baja `-D warnings`.

**Alternativa:** `cargo clippy -p vortex` aparte. Se descartó: el ticket pide `--workspace`.

## Risks / Trade-offs

- **[Script T-00-01-02 vs este ZZ]** → es el BREAKING interno del proposal: el chequeo cambia de “prohibido” a “obligatorio”. No hay consumidores fuera de este repo.
- **[`clippy` sobre `fn main() {}`]** → riesgo bajo; mitigación: comentario de módulo al estilo de los libs.
- **[T-00-07 agrega `clap`]** → este ZZ deja el `[dependencies]` vacío a propósito; T-00-07 relajará esa regla solo para el binario.

## Migration Plan

1. Actualizar `check-workspace.sh` para exigir el binario; el repo actual falla (red).
2. Fixtures unhappy (sin member, sin `main.rs`, con deps, sin lints).
3. Crear `vortex-core/bin/vortex/` y sumar el member.
4. `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo metadata` lista `vortex`.
5. Rollback: borrar `bin/vortex/`, quitar el member y revertir el script. No hay datos.

## Open Questions

Ninguna que cambie el desglose. El nombre de package `vortex` queda como supuesto alineado a T-00-01-02 y a la CLI.
