# Spec Delta

Ticket: T-03-12. HU: HU-03-03.

## ADDED Requirements

### Requirement: Caso de uso de búsqueda por texto

El crate `search` MUST exponer un caso de uso de búsqueda por texto que construye un `SearchQuery` de dominio y delega en el puerto `TextSearch`. MUST NOT abrir HTTP, MUST NOT mapear a `SearchResponse` / `problem+json`, MUST NOT hablar con Meilisearch salvo a través del puerto, y MUST NOT pedir facetas ni degradar a SurrealDB.

El caso de uso MUST aceptar `q`, `filtros` (`categoria_ids`, `marca_id`, `con_stock`, `atributos` con clave de `AtributoDef` **sin** `attr_`), `orden`, `offset` y `limit`. MUST construir el `SearchQuery` (mismas reglas de dominio: `q` ≤ 200, `offset` ≤ 1000, `limit` 1..=50) y MUST llamar `TextSearch::search` con el `IndexUid` inyectado (los tests usan un uid de prueba; el composition root de T-03-13 inyecta `productos`). MUST devolver el `SearchPage` del puerto sin remapear códigos de error.

MUST NOT traducir nombres OpenAPI en inglés (`categoryId`, `brandId`, `inStock`, `relevance`). MUST NOT aplicar `facets=true`. MUST NOT implementar respaldo. MUST exponer una fachada object-safe (`SearchGateway`) con el método de búsqueda, implementada por el caso de uso, para inyección en T-03-13.

#### Scenario: Búsqueda con q y filtros vacíos (happy)

- **WHEN** el caso de uso recibe `q` `"perno"`, filtros vacíos, `orden` `relevancia`, `offset` 0 y `limit` 20
- **THEN** la operación termina sin error
- **AND** el puerto `search` recibe un `SearchQuery` con esos mismos `q`, `offset`, `limit` y `orden`
- **AND** los filtros del `SearchQuery` están vacíos

#### Scenario: Traduce cada filtro al SearchQuery (happy, Listo cuando)

- **WHEN** el caso de uso recibe `categoria_ids` con un id, `marca_id` presente, `con_stock` verdadero y `atributos` `medida` → `"M8"`
- **THEN** la operación termina sin error
- **AND** el `TextSearch` falso recibe exactamente esos cuatro filtros
- **AND** la clave de atributo en el `SearchQuery` es `"medida"` (sin prefijo `attr_`)

#### Scenario: q vacía con filtro de stock (happy)

- **WHEN** el caso de uso recibe `q` vacía y `con_stock` verdadero
- **THEN** la operación termina sin error
- **AND** el puerto recibe `q` vacía y `con_stock` verdadero

#### Scenario: q demasiado larga (unhappy)

- **WHEN** el caso de uso recibe `q` de 201 caracteres
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el puerto `search` no se invoca

#### Scenario: offset o limit fuera de rango (unhappy)

- **WHEN** el caso de uso recibe `offset` 1001, o `limit` 0, o `limit` 51
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el puerto `search` no se invoca

#### Scenario: clave de atributo con prefijo attr_ (unhappy)

- **WHEN** el caso de uso recibe `atributos` con clave `"attr_medida"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el puerto `search` no se invoca

#### Scenario: motor unavailable (unhappy)

- **WHEN** el `TextSearch` falso está configurado para `unavailable` y el caso de uso busca `"perno"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `unavailable`

#### Scenario: motor timeout (unhappy)

