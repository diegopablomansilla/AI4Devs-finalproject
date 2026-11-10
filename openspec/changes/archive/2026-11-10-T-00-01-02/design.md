# Design

## Context

T-00-01-01 ya está en `develop`: canal `1.99.0`, `rustfmt` edición 2024 y `.gitignore` con `target/`. No existe `vortex-core/`. La estructura objetivo está en [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio) y el mapa de crates en [3.2](../../../docs/3-componentes.md#32-mapa-de-crates-y-regla-de-dependencias). Este ZZ crea el workspace de libs; el binario es T-00-01-03. Ver proposal.md para el recorte.

## Goals / Non-Goals

**Goals:**

- Un solo manifiesto de workspace que T-00-01-03 pueda extender con un member más
- Libs que compilan con `cargo clippy --workspace -- -D warnings` sin código de producto
- Tabla `[workspace.dependencies]` que los tickets siguientes referencien con `workspace = true`, sin elegir versión otra vez

**Non-Goals (diseño):**

- Features de crates (`rustls`, `kv-rocksdb`, `macros` de tokio). La tabla solo fija versión; el crate que las use elige features
- `cargo tree` / CI de la regla de `domain` (T-00-02)
- Tests de comportamiento dentro de los libs

## Decisions

### 1. Workspace en `vortex-core/`, no en la raíz del git

**Elección:** `vortex-core/Cargo.toml` y `cargo … --manifest-path vortex-core/Cargo.toml`.

**Por qué:** [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio) separa `vortex-core/` de `vectra-web/`, `docs/` y `scripts/`. Un workspace en la raíz mezclaría el binario con la documentación.

**Alternativa:** workspace virtual en la raíz. Se descartó: contradice 2.8.

### 2. Once libs, paquetes con el nombre del directorio, sin `bin/vortex`

**Elección:** members relativos al manifiesto: `crates/api` … `crates/supervisor`. `package.name` igual al directorio (`domain`, `api`, …). `publish = false`. `src/lib.rs` con un comentario de módulo y nada más.

**Por qué:** el ticket y 3.2 nombran esos crates. Un lib vacío hace pasar `cargo test` (0 tests) y clippy. Meter `bin/vortex` cerraría el Listo cuando del ticket en este ZZ.

**Alternativa:** un solo crate `vortex` con módulos. Se descartó: rompe la regla de dependencias de `domain`.

### 3. Lints heredados, Clippy en `warn`

**Elección:**

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
```

Cada lib: `[lints] workspace = true`. El criterio de salida es `cargo clippy --workspace -- -D warnings`, que promueve esos `warn` a error.

**Por qué:** el ticket pide lints en `[workspace.lints]` y el Listo cuando usa `-D warnings`. `forbid` en `clippy::all` haría fallar el comando por estilo antes de haber código. `priority = -1` deja que un lint puntual futuro lo pise.

**Alternativa:** `pedantic` / `nursery`. Se descartó: ruido y fuera de las ~1 h.

### 4. Pins `=` al estable consultado el 2026-11-09, con dos excepciones de los docs

**Elección:** la tabla de la spec. Todas las entradas son `crate = "=x.y.z"` sin features.

Excepciones deliberadas:

- **surrealdb `=2.7.0`**, no `4.0.1`. [7.2](../../../docs/7-tickets.md) pide SurrealDB 2.x con versión exacta. 2.7.0 es el último 2.x estable en crates.io. 3.x y 4.0.1 existen y no se adoptan en este ZZ.
- **ort `=2.0.0-rc.15`**. La línea 1.x está yanked; no hay 2.0 estable. Ningún lib depende de `ort` todavía, así que el rc no se compila aquí.

**Por qué:** “versiones fijadas” y el `=` que el doc exige para surrealdb se aplican a toda la tabla, para que un `cargo update` no mueva el stack.

**Alternativa:** dejar `^` y fijar solo surrealdb. Se descartó: el ticket pide versiones fijadas en `[workspace.dependencies]`, no solo en el motor.

### 5. Libs sin `[dependencies]`

**Elección:** el script rechaza `[dependencies]` y `[dev-dependencies]` en los once libs. `domain` queda sin SDKs por construcción.

**Por qué:** un lib vacío que ya depende de `surrealdb` compilaría motores nativos y adelantaría T-00-02. La comprobación de este ZZ es estática (el manifiesto); `cargo tree` en CI sigue siendo T-00-02.

### 6. `scripts/check-workspace.sh` más Cargo

**Elección:** el script (raíz opcional `CHECK_ROOT`) valida miembros, lints, pins y ausencia de deps. No llama a `cargo`. Los tres comandos Cargo son el happy path de compilación y se corren aparte, sobre el repo real. Fixtures temporales cubren el unhappy path sin un workspace roto en el árbol de git.

**Por qué:** el mismo patrón que `scripts/check-toolchain.sh`. Un test Rust no puede existir antes del workspace, y un workspace inválido no sirve para demostrar el fallo de forma repetible.

## Risks / Trade-offs

- **[surrealdb 2.7.0 vs 4.0.1]** → se respeta 7.2. Si el equipo quiere 4.x, se cambia la tabla de la spec antes de apply; ningún crate lo compila todavía.
- **[ort rc]** → no entra al grafo de compilación hasta que `vision` lo dependa. El pin documenta la única línea publicable.
- **[`clippy::all` + `-D warnings`]** → un lib vacío no debería emitir warnings. Si rustfmt/clippy 1.99 marca el `lib.rs` mínimo, se ajusta el comentario en el refactor, sin bajar el `-D warnings`.
- **[Cargo.lock casi vacío]** → se versiona igual, para que el alta de la primera dependencia tenga un diff claro.

## Migration Plan

1. Script en rojo contra el repo sin `vortex-core/`.
2. Manifiesto, once libs, lockfile.
3. `cargo build`, `cargo test`, `cargo clippy -- -D warnings`.
4. Repetir fixtures unhappy.
5. Rollback: borrar `vortex-core/` y el script. No hay datos.

## Open Questions

Ninguna que cambie el desglose de tareas. El pin de surrealdb en 2.7.0 (y no en 4.0.1) y el rc de ort quedan como supuestos visibles en la spec; si no cierran, se edita la tabla antes de implementar.
