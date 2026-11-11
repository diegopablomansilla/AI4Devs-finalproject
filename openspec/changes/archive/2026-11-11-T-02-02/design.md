# Design

## Context

En `develop`, `domain` expone `DomainError` (incl. `category_cycle` / `invalid_input`), `Id<T>`, `Timestamp` y `OutboxEvent`. No hay entidades de catálogo ni puertos de repositorio. La tabla `categoria` ya existe (T-02-01). Allowlist de `domain`: `thiserror`, `ulid`, `rust_decimal`, `serde`; `serde_json` solo en dev. Edición 2024, rustc 1.99. Ver `proposal.md` para motivación y `specs/domain-categoria` para el contrato.

Constraints: sin persistencia (T-02-03); ~2 h; sin crates nuevas; `check-domain-deps.sh` sigue vetando SDKs.

## Goals / Non-Goals

**Goals:**

- API pública de `domain` para `Categoria`, *slug*, profundidad, ciclo y `CategoryRepo`
- Tests unitarios que cierran el Listo cuando (happy + unhappy)
- Trait compilable con un dummy en tests; cero I/O

**Non-Goals (diseño):**

- Adaptador SurrealDB, `storage::Tx`, cálculo persistido de `ancestros`/`ruta`
- `update` / mover / renombrar en el trait
- Repositorio en memoria de producción (T-02-05)
- `async-trait` o `tokio` en `domain`
- Object-safety de `CategoryRepo` (`dyn CategoryRepo`)

## Decisions

### 1. Un módulo `categoria.rs` + reexport

**Elección:** `vortex-core/crates/domain/src/categoria.rs` con entidad, `Slug`, profundidad, ciclo y el trait. `lib.rs` hace `pub mod categoria` y `pub use` de `Categoria`, `Slug`, `CategoryRepo`, `NewCategoria`, `MAX_DEPTH` y las funciones puras.

**Por qué:** primer agregado de negocio; un archivo mantiene el ticket en ~2 h. T-02-08 (`Marca`) puede copiar el patrón.

**Alternativa:** `categoria/{mod.rs,slug.rs,repo.rs}`. Se descartó: más archivos que tests.

### 2. `Slug` newtype + tabla de transliteración española

**Elección:** `Slug(String)` opaco. `from_nombre(&str)` aplica: trim, minúsculas, mapa `ÁÉÍÓÚÜÑáéíóúüñ` → `aeiouun`, resto no `[a-z0-9]` → `-`, colapsar/recortar guiones, luego el patrón `^[a-z0-9]+(-[a-z0-9]+)*$`. `FromStr` / constructor explícito solo acepta el patrón (sin transliterar: `"Tornillería"` falla). Errores → `DomainError::InvalidInput` con `FieldError.field` `nombre` (generación) o `slug` (explícito).

**Por qué:** el contrato OpenAPI ya fija el patrón; el comercio piloto es LATAM; una crate Unicode rompería la allowlist.

**Alternativa:** `deunicode` / `unicode-normalization`. Se descartó: deps nuevas. **Alternativa:** `String` suelta. Se descartó: T-02-03 y T-02-08 repetirían el patrón.

### 3. Profundidad = `ancestros.len() + 1`, máximo 4

**Elección:** constante `MAX_DEPTH = 4`. Raíz (`ancestros` vacío) = 1. Validar profundidad 5+ al construir `Categoria` con `invalid_input` y `field` `padre`. Función pura `depth(ancestros: &[Id<Categoria>]) -> usize`.

**Por qué:** 4.4.1 dice “máxima recomendada: 4”; Meilisearch tiene `lvl0`…`lvl3`. Contar por ancestros no requiere I/O.

**Alternativa:** raíz = 0. Se descartó: “4 niveles” del ticket y de las facetas son cuatro nombres en `ruta` (lvl0–lvl3).

### 4. Ciclo por cadena de ancestros del destino

**Elección:** `would_form_cycle(node, new_parent: Option<Id<Categoria>>, new_parent_ancestors: &[Id<Categoria>]) -> bool`. Verdadero si `new_parent == Some(node)` o `new_parent_ancestors` contiene `node`. `None` (raíz) = falso. `reject_cycle(...)` envuelve el `true` en `InvalidInput` + `category_cycle` (no `self_relation`).

