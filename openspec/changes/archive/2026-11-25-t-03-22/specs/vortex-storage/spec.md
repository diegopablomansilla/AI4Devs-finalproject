# Spec Delta

Ticket: T-03-22. HU: HU-03-08.

## ADDED Requirements

### Requirement: ProductRepo find_by_code en SurrealDB

`ProductoRepo` MUST implementar `find_by_code` consultando la tabla `producto` con `activo = true` y (`sku = $code` OR `$code IN codigos.valor`). MUST hidratar `ResumenProducto` con marca y `categoria_ruta` igual que `list`. MUST ordenar por `sku` ascendente. MUST NOT consultar Meilisearch. MUST NOT devolver inactivos.

#### Scenario: product_repo_find_by_code_shared_oem_returns_both (happy, Listo cuando)

- **WHEN** en la base hay dos productos activos con el mismo `codigos[].valor` OEM
- **AND** se llama `find_by_code` con ese valor
- **THEN** se devuelven dos `ResumenProducto`

#### Scenario: product_repo_find_by_code_does_not_prefix_match (unhappy, Listo cuando)

- **WHEN** hay un producto con fabricante `"HX-0840Z"`
- **AND** se llama `find_by_code` con `"HX-0841Z"`
- **THEN** la lista está vacía

#### Scenario: product_repo_find_by_code_excludes_inactive (unhappy)

- **WHEN** un producto inactivo comparte el OEM buscado
- **THEN** no aparece en el resultado

#### Scenario: product_repo_find_by_code_unknown_returns_empty (happy)

- **WHEN** no hay coincidencias exactas
- **THEN** la lista está vacía sin error

#### Scenario: suite find_by_code en storage (happy)

- **WHEN** se ejecuta `cargo test -p storage --test producto product_repo_find_by_code`
- **THEN** todos los tests terminan con código 0