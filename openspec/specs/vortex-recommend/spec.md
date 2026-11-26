# vortex-recommend Specification

Ticket padre: T-04-05. HU: HU-04-01, HU-04-02, HU-04-03.

## Purpose

Orquesta en el crate `recommend` el ABM de relaciones curadas entre productos (crear, editar, eliminar, listar salientes y entrantes) sobre `RelationRepo` y `ProductRepo`, con fachada object-safe para el API Gateway, sin HTTP ni SurrealDB, según DC-05.

## Requirements

### Requirement: Casos de uso de ABM de relaciones

El crate `recommend` MUST exponer casos de uso de relaciones que delegan en `RelationRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT vivir en `catalog` ([DC-05](../../../docs/3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes)).

Crear MUST aceptar `NewRelacion` y reenviarlo al puerto. El resultado de éxito MUST ser las aristas creadas con resumen del destino (`RelacionConDestino`): una si la relación es unidireccional; las dos del grupo si es bidireccional. `bidireccional` ausente MUST usar `default_bidireccional(tipo)`. Los errores del puerto (`self_relation`, `duplicate_relation`, `not_found`, `invalid_input`) MUST propagarse sin remapeo. MUST NOT hidratar stock ni precio reales en el ABM (eso es la consulta de recomendaciones).

Actualizar MUST aceptar `UpdateRelacion` y delegar en el puerto. El resultado de éxito MUST incluir la arista pedida y, si tiene `grupo_bidireccional`, la otra arista del grupo, cada una con `DestinoResumen`. Eliminar MUST delegar en `RelationRepo::delete`.

Listar MUST aceptar `Id<Producto>`, un `tipo` opcional y una dirección (`saliente` por defecto, `entrante`). Antes de consultar el grafo, MUST cargar el producto con `ProductRepo::get`: si no existe, MUST fallar con `not_found`. El resultado de éxito MUST ser la lista plana de `RelacionConDestino` (salientes vía `list_for_product`, entrantes vía `list_incoming_for_product`), filtrada por `tipo` si viene, sin filtrar destinos inactivos ni `stock` 0. Un producto existente sin aristas MUST devolver lista vacía (no `not_found`).

MUST NOT depender de `surrealdb` ni de `storage`. La consulta 4.6.2 MUST vivir en el requirement «Consulta de recomendaciones (4.6.2)», no en el ABM.

#### Scenario: Complementario Perno M8 a Arandela M8 (happy, HU-04-01)

- **WHEN** existen dos productos distintos y el caso de uso crea `tipo` `complementa`, origen Perno, destino Arandela, `prioridad` 1
- **THEN** la operación termina sin error
- **AND** el resultado tiene una arista `complementa` de Perno hacia Arandela
- **AND** el resumen del destino expone el `sku` y `nombre` de Arandela
- **AND** no se crea arista inversa

#### Scenario: Similar bidireccional por defecto (happy, HU-04-02)

- **WHEN** el caso de uso crea `tipo` `similar_a` entre A y B con `bidireccional` ausente
- **THEN** la operación termina sin error
- **AND** el resultado tiene dos aristas del mismo grupo (A→B y B→A)
- **AND** listar A incluye a B y listar B incluye a A

#### Scenario: Edición del grupo desde B (happy, HU-04-02)

- **WHEN** existe un par bidireccional A similar B
- **AND** el caso de uso actualiza la arista B→A con `prioridad` 3
- **THEN** la operación termina sin error
- **AND** el resultado incluye ambas aristas con `prioridad` 3

#### Scenario: Listado admin incluye destino inactivo (happy, HU-04-03)

- **WHEN** un producto existente tiene aristas de los tres tipos, una hacia un destino inactivo
- **AND** el caso de uso lista salientes sin filtro de tipo
- **THEN** la operación termina sin error
- **AND** aparecen las tres aristas
- **AND** el destino inactivo tiene `activo` falso

#### Scenario: Relación consigo mismo (unhappy, HU-04-01)

- **WHEN** el caso de uso crea una relación con el mismo `Id<Producto>` en origen y destino
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `self_relation`
- **AND** no se persiste arista

#### Scenario: Relación duplicada (unhappy, HU-04-01)

- **WHEN** ya existe `complementa` de A hacia B
- **AND** el caso de uso la crea de nuevo
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `duplicate_relation`

#### Scenario: Listar producto inexistente (unhappy)

- **WHEN** el `ProductRepo` no tiene el id pedido
- **AND** el caso de uso lista relaciones de ese id
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

#### Scenario: recommend no vive en catalog (unhappy, DC-05)

- **WHEN** se inspecciona el crate `catalog`
- **THEN** no declara el servicio de relaciones
- **AND** el ABM está en `recommend`

### Requirement: Fachada RelationGateway object-safe

El crate `recommend` MUST exponer un trait object-safe `RelationGateway` (`Send + Sync`) con `create`, `update`, `delete` y `list`, inyectable como `Arc<dyn RelationGateway>` desde el Gateway. `RelationService` MUST implementar ese trait. Los tests de `recommend` MUST usar repositorios en memoria (MUST NOT abrir SurrealDB).

#### Scenario: RelationGateway es object-safe (happy)

- **WHEN** se construye un `Arc<dyn RelationGateway>` sobre el servicio con repos en memoria
- **THEN** se puede clonar el `Arc` y llamar `create` / `list` sin error de object safety

#### Scenario: Tests de recommend no abren SurrealDB (unhappy)

- **WHEN** se ejecutan los tests del package `recommend`
- **THEN** no se abre un `storage::Db`
- **AND** los fakes cubren `RelationRepo` y `ProductRepo`

### Requirement: Consulta de recomendaciones (4.6.2)

El crate `recommend` MUST exponer un caso de uso de consulta de recomendaciones que delega en `RelationRepo::recommendations_for_product` y MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT vivir en `catalog` ([DC-05](../../../../../docs/3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes)). MUST NOT registrar rutas ni DTO JSON (el mapper HTTP vive en `api`, T-04-07).

El caso de uso MUST aceptar `Id<Producto>`. El resultado de éxito MUST ser `Recomendaciones` tal como lo devuelve el puerto (tres bloques, `destacar_sustitutos`, stock/precio/ubicación reales). Los errores del puerto (`not_found`) MUST propagarse sin remapeo. MUST NOT filtrar de nuevo en el servicio (los filtros `activo` / `stock > 0` viven en el adaptador). MUST NOT hidratar imagen. Un producto existente sin aristas MUST devolver vectores vacíos (no `not_found`).

#### Scenario: Tres bloques ordenados por prioridad (happy, HU-04-05)

- **WHEN** el puerto devuelve similares, sustitutos y complementarios para "Perno M8", ordenados por prioridad
- **AND** el caso de uso consulta ese producto
- **THEN** la operación termina sin error
- **AND** el resultado tiene los tres bloques
- **AND** cada bloque conserva el orden del puerto

#### Scenario: Productos inactivos no aparecen (happy, HU-04-05, Listo cuando)

- **WHEN** el puerto excluye un destino dado de baja
- **AND** el caso de uso consulta el origen
- **THEN** el resultado no incluye ese destino

#### Scenario: Producto sin stock destaca sustitutos con stock (happy, HU-04-06)

- **WHEN** el puerto devuelve `destacar_sustitutos` verdadero y un solo sustituto con stock 12 (el de stock 0 ya filtrado)
- **AND** el caso de uso consulta "Perno M8 Fixser"
- **THEN** `destacar_sustitutos` es verdadero
- **AND** `sustitutos` tiene un ítem con stock 12

#### Scenario: Producto con stock no destaca (happy, HU-04-06)

- **WHEN** el puerto devuelve `destacar_sustitutos` falso
- **AND** el caso de uso consulta ese producto
- **THEN** `destacar_sustitutos` es falso

#### Scenario: Producto inexistente (unhappy)

- **WHEN** el puerto falla con `not_found`
- **AND** el caso de uso consulta ese id
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

#### Scenario: recommend no vive en catalog (unhappy, DC-05)

- **WHEN** se inspecciona el crate `catalog`
- **THEN** no declara el servicio de recomendaciones
- **AND** la consulta 4.6.2 está en `recommend`

### Requirement: Fachada RecommendationGateway object-safe

El crate `recommend` MUST exponer un trait object-safe `RecommendationGateway` (`Send + Sync`) con `get` (consulta de recomendaciones), inyectable como `Arc<dyn RecommendationGateway>` desde el Gateway (T-04-07). El servicio de consulta MUST implementar ese trait. Los tests de `recommend` MUST usar un `RelationRepo` en memoria (MUST NOT abrir SurrealDB). MUST NOT registrar HTTP.

#### Scenario: RecommendationGateway es object-safe (happy)

- **WHEN** se construye un `Arc<dyn RecommendationGateway>` sobre el servicio con repo en memoria
- **THEN** se puede clonar el `Arc` y llamar `get` sin error de object safety

#### Scenario: Tests de consulta no abren SurrealDB (unhappy)

- **WHEN** se ejecutan los tests del package `recommend` de este change
- **THEN** no se abre un `storage::Db`
- **AND** el fake cubre `recommendations_for_product`

#### Scenario: recommend no registra el endpoint HTTP (unhappy, DC-05)

- **WHEN** se inspecciona el crate `recommend`
- **THEN** no declara rutas Axum ni DTO JSON `Recommendations`
- **AND** el mapper HTTP está en `api`
