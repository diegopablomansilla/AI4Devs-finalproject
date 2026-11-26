# Design

## Context

En `develop`, T-02-11 dejó `AtributoDef`, `AttributeDefRepo` (`create`/`get`/`list_all`/`update`/`set_activo`) y `AtributoDefRepo { db: Arc<Db> }` con outbox; UNIQUE `clave` → `ConflictCode::Conflict`. T-02-09 dejó `BrandService` + fachada object-safe, `AppState.with_brands`, DTO 5.3, `assert_conforms` y composition root en `vortex serve`. `AttributeDefRepo::list_all` en storage ya ordena `ORDER BY orden ASC, clave ASC`; el puerto no filtra inactivas. OpenAPI de attribute-definitions declara `includeInactive`, listado «ordenadas por `order`», `POST` 409, `PATCH` 409 (texto de `attribute_in_use`) y `DELETE` **sin** `409`. See proposal.md — Why.

Constraints: MVP del curso; ~2 h; sin UI (T-02-13); `domain` no gana deps; sin `attribute_in_use` (T-02-23); `api` no depende de `storage`.

## Goals / Non-Goals

**Goals:**

- Servicio de aplicación de definiciones de atributo en `catalog` sobre `AttributeDefRepo`, espejo plano de marcas (sin árbol ni occupancy)
- HTTP `/api/v1/attribute-definitions` con DTO 5.3 (incl. mapeo de `tipo`) y tests de `api` contra `openapi.yaml`
- Composition root: `vortex serve` inyecta `AtributoDefRepo`

**Non-Goals (diseño):**

- Puerto de occupancy o consulta a `producto`
- Extraer un helper genérico de maestros compartido con marcas/categorías
- Cambiar `AttributeDefRepo` o la migración `0004_atributo_def`
- Añadir `duplicate_key` a `DomainError` (T-02-11 eligió `conflict`)
- `async-trait`

## Decisions

### 1. `AttributeDefService<R: AttributeDefRepo>` sin occupancy

**Elección:** un servicio genérico que delega `create`/`get`/`list_all`/`update`/`set_activo`. `deactivate` no consulta productos. Idempotencia: baja de ya inactiva y reactivar de ya activa → `Ok` (igual que marcas). `update` de `clave`/`tipo` no consulta uso.

**Por qué:** T-02-23 es el ticket de `attribute_in_use`; no hay tabla `producto` (T-02-14). HU-02-03 escenario «Atributo en uso» queda para ese ticket.

**Alternativa:** un `AttributeOccupancy` simétrico a categorías. Se descartó: contradice el No hace de T-02-11/T-02-12 y no hay productos.

### 2. Fachada object-safe `AttributeDefGateway`

**Elección:** trait con futures boxed (`Pin<Box<dyn Future + Send>>`), igual que `BrandGateway`. `AppState` guarda `Option<Arc<dyn AttributeDefGateway>>`. `new`/`with_health` no inyectan atributos → `GET /api/v1/attribute-definitions` es `503 unavailable`. `with_attribute_defs(...)` para tests y `vortex serve`.

**Por qué:** `build_router` no puede ser genérico sobre el servicio; T-02-06/T-02-09 ya fijaron el patrón. Sin `async-trait`.

**Alternativa:** `api` llama a `AttributeDefRepo`. Se descartó: rompe el hexágono 3.2.

### 3. Fake HTTP vs fake `catalog`

**Elección:** tests de `catalog` usan un `HashMap` en memoria que implementa `AttributeDefRepo` (`clave` única → `conflict`; validación de `NewAtributoDef`/`UpdateAtributoDef`). Tests de `api` usan un fake de `AttributeDefGateway` (no reusan el `HashMap`; `#[cfg(test)]` de `catalog` no es público). El fake HTTP puede devolver `AtributoDef` o `DomainError` configurados, incluido un update de `clave` que **no** falla con `attribute_in_use`.

**Por qué:** las reglas de dominio ya están en `catalog`; `api` prueba contrato + DTO + `assert_conforms`. Surreal en tests de `api` acoplaría a `storage`.

### 4. `includeInactive` y orden en el handler

**Elección:** `list_all` en `catalog` es un wrapper del puerto. El Gateway filtra `activo` según `includeInactive` (default `false`) y ordena por `orden` ascendente y `clave` lexicográfica. No hay query `q`.

