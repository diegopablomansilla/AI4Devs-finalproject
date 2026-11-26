# Design

## Context

En `develop`, T-03-05 deja el perfil `productos` (`searchableAttributes` con `nombre` primero, `rankingRules` terminando en `stock_disponible:desc`, `typoTolerance.disableOnAttributes` = `sku`/`codigos`). T-03-12 omite `sort` cuando `orden` es `relevancia`. T-03-13 expone `GET /api/v1/search` con tests sobre un `SearchGateway` falso. El crate `search` ya tiene tests `#[ignore = "meilisearch"]` con índice efímero (`t0304-*`, `t0305-*`) y el job CI `meilisearch` ejecuta `cargo test --workspace -- --include-ignored meilisearch`. T-02-41 fijó en `seed/productos.jsonl` los nombres de las HU y stocks `"40"` / `"3"`, pero el dataset ~200 SKUs no tiene un par de igual relevancia para el desempate. Ver `proposal.md` para motivación y `specs/vortex-search/spec.md` para el contrato.

Constraints: MVP del curso; ~2 h; no benchmarks (T-09-07); no UI; no seed/Surreal en el harness; Listo cuando = tres escenarios en CI.

## Goals / Non-Goals

**Goals:**

- Harness de integración en `search` que aplica el perfil `productos`, indexa documentos fijos y aserta typo, orden de atributo y desempate por stock
- Unhappy path: typo de `sku` no matchea; `q` sin coincidencias → `SearchPage` con `hits` vacío
- Reusar el patrón T-03-04 (`MEILISEARCH_URL`, `create_index` / `delete_index`, `#[ignore = "meilisearch"]`) para que el job existente los recoja

**Non-Goals (diseño):**

- Inyectar Meilisearch en tests de `api` o `vortex serve`
- Cargar `seed/productos.jsonl` ni arrancar el worker
- Cambiar `productos_index_settings`, el adaptador o el handler
- Nuevo job CI o cambio del filtro `meilisearch`

## Decisions

### 1. Crate `search` + `MeiliTextSearch`, no HTTP

**Elección:** tests de integración en `vortex-core/crates/search/tests/meilisearch_ranking.rs`. Hablan el puerto `TextSearch` (`create_index`, `apply_settings`, `upsert_documents`, `search`, `delete_index`). No montan el router Axum.

**Por qué:** el ranking lo define el perfil 4.11.2 + la consulta sin `sort`. T-03-13 ya cubrió el mapper HTTP con fake. Un test `GET /api/v1/search` con sidecar duplicaría composition root (`AppState`, DTO inglés) sin agregar señal de ranking.

**Alternativa:** `api` inyectando `SearchService { MeiliTextSearch }`. Se descartó: más setup, mismos asertos. **Alternativa:** `vortex serve` + seed. Se descartó: Surreal + worker + ~200 SKUs; fuera de ~2 h y ruidoso para orden.

### 2. Índice efímero con documentos fijos, no el dataset completo

**Elección:** cada test crea `t0314-<ulid>`, aplica `productos_index_settings([])` (o `["medida"]`; las claves facetables no afectan typo/ranking de estos escenarios) y hace upsert de 1–3 `SearchDocument`. Nombres de HU copiados como strings literales (misma ortografía que T-02-41). El par de desempate **no** sale del seed: dos documentos con el mismo `nombre` y `tipo`, `sku` distintos que no aparecen en `q`, `stock_disponible` 40 y 3, `con_stock` verdadero.

**Por qué:** buscar `"Perno"` en 200 SKUs (muchos `tipo: Perno`) hace flaky el orden relativo. T-02-41 garantiza que esos nombres existen en el catálogo de demos; no es un harness de importación.

**Alternativa:** `vortex seed` + índice `productos`. Se descartó: acopla a Surreal, outbox y worker; el desempate no es aislable. **Alternativa:** leer `seed/productos.jsonl` desde el test. Se descartó: el crate `search` no debe conocer el árbol `seed/`; el validador de T-02-41 ya cubre las strings.

### 3. Un test por escenario; cleanup en `delete_index`

**Elección:** cinco `#[tokio::test]` + `#[ignore = "meilisearch"]`. Helper compartido: URL, uid `t0314-*`, `apply_settings`, `cleanup_index` (mismo molde que `meilisearch_adapter.rs`). Nombres de test contienen `meilisearch_ranking_` para el filtro CI.

**Por qué:** un test que falle nombra el escenario Gherkin. Compartir un único índice entre tests introduce orden y fugas.

**Alternativa:** un solo test con los tres asertos. Se descartó: un fallo opaca los otros dos del Listo cuando.

### 4. Unhappy path de contrato 4.11.2, no de HTTP

**Elección:** (1) documento `sku = "SKUABCDEFGH"`, `nombre` sin ese token; `q = "SKUABCDEFGI"` → el hit no aparece (`disableOnAttributes` sobre `sku`). Con Meilisearch 1.58, SKUs con guiones y dígitos (p. ej. `TOR-AUTO-10X1`/`10Y1`) siguen matcheando por tokenización del motor; el test usa un sku alfabérico para observablemente cerrar el escenario 4.11.2. (2) `q = "xyzzy-no-match-9f3"` → `hits` vacío, no `DomainError`. No se agrega un test de URL muerta: T-03-04 ya lo cubre y no es ranking.

**Por qué:** el ticket pide tolerancia a errores de **tipeo**, no reabrir 400/503. Config OpenSpec exige unhappy path; estos dos cierran el contrato de typo-off en códigos y de “no match ≠ error”.

**Alternativa:** typo demasiado agresivo (`"trnll"`). Se descartó: umbral de Meilisearch (1–2 typos según longitud) es sensible a la versión. **Alternativa:** 400/503 en `api`. Se descartó: T-03-13.

### 5. Sin crate nueva ni cambio de allowlist

**Elección:** `SearchDocument::try_new` + campos públicos (`nombre`, `tipo`, `sku`, `stock_disponible`, `con_stock`) como el probe `meilisearch_adapter_upserts_full_search_document`. `SearchQuery::try_new(..., SearchOrden::Relevancia, 0, 20)`.

**Por qué:** cero impacto en `cargo-workspace`. El adaptador ya espera la *task* de upsert (timeout 5 s); no hace falta sleep extra.

## Risks / Trade-offs

- [Ranking de `"Perno"` depende de `attribute`/`words` de Meilisearch] → índice mínimo (solo los dos documentos de la HU); si falla, no se tocan settings (T-03-05 cerrado): se ajusta el fixture (p. ej. `tipo` `"Perno"` vs `"Arandela"`) sin cambiar 4.11.2.
- [Desempate: `sku` es searchable y podría romper el empate] → `sku` de ese par no comparte tokens con `q`; solo cambia `stock_disponible`.
- [Tests ignore no corren en `cargo test --workspace`] → el Listo cuando vive en el job `meilisearch` ya existente; documentado en la spec.
- [Falso negativo si el sidecar local es otra versión] → misma URL y pin que T-09-05 (`scripts/meilisearch-version`).

## Migration Plan

No hay migración de datos ni de API. Rollback = borrar `meilisearch_ranking.rs`. El índice `productos` de producción no se toca.

## Open Questions

Ninguna que bloquee apply. El par de desempate usa un `nombre` de prueba (no un SKU del seed); las tres strings de las HU sí se copian literalmente.
