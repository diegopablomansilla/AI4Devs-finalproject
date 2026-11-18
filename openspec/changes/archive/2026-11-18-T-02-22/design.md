# Design

## Context

En `develop`, T-02-21 dejó `POST`/`GET`/`PATCH /api/v1/products`, `ProductGateway` (crear/obtener/editar) y `PATCH { active }` → 400 `invalid_input`. `ProductRepo` no declara `set_activo`. Marcas, categorías y `atributo_def` ya usan `set_activo` + `DELETE` 204 + `PATCH { active: true }` + outbox `upsert`. OpenAPI ya declara `DELETE /api/v1/products/{productId}` (204/404) y `ProductUpdate.active`. See proposal.md — Why.

Constraints: MVP del curso; ~2 h; `api` no depende de `storage`; el worker Meilisearch es T-03-07; tablas de relaciones (T-04-01) e imágenes (T-02-28) aún no existen.

## Goals / Non-Goals

**Goals:**

- Baja lógica y reactivación en el hexágono (puerto → storage → catalog → HTTP)
- Conservar `movimiento_stock` (Listo cuando) y no tocar aristas ni imágenes
- Evento de outbox para que T-03-07 quite el documento del índice

**Non-Goals (diseño):**

- Llamar a Meilisearch desde este ticket
- Incrementar `version` en `set_activo`
- Extraer un `set_activo` genérico compartido entre maestros
- UI (T-02-34)

## Decisions

### 1. `ProductRepo::set_activo`, no `UpdateProducto.activo`

**Elección:** el puerto gana `set_activo(id, activo) -> FichaProducto`. `UpdateProducto` y `PatchProducto` siguen sin `activo`. Catalog expone `deactivate` / `reactivate` en `ProductGateway`.

**Por qué:** mismo molde que `BrandRepo` / `CategoryRepo`. La ficha (`update`) no escribe `activo`; el flag de circulación es otra operación. El Listo cuando (ledger intacto) se testea en storage sobre `set_activo`, no sobre un `UPDATE` de ficha.

**Alternativa:** meter `activo` en `PatchProducto` y reusar `update`. Se descartó: `update` hace `version += 1` y reemplaza arrays; mezclaría ficha con baja. **Alternativa:** `DELETE` físico. Se descartó: [5.1](../../../docs/5-contract-api.md#51-convenciones-generales) y HU-02-06.

### 2. Outbox `upsert`, no `delete`

**Elección:** `set_activo` emite `OutboxEvent` `producto`/`upsert` con `motivo` ausente, igual que marca/categoría. El worker T-03-07 decide borrar el documento Meilisearch cuando `activo` es falso ([4.8](../../../docs/4-modelo.md#48-sincronización-con-meilisearch)).

**Por qué:** el No hace del ticket es explícito (quitar de la búsqueda = T-03-07). Emitir `operacion: delete` ahora acoplaría este ticket al contrato del worker aún no escrito.

**Alternativa:** `operacion: delete` en baja y `upsert` en reactivación. Se pospone a T-03-07 si el worker lo necesita.

### 3. `set_activo` no incrementa `version` ni escribe `stock`

**Elección:** `UPDATE producto SET activo = …` (y `actualizado_en`). `version` y `stock` no cambian. Idempotente si ya tiene ese valor.

**Por qué:** `DELETE` no tiene cuerpo ni `version`; si incrementáramos, el cliente no sabría el nuevo valor para reactivar. El movimiento de stock tampoco incrementa `version` (T-02-24). La reactivación HTTP envía `version` porque OpenAPI lo exige en `ProductUpdate`, no porque haya bloqueo optimista sobre el flag.

**Alternativa:** `WHERE version = $v` y `version += 1` también en DELETE. Se descartó: 204 sin cuerpo.

### 4. PATCH `{ active }` no valida `version`; ficha sí

**Elección:** `{ "version": N, "active": true|false }` (solo esos campos) llama a `reactivate`/`deactivate` y **no** compara `N` con la versión persistida ni la incrementa. Si el mismo PATCH trae campos de ficha (`price`, `name`, …), primero `set_activo` y después `update` (este usa `version` y puede dar `version_conflict`). `{ "version": N }` sin `active` ni ficha sigue siendo 400.

**Por qué:** `ProductUpdate` exige `version` + `minProperties: 2`; marcas no tienen versión. Exigir match en un parche solo de `active` haría fallar reactivaciones inofensivas tras un movimiento de stock (el stock no cambia `version`, pero una edición de ficha concurrente sí). El flag de circulación no es ficha.

**Alternativa:** 409 si `N` no coincide. Se descartó para el parche solo-`active`. **Alternativa:** seguir rechazando `active` con 400. Se descartó: es el Hace de este ticket.

### 5. GET de inactivo sigue 200

**Elección:** `get` no filtra por `activo`. Tras `DELETE`, `GET /products/{id}` devuelve `active: false`.

**Por qué:** el administrador necesita la ficha para reactivar. Quitar de búsqueda/recomendaciones es T-03-07 / T-04-06. T-02-21 ya dejó este MUST NOT.

### 6. Ledger, relaciones e imágenes: no borrar

**Elección:** el SQL de `set_activo` es solo `UPDATE` de `producto`. Tests de storage cuentan `movimiento_stock` y `stock` (Listo cuando). Relaciones (T-04-01) e imágenes (T-02-28) no tienen tabla aún: la spec prohíbe `DELETE` sobre esas tablas; no se inventan tests contra esquema inexistente.

**Por qué:** HU-02-06 «se conservan su ledger, relaciones e imágenes». Conservar = no tocar, no hidratar.

### 7. OpenAPI `503` en DELETE

**Elección:** agregar `'503': Unavailable` a `DELETE /api/v1/products/{productId}` (el path ya existe; faltaba 503 como en marcas). No se agregan paths nuevos.

**Por qué:** T-09-04 falla si respondemos 503 sin schema.

### 8. Fakes HTTP vs storage

**Elección:** tests de `api` usan el fake de `ProductGateway` (extender `deactivate`/`reactivate`). El ledger intacto se aserta en `storage/tests/producto.rs` contra Surreal. `vortex_serve_delete_unknown_product_returns_404` cubre el cableado real (404 ≠ 503).

**Por qué:** mismo corte que T-02-21. El fake HTTP no escribe `movimiento_stock`.

### 9. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-22` y rename a `T-02-22`.

## Risks / Trade-offs

- **[`PATCH { version, active: true }` ignora version]** → un cliente con ficha vieja igual reactiva. Mitigación: documentado; la ficha concurrente no se pisa. Si más adelante hace falta, T-02-34 puede GET antes de PATCH.
- **[Tablas de relaciones/imágenes ausentes]** → el Listo cuando se cubre con ledger; la spec prohíbe borrar el resto. Mitigación: T-04-03 / T-02-30 no deben asumir que la baja limpia aristas.
- **[Worker T-03-07 aún no existe]** → un producto dado de baja sigue «buscable» hasta ese ticket. Aceptado por el No hace.
- **[Alcance ~2 h]** → copiar `MarcaRepo::set_activo` + un handler DELETE. No se divide el ticket.

## Migration Plan

Sin migración de esquema: `producto.activo` ya existe (T-02-14). Rollback = revertir el PR; las filas inactivas quedan inactivas y el GET sigue mostrándolas.
