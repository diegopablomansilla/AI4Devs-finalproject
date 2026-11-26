# Design

## Context

En `develop`, T-04-04 dejó `RelationRepo` completo (create/update/delete/list **saliente**) y tipos `DestinoResumen` / `RelacionesDeProducto`. El crate `recommend` es un lib vacío sin `[dependencies]` (regla empty-lib de `check-workspace.sh`). El router de `api` no registra `/relations`. OpenAPI ya declara los cuatro paths; no declara `503`. T-04-04 difería `direction=incoming` y el 404 de producto a este ticket. See proposal.md — Why.

Constraints: MVP del curso; ~2 h; DC-05 (relaciones no en `catalog`); `api` no depende de `storage`; `domain` no gana crates; T-09-04 exige schema por cada status que respondemos.

## Goals / Non-Goals

**Goals:**

- `RelationService` / `RelationGateway` en `recommend` sobre `RelationRepo` + `ProductRepo`
- HTTP delgado con DTO 5.3 y `assert_conforms`
- Listado incoming (puerto + adaptador) y 404 de producto en GET
- Allowlist `recommend` / `api` / `vortex`

**Non-Goals (diseño):**

- Merge de PATCH o bidireccionalidad en el handler (ya está en `storage`)
- `async-trait`
- Hidratar `ProductSummary` real (stock/precio/tipo)
- Consulta 4.6.2 / métrica `recommendations_duration_seconds`
- Extraer un mapper 5.3 genérico

## Decisions

### 1. Casos de uso en `recommend`, no en `catalog`

**Elección:** `vortex-core/crates/recommend` gana `RelationService<R, P>` (`R: RelationRepo`, `P: ProductRepo`) y trait object-safe `RelationGateway` (`Pin<Box<dyn Future…>>`, mismo molde que `SearchGateway` / `ProductGateway`). `api` inyecta `Option<Arc<dyn RelationGateway>>` con `with_relations`. Sin servicio → `503 unavailable`.

