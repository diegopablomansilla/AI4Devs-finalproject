# domain-text-search Specification

## Purpose

Define el puerto hexagonal `TextSearch` y los tipos de consulta/página de búsqueda de texto en el crate `domain`, para que adaptadores y casos de uso hablen el mismo contrato sin acoplar el dominio a Meilisearch.

Ticket padre: T-03-03. HU: HU-03-01.

## Requirements

### Requirement: Puerto TextSearch en domain

El crate `domain` MUST exponer el puerto `TextSearch` con exactamente estas operaciones: `upsert_documents`, `delete_documents`, `search`, `facet_search`, `apply_settings`, `create_index`, `swap_indexes`, `delete_index` y `health`. Los errores de las operaciones MUST ser `DomainError`. El puerto MUST NOT persistir, MUST NOT abrir HTTP ni sockets, MUST NOT depender de `meilisearch-sdk` ni de `reqwest`, y MUST NOT mapear a `problem+json` ni a estado HTTP.

`upsert_documents` MUST aceptar un identificador de índice y una lista de documentos identificados por ULID de 26 caracteres **sin** prefijo de tabla. `delete_documents` MUST aceptar un identificador de índice y una lista de esos mismos ids. `search` MUST aceptar un identificador de índice y un `SearchQuery`; el resultado de éxito MUST ser `SearchPage`. `facet_search` MUST aceptar un identificador de índice y un `SearchQuery`; el resultado de éxito MUST ser `SearchPage`. `apply_settings` MUST aceptar un identificador de índice y un `IndexSettings`. `create_index` y `delete_index` MUST aceptar un identificador de índice. `swap_indexes` MUST aceptar dos identificadores de índice (origen y destino). `health` MUST devolver `ComponentHealth` de éxito o `DomainError`.

