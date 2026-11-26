# Design

## Context

`mountProducts` usa `mode: "list" | "create" | "edit"`. La ficha de edición (`renderEditProductSheet`) pinta Stock y Relaciones con `aria-disabled="true"`; no hay cambio de pestaña ni `POST .../stock-movements`. `api.request` ya envía `X-Vectra-Terminal` (T-01-06). El backend T-02-25 acepta solo `inbound` / `adjustment`, exige `reason` en ajuste, mapea `negative_stock` y no incrementa `version`. Harness Node sin jsdom (stubs HTML como T-02-37). Ver proposal.md.

## Goals / Non-Goals

**Goals:**

- Estado de pestaña `sheetTab: "datos" | "stock"` solo en `mode === "edit"`
- `renderStockPanel` + `readStockMovement` en `product-sheet.js`
- POST + pintar `resultingStock`; actualizar `editSnapshot.stock`
- Tests happy/unhappy de HU-02-07

**Non-Goals:**

- `GET` historial, cursor, export CSV, habilitar Stock en alta, tabs Web Component, cambios OpenAPI/`showError`

## Decisions

### 1. Stock solo en ficha de edición

El `POST` exige `productId`. En alta el producto aún no existe; Stock sigue deshabilitado (test `sheet_create_stock_tab_stays_disabled` / escenario T-02-35). En edición, quitar `aria-disabled` de Stock; Relaciones sigue disabled.

Alternativa descartada: acción «Stock» en la fila del listado — el ticket pide la **pestaña** de la ficha.

### 2. Tab state en `products.js`, no hash

`sheetTab` default `"datos"`. Click en `[data-tab="stock"]` (si no `aria-disabled`) → `sheetTab = "stock"`, `paint()`. Click en Datos → `"datos"`. Cancelar / éxito de PATCH de ficha / error `not_found` al abrir edición resetean a `"datos"`. `parseHash` no cambia.

`renderEditProductSheet(..., { activeTab, stockFieldErrors, lastResultingStock })`: si `activeTab === "stock"`, el form `edit-product` no se pinta (o queda `hidden`) y se pinta `data-form="stock-movement"`; el tablist queda fuera del form de Datos para que el submit de stock no dispare PATCH.

Alternativa descartada: dos forms siempre en el DOM — el harness stub de `querySelector` se complica; un solo panel visible es suficiente.

### 3. Helpers en `product-sheet.js`

- `renderStockPanel({ stock, fieldErrors, lastResultingStock })` — stock RO, select tipo, `vx-field` cantidad/motivo, botón «Registrar movimiento», `role="status"` si hay `lastResultingStock`.
- `readStockMovement(root, form)` — `{ type, quantity, reason? }` o `null` si cantidad vacía. Si `type === "adjustment"` y motivo vacío → `{ error: "reason" }` (el caller pinta `invalid_input` sin POST). `quantity` siempre string. Omite `reason` en inbound vacío.

Guardia local de `inbound` con cantidad ≤ 0: no es obligatoria (la API responde `invalid_input` en `quantity`); MAY omitir el POST si cantidad no parseable.

### 4. Submit de `stock-movement`

En el listener `submit` de `products.js`, rama `data-form="stock-movement"`: `preventDefault`, `readStockMovement`, POST a `/api/v1/products/${editProductId}/stock-movements`. Éxito: `editSnapshot = { ...editSnapshot, stock: movement.resultingStock }`, `lastResultingStock = movement.resultingStock`, `stockFieldErrors = {}`, `paint()`. Catch: `negative_stock` / `network` / `not_found` → `showError` sin mutar stock; `invalid_input` → asignar `errors[].field` a `stockFieldErrors` + `showError`. MUST NOT llamar PATCH ni GET de historial.

`X-Vectra-Terminal`: no tocar `products.js`; el `request` inyectado en tests no la ve (igual que T-02-35). Manual: terminal «caja-1» en el campo admin.

### 5. CSS

Clases `.vx-product-stock-panel`, `.vx-product-stock-result` con tokens `--vx-*` existentes. Tabs enabled vs `[aria-disabled="true"]` ya está en `base.css`.

### 6. Tests

Extender stubs: `querySelector('[data-tab="stock"]')`, `form[data-form="stock-movement"]`, `select[name="type"]`, `vx-field[name="quantity"|"reason"]`. Helper `openEditStockTab` sobre `openEditSheet` de T-02-37. Ficha sample con `stock: "10"` para el ingreso HU-02-07.

## Risks / Trade-offs

- [Repaint pierde valores del form de Datos al ir a Stock] → aceptable; Cancelar/Guardar de Datos no están en esa pestaña. MAY conservar el form Datos en DOM oculto si el harness lo permite; no es requisito.
- [Stock 201 no refresca `GET` ficha] → `resultingStock` es la proyección; un movimiento concurrente de otra terminal no se ve hasta reabrir. Mitigación: fuera de alcance (sin GET historial ni polling).
- [Rama apilada sobre T-02-37] → si #149 no entra a `develop`, el PR de T-02-38 debe esperar o rebasarse. No se reimplementa Editar aquí.

## Migration Plan

Rollback revierte la pestaña a `aria-disabled` en edición; el listado y PATCH de ficha no cambian. Sin migración de datos.

## Open Questions

Ninguna.
