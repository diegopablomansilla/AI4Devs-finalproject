# Design

## Context

En `develop`, `domain` expone `DomainError` y declara solo `thiserror`. `scripts/check-workspace.sh` permite esa única crate en `domain` y rechaza `[dev-dependencies]`. Los pins `ulid = "=3.0.0"`, `rust_decimal = "=1.43.0"` y `serde = "=1.0.229"` ya están en `[workspace.dependencies]`; `serde_json` y `chrono` no. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: no entidades (T-02-02+); `check-domain-deps.sh` sigue vetando SDKs de motor; ~2 h; alineación a `Ulid` / `Decimal` / `Timestamp` de `docs/openapi.yaml`.

## Goals / Non-Goals

**Goals:**

- Newtypes `Id<T>`, `Decimal`, `Timestamp` parseables y serializables (string + JSON) según el contrato
- Allowlist mínima en `check-workspace.sh` para esas crates en `domain`
- Tests unitarios happy + unhappy que cierren el Listo cuando

**Non-Goals (diseño):**

- `chrono` / `time` como dependencia nueva
- `NonNegativeDecimal` o asserts de stock/precio
- `From<DomainError>` en los errores de parse (los casos de uso pondrán `field`)
- `trybuild` para demostrar que `Id<Producto>` ≠ `Id<Categoria>`
- Features `serde` de `ulid` / `rust_decimal` (el newtype controla el wire format)

## Decisions

### 1. Tres módulos + reexport, `PhantomData` en `Id<T>`

**Elección:** `id.rs`, `decimal.rs`, `timestamp.rs`; `pub use` en `lib.rs` (`domain::Id`, `domain::Decimal`, `domain::Timestamp`). `Id<T>` envuelve `ulid::Ulid` con `PhantomData<fn() -> T>` para que sea `Copy`/`Send` sin definir entidades. Los tests usan un marcador privado (`struct Dummy;`).

**Por qué:** el ticket nombra `Id<T>` y prohíbe entidades. El marcador evita mezclar IDs cuando existan `Producto` / `Categoria`.

**Alternativa:** un `Id` sin parámetro. Se descartó: T-02-02 y T-02-16 perderían seguridad de tipos. **Alternativa:** structs vacíos `ProductId` / `CategoryId` en este ticket. Se descartó: son entidades.

### 2. Parse de `Id` = patrón OpenAPI, no `Ulid::from_string` a ciegas

**Elección:** aceptar solo `^[0-9A-HJKMNP-TV-Z]{26}$` (mayúsculas, 26, Crockford). Rechazar `tabla:`, minúsculas, `I`/`L`/`O`/`U`, longitudes ≠ 26. Luego construir `ulid::Ulid`. `Display` / serde = esas 26 letras. `Id::generate()` usa `Ulid::new()`.