- **WHEN** el `TextSearch` falso está configurado para `timeout` y el caso de uso busca `"perno"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `timeout`

#### Scenario: la fachada es inyectable (happy)

- **WHEN** se usa la fachada object-safe del caso de uso como `dyn` en un contenedor de aplicación
- **THEN** el tipo es object-safe
- **AND** el método de búsqueda está expuesto
- **AND** no expone facetas ni respaldo

### Requirement: Traducción de SearchQuery a consulta Meilisearch

El adaptador `TextSearch` MUST traducir el `SearchQuery` a parámetros de consulta del motor:

- `q`, `offset`, `limit` como hasta T-03-04
- `filtros.categoria_ids` no vacío → filtro `categoria_ids IN [ids]` (ULID sin prefijo de tabla)
- `filtros.marca_id` presente → `marca_id = "<ulid>"`
- `filtros.con_stock` presente → `con_stock = true` o `con_stock = false`
- cada `filtros.atributos` `clave` → valor → `attr_<clave> = "<valor>"` (anteponer `attr_` aquí, no en dominio)
- varios filtros presentes MUST combinarse con `AND`
- filtros todos vacíos / ausentes MUST omitir el parámetro `filter`
- `orden` `relevancia` MUST omitir `sort` (valen las `rankingRules` del índice, incluido el desempate `stock_disponible:desc`)
- `orden` `precio_asc` → `precio:asc`; `precio_desc` → `precio:desc`; `stock_desc` → `stock_disponible:desc`
- MUST pedir resaltado de consulta: `attributesToHighlight` = `nombre`, `descripcion` ([4.11.2](../../../../../docs/4-modelo.md#4112-configuración-del-índice)); MUST NOT PATCH eso como setting de índice
- cada hit MUST mapear `_formatted.nombre` / `_formatted.descripcion` a `SearchHit.resaltado` (ausente si el motor no envía `_formatted`)
- cada hit MUST hidratar los campos 4.11.1 presentes en el documento (`sku`, `nombre`, `tipo`, `marca`, `codigos`, `atributos_texto`, `precio`, `unidad_medida`, `stock_disponible`, `con_stock`, `ubicacion`, `imagen`) además del `id`

MUST NOT enviar `facets` ni `facetDistribution` (T-03-18). MUST NOT cambiar el mapper de errores de T-03-04. MUST NOT exigir Meilisearch en la suite por defecto.

#### Scenario: filtro AND de categoria, marca, stock y attr_medida (happy)

- **WHEN** se traduce un `SearchFilters` con un `categoria_ids`, un `marca_id`, `con_stock` verdadero y `medida` → `"M8"`
- **THEN** el `filter` del motor incluye `categoria_ids IN [...]`, `marca_id = "..."`, `con_stock = true` y `attr_medida = "M8"` combinados con `AND`
- **AND** no aparece la clave `medida` sin el prefijo `attr_`

#### Scenario: sin filtros no envía filter (happy)

- **WHEN** se traduce un `SearchFilters` vacío
- **THEN** la consulta al motor no incluye parámetro `filter`

#### Scenario: sort por precio y stock (happy)

- **WHEN** se traduce `orden` `precio_asc`, `precio_desc` o `stock_desc`
- **THEN** el `sort` del motor es `precio:asc`, `precio:desc` o `stock_disponible:desc` respectivamente

#### Scenario: relevancia no envía sort (happy)

- **WHEN** se traduce `orden` `relevancia`
- **THEN** la consulta al motor no incluye parámetro `sort`

#### Scenario: pide _formatted de nombre y descripcion (happy)

- **WHEN** el adaptador ejecuta `search` con cualquier `SearchQuery` válido
- **THEN** la consulta pide `attributesToHighlight` `nombre` y `descripcion`
- **AND** no se hace PATCH de settings de índice con esos atributos

#### Scenario: hit hidrata documento y resaltado (happy)

- **WHEN** el motor devuelve un documento 4.11.1 con `_formatted.nombre` que contiene `<em>` o `<mark>`
- **THEN** el `SearchHit` expone `id`, `sku`, `nombre` y `resaltado.nombre` a partir de `_formatted`
- **AND** `facetas` de la página sigue ausente

### Requirement: Tests del caso de uso y de la traducción

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST NOT exigir sidecar Meilisearch (sin nuevos `#[ignore = "meilisearch"]` en este ticket).

