# Design

## Context

`CategoryRepo` en `storage` implementa `create`/`get`/`list_all`. Dominio: `reject_cycle`, `Slug::from_nombre`, `MAX_DEPTH`. `Tx` emite un outbox por evento en una transacción.

## Goals / Non-Goals

**Goals:** `update` atómico con recálculo BFS del subárbol; un outbox `upsert` por fila tocada.

**Non-goals:** HTTP, worker, toggling `activo`.

## Decisions

1. **`UpdateCategoria`:** `id`, `nombre`, `padre` (siempre estado completo; `None` = raíz).
2. **Validación:** `get(id)`; si `padre` cambia, `get(padre)` + `reject_cycle`; profundidad del subárbol tras el movimiento ≤ 4.
3. **Descendientes:** `SELECT` categorías con `id = node` o `node IN ancestros`; ordenar por `ancestros.len()` ascendente.
4. **Recálculo:** en memoria, hijo toma `ancestros`/`ruta` del padre ya actualizado en el mapa; renombre solo actualiza segmento de `ruta` en el índice del nodo renombrado para descendientes.
5. **Persistencia:** una `Tx` con `UPDATE` por categoría afectada + `emit` cada una; rollback si falla UNIQUE → `duplicate_slug`.

## Risks

- Muchos `UPDATE` en ramas grandes — MVP aceptable; árbol típico pequeño.
- Slug raíz duplicado al mover a raíz — `SELECT` previo o mapper UNIQUE como en `create`.

## Migration Plan

Sin migración SQL nueva.
