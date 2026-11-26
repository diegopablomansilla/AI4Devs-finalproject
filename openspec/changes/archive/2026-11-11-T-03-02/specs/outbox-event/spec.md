# Spec Delta

Ticket padre: T-03-02. Change: T-03-02. HU: HU-03-01.

## Purpose

Define el tipo de dominio `OutboxEvent` (entidad, registro ULID, operación y motivo) para que las escrituras de catálogo emitan un evento de sincronización sin acoplar `domain` a SurrealDB.

## ADDED Requirements

### Requirement: OutboxEvent en domain

El crate `domain` MUST exponer `OutboxEvent` con exactamente estos campos: `entidad`, `registro`, `operacion` y `motivo`. `entidad` MUST ser uno de `producto`, `categoria`, `marca`, `atributo_def`, `sinonimo`, `nota_tecnica`, `imagen_producto` ([4.8.1](../../../../../docs/4-modelo.md#481-outbox)). `operacion` MUST ser `upsert` o `delete`. `registro` MUST ser un ULID Crockford de 26 caracteres **sin** prefijo de tabla (mismas reglas que `Id<T>`). `motivo` MUST ser `Option` de string (pista para el worker: `stock`, `ficha`, `precio`, u otra). El crate MUST NOT depender de `surrealdb` ni incluir `tabla:` en `registro`. MUST NOT ampliar las variantes ni los `code` de `DomainError`.

#### Scenario: Construcción válida (happy)

- **WHEN** se construye un `OutboxEvent` con `entidad` `producto`, `operacion` `upsert`, `registro` un ULID de 26 caracteres y `motivo` `Some("stock")`
- **THEN** la construcción termina sin error
- **AND** los cuatro campos expuestos coinciden con esos valores

#### Scenario: motivo ausente (happy)

- **WHEN** se construye un `OutboxEvent` con `motivo` vacío (`None`)
- **THEN** la construcción termina sin error
- **AND** `motivo` expuesto es `None`

#### Scenario: Prefijo de tabla en registro (unhappy)

- **WHEN** se intenta construir un `OutboxEvent` con `registro` `producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8` (u otro `tabla:` + ULID)
- **THEN** la operación falla
- **AND** no se produce un `OutboxEvent`

#### Scenario: entidad u operación fuera de lista (unhappy)

- **WHEN** se intenta construir un `OutboxEvent` con `entidad` `foo` o `operacion` `update`
- **THEN** la operación falla
- **AND** no se produce un `OutboxEvent`

### Requirement: Tests de OutboxEvent

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `outbox_event_valid_producto_upsert` | `vortex-core/crates/domain/` (unidad) | happy |
| `outbox_event_motivo_none` | `vortex-core/crates/domain/` (unidad) | happy |
| `outbox_event_rejects_table_prefix` | `vortex-core/crates/domain/` (unidad) | unhappy |
| `outbox_event_rejects_unknown_entidad_or_operacion` | `vortex-core/crates/domain/` (unidad) | unhappy |

`cargo test --workspace` MUST ejecutarlos. MUST NOT usar SurrealDB ni el crate `storage`.

#### Scenario: la suite de OutboxEvent cubre happy y unhappy (happy)

- **WHEN** se ejecuta `cargo test -p domain` en `vortex-core/`
- **THEN** corren los tests de la tabla
- **AND** los de construcción válida y `motivo` ausente pasan
- **AND** los de prefijo de tabla y entidad/operación inválidas pasan
