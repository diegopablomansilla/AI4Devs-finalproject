# Design

## Context

En `develop`, `domain` ya expone `MovimientoStock`, `NewMovimientoStock`, `TipoMovimiento` y el puerto `StockLedger` (`append`, `list_by_producto`) sin adaptador (T-02-17). `storage` tiene `0006_movimiento_stock` (campos `READONLY`), `ProductoRepo::create` que escribe el movimiento `inicial` **inline** (T-02-18), y `Tx` que ya mapea `THROW "stock_negativo"` → `negative_stock` (T-03-02). `ProductRepo::update` incrementa `version`; el ledger no debe tocarla. No hay HTTP de stock (T-02-25). Ver `proposal.md` para motivación y `specs/vortex-storage` para el contrato.

Constraints: MVP del curso; ~2 h; `domain` no gana deps; `api`/`catalog` no se tocan; SurrealDB `=2.7.0`; sin `openapi.yaml`.

## Goals / Non-Goals

**Goals:**

- Adaptador hexagonal de `StockLedger::append` atómico: movimiento + proyección `producto.stock` + outbox `motivo: "stock"`
- Aritmética del stock **dentro** de la transacción SurrealQL (4.5.2), para que dos ingresos concurrentes sumen
- Tests que cierran el Listo cuando: ajuste −8 sobre stock 5; dos ingresos concurrentes

**Non-Goals (diseño):**

- Extraer el `inicial` de `ProductRepo::create` a `StockLedger`
- Caso de uso en `catalog` ni wiring en `vortex serve` (T-02-25)
- `Tx` que acepte un `Tx` padre para ventas multilínea (T-08-03 abre la suya)
- Helper público nuevo en `domain` si `try_new` alcanza para pre-validar
- Paginación de `list_by_producto`

## Decisions

### 1. Adaptador `MovimientoStockRepo` en `storage/src/movimiento_stock.rs`

**Elección:** newtype `MovimientoStockRepo { db: Arc<Db> }` que implementa `domain::StockLedger`. `lib.rs` hace `mod movimiento_stock` y `pub use movimiento_stock::MovimientoStockRepo`. Escrituras vía `Tx::run` + `emit` + `commit`; hidratar el `MovimientoStock` con DTO `serde` + strip de `movimiento_stock:` / `producto:`. Sembrar productos en tests con `ProductoRepo` (mismo helper de categoría que `tests/producto.rs`).

**Por qué:** mismo molde que T-02-08 / T-02-18 / T-02-23. El adaptador pertenece a `storage`, no a `catalog`. Un archivo `tests/stock_ledger.rs` no mezcla esquema T-02-15 (`tests/movimiento_stock.rs`) con el puerto.

**Alternativa:** meter `append` en `ProductoRepo`. Se descartó: el puerto es `StockLedger`; mezclarlo hincharía ficha y rompería el corte T-02-18. **Alternativa:** crate `catalog`. Se descartó: este ticket no tiene caso de uso HTTP.

### 2. Stock resultante en SurrealQL, no en Rust

**Elección:** una sola `Tx` con el script de [4.5.2](../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción):

```text
BEGIN
  LET $nuevo = $producto.stock + $cantidad;
  IF $nuevo < 0 { THROW "stock_negativo"; };
  CREATE movimiento_stock:... CONTENT { ..., stock_resultante: $nuevo, ... };
  UPDATE $producto SET stock = $nuevo;   -- sin version += 1
  CREATE outbox ...                     -- emit(), motivo "stock"
COMMIT
```

El `SELECT` de existencia del producto (→ `not_found`) ocurre **antes** de la `Tx` y **no** se usa ese `stock` para la suma. `cantidad` se interpola como literal `Ndec` (mismo helper que `ProductoRepo`).

**Por qué:** el Listo cuando exige dos ingresos concurrentes **ambos aplicados**. Un RMW en Rust (`get.stock + qty` → `UPDATE`) pierde actualizaciones. `Tx::commit` ya traduce `stock_negativo`.

**Alternativa:** `UPDATE producto SET stock += $cantidad` + chequeo posterior. Se descartó: 4.5.2 fija `LET $nuevo` y el movimiento necesita `stock_resultante` atómico. **Alternativa:** retry optimista con `WHERE stock = $visto`. Se descartó: más complejo; la transacción del motor debe serializar escrituras al mismo `record`.

### 3. Pre-validación con `MovimientoStock::try_new` y `stock_resultante` placeholder

