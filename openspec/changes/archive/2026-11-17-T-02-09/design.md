# Design

## Context

En `develop`, T-02-08 dejó `Marca`, `BrandRepo` (`create`/`get`/`list_all`/`update`/`set_activo`) y `MarcaRepo { db: Arc<Db> }` con outbox. T-02-05/T-02-06 dejaron `CategoryService` + fachada object-safe, `AppState` opcional, DTO 5.3, `assert_conforms` y composition root en `vortex serve`. `BrandRepo::list_all` no ordena ni filtra (`SELECT * FROM marca`). OpenAPI de brands declara `q`, `includeInactive`, `BrandUpdate.slug` y `DELETE` **sin** `409`. See proposal.md — Why.

Constraints: MVP del curso; ~2 h; sin UI (T-02-10); `domain` no gana deps; no occupancy (HU-02-02); `api` no depende de `storage`.

## Goals / Non-Goals

**Goals:**

- Servicio de aplicación de marcas en `catalog` sobre `BrandRepo`, espejo plano de categorías (sin árbol ni H-10)
- HTTP `/api/v1/brands` con DTO 5.3 y tests de `api` contra `openapi.yaml`
- Composition root: `vortex serve` inyecta `MarcaRepo`

**Non-Goals (diseño):**

- Puerto de occupancy o consulta a `producto`
- Extraer un helper genérico de maestros (list/filter DTO) compartido con categorías
- Cambiar `BrandRepo` o la migración `0003_marca`
- `async-trait`

## Decisions

### 1. `BrandService<R: BrandRepo>` sin occupancy

**Elección:** un servicio genérico que delega `create`/`get`/`list_all`/`update`/`set_activo`. `deactivate` no consulta productos. Idempotencia: baja de ya inactiva y reactivar de ya activa → `Ok` (igual que categorías).

**Por qué:** el ticket exige baja con productos asociados permitida. Un puerto de occupancy contradiría HU-02-02 y no hay tabla `producto` (T-02-14).

**Alternativa:** `BrandOccupancy` simétrico a categorías. Se descartó: el Listo cuando y OpenAPI `DELETE` no tienen `409`.

### 2. Fachada object-safe `BrandGateway`

**Elección:** trait con futures boxed (`Pin<Box<dyn Future + Send>>`), igual que `CategoryGateway`. `AppState` guarda `Option<Arc<dyn BrandGateway>>`. `new`/`with_health` no inyectan marcas → `GET /api/v1/brands` es `503 unavailable`. `with_brands(...)` para tests y `vortex serve`.

**Por qué:** `build_router` no puede ser genérico sobre el servicio; T-02-06 ya fijó el patrón. Sin `async-trait` (no está en el workspace).

**Alternativa:** `api` llama a `BrandRepo`. Se descartó: rompe el hexágono 3.2 y mezclaría HTTP con casos de uso.

### 3. Fake HTTP vs fake `catalog`

**Elección:** tests de `catalog` usan un `HashMap` en memoria que implementa `BrandRepo` (slug único → `duplicate_slug`, `Slug::from_nombre`). Tests de `api` usan un fake de `BrandGateway` (no reusan el `HashMap`; `#[cfg(test)]` de `catalog` no es público). El fake HTTP puede devolver `Marca` o `DomainError` configurados, incluida una marca “con productos” que igual acepta `deactivate` (escenario HU-02-02).

**Por qué:** las reglas de dominio ya están en `catalog`; `api` prueba contrato + DTO + `assert_conforms`. Surreal en tests de `api` acoplaría a `storage` (diagrama 3.2).

### 4. `includeInactive`, `q` y orden en el handler

**Elección:** `list_all` en `catalog` es un wrapper del puerto. El Gateway filtra `activo` según `includeInactive` (default `false`), filtra prefijo **case-insensitive** de `nombre` según `q`, y ordena por `nombre` lexicográfico. `q.len() > 100` → `400 invalid_input` field `q`.

**Por qué:** OpenAPI define `q`/`includeInactive`/“ordenadas por nombre” como contrato HTTP, no como regla de `BrandRepo`. Categorías ya filtran `includeInactive` en el handler.

**Alternativa:** `BrandService::list(q, include_inactive)` — se descartó para no duplicar semántica de query en el crate de casos de uso; T-02-06 dejó el filtro en HTTP.

### 5. Mapeo PATCH (slug permitido)

**Elección:** `name` → `update` con el `nombre` nuevo (slug `None` si no viene `slug`); `slug` → `update` conservando `nombre` actual; `active: true` → `reactivate`; `active: false` → `deactivate` (igual que `DELETE`). Si vienen `name` y `slug`, un solo `update`. Cuerpo vacío → `400` (`minProperties: 1`). A diferencia de categorías, `PATCH` solo con `slug` es válido (`UpdateMarca.slug` existe).

**Por qué:** T-02-08 diseñó `UpdateMarca { nombre, slug: Option<Slug> }` alineado a `BrandUpdate`.

### 6. Campos de error 5.3

**Elección:** extender `remap_api_field`: `nombre` → `name` (ya existe). Path inválido → `brandId` (espejo de `invalid_category_id`). `q` demasiado largo se construye en el handler con field `q`.

**Por qué:** 5.3 es la tabla única; los tests unhappy asertan que no aparece `nombre`.

### 7. OpenAPI `503` y `DELETE` sin `409`

**Elección:** agregar `'503': { $ref: '#/components/responses/Unavailable' }` a GET/POST/PATCH/DELETE de brands (si falta). **No** agregar `409` en `DELETE`. El test `delete_brand_with_associated_products_returns_204` aserta 204, no 409. `204` no llama `assert_conforms`.

**Por qué:** T-09-04 falla si respondemos `503` sin schema. El ticket y HU-02-02 prohíben tratar la ocupación como conflicto.

### 8. Composition root

**Elección:** `serve.rs` construye `BrandService::new(MarcaRepo::new(Arc::clone(&db)))` y `AppState::with_brands`. Un test `vortex_serve_get_brands_returns_items` junto al de categorías.

**Por qué:** `MarcaRepo` ya es `'static` (`Arc<Db>`). Allowlist de `vortex` → `catalog` ya existe (T-02-06).

### 9. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-09` y rename a `T-02-09` (igual que tickets previos). `status` / `validate` aceptan el ID de proyecto.

## Risks / Trade-offs

- **[Orden lexicográfico vs locale es-AR]** → `nombre` con mayúsculas/acentos usa orden Unicode de Rust. Mitigación: catálogo piloto es ASCII-dominante; no hay collator en el workspace.
- **[Filtro `q` en HTTP, no en Surreal]** → `list_all` carga el maestro completo. Mitigación: maestro pequeño ([5.3](../../docs/5-contract-api.md) listados sin paginar); T-00-21 no aplica.
- **[Fake HTTP vs repo real]** → el test `vortex_serve_get_brands_returns_items` cubre el cableado Surreal; el resto es contrato HTTP.
- **[Baja con productos no se puede persistir aún]** → no hay tabla `producto`. Mitigación: el servicio no tiene occupancy; el test HTTP documenta 204 vs 409.

## Migration Plan

Sin migración `.surql`. Cambio de contrato: `docs/openapi.yaml` declara `503` en operaciones de brands. Rollback: revertir handlers, `AppState.with_brands` y el YAML.

## Open Questions

Ninguna que altere specs o tareas.
