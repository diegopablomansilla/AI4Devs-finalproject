# Design

## Context

En `develop`, `domain` expone `Relacion`, `NewRelacion`, `UpdateRelacion`, `TipoRelacion` y `RelationRepo` (`create` / `update` / `delete` / `list_for_product` con `list_for_product → Vec<Relacion>`; T-04-02 difería el agrupado y el resumen del destino a T-04-04). `storage::RelacionRepo` implementa `create` (RELATE simple o par con `grupo_bidireccional` en `Tx`, sin outbox); `update` / `delete` / `list_for_product` devuelven `DomainError::unavailable`. El esquema `0007` ya tiene `prioridad`, `nota`, `grupo_bidireccional` y UNIQUE `(in, out)`. `recommend` sigue vacío. Ver `proposal.md` para motivación y las deltas de `vortex-storage` / `domain-relacion` para el contrato.

Constraints: sin recomendaciones (T-04-06); sin HTTP ni `recommend` (T-04-05); ~2 h; SurrealDB `=2.7.0`; MVP del curso.

## Goals / Non-Goals

**Goals:**

- `update` / `delete` atómicos sobre el grupo bidireccional (Listo cuando: cambiar prioridad desde B modifica ambas aristas)
- `list_for_product` saliente agrupado por tipo con `DestinoResumen`
- Reemplazar los stubs `Unavailable` de T-04-03

**Non-Goals (diseño):**

- Consulta 4.6.2, filtros `activo`/`stock`, HTTP, outbox, Meilisearch
- Cambiar `0007_relaciones.surql`, `create` o `DomainError`
- Hidratar `ProductSummary` completo ni listar aristas entrantes

## Decisions

### 1. Tipos de listado en `domain`; firma de `list_for_product`

**Elección:** agregar `DestinoResumen { id, sku, nombre, activo }`, `RelacionConDestino { relacion, destino }` y `RelacionesDeProducto { similar_a, sustituible_por, complementa }`. Cambiar `RelationRepo::list_for_product` a `Result<RelacionesDeProducto, DomainError>`. `update` sigue devolviendo `Relacion` (la arista del `id`). Reexportar los tipos desde `domain::lib`. El dummy de test y `RelacionRepo` de storage se ajustan.

**Por qué:** T-04-02 dejó el agrupado y el resumen para este ticket; el Hace pide ambos en `list_for_product`. Un DTO mínimo cubre HU-04-03 (sku/nombre/activo, incluidos inactivos) sin copiar `ProductSummary` (T-04-05).

**Alternativa:** conservar `Vec<Relacion>`. Se descartó: no hay dónde colgar el resumen. **Alternativa:** `Vec<RelacionConDestino>` plano. Se descartó: el ticket pide agrupado por tipo; T-04-05 puede aplanar a `items[]`. **Alternativa:** resumen en `storage` sin tocar `domain`. Se descartó: el puerto viviría incompleto y `recommend` no podría tipar el listado.

### 2. Grupo bidireccional: una `Tx` por `grupo_bidireccional`

**Elección:**

```text
-- update (con grupo)
UPDATE {tabla} SET prioridad = $p, nota = $n
  WHERE grupo_bidireccional = $g;

-- update (sin grupo)
UPDATE type::thing('{tabla}', '{id}') SET ...;

-- delete (con grupo)
DELETE {tabla} WHERE grupo_bidireccional = $g;

-- delete (sin grupo)
DELETE type::thing('{tabla}', '{id}');
```

Antes de la `Tx`: `SELECT` la arista `tipo`+`id`. Ausente → `not_found` (sin I/O de escritura). Con grupo: mutar por `grupo_bidireccional` (ambas aristas). Sin grupo: mutar solo ese record. Validar `prioridad`/`nota` del parche construyendo un `RelacionDraft` con los valores resultantes (actual ⊕ parche) y `Relacion::try_new` **antes** de la `Tx`. `nota: None` en `UpdateRelacion` = no tocar; `Some(None)` = limpiar.

Tras el commit de `update`, re-fetch de la arista `id` (timestamps del motor). `delete` no re-fetch.

