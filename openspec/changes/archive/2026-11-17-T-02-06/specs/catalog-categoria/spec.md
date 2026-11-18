# Spec Delta

## ADDED Requirements

### Requirement: Fachada object-safe y occupancy de producción

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) de los casos de uso de categoría (crear, get, listar, renombrar, mover, dar de baja, reactivar) para que el Gateway la inyecte como `Arc<dyn …>`. `CategoryService` MUST implementar esa fachada. MUST NOT mapear a HTTP ni abrir SurrealDB. MUST exponer un occupancy de producción que MUST devolver `false` en `has_active_products` (hasta T-02-18). MUST NOT consultar `producto`.

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye un `CategoryService` con un `CategoryRepo` y el occupancy de producción
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones
- **AND** `has_active_products` para cualquier id termina sin error y es `false`

#### Scenario: occupancy de producción no consulta productos (unhappy)

- **WHEN** el occupancy de producción recibe un id de categoría
- **THEN** no abre SurrealDB ni el crate `storage`
- **AND** el resultado es `false` (H-10 de productos no rige en HTTP hasta T-02-18)
