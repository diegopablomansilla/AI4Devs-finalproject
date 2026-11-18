# Design

## Context

En `develop`, `domain` expone `Categoria`, `Marca`, `AtributoDef`/`TipoAtributo`, `CategoryRepo`, `BrandRepo`, `AttributeDefRepo`, `DomainError` (incl. `invalid_input` y `duplicate_sku`), `Id<T>`, `Decimal` y `Timestamp`. `storage` ya tiene la tabla `producto` (T-02-14) y `movimiento_stock` (T-02-15). No hay entidad `Producto`, value objects de ficha ni puerto `ProductRepo`. Allowlist de `domain`: `thiserror`, `ulid`, `rust_decimal`, `serde`; `serde_json` solo en dev. Edición 2024, rustc 1.99. Ver `proposal.md` para motivación y `specs/domain-producto` para el contrato.

Constraints: sin persistencia (T-02-18); ~2 h; sin crates nuevas; `check-domain-deps.sh` sigue vetando SDKs. MVP del curso.

## Goals / Non-Goals

**Goals:**

- API pública de `domain` para `Producto`, VOs, validación de atributos y `ProductRepo`
- Tests unitarios que cierran el Listo cuando (`"cuarenta"` en `numero` y `def` duplicado)
- Trait compilable con un dummy en tests; cero I/O

**Non-Goals (diseño):**

- Adaptador SurrealDB, `storage::Tx`, UNIQUE → `duplicate_sku`, movimiento `inicial`, outbox
- `update` / `set_activo` / listado en el trait
- Repositorio en memoria de producción (T-02-20)
- `async-trait` o `tokio` en `domain`
- Object-safety de `ProductRepo` (`dyn ProductRepo`)
- Ampliar `DomainError` o `AtributoDef`

## Decisions

### 1. Un módulo `producto.rs` + reexport

**Elección:** `vortex-core/crates/domain/src/producto.rs` con entidad, VOs, validación de atributos y el trait. `lib.rs` hace `pub mod producto` y `pub use` de `Producto`, `ProductoDraft`, `Sku`, `Codigo`, `TipoCodigo`, `Ubicacion`, `UnidadMedida`, `ValorAtributo`, `AtributoValor`, `NewProducto` y `ProductRepo`.

**Por qué:** mismo molde que `categoria.rs` / `marca.rs` / `atributo_def.rs`; un archivo mantiene el ticket en ~2 h.

**Alternativa:** `producto/{mod.rs,sku.rs,repo.rs}`. Se descartó: más archivos que tests.

### 2. Campos y enums en español; wire API en T-02-21

