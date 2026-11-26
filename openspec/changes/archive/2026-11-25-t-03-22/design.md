# Design

## Context

Ticket T-03-22, HU-03-08, decisión H-05: `by-code` consulta SurrealDB (no Meilisearch). Índices ya migrados: `uq_producto_sku`, `idx_producto_codigos` sobre `codigos[*].valor`.

## Decisiones

### Coincidencia exacta

Igualdad de string tras percent-decode del path y `trim`. Case-sensitive. Sin edgengram ni `LIKE`/`CONTAINS` sobre el código buscado.

### Campos coincidentes

`sku = $code` **OR** existe un elemento en `codigos` cuyo `valor = $code` (cualquier `tipo`: fabricante, oem, barras). OEM compartido puede devolver N productos.

### Solo activos

`activo = true`. Inactivos nunca aparecen aunque el código coincida.

### Orden

`sku` ascendente (estable; el contrato OpenAPI no ordena).

### HTTP

- Éxito: `200` `{ "items": ProductSummary[] }`. Sin matches → `items: []` (no 404).
- `code` vacío tras trim o `len > 64` → `400 invalid_input`, `errors[].field` = `code`.
- Sin `ProductGateway` → `503 unavailable` (mismo patrón que catálogo). **No** requiere `SearchGateway` (Meilisearch caído no afecta `by-code`).

### Reuso

Mapper `ResumenProducto` → `ProductSummaryResponse` igual que `list_products`; `imageUrl` = `null`.

### SurrealQL

Parámetro `$code` enlazado; no interpolar el path en SQL. Condición de códigos alineada al índice `idx_producto_codigos` (iterar `codigos` y comparar `valor`).

### Ubicación del handler

Extender `api/src/search.rs` con `GET /by-code/{code}` bajo el prefijo de búsqueda, o ruta plana en el router; mismo crate `api`, sin dependencia nueva de `search` para la consulta DB (solo `ProductGateway`).

## Riesgos

- Sintaxis SurrealDB para filtrar array de objetos: validar en integración `storage/tests/producto.rs` (TDD).

## Verificación

[docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5: `cargo fmt`, `clippy -D warnings`, `cargo test --workspace`, `openspec validate t-03-22`.