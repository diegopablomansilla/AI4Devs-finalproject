# Proposal

## Ticket padre

- **Ticket:** [T-02-20](../../../docs/7-tickets.md) — Casos de uso de producto
- **Change:** `T-02-20`
- **HU:** [HU-02-04](../../../docs/6-historias.md#hu-02-04--alta-de-producto), [HU-02-05](../../../docs/6-historias.md#hu-02-05--edición-con-bloqueo-optimista)
- **Depende de:** T-02-19 (en `develop`: `ProductRepo::create` / `get` hidratado / `update` con `version_conflict`; `FichaProducto`; validación de atributos en `Producto::try_new`; storage **no** comprueba categoría/marca activas)
- **Hace:** crear, obtener y editar en `catalog`, cargando las `AtributoDef` para validar y verificando que categoría y marca existan y estén activas.
- **No hace:** baja (T-02-22).
- **Listo cuando:** tests con repositorios en memoria.

## Why

HU-02-04 exige el alta de "Perno hexagonal M8 x 40 zincado" con categoría, marca y atributos válidos, y rechazar SKU duplicado y un valor `"cuarenta"` en un atributo `number`. HU-02-05 exige editar con la `version` leída y `version_conflict` si otra terminal ya guardó. T-02-19 dejó persistencia y bloqueo optimista, pero `catalog` aún no orquesta la ficha: no carga `AtributoDef` ni rechaza categoría/marca inexistentes o inactivas. Sin ese servicio, T-02-21 no puede mapear `POST`/`GET`/`PATCH /products` sobre un hexágono.

## What Changes

- Casos de uso en el crate `catalog`: crear, obtener ficha y editar producto
- Antes de persistir: cargar las `AtributoDef` referenciadas para validar valores; exigir que categoría y marca (si hay) existan y `activo = true`
- Edición: fusionar el parche de ficha (campos opcionales + `version`) sobre la ficha actual y llamar `ProductRepo::update` con snapshot completo; **sin** `stock` ni `activo`
- Repositorios en memoria (`ProductRepo`, `CategoryRepo`, `BrandRepo`, `AttributeDefRepo`) para TDD (Listo cuando)
- Fachada object-safe `ProductGateway` para inyección en T-02-21 (`Arc<dyn …>`)
- Errores de dominio sin remapeo HTTP (`duplicate_sku`, `invalid_input`, `version_conflict`, `not_found`)

## Non-goals

- HTTP `POST`/`GET`/`PATCH /api/v1/products`, DTO inglés, `openapi.yaml`, `assert_conforms`, `problem+json` — **T-02-21**
- Baja y reactivación (`activo`, `set_activo`, `DELETE` / `PATCH { active: true }`) — **T-02-22**
- Listado admin paginado — **T-02-27** / **T-00-21**
- `StockLedger.append` / ingresos y ajustes — **T-02-24** / **T-02-25**
- Imágenes en la ficha — **T-02-30** (v2)
- Worker Meilisearch / aparición en búsqueda en &lt; 1 s — **T-03-07**
- UI de ficha / conflicto de versión en SPA — **T-02-35** / **T-02-37**
- `vortex seed` de productos — **T-02-43**
- Ampliar `DomainError` (`duplicate_sku`, `version_conflict`, `invalid_input` ya existen)
- Spans de `tracing` en `catalog` (el Gateway instrumenta en T-02-21)
- Cambiar `ProductRepo`, hidratación Surreal o migraciones — ya cerrados en T-02-18 / T-02-19

## Capabilities

### New Capabilities

- `catalog-producto`: casos de uso de producto en `catalog` (crear, obtener, editar) con validación de `AtributoDef` y de categoría/marca activas; fake en memoria; fachada object-safe; sin baja

### Modified Capabilities

- (ninguna; `domain-producto` y `vortex-storage` ya delegan existencia/activo de categoría y marca y la orquestación de defs a este ticket)

## Impact

- `vortex-core/crates/catalog/` (`src/producto.rs`, reexport en `lib.rs`); fakes de test locales o reuso de `marca`/`atributo_def` `testing`
- Sin cambios de API HTTP, `openapi.yaml`, `storage` ni `domain`
- Allowlist de `catalog` ya permite `domain` + `tokio` (dev); no hay deps nuevas
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-20 (Listo cuando: tests con repositorios en memoria)
- Ámbito: orquestación sobre puertos existentes; cabe en ~2 h. No se divide el ticket.
