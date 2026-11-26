# Proposal

## Ticket padre

- **Ticket:** [T-02-02](../../../docs/7-tickets.md) — Dominio Categoría y `CategoryRepo`
- **Change:** `T-02-02`
- **HU:** [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías)
- **Depende de:** T-00-03, T-00-04 (en `develop`; T-02-01 ya dejó la tabla `categoria`)
- **Hace:** entidad `Categoria`, generación de *slug* desde el nombre, profundidad máxima 4, detección de ciclo como función pura y puerto `CategoryRepo`.
- **No hace:** persistencia.
- **Listo cuando:** tests unitarios de *slug*, profundidad y ciclo.

## Why

El catálogo (Fase 2) necesita el modelo de árbol de categorías en `domain` antes del repositorio (`T-02-03`). La migración `0002` ya existe; sin entidad, *slug*, profundidad, ciclo y el puerto `CategoryRepo`, el adaptador de SurrealDB no tiene contrato hexagonal que implementar y HU-02-01 no puede rechazar `category_cycle` ni generar el *slug* del contrato.

## What Changes

- Entidad `Categoria` en el crate `domain` (campos de [4.4.1](../../../docs/4-modelo.md#441-categoria--árbol): `id`, `nombre`, `slug`, `padre`, `ancestros`, `ruta`, `activo`, `creado_en`, `actualizado_en`) con `Id<Categoria>` y `Timestamp` de T-00-04
- Generación de *slug* desde `nombre` alineada al patrón OpenAPI `^[a-z0-9]+(-[a-z0-9]+)*$` (transliteración del español, sin crate nueva)
- Función pura de profundidad (raíz = 1, máximo 4, alineado a facetas `categoria.lvl0`…`lvl3`)
- Función pura de ciclo: un nodo no puede ser padre de sí mismo ni moverse debajo de un descendiente → `DomainError::InvalidInput` con `code` `category_cycle` (T-00-03)
- Puerto `CategoryRepo` (`create`, `get`, `list_all`) en `domain`; sin adaptador ni SurrealDB
- Tests unitarios en `crates/domain` (happy + unhappy de *slug*, profundidad y ciclo)
- TDD: red = tests que no compilan / fallan; green = entidad, funciones y trait

## Non-goals

- Persistencia `create`/`get`/`list_all`, cálculo de `ancestros`/`ruta` al persistir, mapeo UNIQUE → `duplicate_slug` — **T-02-03** (No hace del ticket)
- Mover, renombrar, recálculo de rama, `update` en el puerto — **T-02-04**
- Casos de uso en `catalog`, HTTP `/categories`, UI de árbol — **T-02-05**, **T-02-06**, **T-02-07**
- Repositorio en memoria para casos de uso — **T-02-05**
- Tablas / entidades `marca`, `atributo_def`, `producto` — **T-02-08**, **T-02-11**, **T-02-14** / **T-02-16**
- Seed de categorías — **T-02-42**
- Worker / reindexación por cambio de categoría — **T-03-08**
- Ampliar `DomainError`, `Id<T>` o `Timestamp` — ya cubiertos por **T-00-03** / **T-00-04**
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas crates en `domain` (`unicode-normalization`, `async-trait`, …) — la allowlist de T-00-04 se mantiene

## Capabilities

### New Capabilities

- `domain-categoria`: entidad `Categoria`, *slug* desde el nombre, profundidad máxima 4, detección de ciclo como función pura y puerto `CategoryRepo` en el crate `domain`

### Modified Capabilities

- (ninguna; se reutilizan `domain-types` y `domain-error` sin cambiar requisitos)

## Impact

- **Repo:** `vortex-core/crates/domain/` (`src/lib.rs`, módulo de categoría / *slug* / puerto)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `openspec validate T-02-02`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`; `check-domain-deps.sh` y `check-workspace.sh` siguen en verde (sin deps nuevas)
- **Sin** cambios de API HTTP ni de `openapi.yaml`
- **Ticket:** cierra T-02-02 (Listo cuando: tests unitarios de *slug*, profundidad y ciclo)