**Elección:** antes de `Tx`, construir un draft con `stock_resultante = 0` (siempre ≥ 0) para disparar signo, `motivo` y `venta`. Tras `commit`, re-leer la fila y volver a `try_new` con el `stock_resultante` real. No se añade API en `domain`.

**Por qué:** `NewMovimientoStock` no trae `stock_resultante` (T-02-17). Duplicar `validate_motivo` en `storage` divergiría. El `negative_stock` de proyección lo da el `THROW`, no el placeholder.

**Alternativa:** `NewMovimientoStock::validate()` en `domain`. Se descartó: cambio de puerto innecesario si `try_new` cubre el unhappy de `ajuste` sin motivo.

### 4. `tipo` `venta` e `inicial` no se bloquean; no hay tests de venta

**Elección:** `append` persiste cualquier `TipoMovimiento` que pase dominio (incl. `venta` con `Id<Venta>` en el campo `option<record>` genérico). MUST NOT `CREATE venta` ni tocar `contador:venta`. Los tests de este ticket cubren `ingreso` y `ajuste`. `ProductRepo::create` sigue escribiendo `inicial` inline.

**Por qué:** T-02-18 dejó el `MUST NOT` de `append` en `ProductRepo`, no una prohibición de tipos en el ledger. T-08-03 es la transacción de venta completa (varias líneas + contador); no puede reusar un `append` que abre su propia `Tx`. Permitir la fila `venta` no es «confirmar venta».

**Alternativa:** rechazar `tipo` `venta` con `invalid_input`. Se descartó: el No hace es el flujo POS, no el campo del ledger. **Alternativa:** extraer un `append_in(tx: &mut Tx)`. Se descartó: YAGNI hasta T-08-03 y rompería el builder actual de `Tx`.

### 5. `list_by_producto` sin cursor, más reciente primero

**Elección:** `SELECT * FROM movimiento_stock WHERE producto = $p ORDER BY creado_en DESC`. Lista vacía si no hay filas. No es `not_found` por producto ausente.

**Por qué:** el trait ya lo exige; los tests del Listo cuando verifican movimientos. T-02-26 añade cursor en HTTP.

**Alternativa:** dejar `list_by_producto` como `todo!` / `unavailable`. Se descartó: no se puede `impl StockLedger`.

### 6. Producto inactivo acepta movimientos

**Elección:** `append` no mira `activo`.

**Por qué:** T-02-22 conserva el ledger en la baja; un ajuste de conteo sobre un discontinuado no debe bloquearse aquí. T-02-25 decidirá si el caso de uso HTTP filtra.

### 7. Tests en `tests/stock_ledger.rs`; concurrencia con `tokio::join!`

**Elección:** sembrar categoría + producto con stock 5 u 10 vía `ProductoRepo::create`. Prefijo `stock_ledger_`. El concurrente lanza dos `append` ingreso `10` sobre stock `5` y espera stock `25`. El unhappy Listo cuando: stock 5, ajuste −8, `code` `negative_stock`, recuento de movimientos igual, stock 5.

**Por qué:** `tests/movimiento_stock.rs` es el esquema READONLY (T-02-15). `tokio::join!` es el mismo espíritu que `product_repo_concurrent_updates_same_version_conflict`.

### 8. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-24` y rename a `T-02-24`.

## Risks / Trade-offs

- **[Aislamiento de SurrealDB 2.7 en `kv-mem`]** → dos `BEGIN` concurrentes podrían no serializar y perder una suma. Mitigación: el SQL lee `producto.stock` dentro de la Tx; si el test `stock_ledger_concurrent_ingresos_both_apply` falla, reintentar el segundo `append` no está en spec — hay que ajustar el script (p. ej. un solo `UPDATE … SET stock +=`) sin cambiar el contrato observable (ambos aplicados, stock 25).
- **[Tx::run no tiene bindings en sentencias de negocio]** → interpolar ULID y `Ndec` como `ProductoRepo`. Mitigación: reusar escape de strings para `motivo`/`terminal`; no concatenar input crudo sin `sql_string`.
- **[Placeholder `stock_resultante = 0` en pre-validación]** → no detecta `negative_stock` antes del motor. Aceptado: esa regla es 4.5.2 / `THROW`.
- **[Alcance ~2 h]** → un módulo + un archivo de tests; sin HTTP. No se divide el ticket.

## Migration Plan

Sin migración de esquema: `movimiento_stock` y `producto.stock` ya existen. Rollback = revertir el PR; las filas `inicial` de T-02-18 no se reescriben.

## Open Questions

Ninguna que altere specs o tareas. El Listo cuando fija cantidades (stock 5 / ajuste −8; dos ingresos concurrentes).
