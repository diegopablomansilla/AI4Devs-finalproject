# Design

## Context

En `develop`, `domain` expone `Producto` / `ProductRepo`, `DomainError` (incl. `invalid_input` y `negative_stock`), `Id<T>`, `Decimal` con signo y `Timestamp`. `storage` ya tiene la tabla `movimiento_stock` (T-02-15) con campos `READONLY`. No hay entidad `MovimientoStock`, `TipoMovimiento` ni puerto `StockLedger`. Allowlist de `domain`: `thiserror`, `ulid`, `rust_decimal`, `serde`; `serde_json` solo en dev. Edición 2024, rustc 1.99. Ver `proposal.md` para motivación y `specs/domain-movimiento-stock` para el contrato.

Constraints: sin persistencia (T-02-24); ~2 h; sin crates nuevas; `check-domain-deps.sh` sigue vetando SDKs. MVP del curso.

## Goals / Non-Goals

**Goals:**

- API pública de `domain` para `MovimientoStock`, `TipoMovimiento`, validación de reglas 4.5.1 y `StockLedger`
- Tests unitarios que cierran el Listo cuando (signo por tipo, `cantidad == 0`, `ajuste` sin `motivo`)
- Trait compilable con un dummy en tests; cero I/O

**Non-Goals (diseño):**

- Adaptador SurrealDB, `storage::Tx`, proyección `producto.stock`, `THROW "stock_negativo"`, outbox
- Calcular `stock_resultante` a partir del stock actual (T-02-24)
- Paginación de `list_by_producto` (T-00-21 / T-02-26)
- Repositorio en memoria de producción
- `async-trait` o `tokio` en `domain`
- Object-safety de `StockLedger` (`dyn StockLedger`)
- Ampliar `DomainError` o `Decimal`
- Entidad completa `Venta` (T-08-01)

## Decisions

### 1. Un módulo `movimiento_stock.rs` + reexport

**Elección:** `vortex-core/crates/domain/src/movimiento_stock.rs` con entidad, `TipoMovimiento`, marcador `Venta`, validación y el trait. `lib.rs` hace `pub mod movimiento_stock` y `pub use` de `MovimientoStock`, `MovimientoStockDraft`, `TipoMovimiento`, `NewMovimientoStock`, `StockLedger` y `Venta`.

**Por qué:** mismo molde que `producto.rs`; un archivo mantiene el ticket en ~2 h.

**Alternativa:** `movimiento_stock/{mod.rs,tipo.rs,repo.rs}`. Se descartó: más archivos que tests.

### 2. Campos y `TipoMovimiento` en español; wire API en T-02-25

