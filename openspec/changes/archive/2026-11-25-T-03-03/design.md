# Design

## Context

En `develop`, `domain` ya tiene `DomainError` (T-00-03), `Id<T>` (T-00-04), `ComponentHealth` (T-00-13) y puertos hexagonales con RPITIT (`ProductRepo`, `CategoryRepo`, …): `fn … -> impl Future<Output = Result<…, DomainError>> + Send`. El crate `search` sigue vacío salvo el probe `#[ignore = "meilisearch"]` de T-09-05. `scripts/check-domain-deps.sh` prohíbe `meilisearch-sdk` y `reqwest` en el árbol `normal,build` de `domain`; `check-workspace.sh` solo permite `thiserror|ulid|rust_decimal|serde` en `[dependencies]` de `domain` (`serde_json` solo en dev). Ver `proposal.md` para motivación y `specs/domain-text-search/spec.md` para el contrato.

Constraints: sin adaptador (T-03-04); sin 4.11.2 (T-03-05); sin documento 4.11.1 (T-03-06); MVP del curso; ~2 h.

## Goals / Non-Goals

**Goals:**

- Módulo `domain::text_search` con el *trait* y los tipos del ticket, reexportados en `lib.rs`
- Validación de `SearchQuery` / índice / id de documento en constructores (`invalid_input`)
- Dummy de tests que cubre happy (`search` → `SearchPage`, `health` → `up`) y unhappy (`Unavailable`, `Timeout`, `InvalidInput`)
- Cero dependencias nuevas

**Non-Goals (diseño):**

- `async-trait` / `dyn TextSearch` object-safe
- `serde_json::Value` como documento (rompería la allowlist)
- Campos 4.11.1 en `SearchHit` / `SearchDocument` (T-03-06 los agrega o introduce el builder)
- Traducir filtros a sintaxis Meilisearch (T-03-12 / T-03-04)

## Decisions

### 1. El puerto vive en `domain`, no en el crate `search`

**Elección:** `vortex-core/crates/domain/src/text_search.rs` + `pub mod text_search` y `pub use` en `lib.rs`. El crate `search` no se toca.