**Elección:** entidad y VOs usan nombres de [4.4.4](../../docs/4-modelo.md#444-producto): `unidad_medida` = `unidad`/`par`/`caja`/`metro`/`kilogramo`/`litro`; `Codigo.tipo` = `fabricante`/`oem`/`barras`. `parse_wire` de esos enums acepta **solo** valores del modelo. `"unit"` o `"manufacturer"` fallan en dominio.

**Por qué:** `Categoria`/`Marca`/`AtributoDef` ya guardan español; [5.3](../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) traduce en el gateway (T-02-21).

**Alternativa:** aceptar inglés en `parse_wire`. Se descartó: duplicaría el mapa 5.3 y mezclaría capas.

### 3. `ValorAtributo` tipado; `"cuarenta"` es texto frente a `numero`

**Elección:**

```rust
pub enum ValorAtributo {
    Texto(String),
    Numero(Decimal),
    Booleano(bool),
}
```

El Listo cuando se cubre construyendo un `AtributoValor` con `ValorAtributo::Texto("cuarenta")` contra una `AtributoDef` de `TipoAtributo::Numero`. Reutiliza `Decimal` existente (máx. 4 decimales del contrato). No se añade `serde_json::Value`.

**Por qué:** el modelo 4.4.4 es `string | number | bool`; un enum impide el valor ilegible. `"cuarenta"` no parsea como `Decimal`, así que llega como texto y choca con `numero`.

**Alternativa:** todo `String` y parsear según el def. Se descartó: pierde el bool/número del modelo y empuja parseo a cada caller. **Alternativa:** `f64`. Se descartó: ya hay `Decimal` en `domain`.

### 4. Rutas de `FieldError` de atributos en inglés del contrato

**Elección:** mismatch de tipo/valor → `attributes[n].value`; `def` duplicado o desconocido → `attributes[n].definitionId`. El resto de campos de ficha usan nombres del modelo (`sku`, `nombre`, `precio`, `codigos[n].valor`), igual que `Marca.nombre`.

**Por qué:** HU-02-04 y el ticket nombran `attributes[n].value`; `domain-error` ya ejemplifica `attributes[1].value`. T-02-21 reenvía `FieldError.field` a `problem+json` sin remapear esas rutas.

**Alternativa:** `atributos[n].valor`. Se descartó: el Listo cuando / HU no coincidirían con OpenAPI.

### 5. Constructor `Producto::try_new(draft, defs)` valida ficha y atributos

**Elección:** `ProductoDraft` trae el snapshot completo (campos 4.4.4). `try_new` recorta/valida sku, nombre, tipo, descripción, precio ≥ 0, stock ≥ 0, version ≥ 1, y llama a una función pura `validate_atributos(&atributos, defs) -> Result<(), DomainError>`. No consulta repos. No exige que `categoria`/`marca` existan o estén activas.

**Por qué:** T-02-18 hidrata desde Surreal; T-02-20 carga `AtributoDef` y verifica maestros. Mezclar I/O en el constructor rompería el No hace (persistencia) y el alcance de T-02-20.

**Alternativa:** validar atributos solo en `catalog`. Se descartó: el Listo cuando es unitario de dominio.

### 6. `ProductRepo` con `create` y `get`; sin `update` ni listado

**Elección:**

```rust
pub struct NewProducto {
    pub sku: String,
    pub nombre: String,
    pub tipo: String,
    pub descripcion: Option<String>,
    pub marca: Option<Id<Marca>>,
    pub categoria: Id<Categoria>,
    pub atributos: Vec<AtributoValor>,
    pub codigos: Vec<Codigo>,
    pub unidad_medida: UnidadMedida,
    pub precio: Decimal,
    pub ubicacion: Option<Ubicacion>,
    pub stock_inicial: Decimal,
}

pub trait ProductRepo {
    fn create(&self, input: NewProducto) -> impl Future<Output = Result<Producto, DomainError>> + Send;
    fn get(&self, id: Id<Producto>) -> impl Future<Output = Result<Producto, DomainError>> + Send;
}
```

Dummy en `#[cfg(test)]` que implementa el trait y devuelve `Unavailable` (o un `Producto` fijo). T-02-18 implementa `create` de verdad; T-02-19 añade `update` y hidrata `get`; T-02-22 `set_activo`; T-02-27 el listado paginado. `stock_inicial` viaja en `NewProducto` para no cambiar la firma en T-02-18; este ticket no lo aplica.

**Por qué:** mismo recorte que `CategoryRepo` en T-02-02 (`create`/`get`/`list_all` para el ticket de persistencia inmediato). Productos no son maestro pequeño: no hay `list_all`. rustc 1.99 + edición 2024 evitan `async-trait`.

**Alternativa:** trait síncrono. Se descartó: el adaptador SurrealDB es async. **Alternativa:** incluir `update` ahora. Se descartó: T-02-18 tendría que stubbearlo.

### 7. `barras` permitido en dominio; unicidad de OEM no se exige

**Elección:** `TipoCodigo::Barras` forma parte del enum (el modelo lo incluye). Unicidad de `Codigo.valor` / OEM no se valida en dominio (4.4.4: varios productos MAY compartir OEM). Índice `idx_producto_codigos` es de persistencia (T-02-14, ya hecho).

**Por qué:** el ticket no pide rechazar `barras`; el lector es post-MVP (T-02-21/v2), no el value object.

## Risks / Trade-offs

- **[Ámbito ~2 h con varios VOs]** → Mitigación: un solo módulo; tests del Listo cuando + un happy de ficha + VOs mínimos; no se divide el ticket.
- **[`Decimal` para valores de atributo `numero`]** → Mitigación: 4 decimales cubren medidas típicas del piloto; T-02-18 mapea el `number` de Surreal a `Decimal`.
- **[Rutas de error mixtas EN/ES]** → Mitigación: documentado en la spec; solo `attributes[n].*` usa el contrato porque HU-02-04 lo nombra.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-16` y rename a `T-02-16` (igual que tickets previos).

## Migration Plan

1. Red: tests unitarios de VOs, `Producto` y `ProductRepo` (no compilan / fallan).
2. Green: `producto.rs` + reexports; dummy del trait.
3. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` en 0, `openspec validate T-02-16`.
4. Rollback: quitar el módulo y los reexports. Sin migración de datos ni API HTTP.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El detalle de helpers de test (`AtributoDef` de fixture) se copia del estilo de `atributo_def.rs` en apply.
