# Design

## Context

En `develop`, `domain::IndexSettings` es un placeholder `Copy` vacío y `MeiliTextSearch::apply_settings` es `Ok(())` sin HTTP (T-03-04). El puerto ya acepta `apply_settings(index, IndexSettings)`. `meilisearch-sdk` `=0.33.0` expone `Index::set_settings` (PATCH) y `get_settings`; `filterable_attributes` es `Vec<FilterableAttribute>` (string o objeto). `attributesToHighlight` **no** es setting de índice: es parámetro de consulta (T-03-12). Ver `proposal.md` para motivación y `specs/vortex-search/spec.md` / `specs/domain-text-search/spec.md` para el contrato.

Constraints: sin `synonyms` (T-03-27); sin documento 4.11.1 (T-03-06); sin `AttributeDefRepo` / SurrealDB; sin wiring en `bin/vortex` (T-00-27); MVP del curso; ~2 h. CLI OpenSpec kebab-case (`t-03-05` → carpeta `T-03-05`).

## Goals / Non-Goals

**Goals:**

- `IndexSettings` como payload de listas (vacío = no-op)
- Constructor `productos_index_settings` en `search` (4.11.2 + H-03 + H-04 + `attr_<clave>`)
- `apply_settings` PATCH + espera de *task* cuando el payload no está vacío
- Test ignore que GET settings y compara (Listo cuando)
- Actualizar `docs/4-modelo.md` § 4.11

**Non-Goals (diseño):**

- `get_settings` en el puerto `TextSearch` (solo el test de integración lee con el SDK)
- `displayedAttributes` / `stopWords` / faceting `maxValuesPerFacet`
- PATCH de `synonyms` (omitir el campo; T-03-27)
- Aplicar settings al uid fijo `productos` en este ticket (tests usan índice efímero; T-00-27 / T-03-11 usarán el constructor)

## Decisions

### 1. `IndexSettings` es transporte en `domain`; el perfil vive en `search`

**Elección:** ampliar `domain::IndexSettings` a struct con `Vec<String>` para `searchable_attributes`, `ranking_rules`, `filterable_attributes`, `sortable_attributes`, `typo_disable_on_attributes`. `Default` / todas las listas vacías = no-op. `is_empty()` (o equivalente) decide si `apply_settings` llama al motor. El constructor `search::productos_index_settings(claves_facetables)` (nombre exacto al aplicar) arma las constantes 4.11.2, agrega `marca_id`, `categoria_ids`, `tipo`, y `attr_<clave>` por cada clave.

**Por qué:** el puerto ya pasa `IndexSettings`. Las constantes Meilisearch (`categoria.lvl0`, `stock_disponible:desc`) no deben vivir en `domain` (T-00-02). T-03-08 / T-00-27 pasarán las claves facetables sin que este ticket abra SurrealDB.

**Alternativa:** hardcodear 4.11.2 dentro de `apply_settings` e ignorar el payload. Se descartó: no hay forma de inyectar `attr_*` dinámicos. **Alternativa:** `IndexSettings::productos` en `domain`. Se descartó: filtra nombres de motor en el crate prohibido.

`IndexSettings` deja de ser `Copy` (tiene `Vec`). Dummy de `domain` y `apply_settings(..., IndexSettings)` pasan a `IndexSettings::default()`.

### 2. Claves facetables: validar formato `AtributoDef.clave`, no el repo

**Elección:** cada clave MUST coincidir con el patrón de `AtributoDef` (minúscula ASCII inicial, luego `[a-z0-9_]`) y MUST NOT empezar por `attr_`. Prefijo `attr_` lo agrega el constructor. Cero claves es válido. Duplicados: se deduplican preservando orden.

**Por qué:** T-03-05 depende solo de T-03-04. El catálogo no está en el grafo de este ticket. T-03-08 actualizará `filterableAttributes` cuando cambie `facetable`.

**Alternativa:** `list_all` de `AttributeDefRepo`. Se descartó: arrastra `storage`/`catalog` y ~2 h. **Alternativa:** aceptar cualquier string. Se descartó: `attr_Medida` o `attr_attr_medida` romperían T-03-12.

