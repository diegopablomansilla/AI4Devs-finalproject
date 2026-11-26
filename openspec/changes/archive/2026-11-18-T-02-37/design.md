# Design

## Context

`mountProducts` usa `mode: "list" | "create"`. La ficha de alta ya hace POST con atributos/códigos (T-02-36). Reactivar hace `GET` + `PATCH { version, active: true }` sin diff. `messageFor("version_conflict")` existe (T-01-07). `Product` trae `version`, `stock`, `attributes` con `definitionId`/`key`, `codes`, `brand`/`category` refs. Harness Node sin jsdom. Ver proposal.md.

## Goals / Non-Goals

**Goals:**

- `mode: "edit"` con `editProductId` y snapshot `editSnapshot` para diff tras conflicto
- `renderEditProductSheet` / `readProductUpdate` / `diffChangedProductFields`
- PATCH + recarga en `version_conflict`; stock RO
- Tests happy/unhappy; supersede «no ficha de edición» en spec delta

**Non-Goals:**

- parseHash profundo, merge 3-way, pestaña Stock operable, cambios OpenAPI/showError

## Decisions

### 1. Entrada: `data-action="edit"` en cada fila

Al click: `mode = "edit"`, `editProductId`, limpiar `fieldErrors`, pedir `GET /api/v1/products/{id}` + `GET /api/v1/attribute-definitions`, guardar respuesta en `editSnapshot` y `paint()`.

### 2. `product-sheet.js`: edit vs create

- `renderEditProductSheet(product, categories, brands, attributeDefs, fieldErrors, changedFields)` — título «Editar producto», `data-form="edit-product"`, sin `initialStock`, bloque stock RO (`<output>` o texto), prefill vía atributos `value` en `vx-field` y `selected` en selects.
- `readProductUpdate(root, form, attributeDefs)` — mismo parseo que create pero body incluye solo campos presentes + `version` desde `data-product-version` del form.
- `diffChangedProductFields(before, after)` — compare sku, name, productType, description, brandId, categoryId, unitOfMeasure, price, location (aisle/shelf/position), attributes (by definitionId), codes (by type). Retorna `{ keys: string[], labels: string[] }` en español para el status.

Atributos en PATCH: incluir definiciones inactivas y códigos `barcode` si venían en el GET aunque no se pinten.

### 3. Conflicto

En `submit` edit: `PATCH` → catch `version_conflict`: `showError`, `GET` fresco, `diffChangedProductFields(editSnapshot, fresh)`, actualizar `editSnapshot` y `fieldErrors` vacío, `changedFields` para CSS/`role="status"`, `paint()`. **No** conservar valores del form previo al GET.

### 4. Éxito

`201`-equivalent 200: `mode = list`, reload first page.

### 5. CSS

`.vx-product-stock-readonly`, `.vx-product-field-changed` con tokens `--vx-*` existentes.

### 6. Tests

Extender stubs para `data-form="edit-product"`, `openEditSheet` helper, ficha `Product` sample con version 3.

## Risks / Trade-offs

- [Repaint pierde draft local tras conflicto] → esperado (No hace merge).
- [Índice attributes en PATCH] → remap de errores igual que create.

## Migration Plan

Rollback revierte UI; listado vuelve a sin Editar.

## Open Questions

Ninguna.