**Por qué:** 4.6.1: «editar o eliminar una arista del grupo aplica a ambas». Una `Tx` evita el par a medias. Mutar por grupo (no dos `UPDATE` por id) cubre el Listo cuando aunque se identifique la arista B→A.

**Alternativa:** devolver ambas aristas en `update`. Se descartó: el puerto T-04-02 ya firma `Relacion`; T-04-05 arma `items[]`. **Alternativa:** dos commits. Se descartó: el Hace pide aplicar al grupo.

### 3. Listado: tres `SELECT` salientes + join de `out`

**Elección:** para cada tabla, `SELECT` aristas con `in = producto:{id}` y campos de `out` (`id`, `sku`, `nombre`, `activo`). Hidratar `Relacion` con el mapper existente (`record_to_relacion`) y `DestinoResumen` del join. Agrupar en `RelacionesDeProducto`. `ORDER BY prioridad, id`. Destinos inactivos incluidos. Producto desconocido o sin aristas → tres vec vacíos (el 404 es T-04-05). Sin aristas entrantes.

**Por qué:** HU-04-03 es administración (incluye inactivos); T-04-06 filtra. Tres consultas acotadas evitan un `UNION` frágil entre tablas `TYPE RELATION`.

**Alternativa:** un `RETURN { similares, … }` estilo 4.6.2. Se descartó: esa consulta es T-04-06 (filtros distintos). **Alternativa:** comprobar que el producto exista y devolver `not_found`. Se descartó: el ticket no lo pide; T-04-05 hace `GET` del producto.

### 4. Reemplazar stubs `Unavailable`; tests en `relacion_repo.rs`

**Elección:** borrar `relation_repo_update_delete_list_are_unavailable`. Extender `tests/relacion_repo.rs` (mismo seed perno/arandela). Baja lógica de un destino vía `ProductRepo::set_activo` para el escenario de listado con inactivo. Nombres de test = tabla «Tests previstos» de las specs. `create` no cambia.

**Por qué:** el archivo y el seed de T-04-03 ya cubren ULID de dominio. Un test de Listo cuando que actualiza desde B es el contrato de HU-04-02.

**Alternativa:** archivo nuevo `relacion_repo_mut.rs`. Se descartó: mismo puerto y mismo seed.

### 5. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-04-04` y rename a `T-04-04` (igual que tickets previos).

## Risks / Trade-offs

- **[Surreal 2.7 no actualiza `actualizado_en` en `UPDATE` masivo por `WHERE grupo_bidireccional`]** → Mitigación: el VALUE `time::now()` del esquema ya cubre `UPDATE`; el test de Listo cuando lee `prioridad` en ambas aristas, no el timestamp.
- **[`nota: Option<Option<String>>` se serializa mal en SQL]** → Mitigación: ramificar `SET nota = NONE` vs `SET nota = '…'` vs omitir el campo; el escenario de nota unidireccional es el contrato.
- **[Join `out.sku` en tabla RELATION no hidrata]** → Mitigación: `SELECT … FETCH out` o `out.sku` explícito; confirmar en apply; si falla, `SELECT` el producto por `out` en el mismo round (sigue sin HTTP).
- **[Cambio de firma rompe el dummy de `domain`]** → Mitigación: tarea red de `domain` primero; el dummy devuelve `RelacionesDeProducto` vacío.
- **[Ámbito ~2 h]** → Mitigación: sin incoming, sin ProductSummary, sin HTTP; `create` intacto. No se divide el ticket.

## Migration Plan

Sin migración de esquema (`0007` ya está). Despliegue = merge a `develop`. Rollback = revertir adaptador + tipos de listado; las tablas no cambian.

1. Red: tipos + firma en `domain` (tests unitarios) y tests de `tests/relacion_repo.rs` que fallan contra `Unavailable`.
2. Green: `update` / `delete` / `list_for_product`; quitar stubs.
3. Cierre: § 5 en `vortex-core/`, `openspec validate T-04-04`.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. La sintaxis exacta del `SELECT` con campos de `out` se confirma en apply contra SurrealDB 2.7.
