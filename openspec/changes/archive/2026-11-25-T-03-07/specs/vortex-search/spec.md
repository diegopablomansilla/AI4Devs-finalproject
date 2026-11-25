# Spec Delta

Ticket padre: T-03-07. HU: HU-03-01.

## MODIFIED Requirements

### Requirement: Adaptador TextSearch en search

El crate `search` MUST implementar el puerto `domain::TextSearch` contra un Meilisearch HTTP local. El adaptador MUST construirse con una URL base (por defecto la de `MEILISEARCH_URL` / `http://127.0.0.1:7700`) y una `api_key` opcional. MUST esperar cada *task* de escritura (`upsert_documents`, `delete_documents`, `create_index`, `delete_index`, `swap_indexes`, y `apply_settings` cuando no sea no-op) hasta completar o hasta un *timeout* configurable (defecto 5 s). `upsert_documents` MUST serializar cada `SearchDocument` con **todos** los campos [4.11.1](../../../../../docs/4-modelo.md#4111-documento) (`id` ULID de 26 caracteres sin prefijo de tabla como clave primaria, más `sku`, `nombre`, `tipo`, `marca`, `marca_id`, `categoria`, `categoria_ids`, `codigos`, `atributos_texto`, `attr_*`, `descripcion`, `precio`, `unidad_medida`, `stock_disponible`, `con_stock`, `ubicacion`, `imagen`). MUST NOT enganchar el adaptador en `bin/vortex` ni en el supervisor (T-00-27). MUST NOT enviar `synonyms` al motor (T-03-27).

`search` (consulta) MUST enviar `q`, `offset` y `limit` del `SearchQuery` al motor y MUST devolver un `SearchPage` cuyos `hits` exponen el `id` ULID de cada documento. MUST NOT traducir filtros HTTP ni `orden` de ranking a la consulta (T-03-12). `facet_search` MAY delegar en `search` sin agregar facetas (T-03-18). `apply_settings` con `IndexSettings` vacío MUST terminar en éxito **sin** llamar al motor y **sin** aplicar `searchableAttributes` / `filterableAttributes` / `rankingRules`. `apply_settings` con `IndexSettings` no vacío MUST aplicar al uid dado las listas del payload (`searchableAttributes`, `rankingRules`, `filterableAttributes`, `sortableAttributes`, `typoTolerance.disableOnAttributes`) y MUST esperar la *task*. MUST NOT PATCH `synonyms` ni `attributesToHighlight`. `create_index` y `delete_index` MUST operar sobre el uid dado. `swap_indexes` MUST rechazar origen = destino con `invalid_input` (misma guarda de dominio) y, si difieren, pedir el intercambio al motor.

#### Scenario: Alta y consulta de un documento (happy, Listo cuando)

- **WHEN** el adaptador hace `upsert_documents` de un `SearchDocument` con un ULID válido en un índice de prueba y luego `search` con `q` igual a ese ULID (o `q` vacía con `limit` suficiente)
- **THEN** ambas operaciones terminan sin error
- **AND** `SearchPage.hits` incluye un hit cuyo `id` es ese ULID

#### Scenario: Upsert envía el documento 4.11.1 (happy)

- **WHEN** el adaptador hace `upsert_documents` de un `SearchDocument` con `sku` y `nombre` no vacíos y luego se lee el documento del motor por `id`
- **THEN** el JSON persistido incluye `sku` y `nombre` iguales a los del `SearchDocument`
- **AND** la clave primaria sigue siendo `id`

#### Scenario: Borrado de un documento (happy, Listo cuando)

- **WHEN** el adaptador da de alta un documento, confirma que `search` lo encuentra, y luego `delete_documents` de ese id
- **THEN** el borrado termina sin error
- **AND** un `search` posterior no incluye ese `id` en `hits`

#### Scenario: health up (happy)

- **WHEN** Meilisearch responde en la URL configurada y el adaptador llama `health`
- **THEN** la operación termina sin error
- **AND** el `ComponentHealth` tiene estado `up`

#### Scenario: apply_settings vacío no configura 4.11.2 (happy, non-goal)

- **WHEN** el adaptador llama `apply_settings` con un `IndexSettings` vacío
- **THEN** la operación termina sin error
- **AND** no se llama al motor para settings
- **AND** no se exige que el índice tenga `searchableAttributes` ni `filterableAttributes` de [4.11.2](../../../../../docs/4-modelo.md#4112-configuración-del-índice)

#### Scenario: apply_settings aplica el perfil productos y GET coincide (happy, Listo cuando)

- **WHEN** el adaptador crea un índice de prueba distinto de `productos`, aplica el perfil `productos` con clave facetable `medida`, y se leen los settings del motor
- **THEN** `apply_settings` termina sin error
- **AND** `searchableAttributes` leídos coinciden con `nombre`, `tipo`, `sku`, `codigos`, `marca`, `atributos_texto`, `descripcion` en ese orden
- **AND** `rankingRules` leídos coinciden e incluyen `stock_disponible:desc`
- **AND** `filterableAttributes` leídos incluyen `marca_id`, `categoria_ids`, `tipo`, `con_stock` y `attr_medida`
- **AND** `sortableAttributes` leídos incluyen `precio` y `stock_disponible`
- **AND** `typoTolerance.disableOnAttributes` leídos son `sku` y `codigos`
- **AND** el motor no tiene `synonyms` de rubro aplicados por esta operación

### Requirement: Tests del adaptador Meilisearch

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

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
| `productos_index_settings_includes_4_11_2_h03_h04` | `vortex-core/crates/search/` | happy | unidad (sin sidecar) |
| `productos_index_settings_omits_synonyms` | `vortex-core/crates/search/` | happy | unidad (sin sidecar) |
| `productos_index_settings_rejects_invalid_clave` | `vortex-core/crates/search/` | unhappy | unidad (sin sidecar) |

Los tests `#[ignore = "meilisearch"]` MUST usar `MEILISEARCH_URL` (defecto `http://127.0.0.1:7700`) y un uid de índice de prueba distinto de `productos`. `cargo test --workspace` **sin** `--include-ignored` MUST terminar con código 0 aunque el sidecar no esté levantado (los tests ignore se omiten; el unhappy de URL muerta y los unitarios del perfil corren). `cargo test --workspace -- --include-ignored meilisearch` MUST ejecutar los ignore de esta tabla **y** el probe `meilisearch_health_is_available` de T-09-05. MUST NOT borrar ese probe. MUST NOT exigir el job CI `meilisearch` nuevo: el de T-09-05 ya filtra `meilisearch`.

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
