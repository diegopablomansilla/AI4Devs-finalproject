# Design

## Context

En `develop`, `domain` expone `Producto` / `Id<Producto>`, `DomainError` (incl. `self_relation` y `duplicate_relation`) e `Id<T>` / `Timestamp`. `storage` ya tiene las tablas `similar_a`, `sustituible_por` y `complementa` (T-04-01). No hay entidad `Relacion`, `TipoRelacion` ni puerto `RelationRepo`. `recommend` es lib vacía: `check-workspace.sh` / `cargo-workspace` rechazan `[dependencies]` en ese crate. Allowlist de `domain`: `thiserror`, `ulid`, `rust_decimal`, `serde`; `serde_json` solo en dev. Edición 2024, rustc 1.99. Ver `proposal.md` para motivación y `specs/domain-relacion` para el contrato.

Constraints: sin persistencia (T-04-03); ~2 h; sin crates nuevas; `check-domain-deps.sh` sigue vetando SDKs. MVP del curso.

## Goals / Non-Goals

**Goals:**

- API pública de `domain` para `Relacion`, `TipoRelacion`, `default_bidireccional` / resolución del flag, validación 4.6.1 y `RelationRepo`
- Tests unitarios que cierran el Listo cuando (`self_relation`, `prioridad < 1`, `nota` > 500, tipos y default bidireccional)
- Trait compilable con un dummy en tests; cero I/O

**Non-Goals (diseño):**

- Adaptador SurrealDB, `RELATE`, transacción bidireccional, UNIQUE → `duplicate_relation`
- Agrupar `list_for_product` por tipo o hidratar `ResumenProducto` del destino (T-04-04)
- Casos de uso en `recommend`, allowlist de `recommend` en `check-workspace.sh`, HTTP
- Consulta 4.6.2 de recomendaciones (T-04-06)
- `async-trait` o `tokio` en `domain`
- Object-safety de `RelationRepo` (`dyn RelationRepo`)
- Ampliar `DomainError` o tocar `error.rs`

## Decisions

### 1. Un módulo `relacion.rs` + reexport

**Elección:** `vortex-core/crates/domain/src/relacion.rs` con entidad, `TipoRelacion`, marcador `GrupoBidireccional`, validación, helpers de bidireccionalidad y el trait. `lib.rs` hace `pub mod relacion` y `pub use` de `Relacion`, `RelacionDraft`, `TipoRelacion`, `NewRelacion`, `UpdateRelacion`, `RelationRepo`, `GrupoBidireccional`, `default_bidireccional` y `resolve_bidireccional`.

**Por qué:** mismo molde que `movimiento_stock.rs`; un archivo mantiene el ticket en ~2 h.

**Alternativa:** `relacion/{mod.rs,tipo.rs,repo.rs}`. Se descartó: más archivos que tests.

### 2. Campos y `TipoRelacion` en español; wire API en T-04-05

