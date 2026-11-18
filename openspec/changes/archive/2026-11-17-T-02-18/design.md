# Design

## Context

En `develop`, `domain` expone `Producto`, `NewProducto` (`stock_inicial`), `ProductRepo` (`create`, `get` sin adaptador), `MovimientoStock` / `TipoMovimiento::Inicial`, `OutboxEvent` (`OutboxEntity::Producto`) y `DomainError` (`duplicate_sku`, `invalid_input`, `not_found`). `storage` abre SurrealDB, aplica `0000`–`0006` (`producto`, `movimiento_stock`), y ofrece `Tx` + mapper UNIQUE `sku` → `duplicate_sku` + `OutboxNotifier`. Los adaptadores de catálogo (`CategoriaRepo`, `MarcaRepo`, `AtributoDefRepo`) viven en `storage` como `*Repo { db: Arc<Db> }`. No hay adaptador de `ProductRepo`. El puerto `StockLedger` no tiene implementación (T-02-24). Ver `proposal.md` para motivación y `specs/vortex-storage` para el contrato.

Constraints: sin HTTP ni `catalog` (T-02-19…T-02-21); sin `update`/`version_conflict` (T-02-19); ~2 h; `domain` no gana deps ni cambia la firma de `NewProducto`; SurrealDB `=2.7.0`; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Adaptador hexagonal de `ProductRepo::create` atómico: ficha + movimiento `inicial` condicional + proyección `producto.stock` + outbox
- `get` básico para devolver el snapshot persistido (necesario para el retorno de `create`)
- Tests que cierran el Listo cuando: stock 50 → `version = 1`, movimiento `inicial`, evento de outbox; unhappy `duplicate_sku`

**Non-Goals (diseño):**

- Implementar el puerto `StockLedger` (ingresos/ajustes reutilizan el mismo patrón en T-02-24)
- Añadir `terminal` a `NewProducto` (T-02-21 pasa `X-Vectra-Terminal`; el movimiento inicial de este ticket queda sin `terminal`)
- Ficha hidratada (nombre de marca, ruta de categoría, atributos con clave/etiqueta/unidad)
- Casos de uso, HTTP, UI

## Decisions

### 1. Adaptador `ProductoRepo` en `storage/src/producto.rs`

**Elección:** newtype `ProductoRepo { db: Arc<Db> }` que implementa `domain::ProductRepo`. `lib.rs` hace `mod producto` y `pub use producto::ProductoRepo`. Hidratar con DTO `serde` + strip de `producto:` / `marca:` / `categoria:` / `atributo_def:`. Escrituras vía `Tx::run` + `emit` + `commit`; luego `get` para timestamps del motor. Reusar `CategoriaRepo` / `MarcaRepo` / `AtributoDefRepo` solo en tests para sembrar FKs.

**Por qué:** mismo molde que T-02-08 / T-02-11. El adaptador pertenece a `storage`, no a `catalog`.

**Alternativa:** crate `catalog`. Se descartó: `catalog` es casos de uso (T-02-20). **Alternativa:** implementar `StockLedger` aquí y llamarlo desde `create`. Se descartó: T-02-24 es el dueño de `append`; mezclarlo hincharía este ticket.

### 2. Una `Tx`: CREATE con `stock` 0, ledger solo si `stock_inicial > 0`, outbox siempre

**Elección:**

```text
BEGIN
  CREATE producto:... CONTENT { ..., /* sin stock; DEFAULT 0 */ }
  -- si stock_inicial > 0:
  CREATE movimiento_stock:... CONTENT { tipo: 'inicial', cantidad, stock_resultante }
  UPDATE producto:... SET stock = $stock_inicial
  CREATE outbox ...  -- emit()
COMMIT
```

