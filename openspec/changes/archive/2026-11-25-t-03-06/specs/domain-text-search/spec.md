# Spec Delta

Ticket padre: T-03-06. HU: HU-03-01.

## MODIFIED Requirements

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

## ADDED Requirements

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