**Por qué:** [DC-05](../../../docs/3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes) y el Hace del ticket. [3.4.5](../../../docs/3-componentes.md#345-recommendation-service) ya consume `RelationRepo` y `ProductRepo`.

**Alternativa:** handlers llaman a `RelacionRepo`. Se descartó: rompe hexágono 3.2 y acopla `api` a `storage`. **Alternativa:** servicio en `catalog`. Se descartó: DC-05.

### 2. `ProductRepo` solo para 404 del GET

**Elección:** `list` llama `ProductRepo::get` primero; `not_found` se propaga. El repo de grafo sigue devolviendo grupos vacíos si el id no existe (T-04-04). POST/PATCH/DELETE no consultan `ProductRepo`: el `ENFORCED` de Surreal ya es `not_found`.

**Por qué:** vortex-storage documentó «el 404 de producto es T-04-05». Un GET admin de un ULID inventado no debe parecer «cero relaciones».

**Alternativa:** 200 `items: []` para id desconocido. Se descartó: OpenAPI declara 404 y T-04-04 lo dejó explícito. **Alternativa:** el Gateway llama `ProductGateway`. Se descartó: mezcla catálogo HTTP con el caso de uso de relaciones.

### 3. POST/PATCH `{ items }` re-listando tras el write

**Elección:** `RelationRepo::create` / `update` siguen devolviendo **una** `Relacion` (no se reabre el puerto). El servicio, si hay `grupo_bidireccional`, hace `list_for_product(origen)` y `list_for_product(destino)` y arma el vector de `RelacionConDestino` (1 o 2). DELETE no re-lista.

**Por qué:** OpenAPI `minItems: 1` / `maxItems: 2` y HU-04-02 («A ofrece B y B ofrece A») sin cambiar la firma de T-04-03/T-04-04.

**Alternativa:** cambiar `create` a `Vec<Relacion>`. Se descartó: No hace = reabrir create. **Alternativa:** devolver solo la arista pedida en POST bidireccional. Se descartó: el contrato admite 2 ítems y el Listo cuando cubre HU-04-02.

### 4. Incoming como método nuevo del puerto

**Elección:** `RelationRepo::list_incoming_for_product`. El adaptador copia `list_for_product` con `out` = producto y `DestinoResumen` del `in`. `list_for_product` no cambia (sigue sin mezclar entrantes). Query HTTP `direction=incoming` llama ese método; `type` filtra en el servicio.

**Por qué:** T-04-04 lo diferió a este ticket; OpenAPI lo declara. Un SELECT extra cabe en el presupuesto.

**Alternativa:** ignorar `incoming` o 400. Se descartó: contrato y deferral. **Alternativa:** un flag en `list_for_product`. Se descartó: rompe la semántica «solo salientes» ya testeada.

### 5. `ProductSummary` mínimo, no ficha

**Elección:** mapper HTTP rellena `target` desde `DestinoResumen`: `productType` `""`, `price`/`stock` `"0"`, `brand` null, `categoryPath` `[]`, `imageUrl` null. `target` viaja (no se omite) para que T-04-09 tenga `sku`/`name`/`active`.

**Por qué:** OpenAPI exige esos campos si `target` está presente; T-04-04 pidió «mapear el resumen mínimo al contrato». Stock real es T-04-06.

**Alternativa:** omitir `target`. Se descartó: la UI de T-04-09 necesita sku/nombre. **Alternativa:** N+1 `ProductRepo::get` por destino. Se descartó: mezcla alcance de recomendaciones y duplica I/O.

### 6. Fake HTTP vs in-memory `recommend`

**Elección:** tests de `api` (`tests/relations.rs`) usan un fake de `RelationGateway`. Tests de `recommend` usan repos en memoria. `vortex_serve_get_unknown_product_relations_returns_404` cubre el composition root (404 ≠ 503). Test de `storage` cubre incoming.

**Por qué:** `#[cfg(test)]` de `recommend` no es visible desde `api`. Surreal en cada test HTTP acoplaría a `storage`.

### 7. Remap de `errors[].field`

**Elección:** extender `remap_api_field` con `origen`→`sourceProductId`, `destino`→`targetProductId`, `prioridad`→`priority`, `nota`→`note`. `tipo` sigue siendo `type` (vale para relaciones; el caso `productType` ya es local a productos). Path inválido → `productId` / `relationId` / `relationType` en el handler.

**Por qué:** 5.3. `self_relation` ya tiene título en `error.rs`.

### 8. OpenAPI `503` y composition root

**Elección:** agregar `'503': Unavailable` a las cuatro operaciones. `serve.rs` construye `RelationService::new(RelacionRepo, ProductoRepo)` (ambos `Arc<Db>`) y `AppState::with_relations`.

**Por qué:** T-09-04 falla si respondemos 503 sin schema. Mismo molde que search/productos.

### 9. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-04-05` y rename a `T-04-05`.

## Risks / Trade-offs

- **[POST bidireccional = 2 lecturas extra]** → listas de relaciones son chicas; no hay RNF de write. Mitigación: tests de HU-04-02 asertan 2 ítems y el GET simétrico.
- **[`ProductSummary` con price/stock "0"]** → un cliente que pinte precio en la pestaña admin verá cero. Mitigación: T-04-09 usa prioridad/nota/bidireccional; precio real es ficha (T-02-35) o recomendaciones (T-04-07).
- **[Incoming + allowlist + HTTP ~2 h]** → incoming es un SELECT espejo ya diferido; allowlist es el mismo parche que T-03-13. No se divide el ticket.
- **[Fake HTTP no escribe Surreal]** → `vortex_serve_get_unknown_product_relations_returns_404` + tests de `storage` incoming cubren persistencia.

## Migration Plan

N/A: proceso local-first, sin datos que migrar. El schema `0007` no cambia. Rollback = no registrar las rutas (rama sin merge).

## Open Questions

Ninguna que altere specs, enfoque o tareas: incoming, placeholders de `ProductSummary` y `ProductRepo` para 404 quedan cerrados arriba.
