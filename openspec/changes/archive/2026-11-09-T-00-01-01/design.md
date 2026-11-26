# Design

## Context

El repo es documentación + OpenSpec; no hay `vortex-core/` ni `Cargo.toml`. Ya existe un `.gitignore` de curso (macOS, editor, `.env`, `/target/` solo en raíz, `.cursor/` con excepciones de rules). Ver proposal.md para motivación. Recorte de T-00-01 en [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias): este ZZ no crea workspace.

Constraints: Rust edición 2024 y “estable fijado” ([7.2](../../../docs/7-tickets.md)); modelos en `models/` git-ignored ([2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio)); datos de runtime bajo `<datos>/` (db, logs, media, meili, tls, secrets) definidos en tickets posteriores.

## Goals / Non-Goals

**Goals:**

- Dejar toolchain y formato reproducibles desde la raíz, usables por rust-analyzer, rustup y (más adelante) T-09-01
- Corregir `/target/` para que un workspace en `vortex-core/` no commitee `target/`
- Tener un chequeo TDD (happy + unhappy) que no dependa de Cargo

**Non-Goals (diseño):**

- Duplicar `rust-toolchain.toml` / `rustfmt.toml` dentro de `vortex-core/` cuando T-00-01-02 cree el workspace (rustup y rustfmt caminan hacia la raíz)
- Elegir `max_width` u otras knobs de rustfmt más allá de edición y fin de línea
- Definir el schema de `Config.paths` (T-00-05); `data/` es solo el ignore de desarrollo

## Decisions

### 1. Archivos de toolchain en la raíz, no en `vortex-core/`

**Elección:** `rust-toolchain.toml` y `rustfmt.toml` en la raíz del git.

**Por qué:** T-00-01-01 no crea `vortex-core/`. rustup y rustfmt buscan hacia arriba desde el cwd; rust-analyzer en Cursor abre el repo raíz. Un solo pin evita desalineación raíz vs workspace.

**Alternativa:** crear `vortex-core/` solo para esos archivos. Se descartó: mezcla este ZZ con el de workspace y no aporta comportamiento.

### 2. Canal `1.99.0` (estable fijado), no `stable` flotante

**Elección:** `channel = "1.99.0"` más `components = ["rustfmt", "clippy"]`.

**Por qué:** el ticket pide Rust estable *fijado*. En el Nodo Maestro actual `rustc` es `1.99.0 (b940084d7 2026-09-28)`. Clippy se pide ya para que T-00-01-02 y T-09-01 no instalen componentes a mano.

**Alternativa:** `channel = "stable"` (deriva entre máquinas) o pin más viejo “LTS”. Se descartó: no cumple “fijado” y el hardware objetivo ya corre 1.99.0.

```toml
[toolchain]
channel = "1.99.0"
components = ["rustfmt", "clippy"]
```

### 3. `rustfmt.toml` mínimo

**Elección:**

```toml
edition = "2024"
newline_style = "Unix"
```

**Por qué:** la edición es el requisito de stack; Unix evita diffs CRLF. El resto (p. ej. `max_width = 100`) queda en defaults de rustfmt hasta que el equipo tenga código que formatear.

**Alternativa:** copiar un rustfmt “opinionado” (imports, wrap comments). Se descartó: fuera del ~0,5 h y no hay Rust que formatear.

### 4. Extender `.gitignore`; `data/` como datos locales de desarrollo

**Elección:** no reescribir el `.gitignore` de curso. Cambios:

- `models/`
- reemplazar `/target/` por `target/` (cualquier profundidad, cubre `vortex-core/target/`)
- `data/` para db/logs/media locales mientras T-00-05 no fije otra ruta por defecto

**Por qué:** el ticket nombra `models/`, `target/` y “datos locales”. Las rutas de producción son configurables; en el clone de desarrollo lo habitual es un `data/` en la raíz, que no debe entrar a git.

**Alternativa:** ignorar `*.lock`, `*.surql` de runtime, o `~/Library/...`. Se descartó: demasiado amplio o fuera del repo.

### 5. Script POSIX `scripts/check-toolchain.sh` como harness TDD

**Elección:** un script con raíz opcional (`CHECK_ROOT`, default: raíz del repo). Valida presencia y claves de los TOML (grep/sed, sin parser extra) y que `.gitignore` contenga los patrones. Fixtures temporales para unhappy path (árbol incompleto). Exit 0/≠0.

**Por qué:** no hay workspace Cargo; un crate de tests no puede existir aún. El mismo script sirve de “Listo cuando” local de este ZZ.

**Alternativa:** Bats, o esperar a T-00-01-02 para tests Rust. Se descartó: Bats es dependencia nueva; posponer tests viola SDD+TDD de la spec.

El script no es producto de runtime (no lo invoca `vortex`). T-09-01 puede reusar la idea o pasar a `cargo fmt --check` / `clippy` cuando exista el workspace.

## Risks / Trade-offs

- **[Pin 1.99.0 vs runner de CI]** → T-09-01 instalará vía `rust-toolchain.toml` (rustup en `macos-14`). Si el runner no llega a 1.99.0, el job falla de forma visible; no se usa `stable` flotante para “pasar igual”.
- **[T-00-05 elige otra ruta default que no es `data/`]** → se suma el patrón en ese ticket; `data/` queda como ignore inofensivo.
- **[Chequeo TOML por grep]** → no detecta TOML inválido con canal presente en un comentario. Mitigación: fixtures y archivos reales son mínimos; no vale una crate `toml` todavía.
- **[`openspec new change` rechaza `T-00-01-01` (exige kebab-case)]** → el change se creó en minúsculas y se renombró; `status` / `instructions` aceptan el ID de proyecto. Queda como nota operativa, no bloquea este ZZ.

## Migration Plan

1. Añadir archivos y script; ampliar `.gitignore`.
2. Correr `scripts/check-toolchain.sh` (happy) y las fixtures unhappy.
3. Rollback: revertir los tres archivos + script; no hay estado persistido.

No hay deploy ni datos de usuario.
