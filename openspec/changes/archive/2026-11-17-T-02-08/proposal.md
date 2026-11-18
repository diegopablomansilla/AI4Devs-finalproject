# Proposal

## Ticket padre

- **Ticket:** [T-02-08](../../../docs/7-tickets.md) — Marca: migración, dominio y repositorio
- **Change:** `T-02-08`
- **HU:** [HU-02-02](../../../docs/6-historias.md#hu-02-02--marcas)
- **Depende de:** T-03-02 (en `develop`: `storage::Tx` + outbox; `OutboxEntity::Marca` ya existe)
- **Hace:** migración de [4.4.2](../../../docs/4-modelo.md#442-marca), entidad `Marca`, puerto `BrandRepo` y su implementación con outbox; índice único → `duplicate_slug`.
- **No hace:** HTTP.
- **Listo cuando:** tests de integración de alta, edición y baja lógica.

## Why

HU-02-02 exige un maestro plano de marcas (alta "Fixser", edición y baja lógica sin perder la referencia). T-02-01…T-02-07 cubrieron el árbol de categorías; sin tabla `marca`, entidad, puerto e implementación hexagonal, T-02-09 no puede exponer `/api/v1/brands` ni el catálogo puede asociar productos a una marca.

## What Changes

- Migración embebida `0003_marca.surql`: tabla `SCHEMAFULL` `marca` según 4.4.2 (campos, `ASSERT` de `nombre`, índice único `uq_marca_slug` sobre `slug`, `CHANGEFEED 3d`), con `DEFINE … IF NOT EXISTS` (4.10)
- Entidad `Marca` en `domain` (`id`, `nombre`, `slug`, `activo`, `creado_en`, `actualizado_en`) reutilizando `Slug` e `Id<Marca>`
- Puerto `BrandRepo` (`create`, `get`, `list_all`, `update`, `set_activo`) en `domain`; sin HTTP ni crate `catalog`
- Adaptador SurrealDB en `storage` con `Tx` + `OutboxEvent` `marca` / `upsert`; UNIQUE `slug` → `DomainError::Conflict` `duplicate_slug`
- Tests unitarios de entidad/puerto y de integración de esquema + alta/edición/baja (happy + unhappy)
- TDD: red = tests que no compilan / fallan; green = migración + módulo + adaptador

## Non-goals

- Casos de uso en `catalog` y HTTP `GET`/`POST /api/v1/brands`, `PATCH`/`DELETE /api/v1/brands/{brandId}` — **T-02-09** (No hace del ticket: HTTP)
- UI de marcas, logos — **T-02-10**
- Ocupación / rechazo de baja por productos asociados — **T-02-09** permite la baja; **T-02-14** introduce `producto`
- `atributo_def`, `producto`, seed de marcas — **T-02-11**, **T-02-14**, **T-02-42**
- Worker *fan-out* de marca a Meilisearch — **T-03-08**
- Extraer `Slug` de `categoria.rs` a un módulo compartido — reutilizar el tipo existente; no se cambia `domain-categoria`
- Ampliar `DomainError` u `OutboxEntity` — ya cubiertos por **T-00-03** / **T-03-02** (`duplicate_slug`, entidad `marca`)
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas crates en `domain`

## Capabilities

### New Capabilities

- `domain-marca`: entidad `Marca`, reuso de `Slug` desde el nombre, y puerto `BrandRepo` (`create`, `get`, `list_all`, `update`, `set_activo`) en el crate `domain`

### Modified Capabilities

- `vortex-storage`: migración `0003` con tabla `marca` e índice único `slug`; adaptador `BrandRepo` con outbox; UNIQUE `slug` → `duplicate_slug`

## Impact

- **Repo:** `vortex-core/crates/domain/` (módulo `marca`, reexports); `vortex-core/crates/storage/` (`migrations/0003_marca.surql`, adaptador, tests de integración, ajuste de conteo de migraciones)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `cargo test -p storage --test marca`; `openspec validate T-02-08`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`; `check-domain-deps.sh` sigue en verde
- **Sin** cambios de API HTTP ni de `openapi.yaml`
- **Ticket:** cierra T-02-08 (Listo cuando: tests de integración de alta, edición y baja lógica)
- **Ámbito:** marca es plana (sin árbol); cabe en ~2 h copiando el patrón de T-02-01 + T-02-02 + T-02-03/T-02-04 sin lógica de ciclos. No se divide el ticket.
