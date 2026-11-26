# Design

## Context

En `develop`, `domain` expone `Categoria`, `NewCategoria`, `CategoryRepo` (`create`/`get`/`list_all`, sin `update`), `Slug`, `MAX_DEPTH` y `OutboxEvent`. `storage` abre SurrealDB, aplica `0002_categoria.surql` (UNIQUE `(padre, slug)`), y ofrece `Tx` + mapper UNIQUE/`slug` → `duplicate_slug` + `OutboxNotifier`. `Tx::run` acumula SQL sin bindings de negocio (solo el outbox se bindea). `Db::query` no devuelve filas; las lecturas usan `inner.query` + `take`. Allowlist de `storage`: `surrealdb`, `thiserror`, `sha2`, `domain`, `tokio`. Ver `proposal.md` para motivación y `specs/vortex-storage` para el contrato.

Constraints: sin `update` (T-02-04); ~2 h; `domain` no gana deps; SurrealDB `=2.7.0`.

## Goals / Non-Goals

**Goals:**

- Primer adaptador hexagonal de catálogo: `storage` implementa `CategoryRepo`
- `create` materializa `ancestros`/`ruta` y escribe outbox en la misma `Tx`
- Tests de integración que cierran el Listo cuando (happy + unhappy)

**Non-Goals (diseño):**

- Ampliar el trait con `update` / mover / renombrar
- Casos de uso en `catalog`, HTTP, worker
- Cambiar la migración `0002` o la taxonomía de `DomainError`
- Repositorio en memoria

## Decisions

### 1. Struct `CategoriaRepo<'a>` en `storage/src/categoria.rs`

**Elección:** newtype sobre `&Db` que implementa `domain::CategoryRepo`. `lib.rs` hace `mod categoria` y `pub use categoria::CategoriaRepo`. Tests en `crates/storage/tests/categoria.rs` (se suman a los de T-02-01).

**Por qué:** un archivo, mismo crate que `Tx` y la tabla; T-02-08 (`Marca`) puede copiar el patrón. El lifetime evita `Arc<Db>` en este ticket.

**Alternativa:** crate `catalog` con el adaptador. Se descartó: `catalog` es casos de uso (T-02-05); el adaptador de persistencia pertenece a `storage` (diagrama STO → DOM).

### 2. `create`: calcular árbol en Rust, persistir con `Tx`, hidratar con `get`

**Elección:**

1. *Slug*: `input.slug` o `Slug::from_nombre(&input.nombre)`.
2. Si `padre` es `Some`, `get(padre)`; ausencia → `NotFound`. `ancestros = padre.ancestros + padre.id`; `ruta = padre.ruta + nombre`. Profundidad `padre.depth() + 1 > MAX_DEPTH` → `invalid_input`.
3. Raíz: `ancestros = []`, `ruta = [nombre]`.
4. `id = Id::<Categoria>::generate()`.
5. `Tx::run(CREATE type::thing('categoria', id) CONTENT { … })` + `emit(OutboxEvent { entidad: categoria, operacion: upsert, registro: id, motivo: None })` + `commit`.
6. `get(id)` para devolver timestamps del motor (`creado_en` DEFAULT, `actualizado_en` VALUE).

**Por qué:** `Tx::commit` no devuelve filas; el motor es la fuente de `datetime`. Calcular `ancestros`/`ruta` en Rust cumple el ticket y reutiliza el padre ya leído (sin grafo en SurrealQL).

**Alternativa:** un `SELECT` dentro de la misma `Tx`. Se descartó: `Tx` no expone resultados. **Alternativa:** `motivo: Some("ficha")`. Se descartó: 4.8 reserva `motivo` como pista de producto; `None` basta para *fan-out* por `entidad`.

### 3. SQL de escritura: ULID interpolados; nombres escapados

**Elección:** interpolar `id`/`slug`/`padre` (Crockford ULID o patrón de *slug*, sin comillas peligrosas) y escapar `nombre`/`ruta` como strings SurrealQL. `padre` raíz = `NONE`; hija = `type::thing('categoria', padre_id)`. `ancestros` = array de `type::thing`. No se toca la API de `Tx` salvo que apply no pueda expresar arrays de records de forma fiable.

