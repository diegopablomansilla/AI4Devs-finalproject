# Spec Delta

Ticket padre: T-00-09. Change: T-00-09. HU: HU-00-02.

## ADDED Requirements

### Requirement: Motor de migraciones embebidas con checksum

El crate `storage` MUST incluir migraciones en `migrations/NNNN_nombre.surql` embebidas con `include_str!`. MUST existir `0000_migracion.surql` conforme a [4.10](../../../../docs/4-modelo.md#410-operación-del-esquema). `Db::migrate()` MUST aplicar las migraciones embebidas en orden ascendente de `version`, registrar cada una en `migracion` con `checksum` SHA-256 hex del contenido del archivo, y ser idempotente cuando el contenido no cambió. Si una `version` ya registrada tiene un `checksum` distinto al del archivo embebido, la operación MUST fallar sin aplicar más SQL y el error MUST nombrar la versión en `Display`. `Db::open` MUST NOT aplicar migraciones (T-00-08).

#### Scenario: Primera instalación (happy)

- **WHEN** tras `Db::open` se llama `Db::migrate` en un directorio temporal vacío
- **THEN** `migrate` termina sin error
- **AND** existe un registro con `version` 0 y `checksum` igual al SHA-256 de `0000_migracion.surql`

#### Scenario: Re-arranque sin cambios (happy)

- **WHEN** `migrate` se ejecuta dos veces sobre la misma instancia con las mismas migraciones embebidas
- **THEN** la segunda ejecución termina sin error
- **AND** sigue habiendo un solo registro para la `version` 0

#### Scenario: Migración alterada (unhappy)

- **WHEN** la versión 0 ya está registrada y se intenta `apply_migrations` con SQL distinto en la misma versión
- **THEN** la operación falla
- **AND** el mensaje de error incluye la versión (`0` o `0000`)
