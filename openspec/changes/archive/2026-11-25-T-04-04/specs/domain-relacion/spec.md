# Spec Delta

Ticket padre: T-04-04. Change: T-04-04. HU: HU-04-02, HU-04-03.

## ADDED Requirements

### Requirement: DestinoResumen y RelacionesDeProducto

El crate `domain` MUST exponer `DestinoResumen` con `id` (`Id<Producto>`), `sku`, `nombre` y `activo`. MUST NOT incluir `precio`, `stock`, `tipo`, `marca`, `categoryPath` ni `imageUrl` (eso es `ProductSummary` en T-04-05). MUST NOT usar nombres de API en inglés (`sku`/`nombre`/`activo` coinciden con el modelo).

El crate `domain` MUST exponer `RelacionConDestino` con la arista (`Relacion`) y su `DestinoResumen`. MUST exponer `RelacionesDeProducto` con tres vectores: `similar_a`, `sustituible_por` y `complementa`, cada uno de `RelacionConDestino`. MUST NOT persistir. MUST NOT mapear a HTTP.

#### Scenario: DestinoResumen cubre identidad administrativa (happy, HU-04-03)

- **WHEN** se construye un `DestinoResumen` con `sku`, `nombre` y `activo` falso
- **THEN** la construcción termina sin error
- **AND** `activo` es falso
- **AND** el `id` no contiene `:`

#### Scenario: RelacionesDeProducto agrupa los tres tipos (happy, HU-04-03)

- **WHEN** se construye un `RelacionesDeProducto` con un ítem en cada vector
- **THEN** `similar_a`, `sustituible_por` y `complementa` exponen esos ítems
- **AND** cada ítem tiene `Relacion` y `DestinoResumen`

## MODIFIED Requirements

### Requirement: Puerto RelationRepo sin persistencia

El crate `domain` MUST exponer el puerto `RelationRepo` como *trait* con operaciones `create`, `update`, `delete` y `list_for_product`. Los `id` MUST ser `Id<Relacion>` / `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. El consumidor previsto del puerto MUST ser el Recommendation Service (`recommend`, [DC-05](../../../../../docs/3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes)), no Catalog.

`create` MUST aceptar `NewRelacion` con `origen`, `destino`, `tipo`, `prioridad`, `nota` y `bidireccional` (`Option<bool>`: ausente = `default_bidireccional(tipo)`). El resultado de éxito MUST ser `Relacion` (arista saliente origen→destino). MUST NOT crear la arista inversa en el crate `domain` (persistencia en `storage`).

`update` MUST aceptar `UpdateRelacion` con `id`, `tipo`, `prioridad` opcional y `nota` opcional. El resultado de éxito MUST ser `Relacion`. `delete` MUST aceptar `tipo` y `id`. `list_for_product` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `RelacionesDeProducto` (aristas salientes agrupadas por tipo, cada ítem con `DestinoResumen`; T-04-04 hidrata en el adaptador).

El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT ejecutar `RELATE`. MUST NOT traducir el índice UNIQUE a `duplicate_relation`. MUST NOT depender de `surrealdb` ni de otros crates del workspace. MUST NOT declarar el puerto en `catalog`.

#### Scenario: El puerto declara create, update, delete y list_for_product (happy)

- **WHEN** se inspecciona el puerto `RelationRepo` en `domain`
- **THEN** expone `create`, `update`, `delete` y `list_for_product`
- **AND** `create` acepta `NewRelacion` y el resultado de éxito usa `Relacion`
- **AND** `update` acepta `UpdateRelacion` y el resultado de éxito usa `Relacion`
- **AND** `delete` acepta `TipoRelacion` e `Id<Relacion>`
- **AND** `list_for_product` acepta `Id<Producto>` y el resultado de éxito usa `RelacionesDeProducto`
- **AND** los resultados de error usan `DomainError`

#### Scenario: NewRelacion omite bidireccional y usa el default del tipo (happy, HU-04-02)

- **WHEN** se construye un `NewRelacion` con `tipo` `similar_a` y `bidireccional` ausente
- **THEN** el default resuelto es verdadero
- **WHEN** se construye un `NewRelacion` con `tipo` `complementa` y `bidireccional` ausente
- **THEN** el default resuelto es falso

#### Scenario: domain no persiste relaciones (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `RELATE` sobre `similar_a`, `sustituible_por` o `complementa`

#### Scenario: RelationRepo no vive en catalog (unhappy, DC-05)

- **WHEN** se inspecciona el crate `catalog`
- **THEN** no declara el trait `RelationRepo`
- **AND** el puerto está en `domain` para consumo de `recommend`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `destino_resumen_exposes_sku_nombre_activo` | `domain` (`src/relacion.rs`) | unitario happy (HU-04-03) |
| `relaciones_de_producto_groups_three_types` | `domain` (`src/relacion.rs`) | unitario happy (HU-04-03) |
| `relation_repo_trait_exposes_crud` | `domain` (`src/relacion.rs`) | unitario happy (dummy; `list_for_product` → `RelacionesDeProducto`) |
