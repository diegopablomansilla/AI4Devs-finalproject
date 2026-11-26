# Design

## Context

T-02-05 dejó `CategoryService<R, O>` genérico (RPITIT, no object-safe) y `CategoriaRepo<'a>` con `&'a Db`. El Gateway no registra `/api/v1`. `AppState::new` se usa en decenas de tests sin catálogo. `DELETE` en OpenAPI no declara `409`. See proposal.md — Why.

## Goals / Non-Goals

**Goals:** primer recurso de negocio del Gateway; DTO 5.3; tests de `api` contra `openapi.yaml`; composition root en `vortex serve`; allowlists.

**Non-Goals:** UI; occupancy real; reindex; `GET` por id; ampliar `UpdateCategoria` con slug.

## Decisions

1. **Fachada object-safe en `catalog`.** Trait con futures boxed (`Pin<Box<dyn Future + Send>>`) sobre los métodos de `CategoryService`. `AppState` guarda `Option<Arc<dyn …>>`. Sin `async-trait` (no está en el workspace).
   - Alternativa: `api` genérico sobre el servicio — rompería `build_router` y todos los tests.
   - Alternativa: `api` llama a `CategoryRepo` — H-10 quedaría en el Gateway (descartado en T-02-05).

2. **`AppState` opcional.** `new`/`with_health` no inyectan catálogo → `GET /api/v1/categories` es `503 unavailable`. `with_categories(...)` para tests y `vortex serve`. Los tests de `/health` no cambian.
   - Alternativa: stub por defecto que lista vacío — ocultaría un serve sin wiring.

3. **Fake en tests de `api`.** Implementa la fachada; no reusa el `HashMap` de `catalog` (`#[cfg(test)]`). El fake devuelve `Categoria` o `DomainError` configurados. Las reglas de árbol ya están en `catalog`; aquí se prueba HTTP + DTO + `assert_conforms`.
   - Alternativa: Surreal en tests de `api` — más lento y acoplaría `api` a `storage` (el diagrama 3.2 lo prohíbe).

4. **Mapeo PATCH.** `name` → `rename`; `parentId` → `move_category` (`null` = raíz); `active: true` → `reactivate`; `active: false` → `deactivate` (igual que `DELETE`). Si vienen `name` y `parentId`, rename y después move. `slug` solo en PATCH → `400 invalid_input`.
   - Alternativa: un `update` combinado en `catalog` — fuera de T-02-05.

5. **`includeInactive` en el handler.** `list_all` y filtrar `activo` en el Gateway (dato, no regla de negocio). Default `false`.

6. **Campos de error 5.3.** Al traducir `DomainError::InvalidInput`, remapear `nombre`→`name`, `padre`→`parentId` (y `slug` se queda). El resto de `ApiError` no cambia.

7. **`CategoriaRepo` con `Arc<Db>`.** Necesario para `'static` en `AppState`. Tests de `storage` existentes siguen pasando (`CategoriaRepo::new` acepta `Arc` o se construye `Arc::new` en el harness).
   - Alternativa: `Box::leak` del `Db` en serve — inaceptable.

8. **Occupancy stub público** en `catalog` que siempre devuelve `false`. H-10 de productos no rige en HTTP hasta T-02-18. H-10 de hijas activas sí (el servicio usa `list_all`).

9. **OpenAPI `DELETE` + `409`.** Sin ese schema, `assert_conforms` del unhappy H-10 falla (T-09-04). `204` no tiene content: el test aserta estado y cuerpo vacío, no llama `assert_conforms`.

10. **Allowlist.** `api` + `catalog` (path); `vortex` + `catalog` (path). `scripts/check-workspace.sh` y el fixture de `api` fuera de allowlist (p. ej. `clap`) siguen; el happy usa los `Cargo.toml` reales.

## Risks / Trade-offs

- [Occupancy `false`] → se puede dar de baja una categoría con productos activos por HTTP hasta T-02-18. Mitigación: documentado en T-02-05; no hay tabla `producto`.
- [PATCH rename+move en dos round-trips] → no atómico. Mitigación: árbol típico pequeño; T-02-05 no expone update combinado.
- [Fake HTTP vs repo real] → el test `vortex_serve_get_categories_returns_items` cubre el cableado Surreal; el resto es contrato HTTP.

## Migration Plan

Sin migración `.surql`. Cambio de `CategoriaRepo<'a>` a handle poseído: recompilar `storage` tests. `docs/openapi.yaml`: agregar `'409': { $ref: '#/components/responses/Conflict' }` en `DELETE /api/v1/categories/{categoryId}`.

## Open Questions

Ninguna que altere specs o tareas.
