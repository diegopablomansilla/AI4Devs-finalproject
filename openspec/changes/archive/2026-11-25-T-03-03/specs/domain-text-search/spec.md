# Spec Delta

Ticket padre: T-03-03. Change: T-03-03. HU: HU-03-01.

## Purpose

Define el puerto hexagonal `TextSearch` y los tipos de consulta/página de búsqueda de texto en el crate `domain`, para que adaptadores y casos de uso hablen el mismo contrato sin acoplar el dominio a Meilisearch.

## ADDED Requirements

### Requirement: Puerto TextSearch en domain

El crate `domain` MUST exponer el puerto `TextSearch` con exactamente estas operaciones: `upsert_documents`, `delete_documents`, `search`, `facet_search`, `apply_settings`, `create_index`, `swap_indexes`, `delete_index` y `health`. Los errores de las operaciones MUST ser `DomainError`. El puerto MUST NOT persistir, MUST NOT abrir HTTP ni sockets, MUST NOT depender de `meilisearch-sdk` ni de `reqwest`, y MUST NOT mapear a `problem+json` ni a estado HTTP.

`upsert_documents` MUST aceptar un identificador de índice y una lista de documentos identificados por ULID de 26 caracteres **sin** prefijo de tabla. `delete_documents` MUST aceptar un identificador de índice y una lista de esos mismos ids. `search` MUST aceptar un identificador de índice y un `SearchQuery`; el resultado de éxito MUST ser `SearchPage`. `facet_search` MUST aceptar un identificador de índice y un `SearchQuery`; el resultado de éxito MUST ser `SearchPage`. `apply_settings` MUST aceptar un identificador de índice y un `IndexSettings`. `create_index` y `delete_index` MUST aceptar un identificador de índice. `swap_indexes` MUST aceptar dos identificadores de índice (origen y destino). `health` MUST devolver `ComponentHealth` de éxito o `DomainError`.

MUST NOT implementar un adaptador Meilisearch (T-03-04). MUST NOT aplicar la configuración 4.11.2 (T-03-05). MUST NOT construir el documento 4.11.1 (T-03-06).

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
- `SearchFilters`: `categoria_ids` (lista de `Id<Categoria>`; vacía = sin filtro de categoría), `marca_id` (`Option<Id<Marca>>`), `con_stock` (`Option<bool>`; ausente = ambos) y `atributos` (mapa `clave` → valor string; `clave` es la de `AtributoDef`, **sin** prefijo `attr_`). MUST NOT incluir texto de búsqueda.
- `SearchPage`: `hits` (lista de `SearchHit`), `estimado_total` (entero ≥ 0), `offset`, `limit`, `tiempo_ms` (entero ≥ 0) y `facetas` (`Option`; ausente permitido). `SearchHit` MUST exponer `id` (ULID de 26 caracteres sin prefijo de tabla). MUST NOT exigir los campos 4.11.1 (`sku`, `nombre`, `imagen`, …) en este ticket.

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

### Requirement: Identificador de índice y documento

El crate `domain` MUST validar el identificador de índice: MUST ser no vacío tras recortar espacios. Construir un identificador vacío o solo espacios MUST fallar con `DomainError` `code` `invalid_input`. `swap_indexes` con origen igual a destino MUST fallar con `DomainError` `code` `invalid_input`. El `id` de documento MUST ser un ULID Crockford de 26 caracteres **sin** `:` (mismas reglas que `Id<T>`). Un id con prefijo de tabla o vacío MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT producir el documento. `IndexSettings` MUST poder construirse vacío (la configuración 4.11.2 es T-03-05). MUST NOT hablar con Meilisearch.

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

Tras agregar el puerto, el árbol `normal,build` del crate `domain` MUST NOT incluir `meilisearch-sdk`, `reqwest` ni otros packages prohibidos por [domain-deps](../../../../specs/domain-deps/spec.md). `scripts/check-domain-deps.sh` MUST terminar con código 0 sobre el repositorio. MUST NOT relajar `scripts/check-workspace.sh` ni la denylist de T-00-02.

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
- **AND** los de construcción válida, dummy `search`/`health` y el *trait* pasan
- **AND** los de query inválida, dummy `Unavailable`/`Timeout`/`InvalidInput`, índice vacío, swap idéntico y prefijo de tabla pasan
