# Design

## Context

En `develop`, `domain` expone `Producto`, `NewProducto`, `ProductRepo` (`create`, `get` → `Producto`; **sin** `update`), `Marca`/`Categoria` (con `ruta`)/`AtributoDef` (`clave`, `etiqueta`, `unidad`) y `DomainError::Conflict` `version_conflict`. `storage::ProductoRepo` implementa `create` (Tx + movimiento `inicial` + outbox) y `get` básico (`SELECT` + `Producto::try_new` con defs; IDs de marca/categoría, sin nombre ni ruta). `Tx::commit` no devuelve filas afectadas; ya mapea UNIQUE `sku` → `duplicate_sku` y `THROW 'stock_negativo'`. Ver `proposal.md` para motivación y las deltas de `domain-producto` / `vortex-storage` para el contrato.

Constraints: sin HTTP ni `catalog` (T-02-20 / T-02-21); sin `activo` (T-02-22); sin imágenes (T-02-30); ~2 h; SurrealDB `=2.7.0`; MVP del curso.

## Goals / Non-Goals

**Goals:**

- `FichaProducto` en `domain` y `get` hidratado (marca, `ruta`, atributos con clave/etiqueta/unidad)
- `update` atómico con bloqueo optimista `WHERE version = $v` / `version += 1`; 0 filas → `version_conflict`
- Reemplazo entero de `atributos` y `codigos`; nunca escribir `stock`
- Tests que cierran el Listo cuando: dos `update` con la misma `version`

**Non-Goals (diseño):**

- Merge de PATCH parcial (T-02-20 carga la ficha, aplica el body y llama `update` con snapshot completo)
- `set_activo`, listado, `StockLedger`, HTTP, `openapi.yaml`
- Tipos `BrandRef`/`CategoryRef` en `domain` (son del contrato HTTP; se reutilizan `Marca` y `Categoria`)

## Decisions

### 1. `FichaProducto` en `domain`; `get`/`update` la devuelven; `create` sigue en `Producto`

**Elección:**

```rust
pub struct AtributoFicha {
    pub def: Id<AtributoDef>,
    pub clave: String,
    pub etiqueta: String,
    pub unidad: Option<String>,
    pub valor: ValorAtributo,
}

pub struct FichaProducto {
    pub producto: Producto,
    pub marca: Option<Marca>,
    pub categoria: Categoria,
    pub atributos: Vec<AtributoFicha>,
}

pub struct UpdateProducto { /* id, version, ficha; sin stock ni activo */ }
```

`ProductRepo::get` y `update` → `Result<FichaProducto, DomainError>`. `create` no cambia. `lib.rs` reexporta los tipos nuevos. Dummy de tests gana `update`.

**Por qué:** el ticket pide `get` con marca, ruta y atributos resueltos; OpenAPI `Product.brand`/`category`/`attributes.{key,label,unit}` se mapea en T-02-21 desde estas entidades. Meter `nombre` de marca dentro de `Producto` rompería el modelo 4.4.4 (FK `record<marca>`).

**Alternativa:** hidratar solo en `storage` y dejar `get` → `Producto`. Se descartó: T-02-20 no tendría de dónde sacar ruta/clave. **Alternativa:** `MarcaRef`/`CategoriaRef` en `domain`. Se descartó: duplican `Marca`/`Categoria` y mezclan nombres de API.

### 2. Hidratar `get` reusando `MarcaRepo` / `CategoriaRepo` / `AtributoDefRepo`

**Elección:** `fetch_one` interno sigue armando `Producto`. `get` luego: `CategoriaRepo::get` (ya trae `ruta`); `MarcaRepo::get` si hay FK; zip de `atributos` con las defs que `load_atributo_defs` ya carga para `try_new`. MUST NOT `SELECT` `imagen_producto`.

**Por qué:** T-02-18 ya carga defs; categoría y marca tienen adaptadores. Evita SQL de grafo extra y mantiene strip de `categoria:` / `marca:`.

**Alternativa:** un `SELECT` Surreal con `FETCH marca, categoria, atributos.*.def`. Se descartó: el deserializado se vuelve un DTO paralelo y el ticket cabe en ~2 h con los repos existentes.

### 3. `UpdateProducto` es snapshot de ficha, no PATCH

**Elección:** el repo recibe todos los campos de ficha + `version` esperada. Arrays `atributos` y `codigos` se escriben completos (`SET atributos = $arr`). T-02-20 hará el merge del body OpenAPI (`minProperties: 2`).

