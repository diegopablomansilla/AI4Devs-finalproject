# Proposal

## Ticket padre

- **Ticket:** [T-02-37](../../../docs/7-tickets.md) — UI edición con conflicto de versión
- **Change:** `T-02-37`
- **HU:** [HU-02-05](../../../docs/6-historias.md#hu-02-05--edición-con-bloqueo-optimista)
- **Depende de:** T-02-36 (ficha de alta con atributos/códigos, en `develop`)
- **Hace:** edición que envía `version`; ante `version_conflict`, recarga la ficha y muestra qué campos cambió la otra terminal; el stock es solo lectura.
- **No hace:** combinación automática de cambios.
- **Listo cuando:** con dos pestañas se reproduce el escenario «Conflicto» de HU-02-05.

## Why

HU-02-05 exige editar fichas con bloqueo optimista: el administrador envía la `version` leída y, si otra terminal ya guardó, debe ver qué cambió sin perder el contexto del conflicto. T-02-35/36 cubren alta; el backend y `openapi.yaml` ya exponen `PATCH` con `version_conflict` (T-02-21). La SPA aún **prohíbe** abrir ficha de edición y no implementa recarga ni diff tras el 409.

## What Changes

- Acción **Editar** por fila en el listado (`#/admin/productos`); `mode: "edit"` sin cambiar el hash.
- Ficha de edición reutiliza la pestaña Datos de `product-sheet.js`, hidratada con `GET /api/v1/products/{id}` + lookups; **stock** visible solo lectura.
- **Guardar** envía `PATCH` con `version` y campos de ficha (sin `stock`, `initialStock`, `active`).
- Ante `409` `version_conflict`: toast en español, `GET` fresco, repinta el formulario con valores del servidor, `role="status"` listando campos que difirieron entre el snapshot anterior y la ficha nueva, y marca visual en esos controles. **No** reaplica los cambios locales (No hace merge).
- TDD Node: happy (PATCH con version, stock RO) + unhappy (conflicto+diff, invalid_input, not_found, red, vacíos, no-merge).
- Supersede el requirement «MUST NOT abrir ficha de edición»; el listado **sigue sin** edición en línea en la tabla.

## Non-goals

- Merge / resolución automática de conflictos — **No hace** del ticket
- Pestaña Stock (T-02-38), Imágenes (T-02-39), Relaciones (T-04-09)
- `barcode`, deep-link `#/admin/productos/{id}`, cambios a OpenAPI
- Playwright, jsdom, `package.json`, crates Rust nuevos

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `vectra-web`: ficha de **edición** con `PATCH` + `version`; manejo de `version_conflict` con recarga y diff de campos; stock solo lectura; acción Editar en listado; suite Node ampliada.

## Impact

- **Repo:** `vectra-web/js/admin/product-sheet.js`, `vectra-web/js/admin/products.js`, `vectra-web/css/base.css`, `scripts/fixtures/vectra-web/product-sheet.test.mjs`, `scripts/fixtures/vectra-web/products.test.mjs`
- **APIs:** consumo existente de `GET`/`PATCH /api/v1/products/{id}` según [docs/openapi.yaml](../../../docs/openapi.yaml); MUST NOT agregar paths
- **Verificación:** `./scripts/check-vectra-web.sh`, `openspec validate T-02-37`, regresión cargo en `vortex-core/` (§5)