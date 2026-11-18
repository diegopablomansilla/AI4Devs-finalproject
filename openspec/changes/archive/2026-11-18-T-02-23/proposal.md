# Proposal

## Ticket padre

- **Ticket:** [T-02-23](../../../docs/7-tickets.md) — Regla `attribute_in_use`
- **Change:** `T-02-23`
- **HU:** [HU-02-03](../../../docs/6-historias.md#hu-02-03--definición-de-atributos-técnicos) (escenario «Atributo en uso»)
- **Depende de:** T-02-12 (casos de uso y `PATCH /api/v1/attribute-definitions/{attributeId}` sin occupancy); T-02-18 (producto con `atributos[*].def` persistido; T-02-19/T-02-21 ya hidratan y exponen la ficha)
- **Hace:** al editar una `AtributoDef`, si algún producto la usa: cambio de `key` → `attribute_in_use`; cambio de `type` → se rechaza si algún valor existente es incompatible.
- **No hace:** migración de valores.
- **Listo cuando:** tests con un producto que usa "medida".

## Why

HU-02-03 exige que, si "medida" ya está en productos, cambiar su clave o pasar a un tipo incompatible falle con `attribute_in_use`. T-02-12 dejó el `PATCH` y documentó el `409` en OpenAPI, pero el servicio aún aplica cualquier cambio de `key`/`type`. Sin la regla, Meilisearch (`attr_<clave>`) y los valores tipados de la ficha se romperían en silencio.

## What Changes

- Puerto de occupancy `AttributeUsage` en `domain`: lista los `ValorAtributo` persistidos para un `Id<AtributoDef>` (productos activos e inactivos)
- `AttributeDefService::update` aplica la regla **antes** de `AttributeDefRepo::update`:
  - hay valores y cambia `clave` → `DomainError::Conflict` `attribute_in_use`
  - hay valores y el nuevo `tipo`/`opciones` deja algún valor inválido (mismas reglas que `Producto::try_new`) → `attribute_in_use`
  - sin productos, o cambio de etiqueta/unidad/facetable/buscable/orden (y `tipo`/`opciones` compatibles) → el update sigue
- Adaptador SurrealDB de `AttributeUsage` sobre `producto.atributos` (no escribe, no emite outbox)
- `PATCH /api/v1/attribute-definitions/{attributeId}` propaga el conflicto a `409` `attribute_in_use` (`problem+json` ya mapeado en T-00-11); `assert_conforms`
- Tests TDD: catalog (memoria) + storage (producto "medida" = `M8`) + HTTP; baja/reactivación de la definición **no** se bloquea

## Non-goals

- Migrar o convertir valores ya persistidos cuando el tipo o la clave cambiarían — **No hace** del ticket
- Bloquear `DELETE` / `PATCH { active: false }` de una definición usada (T-02-12: occupancy solo en edición de `key`/`type`; OpenAPI no declara `409` en `DELETE`)
- Worker *fan-out* / `filterableAttributes` de Meilisearch — **T-03-08**
- UI de atributos (el toast de T-02-13 ya muestra `attribute_in_use` si llega) — **T-02-13**
- Seed de atributos — **T-02-42**
- Occupancy real de categorías (`CategoryOccupancy` sigue en stub) — no es este ticket
- Ampliar `DomainError` / `ConflictCode::AttributeInUse` (ya existen desde T-00-03)
- Cambiar el contrato OpenAPI (PATCH ya describe `409 attribute_in_use`; no se agregan paths)
- Codegen OpenAPI, e2e Playwright, sidecars

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `domain-atributo-def`: puerto `AttributeUsage` (`valores(id) → Vec<ValorAtributo>`); `AttributeDefRepo` sigue sin occupancy
- `vortex-storage`: adaptador `AttributeUsage` consulta `producto.atributos`; `AtributoDefRepo::update` sigue sin aplicar la regla
- `catalog-atributo-def`: `AttributeDefService::update` inyecta occupancy y rechaza clave/tipo incompatibles en uso
- `vortex-api`: `PATCH` de definición usada con cambio de `key` o `type` incompatible → `409 attribute_in_use`; definición sin productos sigue pudiendo cambiar `key`

## Impact

- `vortex-core/crates/domain/src/atributo_def.rs` (puerto `AttributeUsage`; helper de compatibilidad de valor/tipo si se extrae de `producto.rs`)
- `vortex-core/crates/storage/src/producto.rs` (o módulo de occupancy) + `tests/producto.rs` / `tests/atributo_def.rs`
- `vortex-core/crates/catalog/src/atributo_def.rs` (servicio + fake de occupancy) y reexport
- `vortex-core/crates/api/tests/attribute_definitions.rs`, `tests/support/fake_catalog.rs`
- `vortex-core/bin/vortex/src/serve.rs` (inyectar occupancy real en `AttributeDefService`)
- `docs/openapi.yaml`: sin cambio de schema (el `409` de PATCH ya está)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-23 (Listo cuando: tests con un producto que usa "medida")
- Ámbito: occupancy estrecho + regla en `update`; cabe en ~2 h. No se divide el ticket.
