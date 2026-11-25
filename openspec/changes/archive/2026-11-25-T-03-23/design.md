# Design

## Context

En `develop`, T-03-15/T-03-17: `mountSearch` aborta, llama solo a `GET /api/v1/search?q=` y pinta `vx-product-card` `full`. T-03-22: `GET /api/v1/search/by-code/{code}` devuelve `{ items: ProductSummary[] }` (SurrealDB, exacto, H-05). El Omnibox no conoce HTTP. Tests Node en `search.test.mjs` + `dom-stub.mjs`. Ver `proposal.md` para el porqué; spec para heurística, pin y fallos independientes.

Constraints: MVP del curso; ~2 h; sin lector de barras; sin OpenAPI ni crates; `openspec new change` exige kebab-case (`t-03-23` → rename `T-03-23`).

## Goals / Non-Goals

**Goals:**

- Cliente paralelo testeable: heurística + `by-code` + `/search` + pin «Código exacto»
- Abort de **ambas** patas en cada tecla; pintar solo la ronda vigente
- OEM compartido del seed (`OEM-PER-8840` → `PER-ACERO-01` y `PER-ACERO-02`) primero

**Non-Goals (diseño):**

- Exigir un dígito en la heurística (el ticket no lo pide; `"perno"` dispara `by-code` y suele volver `items: []`)
- Extender `vx-product-card` con un slot de badge
- Fusionar en el servidor (T-03-22 ya cerró el endpoint)

## Decisions

### 1. Heurística local, no un endpoint nuevo

```js
function looksLikeCode(q) {
  return q.length >= 1 && q.length <= 64 && /^[A-Za-z0-9-]+$/.test(q);
}
```

`q` ya viene de `String(text ?? "").trim()`. Espacios, `+`, `/`, `_` o longitud > 64 → solo `/search`. SKUs/OEM del seed (`PER-M8X40-ZN`, `HX-0840Z`, `OEM-PER-8840`) coinciden.

**Por qué:** el Hace del ticket es una heurística de UI; el servidor ya valida `code` 1..=64. Llamar `by-code` con un path > 64 daría 400 innecesario.

**Alternativa:** exigir al menos un dígito. Se descartó: el ticket dice «letras y dígitos o guiones», no «debe tener dígito»; `"perno"` con `items: []` no pinnea. **Alternativa:** mandar todo token a `by-code` sin tope 64. Se descartó: 400 en el path.

### 2. Un `AbortController` + `Promise.allSettled`

En cada `vx-query` texto no vacío: `previous.abort()`, `controller = new AbortController()`, `request("GET", searchPath, { signal })` siempre; si `looksLikeCode(q)`, también `request("GET", `/api/v1/search/by-code/${encodeURIComponent(q)}`, { signal })`. Esperar con `Promise.allSettled` (o equivalente que no rechace el par).

AbortError / `signal.aborted` / `gen !== generation`: return, sin `showError`. Si una pata fulfilled y la otra rejected no-abort: pintar la fulfilled según la spec e invocar `showError` con el rejection.

**Por qué:** `Promise.all` tiraría el 200 de `/search` si `by-code` da 503 (Meilisearch no está en `by-code`, pero Surreal/gateway sí puede fallar). H-05: las exactas deben poder verse aunque `/search` falle.

**Alternativa:** pintar `/search` apenas llegue y reordenar cuando llegue `by-code`. Se descartó: flicker y tests de orden inestables. **Alternativa:** dos controllers. Se descartó: un abort debe cancelar las dos.

### 3. Etiqueta estática + un listbox; `getItems` solo tarjetas

Si hay `items.length > 0`: insertar un nodo de texto/heading `"Código exacto"` (`<p>` o `<h2 class="…">`, `aria-hidden` innecesario si es visible) **antes** de las tarjetas exactas, dentro o encima del `role="listbox"`. `bindKeyboardList({ getItems: () => cards })` — la etiqueta no es opción. Tokens `--vx-*` existentes; no `vx-chip` (es un `<button>`).

`createProductCardForHit` se reutiliza para `ProductSummary` (mismos campos de tarjeta; sin `highlight` ni `location`). Deduplicar hits de `/search` por `id` (Set de ids pinneados). Orden relativo de `items` = el de la API (sku asc en T-03-22).

**Por qué:** un listbox, un helper de teclado, ArrowDown cae en la primera exacta. Un chip seleccionable rompería DoD de teclado.

**Alternativa:** dos listboxes. Se descartó: T-01-10 asume un contenedor. **Alternativa:** badge en cada tarjeta. Se descartó: el ticket pide **la** etiqueta del grupo, no un atributo nuevo del componente.

### 4. Tests: fake `request` ramifica por path

Extender el fake de `search.test.mjs`: si `path` incluye `/search/by-code/`, devolver `{ items }`; si incluye `/search?`, devolver `SearchResponse`. Para abort: ambas llamadas de la ronda `"p"` quedan pendientes y se abortan al `"pe"`. Fixture de Listo cuando: `items` con `sku` `PER-ACERO-01` y `PER-ACERO-02` (OEM `OEM-PER-8840` del seed). El Listo cuando humano (escribir el OEM en `vortex serve`) no se automatiza (T-09-10).

`search_non_text_mode_does_not_call_search` y `search_admin_view_does_not_call_search` MUST seguir fallando si se llama `by-code` (asertar que `request` no recibe ese path).

## Risks / Trade-offs

- **[Palabras sueltas disparan by-code]** → `items: []` no pinnea; costo de un GET extra por tecla en tokens sin espacio. Aceptado (ticket).
- **[Toast doble si fallan ambas patas]** → `showError` al menos una vez; MAY una sola si se agrupa. Tests no exigen dos toasts.
- **[ProductSummary sin location]** → tarjetas exactas más pobres que un `SearchHit`; el pin importa más que la ubicación. T-03-22 no cambia el DTO.
- **[id ausente en un ítem malformado]** → no pinnear ese ítem; no usarlo para dedup (spec: no lanzar).
- **[macOS kebab-case del CLI]** → scaffold `t-03-23` y rename (igual que T-03-15).

## Migration Plan

N/A (SPA local). Rollback = revertir el change. Ningún consumidor nuevo fuera de `mountSearch`.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
