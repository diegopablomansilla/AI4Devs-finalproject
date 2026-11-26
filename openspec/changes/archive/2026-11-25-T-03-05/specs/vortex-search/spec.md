# Spec Delta

Ticket padre: T-03-05. HU: HU-03-04, HU-03-07.

## ADDED Requirements

### Requirement: Perfil de settings del índice productos

El crate `search` MUST exponer un constructor de `IndexSettings` para el perfil `productos` que acepta las `clave` de `AtributoDef` con `facetable = true` (sin prefijo `attr_`) y MUST armar las listas siguientes. MUST NOT consultar SurrealDB ni `AttributeDefRepo`. MUST NOT incluir `synonyms` (T-03-27).

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

## MODIFIED Requirements

### Requirement: Adaptador TextSearch en search

El crate `search` MUST implementar el puerto `domain::TextSearch` contra un Meilisearch HTTP local. El adaptador MUST construirse con una URL base (por defecto la de `MEILISEARCH_URL` / `http://127.0.0.1:7700`) y una `api_key` opcional. MUST esperar cada *task* de escritura (`upsert_documents`, `delete_documents`, `create_index`, `delete_index`, `swap_indexes`, y `apply_settings` cuando no sea no-op) hasta completar o hasta un *timeout* configurable (defecto 5 s). MUST NOT construir el documento 4.11.1 (T-03-06): el alta serializa `SearchDocument` con el campo `id` (ULID de 26 caracteres, sin prefijo de tabla) como clave primaria. MUST NOT enganchar el adaptador en `bin/vortex` ni en el supervisor (T-00-27). MUST NOT enviar `synonyms` al motor (T-03-27).

`search` (consulta) MUST enviar `q`, `offset` y `limit` del `SearchQuery` al motor y MUST devolver un `SearchPage` cuyos `hits` exponen el `id` ULID de cada documento. MUST NOT traducir filtros HTTP ni `orden` de ranking a la consulta (T-03-12). `facet_search` MAY delegar en `search` sin agregar facetas (T-03-18). `apply_settings` con `IndexSettings` vacío MUST terminar en éxito **sin** llamar al motor y **sin** aplicar `searchableAttributes` / `filterableAttributes` / `rankingRules`. `apply_settings` con `IndexSettings` no vacío MUST aplicar al uid dado las listas del payload (`searchableAttributes`, `rankingRules`, `filterableAttributes`, `sortableAttributes`, `typoTolerance.disableOnAttributes`) y MUST esperar la *task*. MUST NOT PATCH `synonyms` ni `attributesToHighlight`. `create_index` y `delete_index` MUST operar sobre el uid dado. `swap_indexes` MUST rechazar origen = destino con `invalid_input` (misma guarda de dominio) y, si difieren, pedir el intercambio al motor.

#### Scenario: Alta y consulta de un documento (happy, Listo cuando)

- **WHEN** el adaptador hace `upsert_documents` de un `SearchDocument` con un ULID válido en un índice de prueba y luego `search` con `q` igual a ese ULID (o `q` vacía con `limit` suficiente)
- **THEN** ambas operaciones terminan sin error
- **AND** `SearchPage.hits` incluye un hit cuyo `id` es ese ULID

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

### Requirement: Traducción de errores del motor

El adaptador MUST traducir fallos del motor a `DomainError` sin HTTP ni `problem+json`:

- conexión rechazada, DNS, o motor inalcanzable → `code` `unavailable`
- espera de *task* o de la llamada HTTP agotada → `code` `timeout`
- documento rechazado por el motor (payload inválido / error de documento) → `code` `invalid_input`

MUST NOT introducir `code` nuevos (`meilisearch_unavailable`, etc.). MUST reutilizar `DomainError::unavailable(UnavailableCode::Unavailable)`, `DomainError::timeout()` e `InvalidInputCode::InvalidInput`.

#### Scenario: Motor detenido o inalcanzable (unhappy, Listo cuando)

- **WHEN** el adaptador apunta a una URL donde no hay Meilisearch escuchando (p. ej. puerto cerrado) y se invoca `health`, `search` o `upsert_documents`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `unavailable`

#### Scenario: apply_settings con motor inalcanzable (unhappy)

- **WHEN** el adaptador apunta a una URL donde no hay Meilisearch escuchando y se invoca `apply_settings` con un `IndexSettings` no vacío
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `unavailable`

#### Scenario: Timeout de task (unhappy)

- **WHEN** la espera de una *task* o de la llamada al motor supera el *timeout* configurado
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `timeout`

#### Scenario: Documento inválido (unhappy)

- **WHEN** el motor rechaza un documento (payload inválido)
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`

### Requirement: Tests del adaptador Meilisearch

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo | Motor |
|---|---|---|---|
| `meilisearch_adapter_upserts_then_searches_document` | `vortex-core/crates/search/tests/` | happy | `#[ignore = "meilisearch"]` |
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