`stock` del producto es proyección del ledger ([4.4.4](../../docs/4-modelo.md#444-producto) / [4.5.2](../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción)): no se escribe en el `CONTENT` del alta. `version` queda en DEFAULT 1; el movimiento MUST NOT hacer `version += 1`. `stock_inicial == 0` no inserta movimiento (`cantidad != 0` / signo de `inicial`). `stock_inicial < 0` se rechaza en Rust (`invalid_input`) antes de abrir `Tx`. Comparación con `domain::Decimal`.

Outbox: un evento `producto`/`upsert` **siempre** (un producto con stock 0 también debe indexarse). `motivo = Some("stock")` si hubo movimiento inicial; `None` si no (mismo criterio de pista que T-02-24 / maestros).

**Por qué:** el ticket lista movimiento + stock + evento en la rama `initialStock > 0`, pero HU-02-04 exige que el alta llegue a búsqueda; el worker (T-03-07) consume outbox. Un evento cubre ficha y stock.

**Alternativa:** setear `stock` en el `CREATE`. Se descartó: rompe la regla «stock solo cambia por el ledger». **Alternativa:** dos eventos (`ficha` + `stock`). Se descartó: el worker reindexa el documento completo; el Listo cuando pide «el evento» (singular). **Alternativa:** no emitir outbox si stock es 0. Se descartó: productos sin stock no aparecerían en Meilisearch.

### 3. Movimiento `inicial` inline; no se usa `StockLedger`

**Elección:** SQL de `movimiento_stock` dentro de `ProductoRepo::create`. `terminal`, `motivo` y `venta` ausentes. `cantidad` y `stock_resultante` iguales a `stock_inicial`. T-02-24 generalizará el patrón a `append` (ingreso/ajuste, `THROW stock_negativo`, `motivo` de outbox `"stock"`).

**Por qué:** el ticket pide el movimiento como parte del alta, no el puerto del ledger. Reusar `StockLedger` ahora forzaría implementarlo a medias.

**Alternativa:** extraer un helper `append_inicial` compartido. Se descartó: YAGNI hasta T-02-24.

### 4. UNIQUE `sku` → `duplicate_sku` vía mapper existente

**Elección:** no tocar `tx.rs`. `Tx::commit` ya mapea UNIQUE + token `sku` a `ConflictCode::DuplicateSku` (T-03-02 / tests `tx_unique_sku_is_duplicate_sku`). El rollback de la transacción cubre producto + movimiento + outbox.

**Por qué:** el Listo cuando unhappy de HU-02-04 es `duplicate_sku`; el mapper ya existe.

**Alternativa:** pre-check `SELECT` por SKU. Se descartó: race y duplica el índice.

### 5. `get` básico con `Producto::try_new`; ficha resuelta queda en T-02-19

**Elección:** `get` lee la fila, convierte `record` → `Id<T>` sin prefijo, mapea `atributos[*].valor` (`string`/`number`/`bool` → `ValorAtributo`) y `codigos`. Si hay `atributos`, carga las `AtributoDef` referenciadas para `Producto::try_new` (el constructor las exige). No resuelve nombre de marca, `ruta` de categoría ni etiqueta/unidad del def.

**Por qué:** T-02-16 diseñó `try_new(draft, defs)` para hidratar desde Surreal. T-02-19 añade la ficha rica; este `get` solo cierra el retorno de `create` y el escenario «get de producto existente».

**Alternativa:** bypass del constructor. Se descartó: duplicaría invariantes. **Alternativa:** dejar `get` como `Unavailable`. Se descartó: el trait ya lo declara y `create` necesita releer timestamps.

### 6. Tests en `tests/producto.rs` (extender el archivo de T-02-14)

**Elección:** sembrar categoría (y marca si el caso la usa) con los repos existentes o SQL mínimo, igual que `producto_creates_minimal_sku`. Tests nuevos con prefijo `product_repo_`. No tocar conteo de migraciones.

**Por qué:** el archivo de esquema ya tiene el harness y el seed de `categoria`. Un segundo `producto_repo.rs` partiría el Listo cuando.

**Alternativa:** archivo nuevo. Se descartó: ruido; `atributo_def.rs` ya mezcla esquema + repo.

### 7. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-18` y rename a `T-02-18` (igual que tickets previos). `status` / `validate` aceptan el ID de proyecto.

## Risks / Trade-offs

- **[Ámbito ~2 h con ficha + ledger]** → Mitigación: un solo método `create`; `get` mínimo; sin `StockLedger`; tests del Listo cuando + zero-stock + duplicate_sku. No se divide el ticket.
- **[Deserializer de `atributos[*].valor` union string|number|bool]** → Mitigación: DTO intermedio `serde_json::Value` o enum untagged; el happy path del Listo cuando MAY omitir atributos (el escenario los marca opcionales).
- **[FK `record<categoria>` si el test no siembra]** → Mitigación: seed obligatorio en helpers; no se mapea a un `code` nuevo (T-02-20 es el dueño de «categoría inactiva»).
- **[T-02-21 necesitará `terminal` en el movimiento inicial]** → Mitigación: documentado; `NewProducto` no se cambia aquí. T-02-20/T-02-21 podrán extender el input o pasar el dato en el caso de uso.

## Migration Plan

Sin migración de esquema (0005/0006 ya están). Despliegue = merge a `develop`. Rollback = revertir el adaptador; las tablas no cambian.

## Open Questions

Ninguna que altere spec, enfoque o tasks.
