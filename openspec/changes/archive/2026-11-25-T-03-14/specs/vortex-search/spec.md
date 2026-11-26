# Spec Delta

Ticket: T-03-14. HU: HU-03-03, HU-03-04.

## ADDED Requirements

### Requirement: Ranking y tolerancia a errores contra Meilisearch real

El crate `search` MUST demostrar, contra un Meilisearch HTTP real de la versión fijada, el ranking y la tolerancia a errores de [HU-03-03](../../../../../docs/6-historias.md#hu-03-03--búsqueda-mientras-se-escribe-tolerante-a-errores) y [HU-03-04](../../../../../docs/6-historias.md#hu-03-04--ranking-de-relevancia) con el perfil `productos` ([4.11.2](../../../../../docs/4-modelo.md#4112-configuración-del-índice)).

Los tests MUST usar un uid de índice distinto de `productos`, MUST aplicar `productos_index_settings` (cero o más claves facetables) y MUST indexar documentos fijos. Los nombres de producto de las HU MUST coincidir con los de T-02-41 / `seed/productos.jsonl`: `"Tornillo autoperforante 10 x 1"`, `"Perno de Acero"`, `"Arandela para Perno"`. MUST NOT sembrar el dataset completo, MUST NOT abrir SurrealDB, MUST NOT llamar `GET /api/v1/search`, y MUST NOT medir latencia.

`search` con `orden` `relevancia` MUST omitir `sort` para que valgan las `rankingRules` del índice, incluido el desempate `stock_disponible:desc`.

#### Scenario: tornilo encuentra Tornillo autoperforante (happy, Listo cuando)

- **WHEN** el índice de prueba tiene un documento con `nombre` `"Tornillo autoperforante 10 x 1"` y el perfil `productos` aplicado
- **AND** se busca `q` `"tornilo"` con `orden` `relevancia`
- **THEN** la operación termina sin error
- **AND** `hits` incluye un hit cuyo `nombre` es `"Tornillo autoperforante 10 x 1"`

#### Scenario: Perno ordena Perno de Acero antes que Arandela para Perno (happy, Listo cuando)

- **WHEN** el índice de prueba tiene documentos con `nombre` `"Perno de Acero"` y `"Arandela para Perno"` y el perfil `productos` aplicado
- **AND** se busca `q` `"Perno"` con `orden` `relevancia`
- **THEN** la operación termina sin error
- **AND** ambos nombres aparecen en `hits`
- **AND** el hit `"Perno de Acero"` aparece antes que `"Arandela para Perno"`

#### Scenario: a igual relevancia gana el stock 40 (happy, Listo cuando)

- **WHEN** el índice de prueba tiene dos documentos con el mismo `nombre` y el mismo `tipo`, `stock_disponible` 40 y 3 respectivamente, y el perfil `productos` aplicado
- **AND** se busca ese `nombre` con `orden` `relevancia` (sin `sort` de consulta)
- **THEN** la operación termina sin error
- **AND** el hit con `stock_disponible` 40 aparece antes que el de `stock_disponible` 3

#### Scenario: typo en sku no encuentra el producto (unhappy)

- **WHEN** el índice de prueba tiene un documento con `sku` `"SKUABCDEFGH"` y `nombre` que no contiene ese sku, y el perfil `productos` aplicado (`typoTolerance.disableOnAttributes` incluye `sku`)
- **AND** se busca `q` `"SKUABCDEFGI"` (un carácter distinto del sku) con `orden` `relevancia`
- **THEN** la operación termina sin error
- **AND** `hits` no incluye ese documento

#### Scenario: consulta sin coincidencias devuelve página vacía (unhappy)

- **WHEN** el índice de prueba tiene al menos un documento y el perfil `productos` aplicado
- **AND** se busca `q` `"xyzzy-no-match-9f3"` con `orden` `relevancia`
- **THEN** la operación termina sin error
- **AND** `hits` está vacío
- **AND** el resultado no es `DomainError`

### Requirement: Tests de ranking y tolerancia a errores

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. Los tres Listo cuando MUST correr en el job CI `meilisearch` (filtro `--include-ignored meilisearch` de T-09-05).

| Test | Ubicación | Tipo | Motor |
|---|---|---|---|
| `meilisearch_ranking_tornilo_finds_tornillo_autoperforante` | `vortex-core/crates/search/tests/` | happy, Listo cuando | `#[ignore = "meilisearch"]` |
| `meilisearch_ranking_perno_orders_steel_before_washer` | `vortex-core/crates/search/tests/` | happy, Listo cuando | `#[ignore = "meilisearch"]` |
| `meilisearch_ranking_equal_relevance_orders_stock_40_before_3` | `vortex-core/crates/search/tests/` | happy, Listo cuando | `#[ignore = "meilisearch"]` |
| `meilisearch_ranking_sku_typo_does_not_match` | `vortex-core/crates/search/tests/` | unhappy | `#[ignore = "meilisearch"]` |
| `meilisearch_ranking_unknown_query_returns_empty_hits` | `vortex-core/crates/search/tests/` | unhappy | `#[ignore = "meilisearch"]` |

Los tests `#[ignore = "meilisearch"]` MUST usar `MEILISEARCH_URL` (defecto `http://127.0.0.1:7700`) y un uid de índice de prueba cuyo nombre contenga `t0314`. `cargo test --workspace` **sin** `--include-ignored` MUST terminar con código 0 aunque el sidecar no esté levantado. MUST NOT borrar los probes de T-03-04 / T-03-05 / T-09-05. MUST NOT exigir un job CI nuevo.

#### Scenario: los tres escenarios Listo cuando corren en CI (happy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test --workspace -- --include-ignored meilisearch`
- **THEN** corren `meilisearch_ranking_tornilo_finds_tornillo_autoperforante`, `meilisearch_ranking_perno_orders_steel_before_washer` y `meilisearch_ranking_equal_relevance_orders_stock_40_before_3`
- **AND** terminan con código 0

#### Scenario: el typo de sku y la consulta vacía corren con sidecar (unhappy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test -p search -- --include-ignored meilisearch_ranking_sku_typo meilisearch_ranking_unknown_query`
- **THEN** ambos tests corren y terminan con código 0
- **AND** demuestran hits vacíos sin `DomainError`

#### Scenario: la suite por defecto no exige el sidecar (happy de aislamiento)

- **WHEN** Meilisearch no está levantado
- **AND** se ejecuta `cargo test --workspace` **sin** `--include-ignored`
- **THEN** el comando termina con código 0
- **AND** los tests `meilisearch_ranking_*` figuran como ignorados