**Por qué:** [3.3](../../../docs/3-componentes.md#33-puertos-y-adaptadores) y el ticket ponen el *trait* en `domain`. T-03-04 implementará el adaptador en `search`. Mover el puerto al crate del SDK rompería T-00-02.

**Alternativa:** definir el *trait* en `search`. Se descartó: los casos de uso (T-03-12) y el worker (T-03-07) dependerían del crate de infraestructura.

### 2. RPITIT como el resto de puertos, no `async-trait`

**Elección:** misma firma que `ProductRepo`:

```rust
pub trait TextSearch {
    fn search(
        &self,
        index: &IndexUid,
        query: SearchQuery,
    ) -> impl Future<Output = Result<SearchPage, DomainError>> + Send;
    // …resto de métodos igual
}
```

No `Send + Sync` en el *trait* (tampoco lo tiene `ProductRepo`; `HealthReport` sí porque es síncrono).

**Por qué:** un solo estilo de puerto; `domain` no declara `async-trait` ni `tokio`. El dummy usa `async { … }`.

**Alternativa:** `async-trait` + `dyn TextSearch`. Se descartó: nueva dep y allowlist. T-03-04 puede envolver en `Arc<impl TextSearch>` desde `bin/vortex`.

### 3. Validar `SearchQuery` al construir, con límites de OpenAPI y nombres en español

**Elección:** `SearchQuery::try_new` (o equivalente) rechaza `q.len() > 200`, `offset > 1000`, `limit` fuera de 1..=50. `SearchOrden` es un enum (`Relevancia`, `PrecioAsc`, `PrecioDesc`, `StockDesc`); parsear `"relevance"` / `"price_asc"` falla. `q` vacía es válida (explorar con filtros). Campos: `q`, `filtros`, `orden`, `offset`, `limit`.

`SearchFilters`: `categoria_ids: Vec<Id<Categoria>>`, `marca_id: Option<Id<Marca>>`, `con_stock: Option<bool>`, `atributos: BTreeMap<String, String>` con claves = `AtributoDef.clave` (sin `attr_`). El adaptador/caso de uso (T-03-04 / T-03-12) antepone `attr_`.

**Por qué:** T-03-12 ya fija esos techos y filtros; validarlos en dominio evita que un dummy o un adaptador reciba basura. [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) manda español en el modelo. El prefijo `attr_` es detalle de Meilisearch (4.11.1), no del puerto.

**Alternativa:** structs públicos sin `try_new`. Se descartó: el unhappy path del ticket quedaría solo en el dummy. **Alternativa:** nombres OpenAPI (`inStock`, `price_asc`). Se descartó: rompe 5.3.

### 4. `SearchDocument` / `SearchHit` solo con `id`; `IndexSettings` vacío

**Elección:**

```rust
pub struct SearchDocument { /* id: ULID sin ':' */ }
pub struct SearchHit { pub id: String /* o Id<Producto> vía Display */ }
pub struct IndexSettings; // unidad; T-03-05 le suma searchable/filterable/ranking
pub struct IndexUid(/* NonEmpty string */);
```

`upsert_documents(index, &[SearchDocument])`. T-03-06 MAY agregar campos a `SearchDocument` (documento 4.11.1 + `marca_id` / `categoria_ids`); este ticket no los inventa.

`SearchPage.facetas: Option<()>` o un tipo vacío `SearchFacets` sin campos — ausente en el dummy. T-03-18 rellena el tipo.

`swap_indexes` valida `origen != destino` **antes** de llamar al motor (función de dominio o early-return del dummy / helper `IndexUid::swap_pair`).

**Por qué:** el ticket no nombra `SearchDocument` ni `IndexSettings`; hacen falta para firmar el *trait* sin `serde_json`. Completar 4.11.1/4.11.2 aquí duplicaría T-03-06 / T-03-05 y pasaría de ~2 h.

**Alternativa:** `serde_json::Value` como documento. Se descartó: `serde_json` no está en la allowlist de `[dependencies]` de `domain`. **Alternativa:** associated type `Document`. Se descartó: complica el dummy y T-03-12 sin ganancia en este ticket.

### 5. `health()` reutiliza `ComponentHealth`; errores reutilizan `DomainError`

**Elección:** `health() -> Result<ComponentHealth, DomainError>`. El dummy happy devuelve `ComponentHealth::new(ComponentStatus::Up)`. Unhappy de motor: `DomainError::unavailable(UnavailableCode::Unavailable)` y `DomainError::Timeout`. Documento inválido: `DomainError::invalid_input(InvalidInputCode::InvalidInput, …)`. No se agregan `code` (`meilisearch_unavailable`, etc.).

**Por qué:** [3.3](../../../docs/3-componentes.md#33-puertos-y-adaptadores) pide `health()` en cada adaptador; T-00-13 ya modela `meilisearch` en `/health`. T-03-04 mapea conexión → `Unavailable`, timeout → `Timeout`. Ampliar T-00-03 está fuera de alcance.

**Alternativa:** `health() -> Result<(), DomainError>`. Se descartó: pierde el `ComponentStatus` que `/health` ya usa. **Alternativa:** devolver `HealthSnapshot`. Se descartó: eso es el agregado de T-00-13, no un sidecar.

### 6. Dummy y tests en `#[cfg(test)]` del módulo

**Elección:** `DummyTextSearch` (o dos variantes: ok / fail) en `text_search.rs` bajo `#[cfg(test)]`, igual que `DummyProductRepo`. Un test por fila de la tabla en la spec. `scripts/check-domain-deps.sh` en la tarea de cierre; el unhappy `meilisearch-sdk` lo cubren las fixtures existentes de T-00-02 (no se duplican).

**Por qué:** Listo cuando = compila + `domain` sin SDKs. No hay frontera HTTP ni motor real (T-09-05 / T-03-04).

**Alternativa:** `crates/domain/tests/text_search.rs`. Innecesario: no hay I/O.

## Risks / Trade-offs

- [T-03-04/T-03-06 tendrán que extender `SearchDocument`] → Mitigation: el delta lo declara; el cambio es aditivo de campos, no un puerto distinto. Si T-03-06 prefiere un tipo nuevo, el *trait* se ajusta en ese ticket (un método).
- [`facet_search` en MVP sin HU-03-07] → Mitigation: el ticket lo pide en el *trait*; HTTP/UI de facetas sigue en T-03-18 (v2). El dummy no calcula facetas.
- [Límites 200/1000/50 en dominio vs Gateway] → Mitigation: T-03-13 / OpenAPI vuelven a validar en HTTP; duplicar el techo en `try_new` es barato y habilita tests del puerto sin `api`.

## Migration Plan

Cambio aditivo en `domain`. No hay datos que migrar ni rollback. `cargo test -p domain` y `scripts/check-domain-deps.sh` bastan para revertir si hace falta.