**Por qué:** HU-02-01 mueve `"Fijaciones"` debajo de `"Pernos"`: el `id` de Fijaciones está en `ancestros` de Pernos. El caller (T-02-04) ya tiene `Categoria.ancestros` del destino. Función pura: sin grafo ni repo.

**Alternativa:** conjunto de descendientes. Se descartó: exige recorrer el árbol (persistencia). **Alternativa:** devolver solo `bool` sin `reject_cycle`. Se descartó: el Listo cuando pide el `code` `category_cycle`.

### 5. Constructor `Categoria::try_new` valida nombre, slug y profundidad; no el ciclo de movimiento

**Elección:** `try_new` recibe el snapshot completo (campos 4.4.1). Valida nombre (1..=100 tras trim), `Slug`, profundidad ≤ 4, y `ruta.len() == depth`. No recorre un grafo. El ciclo de *mover* se testea con las funciones del punto 4.

**Por qué:** una fila ya persistida no “se mueve” al hidratarla; T-02-03 hidratará desde SurrealDB. Mezclar ciclo de update en el constructor acoplaría T-02-04.

**Alternativa:** `try_new` exige `id ∉ ancestros`. Se deja como invariante opcional en apply si no hincha el ticket; no es el escenario de HU-02-01.

### 6. `CategoryRepo` con `async fn` nativo; métodos de T-02-03 solamente

**Elección:**

```rust
pub struct NewCategoria {
    pub nombre: String,
    pub slug: Option<Slug>,
    pub padre: Option<Id<Categoria>>,
}

pub trait CategoryRepo {
    fn create(&self, input: NewCategoria) -> impl Future<Output = Result<Categoria, DomainError>> + Send;
    fn get(&self, id: Id<Categoria>) -> impl Future<Output = Result<Categoria, DomainError>> + Send;
    fn list_all(&self) -> impl Future<Output = Result<Vec<Categoria>, DomainError>> + Send;
}
```

(o `async fn` equivalente en edición 2024). Sin `update`. Dummy en `#[cfg(test)]` que implementa el trait y devuelve `Unavailable` (o una `Categoria` fija) para el test de forma. T-02-03 implementa de verdad; T-02-04 amplía el trait.

**Por qué:** rustc 1.99 + edición 2024 evitan `async-trait`. Object-safety no hace falta: T-02-05 usará `R: CategoryRepo`. `NewCategoria` es el input de alta; calcular `ancestros`/`ruta` queda en el adaptador.

**Alternativa:** trait síncrono. Se descartó: el adaptador SurrealDB es async. **Alternativa:** incluir `update` ahora. Se descartó: T-02-03 tendría que stubbearlo.

### 7. Tests `#[cfg(test)]` en el mismo módulo

**Elección:** nombres de `specs/domain-categoria` (tabla Tests previstos). Sin tests de `storage`. `check-domain-deps.sh` y `check-workspace.sh` sin cambios.

**Por qué:** el Listo cuando es unitario. Un test de integración contra `0002` sería persistencia (T-02-03).

## Risks / Trade-offs

- **[Transliteración solo español]** → Mitigación: el MVP es ferretería LATAM; casos cubiertos (`Tornillería`, `Caños`). Otros scripts fallan como `invalid_input`.
- **[async fn en trait no es `dyn`-safe]** → Mitigación: consumidores genéricos; no se publica `Box<dyn CategoryRepo>` en este ticket.
- **[Dummy del trait no cubre create real]** → Mitigación: T-02-03; el dummy solo prueba que el puerto existe.
- **[UNIQUE de raíces con `padre` NONE]** → Fuera de alcance; T-02-01/T-02-03.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-02` y rename a `T-02-02` (igual que T-02-01).

## Migration Plan

1. Red: tests de slug, profundidad, ciclo y dummy del trait (no compilan).
2. Green: `categoria.rs` + reexports; `cargo test -p domain` en verde.
3. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` y `check-workspace.sh` en 0, `openspec validate T-02-02`.
4. Rollback: revertir el módulo y el `pub use`. Sin datos, sin API HTTP, sin migración SQL.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El detalle `async fn` vs `fn ... -> impl Future` queda al apply mientras el trait compile sin `async-trait` y sin `tokio` en `domain`.
