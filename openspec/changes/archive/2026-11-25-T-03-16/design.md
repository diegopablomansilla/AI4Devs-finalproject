# Design

## Context

T-03-15 lista en `data-region="results"` con `textContent`. API devuelve `highlight.name` / `highlight.description` con `<mark>` (T-03-13). `vx-product-card` ya tiene slot `name` (T-01-05) pero no se usa en la lista aún.

## Goals / Non-Goals

**Goals:** utilidad pura testeable en Node; nodos DOM (`mark` + texto); sin cambiar contrato API.

**Non-Goals:** tarjetas (T-03-17); tags distintos de `mark`; DOMPurify/npm.

## Decisions

### 1. `sanitizeHighlight` + parser de nodos

Escape `& < > " '`, luego restaurar solo `&lt;mark&gt;` / `&lt;/mark&gt;` (case-insensitive). `appendSanitizedHtml` divide por tokens `<mark>`/`</mark>` y crea `Text` + `mark` con `textContent` en hijos (nunca `innerHTML` del payload).

### 2. `search.js` pinta highlight en nombre y descripción

Preferir `highlight.name` si presente; si no, `name`. Segunda línea opcional si `highlight.description`. SKU con `textContent`/`TextNode`. Mantener `role="listbox"` y `bindKeyboardList`.

### 3. Tests Node

`highlight.test.mjs` unitario; integración en `search.test.mjs`. Extender `dom-stub` con `createTextNode`, `textContent` agregado y `querySelector('mark'|'img')` mínimo.

## Open Questions

Ninguna.