# Design

## Context

En `develop`, `vortex-core/crates/domain` es un lib vacío (`src/lib.rs` placeholder, sin `[dependencies]`). `scripts/check-workspace.sh` rechaza **cualquier** `[dependencies]` en los once libs; T-00-02-01 dejó explícito que T-00-03 debía relajarlo al agregar `thiserror`. La tabla HTTP vive en [5.4](../../docs/5-contract-api.md#54-errores); el Gateway (T-00-11) todavía no existe. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: `thiserror = "=2.0.21"` ya está en `[workspace.dependencies]`; `domain` no puede tomar SDKs de motor (`check-domain-deps.sh`); ~2 h.

## Goals / Non-Goals

**Goals:**

- Un módulo `domain::error` con variantes y `code` tipados para que T-00-08 / T-00-11 / catálogo no reinventen el enum
- Relajar el harness de workspace lo mínimo para `thiserror` en `domain`
- Tests unitarios en el crate `domain` (happy + unhappy) que cierren el Listo cuando

**Non-Goals (diseño):**

- `IntoResponse` / `problem+json` / títulos en español
- `serde` en `DomainError` o `FieldError`
- Permitir `ulid` / `rust_decimal` / `serde` en `domain` (T-00-04) u otras crates
- Parser de OpenAPI para generar el enum (la tabla 5.4 cabe a mano)

## Decisions

### 1. Códigos anidados por variante, más un `DomainCode` unificado

**Elección:** cada variante carga un enum de códigos de su fila de 5.4 (`InvalidInputCode`, `ConflictCode`, `UnavailableCode`; `NotFound` y `Timeout` no necesitan enum extra). `DomainError::code()` devuelve un `DomainCode` (enum único) cuyo `as_str()` / `Display` es el snake_case de OpenAPI. `FromStr` de `DomainCode` acepta solo esos valores.

Esqueleto (nombres exactos en apply):

```rust
pub enum DomainError {
    NotFound,
    Conflict { code: ConflictCode },
    InvalidInput { code: InvalidInputCode, errors: Vec<FieldError> },
    Unavailable { code: UnavailableCode },
    Timeout,
}
```

**Por qué:** el ticket pide «enums de `code`» y la tabla 5.4 ya agrupa por variante. El compilador cubre «code de otra variante»; el unhappy de `version_conflict` en `InvalidInput` queda como test que no compila o, si hay un constructor genérico, `Err`. T-00-11 mapea variante → HTTP y `code()` → campo `code`.

**Alternativa:** un solo `enum DomainCode` en todas las variantes y validar el par en runtime. Se descartó: permite `InvalidInput { code: VersionConflict }` y empuja el invariante a cada caller.

**Alternativa:** incluir `payload_too_large` / `internal` en `DomainCode`. Se descartó: filtra Gateway al hexágono; T-00-11 / T-00-12 los definen en `api`.

### 2. `thiserror` en `domain`, no `std::error::Error` a mano

**Elección:** `thiserror` con `workspace = true`. `#[derive(Debug, thiserror::Error)]` y mensajes de `Display` técnicos (inglés o el `code`); los `title` en español son T-00-11.

**Por qué:** [7.2](../../docs/7-tickets.md) lista `thiserror` en el stack; el pin ya existe. Implementar `Error` + `Display` a mano ahorra la relajación del harness pero pelea con el default del proyecto.

**Alternativa:** `anyhow` / `snafu`. Fuera de `[workspace.dependencies]`.

### 3. `FieldError::try_new` y lista vacía permitida

**Elección:** `FieldError` con `try_new(field, message) -> Result<FieldError, …>` que rechaza `field` o `message` vacíos/whitespace. `InvalidInput` acepta `Vec<FieldError>` vacío (error de validación sin ruta de campo, `code = invalid_input`). El constructor de `InvalidInput` no acepta `FieldError` inválido porque no se puede construir.

**Por qué:** 5.4 muestra `errors: [{ field, message }]` con rutas tipo `attributes[1].value`; un `field` vacío no sirve a T-01-07. La lista vacía cubre `invalid_input` genérico (p. ej. cuerpo que no es de un campo).

**Alternativa:** `field: String` público sin validar. Se descartó: el unhappy del ticket quedaría solo en parse de `code`.

### 4. Relajar `check-workspace.sh` solo para `thiserror` en `domain`

**Elección:** si el crate es `domain` y `[dependencies]` contiene únicamente `thiserror` (línea `workspace = true`), el script no falla. Cualquier otra crate en `domain`, `[dev-dependencies]` en `domain`, o `[dependencies]` en otro lib / `bin/vortex`, sigue fallando. Comentario de cabecera: T-00-03.

**Por qué:** T-00-01 usó «cero deps» como proxy de hexágono; T-00-02 ya cubre motores con `cargo tree`. Sin este recorte, el Listo cuando de T-00-03 no puede convivir con `check-workspace.sh`. T-00-04 ampliará el allowlist (`ulid`, `rust_decimal`, `serde`).

**Alternativa:** dejar de chequear `[dependencies]` en todos los libs y fiarse solo de `check-domain-deps.sh`. Se descartó: `api` / `catalog` vacíos podrían acoplarse sin que nadie lo vea todavía.

**Alternativa:** no tocar el script y no usar `thiserror`. Se descartó en la decisión 2.

### 5. Tests unitarios en `crates/domain/src/error.rs`

**Elección:** `#[cfg(test)] mod tests` junto al módulo (o `error/tests.rs` si el archivo crece). Un test por escenario de `domain-error`. Fixtures temporales `CHECK_ROOT` para los unhappy de `check-workspace.sh` (mismo patrón POSIX que T-00-01-03), no una suite permanente salvo que el apply las deje en `scripts/fixtures/workspace/` de forma barata.

**Por qué:** el Listo cuando pide tests unitarios de construcción y `code`, no integración HTTP.

**Alternativa:** `crates/domain/tests/*.rs` de integración. Innecesario: no hay frontera de crate que probar más allá de `pub use`.

### 6. Sin `serde`, sin reexport obligatorio extra

**Elección:** `pub mod error;` y `pub use error::{DomainError, FieldError, DomainCode, …}` en `lib.rs` para que `domain::error::DomainError` (ticket) y `domain::DomainError` convivan. Sin `Serialize`.

**Por qué:** T-00-11 arma el JSON; serializar el enum de dominio ahora fijaría un shape HTTP prematuro.

## Risks / Trade-offs

- **[T-00-04 vuelve a tocar `check-workspace.sh`]** → el allowlist de `domain` se amplía otra vez. Mitigación: dejar la regla en una función `domain_deps_ok` con lista explícita de un elemento (`thiserror`).
- **[Códigos nuevos en OpenAPI]`** → hay que agregar la variante de enum a mano. Mitigación: el enum es corto; T-09-03 no genera Rust.
- **[thiserror 2 Display vs. `code()`]** → un caller podría parsear el `Display`. Mitigación: T-00-11 usará `code()`, no el mensaje; tests fijan `as_str()`.
- **[Grep ingenuo de `thiserror` en el script]** → un comentario podría falsear. Mitigación: exigir la sección `[dependencies]` y la clave `thiserror`.

## Migration Plan

1. Red: tests de `DomainError` / `code` / `FieldError` en `domain` (no compilan).
2. Red: `check-workspace.sh` exige allowlist; el repo actual (sin deps en `domain`) sigue en 0; fixture con `ulid` en `domain` falla.
3. Green: módulo `error`, `thiserror` en `Cargo.toml`, relajar el script, tests verdes.
4. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` en 0, `openspec validate T-00-03`.
5. Rollback: revertir `crates/domain` y el bloque nuevo de `check-workspace.sh`. No hay datos ni API.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. Los nombres de los enums internos (`InvalidInputCode` vs `InvalidInputKind`) quedan al apply mientras `code()` y las cinco variantes cumplan la spec.
