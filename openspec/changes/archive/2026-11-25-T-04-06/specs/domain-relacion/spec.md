# Spec Delta

Ticket padre: T-04-06. Change: T-04-06. HU: HU-04-05, HU-04-06.

## ADDED Requirements

### Requirement: Tipos Recomendaciones y ProductoRecomendado

El crate `domain` MUST exponer `ProductoRecomendado` con nombres en español ([5.3](../../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `relacion_id` (`Id<Relacion>`), `id` (`Id<Producto>`), `sku`, `nombre`, `stock` (`Decimal`), `precio` (`Decimal`), `ubicacion` (`Option<string>`: línea formateada como el documento de búsqueda, p. ej. `P3 · E-B · 12`), `prioridad` (`int` ≥ 1), `nota` (`Option<string>`). MUST NOT incluir `imageUrl` ni campo de imagen (T-02-28 es v2; el recableo MVP de T-04-06 deja la imagen ausente). MUST NOT usar nombres de API en inglés (`relationId`, `name`, `price`, `location`, `priority`, `note`, `highlightSubstitutes`).

El crate `domain` MUST exponer `Recomendaciones` con `producto_id` (`Id<Producto>`), `destacar_sustitutos` (`bool`), `similares`, `sustitutos` y `complementarios` (cada uno `Vec<ProductoRecomendado>`). MUST NOT persistir. MUST NOT mapear a HTTP.

#### Scenario: ProductoRecomendado cubre el ítem del panel (happy, HU-04-05)

- **WHEN** se construye un `ProductoRecomendado` con `sku`, `nombre`, `stock` `"12"`, `precio` `"5.00"`, `ubicacion` `"P3 · E-B · 12"`, `prioridad` 1 y `nota` ausente
- **THEN** la construcción termina sin error
- **AND** `relacion_id` e `id` no contienen `:`
- **AND** `ubicacion` es `"P3 · E-B · 12"`
- **AND** no hay campo de imagen ni `imageUrl`

#### Scenario: Recomendaciones agrupa los tres bloques (happy, HU-04-05)

- **WHEN** se construye un `Recomendaciones` con un ítem en `similares`, uno en `sustitutos`, uno en `complementarios` y `destacar_sustitutos` verdadero
- **THEN** los tres vectores exponen esos ítems
- **AND** `destacar_sustitutos` es verdadero

### Requirement: Puerto recommendations_for_product

El crate `domain` MUST añadir al puerto `RelationRepo` la operación `recommendations_for_product` que acepta `Id<Producto>` y cuyo resultado de éxito MUST ser `Recomendaciones`. Los errores MUST ser `DomainError`. MUST NOT persistir ni abrir SurrealDB. MUST NOT mapear a HTTP.

#### Scenario: El puerto declara recommendations_for_product (happy)

- **WHEN** se inspecciona el puerto `RelationRepo` en `domain`
- **THEN** expone `recommendations_for_product`
- **AND** acepta `Id<Producto>`
- **AND** el resultado de éxito usa `Recomendaciones`
- **AND** los resultados de error usan `DomainError`

## Tests previstos

| Test | Módulo | Tipo |
|---|---|---|
| `producto_recomendado_covers_panel_item` | `crates/domain/src/relacion.rs` | unitario / happy |
| `recomendaciones_groups_three_blocks` | `crates/domain/src/relacion.rs` | unitario / happy |
| `relation_repo_trait_exposes_recommendations_for_product` | `crates/domain/src/relacion.rs` | unitario / happy |