**Por qué:** mismo molde que `UpdateMarca` / `UpdateCategoria`. El ticket nombra el reemplazo entero de arrays; el PATCH HTTP no es este crate.

**Alternativa:** `Option<T>` por campo en el repo. Se descartó: `None` vs «borrar descripción» se pelea con OpenAPI y duplica lógica de T-02-20.

### 4. `THROW 'version_conflict'` en la misma `Tx` que el `UPDATE` y el outbox

**Elección:**

```text
BEGIN
  LET $r = UPDATE producto:… SET …, version += 1 WHERE version = $v RETURN AFTER;
  IF array::len($r) = 0 { THROW 'version_conflict' };
  CREATE outbox …  -- emit(), motivo ausente
COMMIT
```

Si el `id` no existe, `SELECT` previo → `not_found` **antes** de abrir `Tx` (como `MarcaRepo::update`). Mapper en `tx.rs`: token `version_conflict` → `DomainError::conflict(ConflictCode::VersionConflict)`, igual que `stock_negativo`. UNIQUE `sku` sigue en el mapper existente. `SET` omite `stock` y `activo`. Sintaxis exacta de `LET`/`THROW` se ajusta contra Surreal 2.7.0 en apply.

**Por qué:** `Tx::commit` no expone filas afectadas. Si se emite outbox y el `UPDATE` no toca filas, el worker reindexaría basura. `THROW` aborta la transacción (outbox incluido). Convención [4.3](../../docs/4-modelo.md#43-convenciones).

**Alternativa:** `UPDATE` fuera de `Tx` y outbox después. Se descartó: no es atómico. **Alternativa:** comparar `version` en Rust tras commit. Se descartó: el evento de outbox ya se habría escrito.

### 5. Listo cuando = dos `update` con la misma `version`

**Elección:** test `product_repo_concurrent_updates_same_version_conflict`. Preferir `tokio::join!` de dos `update` con `version` 1; si el motor embebidо serializa, el segundo ve `version` 2 y sigue siendo el escenario. Un fallback secuencial (primer `update` ok, segundo con `version` 1 → `version_conflict`) cubre el Listo cuando: es exactamente el conflicto de HU-02-05 / 5.5.

**Por qué:** el ticket dice «ediciones concurrentes»; el mecanismo es bloqueo **optimista**, no un lock de fila. Lo observable es «una pasa, la otra `version_conflict`».

**Alternativa:** exigir interleaving real. Se descartó: frágil en Surreal embedded y no cambia el contrato.

### 6. Tests en `tests/producto.rs`; seed de marca, categoría hija y `AtributoDef`

**Elección:** extender el archivo de T-02-14/T-02-18. Helpers de seed con `CategoriaRepo` (padre + hija para `ruta`), `MarcaRepo`, `AtributoDefRepo`. Prefijo `product_repo_`. Actualizar `product_repo_get_returns_persisted` al nuevo retorno `FichaProducto` (campos de `producto` + hidratación mínima). No tocar conteo de migraciones.

**Por qué:** el harness y el Listo cuando de T-02-18 viven ahí; un segundo archivo partiría get/update.

**Alternativa:** `producto_ficha.rs`. Se descartó: ruido.

### 7. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-19` y rename a `T-02-19` (igual que T-02-18).

## Risks / Trade-offs

- **[`Tx` no reporta filas afectadas]** → Mitigación: `THROW 'version_conflict'` dentro de la transacción; mapper en `tx.rs`. Ajustar SQL a 2.7.0 en apply.
- **[Cambio de firma de `get` rompe tests de T-02-18]** → Mitigación: actualizar `product_repo_get_returns_persisted` / dummy en el mismo change (working tree; commit solo post-archive).
- **[Ámbito ~2 h con hidratación + update]** → Mitigación: reusar repos de marca/categoría/def; sin PATCH; sin imágenes; tests del Listo cuando + stock intacto + `not_found`. No se divide el ticket.
- **[FK huérfana de categoría]** → Mitigación: el esquema `record<categoria>` lo impide en el happy path; si `CategoriaRepo::get` falla, se propaga `not_found`/`unavailable` existente.

## Migration Plan

Sin migración de esquema (`0005` ya tiene `version`). Despliegue = merge a `develop`. Rollback = revertir adaptador + tipos de `domain`; las tablas no cambian.

## Open Questions

Ninguna que altere spec, enfoque o tasks.
