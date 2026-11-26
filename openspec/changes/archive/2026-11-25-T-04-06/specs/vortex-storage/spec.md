# Spec Delta

Ticket padre: T-04-06. Change: T-04-06. HU: HU-04-05, HU-04-06.

## ADDED Requirements

### Requirement: Adaptador RelationRepo recommendations_for_product

El crate `storage` MUST implementar `RelationRepo::recommendations_for_product` contra las tablas `similar_a`, `sustituible_por` y `complementa` con **una** consulta al grafo ([4.6.2](../../../../../docs/4-modelo.md#462-consulta-de-recomendaciones)). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT emitir evento de outbox. MUST NOT reindexar Meilisearch. MUST NOT cambiar el esquema `0007`. MUST NOT declarar el adaptador en `catalog` ni en `recommend`. MUST NOT usar `list_for_product` ni N+1 `ProductRepo::get` para armar los bloques.

Si el `Id<Producto>` no existe, MUST fallar con `DomainError` `code` `not_found`. Un producto existente sin aristas MUST devolver los tres vectores vacíos (no `not_found`); `destacar_sustitutos` MUST ser verdadero si y solo si `stock` del producto consultado es 0.

El resultado de éxito MUST ser `Recomendaciones` con `producto_id` igual al pedido y tres bloques ordenados por `prioridad` ascendente (1 = más alta) y, a igualdad, por `relacion_id`:

- `similares`: aristas salientes `similar_a` con destino `activo = true`
- `sustitutos`: aristas salientes `sustituible_por` con destino `activo = true` **y** `stock > 0`
- `complementarios`: aristas salientes `complementa` con destino `activo = true`

Cada `ProductoRecomendado` MUST hidratar `relacion_id`, `id`, `sku`, `nombre`, `stock`, `precio`, `prioridad` y `nota` del destino/arista, y `ubicacion` como string formateado (`P{pasillo} · E-{estante} · {posicion}`, omitiendo partes vacías; `None` si no hay ubicación). MUST NOT hidratar imagen. Destinos inactivos MUST NOT aparecer en ningún bloque. Un sustituto con `stock` 0 MUST NOT aparecer en `sustitutos`. Un similar o complementario activo con `stock` 0 MUST seguir apareciendo. `list_for_product` MUST seguir incluyendo destinos inactivos y sin stock (el listado admin no cambia).

#### Scenario: Tres bloques ordenados por prioridad (happy, HU-04-05)

- **WHEN** "Perno M8" existe con al menos una arista saliente activa de cada tipo (`similar_a`, `sustituible_por`, `complementa`) y dos similares con `prioridad` 2 y 1
- **AND** se llama `recommendations_for_product` con ese producto
- **THEN** la operación termina sin error
- **AND** `similares`, `sustitutos` y `complementarios` contienen esas aristas
- **AND** cada bloque está ordenado por `prioridad` ascendente
- **AND** cada ítem expone `sku`, `nombre`, `stock`, `precio` y `relacion_id` sin prefijo de tabla

#### Scenario: Productos inactivos no aparecen (happy, HU-04-05, Listo cuando)

- **WHEN** existe una arista hacia un producto dado de baja y otra hacia un destino activo del mismo tipo
- **AND** se llama `recommendations_for_product`
- **THEN** la operación termina sin error
- **AND** el destino inactivo no aparece en ningún bloque
- **AND** el destino activo sí aparece

#### Scenario: Producto sin stock destaca sustitutos con stock (happy, HU-04-06)

- **WHEN** "Perno M8 Fixser" tiene `stock` 0 y dos destinos `sustituible_por` activos con stock 12 y 0
- **AND** se llama `recommendations_for_product`
- **THEN** la operación termina sin error
- **AND** `destacar_sustitutos` es verdadero
- **AND** `sustitutos` contiene solo el destino con stock 12

#### Scenario: Producto con stock no destaca sustitutos (happy, HU-04-06)

- **WHEN** el producto consultado tiene `stock` mayor a 0 y un sustituto activo con stock > 0
- **AND** se llama `recommendations_for_product`
- **THEN** la operación termina sin error
- **AND** `destacar_sustitutos` es falso
- **AND** el sustituto con stock aparece en `sustitutos`

#### Scenario: Ubicación formateada y sin imagen (happy)

- **WHEN** un destino activo tiene `ubicacion` pasillo `"3"`, estante `"B"`, posicion `"12"`
- **AND** se llama `recommendations_for_product`
- **THEN** ese ítem tiene `ubicacion` `"P3 · E-B · 12"`
- **AND** el ítem no expone imagen ni `imageUrl`

#### Scenario: Producto existente sin aristas (happy)

- **WHEN** existe un producto sin aristas salientes y se llama `recommendations_for_product`
- **THEN** la operación termina sin error
- **AND** `similares`, `sustitutos` y `complementarios` están vacíos
- **AND** `destacar_sustitutos` refleja el `stock` del producto (verdadero si es 0)

#### Scenario: Producto inexistente (unhappy)

- **WHEN** se llama `recommendations_for_product` con un `Id<Producto>` que no existe
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

## Tests previstos

| Test | Módulo | Tipo |
|---|---|---|
| `relation_repo_recommendations_three_blocks_ordered` | `crates/storage/tests/relacion_repo.rs` | integración / happy |
| `relation_repo_recommendations_excludes_inactive_target` | `crates/storage/tests/relacion_repo.rs` | integración / happy |
| `relation_repo_recommendations_highlights_substitutes_when_stock_zero` | `crates/storage/tests/relacion_repo.rs` | integración / happy |
| `relation_repo_recommendations_does_not_highlight_when_in_stock` | `crates/storage/tests/relacion_repo.rs` | integración / happy |
| `relation_repo_recommendations_includes_formatted_location` | `crates/storage/tests/relacion_repo.rs` | integración / happy |
| `relation_repo_recommendations_empty_when_no_edges` | `crates/storage/tests/relacion_repo.rs` | integración / happy |
| `relation_repo_recommendations_unknown_product_is_not_found` | `crates/storage/tests/relacion_repo.rs` | integración / unhappy |
