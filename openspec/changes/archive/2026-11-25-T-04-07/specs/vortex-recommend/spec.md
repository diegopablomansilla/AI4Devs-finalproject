# Spec Delta

Ticket: T-04-07. HU: HU-04-05.

## MODIFIED Requirements

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