MUST NOT implementar un adaptador Meilisearch (T-03-04). MUST NOT aplicar la configuración 4.11.2 (T-03-05). El crate `domain` MUST exponer `build_search_document` (documento [4.11.1](../../../../../docs/4-modelo.md#4111-documento)). MUST NOT enviar documentos a Meilisearch.

#### Scenario: El puerto declara las nueve operaciones (happy)

- **WHEN** se inspecciona el puerto `TextSearch` en `domain`
- **THEN** expone `upsert_documents`, `delete_documents`, `search`, `facet_search`, `apply_settings`, `create_index`, `swap_indexes`, `delete_index` y `health`
- **AND** los resultados de error usan `DomainError`
- **AND** `search` y `facet_search` de éxito usan `SearchPage`
- **AND** `health` de éxito usa `ComponentHealth`

#### Scenario: Dummy search happy path

- **WHEN** un doble de prueba que implementa `TextSearch` recibe `search` con un `SearchQuery` válido
- **THEN** la operación termina sin error
- **AND** el resultado es un `SearchPage`

#### Scenario: Dummy health up (happy)

- **WHEN** un doble de prueba que implementa `TextSearch` recibe `health`
- **THEN** la operación termina sin error
- **AND** el `ComponentHealth` expuesto tiene estado `up`

#### Scenario: Dummy search Unavailable (unhappy)

- **WHEN** un doble de prueba configurado para fallo de disponibilidad recibe `search`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `unavailable`

#### Scenario: Dummy search Timeout (unhappy)

- **WHEN** un doble de prueba configurado para timeout recibe `search`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `timeout`

#### Scenario: Dummy upsert InvalidInput (unhappy)

- **WHEN** un doble de prueba configurado para documento inválido recibe `upsert_documents`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`

### Requirement: SearchQuery, SearchFilters y SearchPage

El crate `domain` MUST exponer:

- `SearchQuery`: `q` (string, longitud ≤ 200; vacía permitida), `filtros` (`SearchFilters`), `orden` (uno de `relevancia`, `precio_asc`, `precio_desc`, `stock_desc`), `offset` (entero 0..=1000) y `limit` (entero 1..=50). MUST NOT usar nombres de API en inglés (`categoryId`, `brandId`, `inStock`, `price_asc` como campo).
- `SearchFilters`: `categoria_ids` (lista de `Id<Categoria>`; vacía = sin filtro de categoría), `marca_id` (`Option<Id<Marca>>`), `con_stock` (`Option<bool>`; ausente = ambos) y `atributos` (mapa `clave` → valor string; `clave` es la de `AtributoDef`, **sin** prefijo `attr_`). MUST NOT incluir texto de búsqueda. Una `clave` vacía, con formato distinto al de `AtributoDef.clave` (minúsculas ASCII, dígitos, `_`, primer carácter letra) o que ya lleve el prefijo `attr_` MUST fallar con `DomainError` `code` `invalid_input` al construir el `SearchQuery` y MUST NOT producir un `SearchQuery`.
- `SearchPage`: `hits` (lista de `SearchHit`), `estimado_total` (entero ≥ 0), `offset`, `limit`, `tiempo_ms` (entero ≥ 0) y `facetas` (`Option`; ausente permitido). `SearchHit` MUST exponer `id` (ULID de 26 caracteres sin prefijo de tabla) y MAY hidratar los campos 4.11.1 del documento (`sku`, `nombre`, `tipo`, `marca`, `codigos`, `atributos_texto`, `precio`, `unidad_medida`, `stock_disponible`, `con_stock`, `ubicacion`, `imagen`) con valores por defecto vacíos/cero cuando no hay documento. `SearchHit.resaltado` MUST ser `Option` con `nombre` y `descripcion` opcionales (fragmentos `_formatted`; ausente = sin resaltado). MUST NOT exigir facetas.

Construir `SearchQuery` con `q` de más de 200 caracteres, `offset` > 1000, `limit` 0 o `limit` > 50 MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT producir un `SearchQuery`. Un `orden` fuera de la lista MUST fallar igual. MUST NOT mapear a HTTP ni a `SearchResponse`.

#### Scenario: SearchQuery válida (happy)

- **WHEN** se construye un `SearchQuery` con `q` `"perno"`, `offset` 0, `limit` 20, `orden` `relevancia` y filtros vacíos
- **THEN** la construcción termina sin error
- **AND** `q` es `"perno"`
- **AND** `offset` es 0
- **AND** `limit` es 20
- **AND** `orden` es `relevancia`

#### Scenario: q vacía permitida (happy)

- **WHEN** se construye un `SearchQuery` con `q` vacía, `offset` 0 y `limit` 20
- **THEN** la construcción termina sin error
- **AND** `q` es la cadena vacía

#### Scenario: SearchFilters con categoria, marca, stock y atributo (happy)

- **WHEN** se construye un `SearchFilters` con un `categoria_ids` de un `Id<Categoria>`, un `marca_id` presente, `con_stock` verdadero y `atributos` `medida` → `"M8"`
- **THEN** la construcción termina sin error
- **AND** los cuatro filtros expuestos coinciden con esos valores
- **AND** la clave de atributo no lleva prefijo `attr_`

#### Scenario: SearchPage con hit y facetas ausentes (happy)

- **WHEN** se construye un `SearchPage` con un `SearchHit` cuyo `id` es un ULID de 26 caracteres, `estimado_total` 1, `offset` 0, `limit` 20, `tiempo_ms` 0 y `facetas` ausentes
- **THEN** la construcción termina sin error
- **AND** `hits` tiene un elemento
- **AND** `facetas` está ausente

#### Scenario: SearchHit con resaltado (happy)

- **WHEN** se construye un `SearchHit` con `id` ULID y `resaltado.nombre` `"<mark>Perno</mark> hexagonal"`
- **THEN** la construcción termina sin error
- **AND** `resaltado` está presente
- **AND** `resaltado.nombre` contiene `Perno`

#### Scenario: q demasiado larga (unhappy)

- **WHEN** se intenta construir un `SearchQuery` con `q` de 201 caracteres
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `SearchQuery`

#### Scenario: offset o limit fuera de rango (unhappy)

- **WHEN** se intenta construir un `SearchQuery` con `offset` 1001, o con `limit` 0, o con `limit` 51
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `SearchQuery`

#### Scenario: orden desconocido (unhappy)

- **WHEN** se intenta construir un `SearchQuery` con `orden` `"relevance"` u otro valor fuera de `relevancia` / `precio_asc` / `precio_desc` / `stock_desc`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `SearchQuery`

#### Scenario: clave de atributo con prefijo attr_ (unhappy)

- **WHEN** se intenta construir un `SearchQuery` cuyo `filtros.atributos` tiene la clave `"attr_medida"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `SearchQuery`

#### Scenario: clave de atributo inválida (unhappy)

- **WHEN** se intenta construir un `SearchQuery` cuyo `filtros.atributos` tiene la clave `"Medida"` o `"medida-mm"` o vacía
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `SearchQuery`

### Requirement: Identificador de índice y documento

El crate `domain` MUST validar el identificador de índice: MUST ser no vacío tras recortar espacios. Construir un identificador vacío o solo espacios MUST fallar con `DomainError` `code` `invalid_input`. `swap_indexes` con origen igual a destino MUST fallar con `DomainError` `code` `invalid_input`. El `id` de documento MUST ser un ULID Crockford de 26 caracteres **sin** `:` (mismas reglas que `Id<T>`). Un id con prefijo de tabla o vacío MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT producir el documento.

`IndexSettings` MUST poder construirse vacío (payload no-op para `apply_settings`). MUST poder construirse con las listas de settings: `searchableAttributes`, `rankingRules`, `filterableAttributes`, `sortableAttributes` y `typoTolerance.disableOnAttributes` (vectores de string). MUST NOT incluir `synonyms` ni `attributesToHighlight` (resaltado es T-03-12). MUST NOT hablar con Meilisearch. MUST NOT depender de `meilisearch-sdk`. El perfil 4.11.2 + H-03/H-04 (constantes y `attr_<clave>`) MUST vivir en el crate `search`, no en `domain`.

#### Scenario: índice productos (happy)

- **WHEN** se construye un identificador de índice `"productos"`
- **THEN** la construcción termina sin error
- **AND** el valor expuesto es `"productos"`

#### Scenario: documento con ULID (happy)

- **WHEN** se construye un documento de búsqueda con `id` `01J9ZKA7M3Q8R2T5V9X1Z4B6D8`
- **THEN** la construcción termina sin error
- **AND** `id` no contiene `:`

#### Scenario: IndexSettings vacío (happy)

- **WHEN** se construye un `IndexSettings` sin campos de 4.11.2
- **THEN** la construcción termina sin error
- **AND** el payload se considera vacío (no-op)

#### Scenario: IndexSettings con listas (happy)

- **WHEN** se construye un `IndexSettings` con `searchableAttributes` `["nombre"]` y `filterableAttributes` `["marca_id"]`
- **THEN** la construcción termina sin error
- **AND** las listas expuestas coinciden con esos valores
- **AND** el payload no se considera vacío

#### Scenario: índice vacío (unhappy)

- **WHEN** se intenta construir un identificador de índice vacío o solo espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`

#### Scenario: swap con el mismo índice (unhappy)

- **WHEN** se intenta `swap_indexes` con origen y destino iguales a `"productos"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`

#### Scenario: id de documento con prefijo de tabla (unhappy)

- **WHEN** se intenta construir un documento con `id` `producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce el documento

### Requirement: domain sigue sin SDKs

Tras agregar el puerto, el árbol `normal,build` del crate `domain` MUST NOT incluir `meilisearch-sdk`, `reqwest` ni otros packages prohibidos por [domain-deps](../domain-deps/spec.md). `scripts/check-domain-deps.sh` MUST terminar con código 0 sobre el repositorio. MUST NOT relajar `scripts/check-workspace.sh` ni la denylist de T-00-02.

#### Scenario: check-domain-deps sigue verde (happy, Listo cuando)

- **WHEN** se ejecuta `scripts/check-domain-deps.sh` sobre el repositorio con el puerto `TextSearch` presente
- **THEN** termina con código 0

#### Scenario: meilisearch-sdk en domain (unhappy)

- **WHEN** el árbol `normal,build` de `domain` incluye `meilisearch-sdk`
- **THEN** `scripts/check-domain-deps.sh` termina con código distinto de 0
- **AND** el mensaje nombra `meilisearch-sdk`

### Requirement: Tests del puerto TextSearch

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `text_search_trait_exposes_nine_ops` | `vortex-core/crates/domain/` (unidad) | happy |
| `text_search_dummy_search_returns_page` | `vortex-core/crates/domain/` (unidad) | happy |
| `text_search_dummy_health_is_up` | `vortex-core/crates/domain/` (unidad) | happy |
| `search_query_constructs_perno` | `vortex-core/crates/domain/` (unidad) | happy |
| `search_query_allows_empty_q` | `vortex-core/crates/domain/` (unidad) | happy |
| `search_filters_constructs_categoria_marca_stock_attr` | `vortex-core/crates/domain/` (unidad) | happy |
| `search_page_constructs_hit_without_facets` | `vortex-core/crates/domain/` (unidad) | happy |
| `index_uid_constructs_productos` | `vortex-core/crates/domain/` (unidad) | happy |
| `search_document_constructs_ulid` | `vortex-core/crates/domain/` (unidad) | happy |
| `index_settings_constructs_empty` | `vortex-core/crates/domain/` (unidad) | happy |
| `index_settings_constructs_with_lists` | `vortex-core/crates/domain/` (unidad) | happy |
| `search_query_rejects_q_too_long` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `search_query_rejects_offset_or_limit_out_of_range` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `search_query_rejects_english_or_unknown_orden` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `text_search_dummy_search_unavailable` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `text_search_dummy_search_timeout` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `text_search_dummy_upsert_invalid_input` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `index_uid_rejects_blank` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `swap_indexes_rejects_same_uid` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `search_document_rejects_table_prefix` | `vortex-core/crates/domain/` (unidad) | unhappy |

`cargo test --workspace` MUST ejecutarlos. MUST NOT usar Meilisearch, `meilisearch-sdk` ni el crate `search`. El Listo cuando de T-00-02 se verifica con `scripts/check-domain-deps.sh` (el escenario `meilisearch-sdk en domain` ya lo cubre el harness de T-00-02; este ticket no agrega fixtures nuevas).

#### Scenario: la suite de TextSearch cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p domain` en `vortex-core/`
- **THEN** corren los tests de la tabla
- **AND** los de construcción válida, dummy `search`/`health`, el *trait* e `IndexSettings` con listas pasan
- **AND** los de query inválida, dummy `Unavailable`/`Timeout`/`InvalidInput`, índice vacío, swap idéntico y prefijo de tabla pasan

### Requirement: SearchDocument 4.11.1 y build_search_document

El crate `domain` MUST exponer `SearchDocument` con los campos del documento Meilisearch [4.11.1](../../../../../docs/4-modelo.md#4111-documento): `id` (ULID sin prefijo), `sku`, `nombre`, `tipo`, `marca` y `marca_id` opcionales, `categoria` con `lvl0`…`lvl3` según profundidad de `ruta`, `categoria_ids`, `codigos` (solo valores string), `atributos_texto`, mapa plano `attr_<clave>` para facetables, `descripcion` opcional, `precio` (número JSON), `unidad_medida` (wire español), `stock_disponible`, `con_stock`, `ubicacion` opcional formateada `P{n} · E-{e} · {pos}`, `imagen` MUST ser `null` en el MVP (sin T-02-28).

`build_search_document(producto, marca, categoria, defs)` MUST ser función pura sin I/O. `atributos_texto` MUST incluir solo atributos cuya `AtributoDef` tiene `buscable = true`, con formato `"{etiqueta}: {valor}"` y unidad si la def la define. `attr_<clave>` MUST incluir solo defs con `facetable = true`. `con_stock` MUST ser `producto.stock > 0`.

`SearchDocument::try_new(id)` y `try_from_ulid_str` MUST seguir construyendo un documento mínimo para no romper tests del adaptador.

Inconsistencias marca/categoría/def ausente MUST fallar con `invalid_input`.

#### Scenario: Snapshot producto ejemplo 4.4.4 (happy, Listo cuando)

- **WHEN** se llama `build_search_document` con el producto de ejemplo de 4.4.4 / 4.11.1
- **THEN** la operación termina sin error
- **AND** la serialización JSON coincide con el objeto de [4.11.1](../../../../../docs/4-modelo.md#4111-documento)

#### Scenario: Ubicación formateada (happy)

- **WHEN** el producto tiene `ubicacion` pasillo/estante/posición `3`/`B`/`12`
- **THEN** `ubicacion` en el documento es `P3 · E-B · 12`

#### Scenario: Sin marca en producto (happy)

- **WHEN** `producto.marca` es `None` y `marca` es `None`
- **THEN** `marca` y `marca_id` en el documento son null

#### Scenario: Stock cero (happy)

- **WHEN** `producto.stock` es 0
- **THEN** `con_stock` es false

#### Scenario: Marca inconsistente (unhappy)

- **WHEN** `producto.marca` no coincide con `marca` pasada
- **THEN** la operación falla con `invalid_input`

#### Scenario: Categoría inconsistente (unhappy)

- **WHEN** `producto.categoria` ≠ `categoria.id`
- **THEN** la operación falla con `invalid_input`

#### Scenario: Def de atributo ausente (unhappy)

- **WHEN** un atributo del producto no tiene def en `defs`
- **THEN** la operación falla con `invalid_input`

### Requirement: Tests build_search_document

La suite MUST incluir: `build_search_document_snapshot_4_4_4`, `build_search_document_formats_ubicacion`, `build_search_document_atributos_texto_only_buscable`, `build_search_document_attr_keys_only_facetable`, `build_search_document_imagen_is_null`, `build_search_document_categoria_ids_ancestors_and_self`, `build_search_document_con_stock_false_when_zero`, `build_search_document_without_marca`, `build_search_document_rejects_marca_mismatch`, `build_search_document_rejects_categoria_mismatch`, `build_search_document_rejects_missing_atributo_def` en `vortex-core/crates/domain/`.

#### Scenario: Suite build_search_document (happy, Listo cuando)

- **WHEN** se ejecuta `cargo test -p domain build_search_document`
- **THEN** todos los tests anteriores corren y terminan con código 0

### Requirement: productos_index_settings en domain

El crate `domain` MUST exponer `productos_index_settings` (constructor puro de `IndexSettings` para el perfil `productos`) que acepta las `clave` de `AtributoDef` facetables **sin** prefijo `attr_`. MUST armar las mismas listas que [4.11.2](../../../../../docs/4-modelo.md#4112-configuración-del-índice) + H-03 + H-04. MUST NOT consultar SurrealDB ni `AttributeDefRepo`. MUST NOT incluir `synonyms` (T-03-27). MUST NOT abrir HTTP ni depender de `meilisearch-sdk`.

`searchableAttributes` (orden de peso): `nombre`, `tipo`, `sku`, `codigos`, `marca`, `atributos_texto`, `descripcion`.

`rankingRules`: `words`, `typo`, `proximity`, `attribute`, `sort`, `exactness`, `stock_disponible:desc`.

`filterableAttributes` MUST incluir: `categoria.lvl0`, `categoria.lvl1`, `categoria.lvl2`, `categoria.lvl3`, `marca`, `con_stock`, `marca_id`, `categoria_ids`, `tipo`, y `attr_<clave>` por cada clave facetable recibida.

`sortableAttributes`: `precio`, `stock_disponible`.

`typoTolerance.disableOnAttributes`: `sku`, `codigos`.

Una clave vacía, con espacios, con formato distinto al de `AtributoDef.clave` (minúsculas ASCII, dígitos, `_`, primer carácter letra) o que ya lleve el prefijo `attr_` MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT producir `IndexSettings`. Cero claves facetables es válido (solo los filtrables fijos).

El crate `sync` MUST usar esta función (no el crate `search`) para `apply_settings` en el *fan-out* de `atributo_def`.

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

### Requirement: Tests de productos_index_settings en domain

Antes de archive, la suite MUST incluir estos tests (nombres estables) en `vortex-core/crates/domain/`. Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `productos_index_settings_includes_4_11_2_h03_h04` | `vortex-core/crates/domain/` | happy |
| `productos_index_settings_omits_synonyms` | `vortex-core/crates/domain/` | happy |
| `productos_index_settings_rejects_invalid_clave` | `vortex-core/crates/domain/` | unhappy |

`cargo test --workspace` MUST ejecutarlos. MUST NOT usar SurrealDB, `search` ni `meilisearch-sdk`.

#### Scenario: La suite de productos_index_settings cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p domain productos_index_settings`
- **THEN** corren los tests de la tabla
- **AND** los de perfil con `medida` y omisión de `synonyms` pasan
- **AND** el de clave inválida pasa

### Requirement: Tests de SearchHit y claves de atributo

Antes de archive, la suite MUST incluir estos tests (nombres estables) en `vortex-core/crates/domain/`. Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `search_hit_constructs_with_resaltado` | `vortex-core/crates/domain/` | happy |
| `search_query_rejects_attr_prefix_clave` | `vortex-core/crates/domain/` | unhappy |
| `search_query_rejects_invalid_atributo_clave` | `vortex-core/crates/domain/` | unhappy |

`cargo test --workspace` MUST ejecutarlos. MUST NOT usar Meilisearch ni el crate `search`.

#### Scenario: la suite de hit y claves cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p domain search_hit_constructs_with_resaltado search_query_rejects_attr_prefix_clave search_query_rejects_invalid_atributo_clave`
- **THEN** corren los tests de la tabla
- **AND** el de `resaltado` pasa
- **AND** los de clave `attr_` e inválida pasan
