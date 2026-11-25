# Spec Delta

Ticket: T-03-08. HU: HU-03-01.

## MODIFIED Requirements

### Requirement: Perfil de settings del índice productos

El crate `search` MUST reexportar `domain::productos_index_settings` (mismo nombre público `search::productos_index_settings`) para no romper los tests T-03-05. MUST NOT duplicar las listas 4.11.2 en `search`. MUST NOT consultar SurrealDB ni `AttributeDefRepo`. MUST NOT incluir `synonyms` (T-03-27). El contrato observable del constructor (listas, validación de `clave`, `invalid_input`) MUST ser el de `domain-text-search`.

`searchableAttributes` (orden de peso): `nombre`, `tipo`, `sku`, `codigos`, `marca`, `atributos_texto`, `descripcion`.

`rankingRules`: `words`, `typo`, `proximity`, `attribute`, `sort`, `exactness`, `stock_disponible:desc`.

`filterableAttributes` MUST incluir: `categoria.lvl0`, `categoria.lvl1`, `categoria.lvl2`, `categoria.lvl3`, `marca`, `con_stock`, `marca_id`, `categoria_ids`, `tipo`, y `attr_<clave>` por cada clave facetable recibida.

`sortableAttributes`: `precio`, `stock_disponible`.

`typoTolerance.disableOnAttributes`: `sku`, `codigos`.

MUST NOT tratar `attributesToHighlight` como setting de índice (es parámetro de consulta; T-03-12).

Una clave vacía, con espacios, con formato distinto al de `AtributoDef.clave` (minúsculas ASCII, dígitos, `_`, primer carácter letra) o que ya lleve el prefijo `attr_` MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT producir `IndexSettings`. Cero claves facetables es válido (solo los filtrables fijos).

#### Scenario: Perfil productos con un atributo facetable (happy)

- **WHEN** se construye el perfil `productos` con la clave facetable `medida`
- **THEN** la construcción termina sin error
- **AND** `searchableAttributes` es `nombre`, `tipo`, `sku`, `codigos`, `marca`, `atributos_texto`, `descripcion` en ese orden
- **AND** `rankingRules` termina en `stock_disponible:desc`
- **AND** `filterableAttributes` incluye `marca_id`, `categoria_ids`, `tipo` y `attr_medida`
- **AND** `sortableAttributes` incluye `precio` y `stock_disponible`
- **AND** `typoTolerance.disableOnAttributes` es `sku` y `codigos`
- **AND** no hay lista de `synonyms`

#### Scenario: Perfil sin claves facetables (happy)

- **WHEN** se construye el perfil `productos` con cero claves facetables
- **THEN** la construcción termina sin error
- **AND** `filterableAttributes` incluye `marca_id`, `categoria_ids` y `tipo`
- **AND** no incluye ningún `attr_*`

#### Scenario: Clave facetable inválida (unhappy)

- **WHEN** se intenta construir el perfil `productos` con una clave vacía, `Medida`, `attr_medida` o `medida-mm`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce `IndexSettings`

### Requirement: Tests del adaptador Meilisearch

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. Los unitarios del perfil `productos` viven en `domain` (T-03-08); este crate conserva el probe de integración y el reexport.

| Test | Ubicación | Tipo | Motor |
|---|---|---|---|
| `meilisearch_adapter_upserts_then_searches_document` | `vortex-core/crates/search/tests/` | happy | `#[ignore = "meilisearch"]` |
| `meilisearch_adapter_upserts_full_search_document` | `vortex-core/crates/search/tests/` | happy | `#[ignore = "meilisearch"]` |
| `meilisearch_adapter_deletes_document` | `vortex-core/crates/search/tests/` | happy | `#[ignore = "meilisearch"]` |
| `meilisearch_adapter_health_is_up` | `vortex-core/crates/search/tests/` | happy | `#[ignore = "meilisearch"]` |
| `meilisearch_apply_productos_settings_matches_expected` | `vortex-core/crates/search/tests/` | happy | `#[ignore = "meilisearch"]` |
| `meilisearch_adapter_unavailable_when_unreachable` | `vortex-core/crates/search/tests/` | unhappy | siempre (URL muerta; no exige sidecar) |
| `meilisearch_apply_settings_unavailable_when_unreachable` | `vortex-core/crates/search/tests/` | unhappy | siempre (URL muerta; no exige sidecar) |
| `meilisearch_error_maps_timeout` | `vortex-core/crates/search/` | unhappy | unidad (mapper) |
| `meilisearch_error_maps_invalid_document` | `vortex-core/crates/search/` | unhappy | unidad (mapper) |

Los tests `#[ignore = "meilisearch"]` MUST usar `MEILISEARCH_URL` (defecto `http://127.0.0.1:7700`) y un uid de índice de prueba distinto de `productos`. `cargo test --workspace` **sin** `--include-ignored` MUST terminar con código 0 aunque el sidecar no esté levantado (los tests ignore se omiten; el unhappy de URL muerta corre). `cargo test --workspace -- --include-ignored meilisearch` MUST ejecutar los ignore de esta tabla **y** el probe `meilisearch_health_is_available` de T-09-05. MUST NOT borrar ese probe. MUST NOT exigir el job CI `meilisearch` nuevo: el de T-09-05 ya filtra `meilisearch`.

#### Scenario: Alta, consulta y borrado contra el motor (happy, Listo cuando)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test --workspace -- --include-ignored meilisearch`
- **THEN** `meilisearch_adapter_upserts_then_searches_document` y `meilisearch_adapter_deletes_document` corren y terminan con código 0

#### Scenario: Upsert de documento completo (happy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test -p search -- --include-ignored meilisearch_adapter_upserts_full_search_document`
- **THEN** el test corre y termina con código 0
- **AND** el documento en el motor conserva `sku` y `nombre` además de `id`

#### Scenario: Configuración aplicada coincide con la esperada (happy, Listo cuando)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test -p search -- --include-ignored meilisearch_apply_productos_settings_matches_expected`
- **THEN** el test corre y termina con código 0
- **AND** demuestra GET settings igual al perfil `productos` (4.11.2 + H-03 + H-04 + `attr_medida`)

#### Scenario: Error con motor inalcanzable (unhappy, Listo cuando)

- **WHEN** se ejecuta `cargo test -p search meilisearch_adapter_unavailable_when_unreachable` **sin** `--include-ignored`
- **THEN** el test corre (no se omite)
- **AND** termina con código 0
- **AND** demuestra `code` `unavailable`

#### Scenario: apply_settings inalcanzable corre sin sidecar (unhappy)

- **WHEN** se ejecuta `cargo test -p search meilisearch_apply_settings_unavailable_when_unreachable` **sin** `--include-ignored`
- **THEN** el test corre (no se omite)
- **AND** termina con código 0
- **AND** demuestra `code` `unavailable`

#### Scenario: La suite por defecto no exige el sidecar (happy de aislamiento)

- **WHEN** Meilisearch no está levantado
- **AND** se ejecuta `cargo test --workspace` **sin** `--include-ignored`
- **THEN** el comando termina con código 0
- **AND** los tests `#[ignore = "meilisearch"]` figuran como ignorados