### 3. `apply_settings` PATCH parcial; no tocar `synonyms`

**Elección:** si `settings.is_empty()` → `Ok(())` sin HTTP (regresión T-03-04). Si no, `index.set_settings` con solo los campos del payload (`Option::Some` en el SDK). Omitir `synonyms` en el PATCH para no borrar un diccionario futuro. Esperar *task* con el mismo *timeout* de 5 s.

GET en el test: el crate `search` ya depende de `meilisearch-sdk`; el test ignore usa `Client::index(...).get_settings()` (no se agrega método al *trait*). Comparar `searchable_attributes` y `ranking_rules` **en orden**; `filterable_attributes` / `sortable` / `typo.disable_on_attributes` por conjunto (el motor puede reordenar filtrables). Extraer nombres de `FilterableAttribute::Attribute`.

**Por qué:** PATCH de Meilisearch deja intactos los campos omitidos. Un PUT/reset wipearia defaults y `synonyms`.

**Alternativa:** `reset_settings` + set. Se descartó: más lento y borra todo. **Alternativa:** `get_settings` en `TextSearch`. Se descartó: el puerto no lo pide; solo el Listo cuando.

### 4. `attributesToHighlight` no se aplica aquí

**Elección:** documentar en 4.11.2 que el resaltado se pide en la consulta (`_formatted`; T-03-12). No forma parte de `IndexSettings` ni del PATCH.

**Por qué:** en Meilisearch 1.x es parámetro de search, no de `/settings`. Aplicarlo como setting fallaría o sería no-op.

### 5. Tests: unidad del perfil + ignore GET + unhappy URL muerta

**Elección:** nombres de las tablas en las specs. Unidad del constructor (happy H-03/H-04/`attr_medida`, omite synonyms, rechaza clave inválida) **sin** sidecar. Integración `meilisearch_apply_productos_settings_matches_expected` (`#[ignore = "meilisearch"]`, uid `t0305-<ulid>`). Unhappy `meilisearch_apply_settings_unavailable_when_unreachable` (payload no vacío, `http://127.0.0.1:1`). Conservar tests T-03-04 y el probe T-09-05.

**Por qué:** Listo cuando = leer config aplicada. CI T-09-05 ya filtra `meilisearch`. `cargo test --workspace` sin ignore sigue verde.

### 6. Doc 4.11 en el mismo ticket

**Elección:** actualizar [4.11.2](../../../docs/4-modelo.md#4112-configuración-del-índice): `filterableAttributes` incluye `marca_id`, `categoria_ids`, `tipo` y `attr_*`; nota de que `synonyms` entra en T-03-27; resaltado remitido a la consulta. En [4.11.1](../../../docs/4-modelo.md#4111-documento), agregar `marca_id` y `categoria_ids` al ejemplo JSON (H-03) para que el modelo coincida; la función `build_search_document` sigue siendo T-03-06.

**Por qué:** el ticket exige «Actualiza 4.11». H-03 es T-03-05 + T-03-06: settings ahora, documento después.

## Risks / Trade-offs

- [Meilisearch puede devolver `filterable_attributes` como objetos, no strings] → Mitigation: el test acepta `FilterableAttribute::Attribute(nombre)` y compara nombres; no exige `Settings` objects.
- [Orden de filtrables distinto al PATCH] → Mitigation: aserción por conjunto; searchable y ranking sí en orden.
- [`IndexSettings` pierde `Copy`] → Mitigation: un `Default`; dummy y tests de `domain` se actualizan en el mismo change.
- [T-00-27 / T-03-07 aún no llaman al constructor] → Mitigation: el índice real `productos` queda para esos tickets; este deja la API lista.
- [Claves facetables desactualizadas hasta T-03-08] → Mitigation: aceptable: no hay worker ni catálogo indexado todavía.

## Migration Plan

Cambio aditivo. Sin datos que migrar. Rollback: revertir `IndexSettings` a unidad, `apply_settings` a no-op y el párrafo de 4.11. `cargo test --workspace` (sin ignore) basta.

## Open Questions

Ninguna que altere spec, enfoque o tasks. El nombre público `productos_index_settings` vs `ProductosIndexSettings::build` se decide en apply.
