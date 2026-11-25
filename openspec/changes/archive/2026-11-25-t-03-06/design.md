# Design

## Context

`SearchDocument` solo tiene `id` (T-03-03). `MeiliTextSearch` mapea a `{ id }` (T-03-04); **no** se cambia en este ticket. Tipos de catálogo (`Producto`, `Marca`, `Categoria`, `AtributoDef`) viven en `domain`. Snapshot del Listo cuando usa el ejemplo [4.11.1](../../../docs/4-modelo.md#4111-documento) / producto [4.4.4](../../../docs/4-modelo.md#444-producto).

## Goals / Non-Goals

**Goals:** builder puro, `SearchDocument` serializable a JSON Meilisearch, tests stable names, `try_new(id)` compatible.

**Non-Goals:** HTTP, `meilisearch-sdk` en domain, imagen, wiring worker.

## Decisions

### 1. Builder en `domain::search_document`

**Elección:** `pub fn build_search_document(producto: &Producto, marca: Option<&Marca>, categoria: &Categoria, defs: &[AtributoDef]) -> Result<SearchDocument, DomainError>`.

**Validación:** `producto.categoria == categoria.id`; `producto.marca` consistente con `marca` (Some/Some mismo id, None/None); cada `atributos[*].def` presente en `defs`.

### 2. `categoria.lvlN` desde `categoria.ruta`

`lvl{i}` = `ruta[..=i].join(" > ")`, hasta `min(ruta.len(), 4)` (alineado `MAX_DEPTH`).

### 3. `categoria_ids`

Orden: `categoria.ancestros` (raíz → padre) + `categoria.id` como ULID string sin prefijo.

### 4. `attr_*` en `BTreeMap<String, String>`

Claves ya con prefijo `attr_`. Snapshot/plano JSON con las mismas claves que 4.11.1.

### 5. `ubicacion`

Partes presentes: `pasillo` → `P{pasillo}`, `estante` → `E-{estante}`, `posicion` tal cual; unir con ` · `. Si ninguna parte → `None` (JSON `null`).

### 6. `precio` en JSON

`f64` desde `Decimal` (wire del snapshot `120.5`). Struct interno puede guardar `Decimal`; `Serialize` emite número.

### 7. Tests

Fixture canónico en `search_document.rs` tests; ULIDs fijos del ejemplo 4.11.1. Sin `insta`; `serde_json::json!` comparado con `to_value`.

## Risks / Trade-offs

- [Adaptador sigue mandando solo `id`] → Mitigation: explícito en non-goals; T-03-07 extiende `MeiliDocument`.
- [`SearchDocument` más grande rompe PartialEq en tests viejos] → Mitigation: `try_new` rellena defaults; tests existentes solo comparan `id`.