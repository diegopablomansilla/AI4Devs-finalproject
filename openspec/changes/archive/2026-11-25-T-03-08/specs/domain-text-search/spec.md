# Spec Delta

Ticket: T-03-08. HU: HU-03-01.

## ADDED Requirements

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