**Por qué:** el contrato HTTP y el dominio deben hablar el mismo alfabeto; `ulid` 3 puede ser más permisivo (minúsculas). El prefijo es el unhappy explícito de [5.1](../../docs/5-contract-api.md#51-convenciones-generales).

**Alternativa:** normalizar minúsculas a Crockford. Se descartó: el esquema OpenAPI no las admite; mejor fallar temprano.

### 3. `Decimal`: gramática OpenAPI, serde solo string

**Elección:** `FromStr` exige `^-?\d{1,12}(\.\d{1,4})?$` y después `rust_decimal::Decimal`. Serde: `Visitor` con `visit_str` / `visit_string` únicamente (`visit_i64`/`visit_f64` → error). `Serialize` emite string. Igualdad por valor decimal, no por ceros a la derecha.

**Por qué:** [5.1] prohíbe float en el cable; `rust_decimal` con feature `serde` aún puede aceptar números. El patrón recorta notación científica y más de 4 decimales.

**Alternativa:** `serde-with-str` de `rust_decimal`. Se descartó: no garantiza el rechazo de JSON number ni el tope de 4 decimales.

### 4. `Timestamp` sin `chrono`: parser UTC propio

**Elección:** newtype sobre un instante UTC (`i64` segundos Unix + `u32` nanos, o equivalente `std`). Parser RFC 3339 que acepta `…Z` y `…+00:00` (y `+00:00` con fracción); serializa siempre `YYYY-MM-DDTHH:MM:SS[.frac]Z`. Rechaza naive, `-03:00` y epoch numérico.

**Por qué:** `chrono`/`time` no están en el stack T-00-01; agregarlas cambia `cargo-workspace` más allá de la allowlist. El subconjunto UTC del contrato cabe en un parser pequeño y testeable.

**Alternativa:** pin `time` o `chrono`. Se descartó para no ampliar el catálogo de producto. **Alternativa:** guardar `String` validada. Se descartó: no da igualdad de instante ni orden.

### 5. Errores de parse locales, no `DomainError`

**Elección:** `InvalidId`, `InvalidDecimal`, `InvalidTimestamp` (unit o con motivo interno). `FromStr::Err` = ese tipo. No se mapean a `InvalidInput` aquí.

**Por qué:** `FieldError` necesita `field` (p. ej. `price`); el parse de un newtype no lo tiene. T-02-16 / T-00-11 arman el `DomainError`.

**Alternativa:** `DomainError::invalid_input(InvalidInput, vec![])`. Se descartó: pierde la ruta de campo y mezcla capas.

### 6. Allowlist de `check-workspace.sh` + pin `serde_json = "=1.0.154"`

**Elección:** `domain_deps_allowed` acepta en `[dependencies]` exactamente el subconjunto de `{thiserror, ulid, rust_decimal, serde}` (todas `workspace = true`) y en `[dev-dependencies]` solo `serde_json` con `workspace = true`. Cualquier otra clave, `serde_json` en runtime, o deps en otro lib / `bin/vortex`, falla. Agregar `serde_json = "=1.0.154"` a `[workspace.dependencies]` y `check_pin`. Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: el caso `ulid` pasa a happy (dentro de la allowlist); el unhappy usa `clap` o `surrealdb`.

**Por qué:** T-00-03 dejó la lista de un elemento a propósito para este ticket. `serde_json` es la forma barata de cumplir «ida y vuelta JSON» del Listo cuando; 1.0.154 es la versión publicada al planificar.

**Alternativa:** tests de serde con `serde::de::value` y un `Serializer` casero. Se descartó: más código de test que el pin. **Alternativa:** no pinnear `serde_json` y versionarla suelta en el crate. Se descartó: rompe «pins exactos» del workspace.

### 7. Tests unitarios junto a cada módulo

**Elección:** `#[cfg(test)]` en `id.rs` / `decimal.rs` / `timestamp.rs` (nombres de `specs/domain-types`). Fixtures POSIX `CHECK_ROOT` para `check-workspace.sh`, reusando `run-script-tests.sh`.

**Por qué:** el Listo cuando es parse/serde, no HTTP ni SurrealDB.

## Risks / Trade-offs

- **[Parser RFC 3339 incompleto]** → Mitigación: cubrir Z, fracción, `+00:00`; unhappy naive y offset no UTC. No soportar leap seconds ni week-dates.
- **[Display de Decimal cambia ceros (`120.50` → `120.5`)]** → Mitigación: la spec exige patrón e igualdad de valor, no el string original.
- **[serde_json entra a la tabla del stack]** → Mitigación: solo `[dev-dependencies]` de `domain`; el binario no la toma.
- **[ulid 3 y rand/entropy en tests]** → Mitigación: `generate` se testea por patrón, no por valor fijo; parse usa el ejemplo del contrato.
- **[T-00-05+ volverán a tocar la allowlist]** → Mitigación: lista explícita en una función, no un booleano `thiserror`.

## Migration Plan

1. Red: tests de `Id` / `Decimal` / `Timestamp` (no compilan).
2. Red: `check-workspace.sh` still green on repo (aún sin las deps); fixture `clap` en `domain` falla.
3. Green: pin `serde_json`, allowlist, módulos, tests verdes.
4. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` en 0, `openspec validate T-00-04`.
5. Rollback: revertir `crates/domain`, el pin y el bloque de allowlist. Sin datos ni API.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. Fracción de segundo en el `Display` canónico (omitir `.0` vs conservarla) queda al apply mientras el roundtrip JSON y el sufijo `Z` se cumplan.
