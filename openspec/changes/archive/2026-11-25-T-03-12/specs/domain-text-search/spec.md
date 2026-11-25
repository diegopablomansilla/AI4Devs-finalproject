# Spec Delta

Ticket: T-03-12. HU: HU-03-03.

## MODIFIED Requirements

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

## ADDED Requirements

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
