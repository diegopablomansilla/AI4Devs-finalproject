# Spec Delta

Ticket padre: T-03-05. HU: HU-03-04, HU-03-07.

## MODIFIED Requirements

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