**Elección:** entidad y enum usan nombres de [4.5.1](../../docs/4-modelo.md#451-movimiento_stock): `tipo` = `inicial`/`ingreso`/`ajuste`/`venta`. `parse_wire` acepta **solo** esos valores. `"inbound"` o `"adjustment"` fallan en dominio.

**Por qué:** `Producto`/`Categoria` ya guardan español; [5.3](../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) traduce en el gateway (T-02-25).

**Alternativa:** aceptar inglés en `parse_wire`. Se descartó: duplicaría el mapa 5.3 y mezclaría capas.

### 3. `FieldError.field` en español del modelo

**Elección:** violaciones de signo o cero → `cantidad`; `ajuste` sin motivo → `motivo`; `venta` mal usada → `venta`. No se usan las rutas OpenAPI (`quantity`, `reason`, `saleId`).

**Por qué:** HU-02-07 no nombra rutas de API; `Producto` ya usa `sku`/`nombre`/`tipo` para la ficha (el inglés quedó reservado a `attributes[n].*` porque HU-02-04 lo nombra). T-02-25 mapea a `quantity`/`reason` si el contrato lo exige.

**Alternativa:** `quantity`/`reason`. Se descartó: rompería la convención de dominio y el ticket no lo pide.

### 4. Constructor `MovimientoStock::try_new(draft)` valida snapshot; `append` no trae `stock_resultante`

**Elección:** `MovimientoStockDraft` trae el snapshot 4.5.1 (incl. `stock_resultante`). `try_new` aplica signo, `motivo`, `venta`, `cantidad ≠ 0` y `stock_resultante ≥ 0`. No consulta repos ni el stock actual del producto.

```rust
pub struct NewMovimientoStock {
    pub producto: Id<Producto>,
    pub tipo: TipoMovimiento,
    pub cantidad: Decimal,
    pub motivo: Option<String>,
    pub venta: Option<Id<Venta>>,
    pub terminal: Option<String>,
}
```

T-02-24 calcula `$nuevo = $producto.stock + $cantidad` y recién entonces hidrata el snapshot. Este ticket no suma.

**Por qué:** mezclar I/O o la proyección en el constructor rompería el No hace (persistencia) y el alcance de T-02-24.

**Alternativa:** helper `proyectar_stock(actual, cantidad)` que devuelve `negative_stock`. Se descartó: T-02-24 ya tiene `THROW "stock_negativo"` y el Listo cuando de este ticket son las reglas 4.5.1, no la transacción.

### 5. `stock_resultante < 0` usa `negative_stock`; el resto `invalid_input`

**Elección:** `DomainError::invalid_input(InvalidInputCode::NegativeStock, …)` cuando `stock_resultante < 0`. Signo, cero, `motivo` y `venta` usan `InvalidInputCode::InvalidInput`. No se toca `error.rs`.

**Por qué:** T-00-03 ya reserva `negative_stock` para este invariante; T-02-24 traducirá el `THROW` al mismo `code`.

**Alternativa:** todo `invalid_input`. Se descartó: perdería el `code` que HU-02-07 / 5.4 ya definen.

### 6. Marcador `Venta` para `Id<Venta>` hasta T-08-01

**Elección:** `pub struct Venta;` vacío en el mismo módulo, solo para tipar `Id<Venta>`. Sin campos, sin repo, sin migración.

**Por qué:** `Id<T>` exige un `T` distinto; usar `Id<Producto>` o `String` perdería el invariante de identidad. T-08-01 puede sustituir el marcador por la entidad real (mismo nombre, mismo `Id<Venta>`).

**Alternativa:** `Option<String>` con ULID. Se descartó: no reutiliza `Id<T>`. **Alternativa:** no validar `venta` hasta T-08-01. Se descartó: 4.5.1 ya la marca obligatoria en tipo `venta`.

### 7. `StockLedger` con `append` y `list_by_producto`; sin `update`/`delete`

**Elección:**

```rust
pub trait StockLedger {
    fn append(
        &self,
        input: NewMovimientoStock,
    ) -> impl Future<Output = Result<MovimientoStock, DomainError>> + Send;

    fn list_by_producto(
        &self,
        producto: Id<Producto>,
    ) -> impl Future<Output = Result<Vec<MovimientoStock>, DomainError>> + Send;
}
```

Dummy en `#[cfg(test)]` que implementa el trait y devuelve `Unavailable`. T-02-24 implementa `append`; T-02-26 hidrata el listado paginado (este `list_by_producto` queda como consulta mínima del puerto). rustc 1.99 + edición 2024 evitan `async-trait`. El test `stock_ledger_trait_has_no_update_or_delete` lee `include_str!("movimiento_stock.rs")` y afirma que no hay `fn update(` ni `fn delete(`.

**Por qué:** el Hace pide «solo `append` y consultas»; HU-02-07 inmutabilidad se verifica en el puerto, no en SQL (eso ya lo cubrió T-02-15 con `READONLY`).

**Alternativa:** trait síncrono. Se descartó: el adaptador SurrealDB es async. **Alternativa:** no declarar `list_by_producto` hasta T-02-26. Se descartó: el ticket nombra consultas ahora.

## Risks / Trade-offs

- **[Marcador `Venta` vs entidad T-08-01]** → Mitigación: mismo nombre y `Id<Venta>`; T-08-01 mueve o rellena el tipo, no cambia el campo del movimiento.
- **[`stock_resultante < 0` en constructor vs `THROW` en T-02-24]** → Mitigación: mismo `code` `negative_stock`; el adaptador no construye un snapshot inválido.
- **[`list_by_producto` sin cursor]** → Mitigación: T-02-26 / T-00-21 añaden paginación en API; el trait puede evolucionar entonces.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-17` y rename a `T-02-17` (igual que tickets previos).

## Migration Plan

1. Red: tests unitarios de `TipoMovimiento`, `MovimientoStock` y `StockLedger` (no compilan / fallan).
2. Green: `movimiento_stock.rs` + reexports; dummy del trait.
3. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` en 0, `openspec validate T-02-17`.
4. Rollback: quitar el módulo y los reexports. Sin migración de datos ni API HTTP.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El detalle de helpers de test (`Id<Producto>` de fixture) se copia del estilo de `producto.rs` en apply.
