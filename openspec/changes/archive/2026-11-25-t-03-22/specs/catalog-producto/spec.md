# Spec Delta

Ticket: T-03-22. HU: HU-03-08.

## ADDED Requirements

### Requirement: Buscar productos por código exacto

El crate `catalog` MUST exponer `ProductService::find_by_code` que delega en `ProductRepo::find_by_code` sin reglas de negocio adicionales. MUST NOT abrir SurrealDB ni mapear a HTTP.

`ProductGateway` MUST declarar `find_by_code` y el adaptador object-safe MUST reenviar al servicio.

#### Scenario: find_by_code delega al repo (happy)

- **WHEN** el caso de uso recibe `code` `"HX-0840Z"`
- **THEN** invoca `ProductRepo::find_by_code` con ese string
- **AND** devuelve el mismo `Vec<ResumenProducto>` que el puerto

#### Scenario: suite catalog find_by_code (happy)

- **WHEN** se ejecuta `cargo test -p catalog product_service_find_by_code_delegates`
- **THEN** el test termina con código 0