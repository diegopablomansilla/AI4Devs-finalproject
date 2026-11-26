# Proposal

## Ticket padre

- **Ticket:** [T-02-11](../../../docs/7-tickets.md) — `atributo_def`: migración, dominio y repositorio
- **Change:** `T-02-11`
- **HU:** [HU-02-03](../../../docs/6-historias.md#hu-02-03--definición-de-atributos-técnicos)
- **Depende de:** T-03-02 (en `develop`: `storage::Tx` + outbox; `OutboxEntity::AtributoDef` ya existe)
- **Hace:** migración de [4.4.3](../../../docs/4-modelo.md#443-atributo_def--definición-de-atributos-técnicos); entidad `AtributoDef` (clave en `snake_case`, `opciones` obligatorias si es `enum`); puerto `AttributeDefRepo` con outbox.
- **No hace:** la regla "en uso" (necesita `producto`, T-02-23).
- **Listo cuando:** tests cubren "enum sin opciones" → `invalid_input` y clave duplicada → `409`.

## Why

HU-02-03 exige un maestro de atributos técnicos tipados (medida, rosca, grado) para describir productos sin migrar el esquema. Categorías y marcas ya existen; sin tabla `atributo_def`, entidad, puerto e implementación hexagonal, T-02-12 no puede exponer `/api/v1/attribute-definitions` ni T-02-16 validar valores de producto contra definiciones.

## What Changes

- Migración embebida `0004_atributo_def.surql`: tabla `SCHEMAFULL` `atributo_def` según 4.4.3 (campos, `ASSERT` de `tipo`, índice único `uq_atributo_clave` sobre `clave`, `CHANGEFEED 3d`), con `DEFINE … IF NOT EXISTS` (4.10)
- Entidad `AtributoDef` en `domain` (`id`, `clave`, `etiqueta`, `tipo`, `unidad`, `opciones`, `facetable`, `buscable`, `orden`, `activo`, `creado_en`, `actualizado_en`) con `Id<AtributoDef>`
- Validación de dominio: `clave` `snake_case` (`^[a-z][a-z0-9_]*$`); `tipo` ∈ {`texto`, `numero`, `booleano`, `enum`}; si `tipo = enum`, `opciones` no vacías; `enum` sin opciones → `invalid_input`
- Puerto `AttributeDefRepo` (`create`, `get`, `list_all`, `update`, `set_activo`) en `domain`; sin HTTP ni crate `catalog`
- Adaptador SurrealDB en `storage` con `Tx` + `OutboxEvent` `atributo_def` / `upsert`; UNIQUE `clave` → `DomainError::Conflict` `conflict` (HTTP `409` en T-02-12)
- Tests unitarios de entidad/puerto y de integración de esquema + alta/edición/baja (happy + unhappy), incluyendo los dos del Listo cuando
- TDD: red = tests que no compilan / fallan; green = migración + módulo + adaptador

## Non-goals

- Casos de uso en `catalog` y HTTP `GET`/`POST /api/v1/attribute-definitions`, `PATCH`/`DELETE …/{attributeId}` — **T-02-12**
- UI de atributos técnicos — **T-02-13**
- Regla `attribute_in_use` (clave o tipo inmutables cuando hay productos) — **T-02-23** (No hace del ticket)
- Validación de valores de producto contra `AtributoDef` — **T-02-16**
- Tabla `producto`, seed de atributos — **T-02-14**, **T-02-42**
- Worker *fan-out* de atributo a Meilisearch / `filterableAttributes` — **T-03-08**
- Mapeo API inglés (`key`/`label`/`type`/`searchable`) ↔ modelo español — **T-02-12** ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos))
- Ampliar `DomainError` u `OutboxEntity` — ya cubiertos por **T-00-03** / **T-03-02** (`conflict`, entidad `atributo_def`). No se añade `duplicate_key`
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas crates en `domain`

## Capabilities

### New Capabilities

- `domain-atributo-def`: entidad `AtributoDef` (clave `snake_case`, `opciones` obligatorias si `enum`) y puerto `AttributeDefRepo` (`create`, `get`, `list_all`, `update`, `set_activo`) en el crate `domain`

### Modified Capabilities

- `vortex-storage`: migración `0004` con tabla `atributo_def` e índice único `clave`; adaptador `AttributeDefRepo` con outbox; UNIQUE `clave` → `conflict` (409)

## Impact

- **Repo:** `vortex-core/crates/domain/` (módulo `atributo_def`, reexports); `vortex-core/crates/storage/` (`migrations/0004_atributo_def.surql`, adaptador, tests de integración, ajuste de conteo de migraciones)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `cargo test -p storage --test atributo_def`; `openspec validate T-02-11`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`; `check-domain-deps.sh` sigue en verde
- **Sin** cambios de API HTTP ni de `openapi.yaml`
- **Ticket:** cierra T-02-11 (Listo cuando: enum sin opciones → `invalid_input`; clave duplicada → 409 / `conflict`)
- **Ámbito:** maestro plano como marca (T-02-08) más validación de `tipo`/`opciones`; cabe en ~2 h. No se divide el ticket.
