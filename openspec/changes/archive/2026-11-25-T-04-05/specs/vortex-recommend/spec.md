# Spec Delta

Ticket padre: T-04-05. Change: T-04-05. HU: HU-04-01, HU-04-02, HU-04-03.

## Purpose

Orquesta en el crate `recommend` el ABM de relaciones curadas entre productos (crear, editar, eliminar, listar salientes y entrantes) sobre `RelationRepo` y `ProductRepo`, con fachada object-safe para el API Gateway, sin HTTP ni SurrealDB, según DC-05.

## ADDED Requirements

### Requirement: Casos de uso de ABM de relaciones

El crate `recommend` MUST exponer casos de uso de relaciones que delegan en `RelationRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT vivir en `catalog` ([DC-05](../../../../../docs/3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes)).

Crear MUST aceptar `NewRelacion` y reenviarlo al puerto. El resultado de éxito MUST ser las aristas creadas con resumen del destino (`RelacionConDestino`): una si la relación es unidireccional; las dos del grupo si es bidireccional. `bidireccional` ausente MUST usar `default_bidireccional(tipo)`. Los errores del puerto (`self_relation`, `duplicate_relation`, `not_found`, `invalid_input`) MUST propagarse sin remapeo. MUST NOT hidratar stock ni precio reales (T-04-06).

Actualizar MUST aceptar `UpdateRelacion` y delegar en el puerto. El resultado de éxito MUST incluir la arista pedida y, si tiene `grupo_bidireccional`, la otra arista del grupo, cada una con `DestinoResumen`. Eliminar MUST delegar en `RelationRepo::delete`.

Listar MUST aceptar `Id<Producto>`, un `tipo` opcional y una dirección (`saliente` por defecto, `entrante`). Antes de consultar el grafo, MUST cargar el producto con `ProductRepo::get`: si no existe, MUST fallar con `not_found`. El resultado de éxito MUST ser la lista plana de `RelacionConDestino` (salientes vía `list_for_product`, entrantes vía `list_incoming_for_product`), filtrada por `tipo` si viene, sin filtrar destinos inactivos ni `stock` 0. Un producto existente sin aristas MUST devolver lista vacía (no `not_found`).

MUST NOT implementar la consulta de recomendaciones 4.6.2. MUST NOT depender de `surrealdb` ni de `storage`.

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
