# vortex-storage (delta)

## ADDED Requirements

### Requirement: CategoryRepo update con recálculo de rama

El adaptador `CategoryRepo` en `storage` MUST implementar `update(UpdateCategoria)`. Al cambiar `nombre` y/o `padre`, MUST recalcular `slug`, `ancestros` y `ruta` del nodo y de todo su subárbol en una sola `Tx`. MUST emitir un `OutboxEvent` `categoria`/`upsert` por cada categoría actualizada. MUST usar `reject_cycle` del dominio antes de escribir. MUST NOT reindexar productos (T-03-08).

#### Scenario: Renombrar rama de 3 niveles (happy)

- **WHEN** existen Fijaciones > Pernos > Hex y `update` renombra la raíz a Tornillería sin cambiar `padre`
- **THEN** las rutas persistidas son `["Tornillería"]`, `["Tornillería", "Pernos"]` y `["Tornillería", "Pernos", "Hex"]`
- **AND** existen tres eventos outbox adicionales

#### Scenario: Mover debajo de descendiente (unhappy)

- **WHEN** Pernos es hija de Fijaciones y `update` asigna padre Pernos a Fijaciones
- **THEN** la operación falla con `code` `category_cycle`
- **AND** no se añaden filas ni eventos outbox

#### Scenario: Suite de integración update (happy)

- **WHEN** se ejecuta `cargo test -p storage --test categoria` filtrando tests `category_repo_renames_three_level_branch_ruta`, `category_repo_update_emits_outbox_per_affected` y unhappy `category_repo_move_under_descendant_is_category_cycle`
- **THEN** todos pasan
