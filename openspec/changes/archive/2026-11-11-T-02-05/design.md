# Design

## Context

`CategoryRepo` en `domain`/`storage` ya cubre `create`/`get`/`list_all`/`update`. El crate `catalog` es un placeholder sin `[dependencies]`; `check-workspace.sh` falla si `catalog` declara deps. No existe `producto`. See proposal.md — Why.

## Goals / Non-Goals

**Goals:** servicio de aplicación en `catalog` sobre puertos; fake en memoria para TDD; `set_activo` en el trait para que el workspace compile; allowlist de `catalog`.

**Non-Goals:** HTTP, consulta SQL de productos, `tracing` en `catalog`, ampliar `DomainError`.

## Decisions

1. **`CategoryService<R, U>`** genérico (`R: CategoryRepo`, `U: CategoryOccupancy`). `api` (T-02-06) llama al servicio, no al repo. Consultas `get`/`list_all` son wrappers para no romper 3.2.
   - Alternativa: `api` usa `CategoryRepo` directo — se descartó (lógica de H-10 quedaría en el Gateway).

2. **`set_activo` en `CategoryRepo`**, no en `UpdateCategoria`. El recálculo de rama no aplica al flag `activo`.
   - Alternativa: `UpdateCategoria { activo }` — mezclaría rename/move con lifecycle.

3. **`CategoryOccupancy` en `catalog`** (`has_active_products`), no en `domain`. Hasta T-02-14/T-02-18 el único impl es el fake de tests; T-02-06 puede inyectar un stub que devuelve `false`.
   - Alternativa: método en `CategoryRepo` — acoplaría categorías a productos en el puerto de persistencia.

4. **H-10:** rechazo si hay **hija con `activo = true`** (vía `list_all`) **o** occupancy verdadero. `code` = `conflict` (tabla 5.4). Hijas inactivas no bloquean (baja de abajo hacia arriba).
   - Alternativa: `category_in_use` — no existe en 5.4.
   - Alternativa: cualquier hija (también inactiva) — impediría dar de baja el padre después de dar de baja las hojas.

5. **Idempotencia:** baja de ya inactiva y reactivar de ya activa → `Ok` sin occupancy. `not_found` solo si no existe la fila.

6. **Fake en memoria** en tests de `catalog`: `HashMap`, calcula `ancestros`/`ruta`/`slug` con las funciones de `domain` (`Slug::from_nombre`, `reject_cycle`, `MAX_DEPTH`) lo bastante como para los escenarios HU-02-01. No vive en `storage`.

7. **Outbox de `set_activo`:** `upsert`, no `delete`. La baja es lógica; el worker (T-03-08) debe ver el cambio de `activo`.

8. **Allowlist:** `catalog` = path `domain` + `tokio` (dev). Actualizar `scripts/check-workspace.sh` y el fixture `catalog con dependencies` (pasa a «fuera de allowlist»; el happy usa el `Cargo.toml` real).

## Risks / Trade-offs

- [Stub occupancy `false` en T-02-06] → H-10 de productos no rige en HTTP hasta T-02-18. Mitigación: tests de catalog ya cubren el puerto; ticket de producto implementa el adaptador.
- [Fake duplica reglas de árbol] → solo el mínimo para ruta/ciclo/slug; storage sigue siendo la fuente de tests de persistencia.
- [OpenAPI `DELETE` sin `409`] → T-02-06 alinea el contrato; este change no toca `openapi.yaml`.

## Migration Plan

Sin migración `.surql`. `activo` ya existe en `0002`.

## Open Questions

Ninguna que altere specs o tareas. T-02-06 decide cómo PATCH `{ active: true }` vs DELETE se mapean a reactivar/baja.