**Por qué:** `Tx::run` hoy es `&str`; T-03-02 ya interpola en tests. Evita crate `serde` si las lecturas pueden hidratarse sin un DTO derivado.

**Alternativa:** `Tx::bind` genérico. Queda como plan B si el dialecto 2.7 rechaza el `CREATE` interpolado; no se cambia la spec.

### 4. Lecturas: strip de `categoria:` y `Categoria::try_new`

**Elección:** `SELECT` por `type::thing('categoria', $id)` / `SELECT * FROM categoria`. El `id` del motor llega como `categoria:<ulid>`; el adaptador MUST recortar el prefijo de tabla antes de `Id::from_str` (T-00-04 rechaza `:`). `padre` y `ancestros` igual. `datetime` del motor se normaliza a RFC 3339 con `Z` para `Timestamp`. Fila ausente → `NotFound`. Hidratar con `CategoriaDraft` + `try_new`.

**Por qué:** el dominio no conoce records Surreal. Un DTO interno (privado al módulo) aísla el mapeo.

**serde:** si `Response::take` exige `Deserialize`, se declara `serde` (workspace, `derive`) en `[dependencies]` de `storage` y se actualiza `storage_deps_allowed` + fixture happy. Si se puede `take` de primitivas (`String`, `Vec<String>`, `bool`) sin crate nueva, no se toca la allowlist.

**Alternativa:** persistir ULIDs como string sin `record<categoria>`. Se descartó: el esquema 4.4.1 ya es `option<record<categoria>>`.

### 5. UNIQUE de raíces (`padre` NONE) y mapper `duplicate_slug`

**Elección:** el alta pasa por `Tx::commit`, que ya mapea UNIQUE + `slug` → `duplicate_slug` (índice `uq_categoria_slug`). El test unhappy de raíces es obligatorio. Si SurrealDB 2.7 trata `NONE` como distinto en UNIQUE (T-02-01, decisión 4), `create` MUST hacer un `SELECT` previo de raíces con ese *slug* y devolver `duplicate_slug` sin insertar. Si el índice ya rechaza, no hace falta el `SELECT`.

**Por qué:** HU-02-01 / spec exigen `duplicate_slug` también entre raíces. No se cambia el índice.

**Alternativa:** índice extra solo para raíces. Se descartó: fuera de T-02-01 y de ~2 h.

### 6. Tests de integración en el archivo existente

**Elección:** añadir los tests de la tabla «Tests previstos» en `tests/categoria.rs` usando `CategoriaRepo` + `test_db()`. Los tests de esquema de T-02-01 se conservan. Helpers de alta (`create` raíz `"Fijaciones"`, luego hija `"Pernos"`) para el Listo cuando.

**Por qué:** un `[[test]]` ya cubre la tabla; CI `cargo test --workspace` los ejecuta. Sin HTTP.

## Risks / Trade-offs

- **[UNIQUE con `padre` NONE]** → Mitigación: decisión 5; el test `category_repo_rejects_duplicate_root_slug` decide si hace falta el `SELECT` previo.
- **[Mensaje UNIQUE sin la palabra `slug`]** → Mitigación: si el mapper devuelve `conflict` genérico, endurecer `map_surreal_message` con el SQL acumulado (ya se hace para `stock_negativo`) sin ampliar `DomainError`.
- **[Datetime del motor ≠ RFC 3339 `Z`]** → Mitigación: normalizar en el DTO; si falla un test de hidratación, ajustar el mapper, no `Timestamp`.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-03` y rename a `T-02-03` (igual que T-02-01 / T-02-02).
- **[Ámbito ~2 h]** → Mitigación: un módulo, sin `update`, sin bind genérico salvo plan B; no se divide el ticket.

## Migration Plan

Ninguna migración `.surql` nueva. `create` escribe filas de negocio por primera vez; bases ya migradas a `0002` siguen. Rollback de código: quitar `CategoriaRepo` y los tests nuevos. Sin API HTTP. Cierre: § 5 en `vortex-core/`, `openspec validate T-02-03`.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El detalle `serde` vs `take` primitivo y el `SELECT` previo de raíces UNIQUE se resuelven en apply según el dialecto 2.7.