**Por qué:** OpenAPI define `includeInactive` y «ordenadas por `order`» como contrato HTTP. Storage ya ordena, pero el fake de `catalog` no; el handler MUST ordenar para que los tests HTTP sean deterministas. Marcas filtraron en HTTP, no en el caso de uso.

**Alternativa:** `AttributeDefService::list(include_inactive)` — se descartó para no duplicar semántica de query en `catalog`.

### 5. Mapeo PATCH (merge + `active`)

**Elección:** `UpdateAtributoDef` exige el snapshot editable completo (T-02-11). El handler carga la fila, fusiona los campos presentes (`key`/`label`/`type`/`unit`/`options`/`facetable`/`searchable`/`order`) y llama `update`. `active: true` → `reactivate`; `active: false` → `deactivate` (igual que `DELETE`), después del update si también hay otros campos. Cuerpo vacío → `400` (`minProperties: 1`). Defaults de `POST`: `facetable = false`, `searchable = true`, `order = 0`.

**Por qué:** el puerto no es patch-parcial; el contrato HTTP sí lo es. Mismo patrón que marcas (`get` + merge).

### 6. Campos de error 5.3 y enum de tipo

**Elección:** extender `remap_api_field`: `clave` → `key`, `etiqueta` → `label`, `tipo` → `type`, `unidad` → `unit`, `opciones` → `options`, `buscable` → `searchable`, `orden` → `order` (`activo` no suele ir en `errors[]`; path inválido → `attributeId`). El DTO HTTP usa `text`/`number`/`boolean`/`enum`; el dominio sigue en español (`TipoAtributo::parse_wire`). Un `type` JSON desconocido → `400 invalid_input` field `type`.

**Por qué:** 5.3 es la tabla única; T-02-11 persistió valores de modelo, no de API.

### 7. OpenAPI `503` y `409 conflict` (no `attribute_in_use` aún)

**Elección:** agregar `'503': { $ref: '#/components/responses/Unavailable' }` a GET/POST/PATCH/DELETE de attribute-definitions (si falta). POST/PATCH ya declaran `409`; el código de duplicado es `conflict` (T-02-11), no `duplicate_key`. **No** quitar el texto de `attribute_in_use` del `description` de PATCH (contrato futuro de T-02-23). **No** agregar `409` en `DELETE`. El test `patch_attribute_def_changes_key_without_in_use` aserta 200, no 409. `204` no llama `assert_conforms`.

**Por qué:** T-09-04 falla si respondemos `503` sin schema. El Listo cuando de T-02-11 ya fijó 409/`conflict` para clave duplicada.

### 8. Composition root

**Elección:** `serve.rs` construye `AttributeDefService::new(AtributoDefRepo::new(Arc::clone(&db)))` y `AppState::with_attribute_defs`. Un test `vortex_serve_get_attribute_definitions_returns_items` junto al de marcas.

**Por qué:** `AtributoDefRepo` ya es `'static` (`Arc<Db>`). Allowlist de `vortex` → `catalog` ya existe.

### 9. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-12` y rename a `T-02-12` (igual que tickets previos). `status` / `validate` aceptan el ID de proyecto.

## Risks / Trade-offs

- **[Orden lexicográfico vs locale es-AR]** → `clave` es `snake_case` ASCII; el desempate no necesita collator.
- **[Filtro `includeInactive` en HTTP, no en Surreal]** → `list_all` carga el maestro completo. Mitigación: maestro pequeño; T-00-21 no aplica.
- **[Fake HTTP vs repo real]** → el test `vortex_serve_get_attribute_definitions_returns_items` cubre el cableado Surreal; el resto es contrato HTTP.
- **[PATCH 409 en OpenAPI sin `attribute_in_use`]** → el schema `Conflict` admite `conflict` (duplicado) y `attribute_in_use` (futuro). Mitigación: tests de este ticket asertan `conflict` en POST duplicado y 200 al cambiar `key`; T-02-23 agregará el 409 de uso.

## Migration Plan

Sin migración `.surql`. Cambio de contrato: `docs/openapi.yaml` declara `503` en operaciones de attribute-definitions. Rollback: revertir handlers, `AppState.with_attribute_defs` y el YAML.

## Open Questions

Ninguna que altere specs o tareas.