**Elección:** entidad y enum usan nombres de [4.6.1](../../docs/4-modelo.md#461-tablas-de-relación) / [5.3](../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos): `tipo` = `similar_a` / `sustituible_por` / `complementa`; extremos = `origen` / `destino`. `parse` acepta **solo** esos valores. `"similar"`, `"substitute"` o `"complement"` fallan en dominio.

**Por qué:** `Producto` / `MovimientoStock` ya guardan español; el gateway traduce en T-04-05. `in` es keyword de Rust; `origen`/`destino` evitan `r#in` y coinciden con la lectura "al consultar origen, ofrecer destino".

**Alternativa:** enum en inglés del contrato (`similar`/`substitute`/`complement`). Se descartó: duplicaría el mapa 5.3 y mezclaría capas (mismo rechazo que T-02-17 con `"inbound"`).

### 3. `FieldError.field` en español del modelo

**Elección:** `origen == destino` → `destino` + `code` `self_relation`; `prioridad < 1` → `prioridad`; `nota` > 500 → `nota`. No se usan las rutas OpenAPI (`targetProductId`, `priority`, `note`).

**Por qué:** HU-04-01 nombra el `code` `self_relation`, no la ruta JSON; `Categoria::reject_cycle` ya usa `padre`. T-04-05 mapea a `targetProductId` si el contrato lo exige.

**Alternativa:** `targetProductId`. Se descartó: rompería la convención de dominio.

### 4. `self_relation` reutiliza `InvalidInputCode::SelfRelation`; no se toca `error.rs`

**Elección:** `DomainError::invalid_input(InvalidInputCode::SelfRelation, …)` cuando `origen == destino`. `prioridad` y `nota` usan `InvalidInputCode::InvalidInput`. `duplicate_relation` no se emite aquí (índice UNIQUE en T-04-03).

**Por qué:** T-00-03 ya reserva `self_relation` para esta regla; T-02-02 documentó que `category_cycle` no la usa.

**Alternativa:** `invalid_input` genérico. Se descartó: HU-04-01 / 5.4 ya fijan el `code`.

### 5. Marcador `GrupoBidireccional` para `Id<GrupoBidireccional>`

**Elección:** `pub struct GrupoBidireccional;` vacío en el mismo módulo, solo para tipar el ULID compartido. Sin campos, sin repo, sin tabla.

**Por qué:** `Id<T>` exige un `T` distinto; `Option<String>` perdería el invariante ULID. T-04-03 genera el ULID al persistir el par.

**Alternativa:** `Option<String>`. Se descartó: no reutiliza `Id<T>`.

### 6. `default_bidireccional` + `resolve_bidireccional`; el flag no se guarda en la arista

**Elección:**

```rust
pub fn default_bidireccional(tipo: TipoRelacion) -> bool {
    matches!(tipo, TipoRelacion::SimilarA)
}

pub fn resolve_bidireccional(tipo: TipoRelacion, flag: Option<bool>) -> bool {
    flag.unwrap_or_else(|| default_bidireccional(tipo))
}
```

El snapshot `Relacion` no tiene campo `bidireccional`: la bidireccionalidad se observa por `grupo_bidireccional` presente (dos aristas, T-04-03). `NewRelacion.bidireccional: Option<bool>` es input de `create`.

**Por qué:** 4.6.1 modela dos aristas + ULID, no un bool persistido. HU-04-02 "la opción viene marcada" es el default del tipo.

**Alternativa:** persistir `bidireccional: bool` en la entidad. Se descartó: no está en 4.6.1 y duplicaría `grupo_bidireccional`.

### 7. `RelationRepo` con CRUD mínimo; `recommend` sigue vacío

**Elección:**

```rust
pub struct NewRelacion {
    pub origen: Id<Producto>,
    pub destino: Id<Producto>,
    pub tipo: TipoRelacion,
    pub prioridad: i32,
    pub nota: Option<String>,
    pub bidireccional: Option<bool>,
}

pub struct UpdateRelacion {
    pub id: Id<Relacion>,
    pub tipo: TipoRelacion,
    pub prioridad: Option<i32>,
    pub nota: Option<Option<String>>,
}

pub trait RelationRepo {
    fn create(
        &self,
        input: NewRelacion,
    ) -> impl Future<Output = Result<Relacion, DomainError>> + Send;

    fn update(
        &self,
        input: UpdateRelacion,
    ) -> impl Future<Output = Result<Relacion, DomainError>> + Send;

    fn delete(
        &self,
        tipo: TipoRelacion,
        id: Id<Relacion>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn list_for_product(
        &self,
        producto: Id<Producto>,
    ) -> impl Future<Output = Result<Vec<Relacion>, DomainError>> + Send;
}
```

Dummy en `#[cfg(test)]` que implementa el trait y devuelve `Unavailable`. `tipo` en `update`/`delete` elige la tabla (`similar_a` / `sustituible_por` / `complementa`); el `id` no lleva prefijo. rustc 1.99 + edición 2024 evitan `async-trait`. El test DC-05 lee `catalog` con `include_str!` (o path de workspace) y afirma que no declara `RelationRepo`.

`recommend` **no** gana `domain` en este ticket: la allowlist de `cargo-workspace` todavía trata ese crate como lib vacía. DC-05 queda como comentario de módulo + test negativo sobre `catalog`. T-04-05 abre la allowlist y escribe los casos de uso.

**Por qué:** el Hace pide el puerto "usado por recommend (DC-05)"; cablear `recommend` ahora rompería `check-workspace.sh` y mezclaría alcance de T-04-05. Declarar `update`/`delete`/`list_for_product` ahora evita rediseñar el trait en T-04-04 (mismo criterio que `ProductRepo` vs ir agregando métodos).

**Alternativa:** trait solo con `create`. Se descartó: T-04-04 tendría que mutar el puerto en el mismo sprint. **Alternativa:** dependenciar `recommend` → `domain` y extender `cargo-workspace`. Se descartó: ~allowlist + fixtures, fuera del Listo cuando y del ~2 h.

## Risks / Trade-offs

- **[`list_for_product` sin agrupado ni resumen]** → Mitigación: T-04-04 puede envolver el `Vec<Relacion>` o ampliar el tipo de retorno; el trait ya nombra la operación.
- **[`UpdateRelacion.nota: Option<Option<String>>`]** → Mitigación: `None` = no tocar; `Some(None)` = borrar nota; `Some(Some(s))` = set. T-04-04 lo implementa; este ticket solo declara el input.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-04-02` y rename a `T-04-02` (igual que tickets previos).
- **[DC-05 sin dep en `recommend`]** → Mitigación: test de que el puerto no está en `catalog`; T-04-05 cablea el crate.

## Migration Plan

1. Red: tests unitarios de `TipoRelacion`, `Relacion` y `RelationRepo` (no compilan / fallan).
2. Green: `relacion.rs` + reexports; dummy del trait.
3. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` en 0, `openspec validate T-04-02`.
4. Rollback: quitar el módulo y los reexports. Sin migración de datos ni API HTTP.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El detalle de fixtures (`Id<Producto>` de prueba) se copia del estilo de `producto.rs` / `movimiento_stock.rs` en apply.