| Test | Ubicación | Tipo |
|---|---|---|
| `search_service_forwards_q_with_empty_filters` | `vortex-core/crates/search/` | happy |
| `search_service_forwards_each_filter` | `vortex-core/crates/search/` | happy, Listo cuando |
| `search_service_allows_empty_q_with_con_stock` | `vortex-core/crates/search/` | happy |
| `search_gateway_is_object_safe` | `vortex-core/crates/search/` | happy |
| `meili_filter_joins_categoria_marca_stock_and_attr` | `vortex-core/crates/search/` | happy |
| `meili_filter_omitted_when_empty` | `vortex-core/crates/search/` | happy |
| `meili_sort_maps_precio_and_stock` | `vortex-core/crates/search/` | happy |
| `meili_sort_omitted_for_relevancia` | `vortex-core/crates/search/` | happy |
| `meili_search_requests_nombre_descripcion_highlight` | `vortex-core/crates/search/` | happy |
| `search_service_rejects_q_too_long` | `vortex-core/crates/search/` | unhappy |
| `search_service_rejects_offset_or_limit_out_of_range` | `vortex-core/crates/search/` | unhappy |
| `search_service_rejects_attr_prefix_clave` | `vortex-core/crates/search/` | unhappy |
| `search_service_propagates_unavailable` | `vortex-core/crates/search/` | unhappy |
| `search_service_propagates_timeout` | `vortex-core/crates/search/` | unhappy |

`cargo test --workspace` MUST ejecutarlos. El Listo cuando de T-03-12 es `search_service_forwards_each_filter` (doble de `TextSearch`, sin motor).

#### Scenario: la suite de búsqueda cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p search search_service meili_filter meili_sort meili_search_requests search_gateway`
- **THEN** corren los tests de la tabla
- **AND** los de reenvío de filtros, traducción AND, omisión de `filter`/`sort` y highlight pasan
- **AND** los de `invalid_input`, `unavailable` y `timeout` pasan

## MODIFIED Requirements

### Requirement: Adaptador TextSearch en search

El crate `search` MUST implementar el puerto `domain::TextSearch` contra un Meilisearch HTTP local. El adaptador MUST construirse con una URL base (por defecto la de `MEILISEARCH_URL` / `http://127.0.0.1:7700`) y una `api_key` opcional. MUST esperar cada *task* de escritura (`upsert_documents`, `delete_documents`, `create_index`, `delete_index`, `swap_indexes`, y `apply_settings` cuando no sea no-op) hasta completar o hasta un *timeout* configurable (defecto 5 s). `upsert_documents` MUST serializar cada `SearchDocument` con **todos** los campos [4.11.1](../../../../../docs/4-modelo.md#4111-documento) (`id` ULID de 26 caracteres sin prefijo de tabla como clave primaria, más `sku`, `nombre`, `tipo`, `marca`, `marca_id`, `categoria`, `categoria_ids`, `codigos`, `atributos_texto`, `attr_*`, `descripcion`, `precio`, `unidad_medida`, `stock_disponible`, `con_stock`, `ubicacion`, `imagen`). MUST NOT enganchar el adaptador en `bin/vortex` ni en el supervisor (T-00-27). MUST NOT enviar `synonyms` al motor (T-03-27).

`search` (consulta) MUST enviar `q`, `offset` y `limit` del `SearchQuery` al motor, MUST traducir `SearchFilters` y `SearchOrden` a `filter`/`sort` según el requirement «Traducción de SearchQuery a consulta Meilisearch», MUST pedir `_formatted` con `attributesToHighlight` `nombre` y `descripcion`, y MUST devolver un `SearchPage` cuyos `hits` exponen el `id` ULID y los campos 4.11.1 + `resaltado`. MUST NOT traducir nombres HTTP en inglés (`categoryId`, `inStock`) — eso es T-03-13. `facet_search` MAY delegar en `search` sin agregar facetas (T-03-18). `apply_settings` con `IndexSettings` vacío MUST terminar en éxito **sin** llamar al motor y **sin** aplicar `searchableAttributes` / `filterableAttributes` / `rankingRules`. `apply_settings` con `IndexSettings` no vacío MUST aplicar al uid dado las listas del payload (`searchableAttributes`, `rankingRules`, `filterableAttributes`, `sortableAttributes`, `typoTolerance.disableOnAttributes`) y MUST esperar la *task*. MUST NOT PATCH `synonyms` ni `attributesToHighlight`. `create_index` y `delete_index` MUST operar sobre el uid dado. `swap_indexes` MUST rechazar origen = destino con `invalid_input` (misma guarda de dominio) y, si difieren, pedir el intercambio al motor.

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
