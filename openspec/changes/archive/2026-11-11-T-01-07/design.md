# Design

## Context

`develop` ya tiene `vectra-web/js/api.js` (`ApiError` con `code` / `errors[]`, abort ≠ `network`) y los overlays `vx-toast` / `vx-field` (T-01-04). El Gateway titula cada `DomainCode` en español (`api::title_for_code`). El enum `ErrorCode` vive en `docs/openapi.yaml`. Harness: `check-vectra-web.sh` + `node --test` sin npm, con `dom-stub.mjs` para custom elements. Ver proposal.md para el porqué; la spec para el contrato de `messageFor` / `showError` y la tabla de mensajes.

`vx-toast` copia `textContent` del host al live region y observa `variant`. `vx-field` pinta el atributo `error` con `aria-invalid`. No hay vistas ABM que llamen a `showError` todavía.

## Goals / Non-Goals

**Goals:**

- Un mapa único testeable: enum OpenAPI + `network` → español
- `showError` reutilizable por tickets posteriores, con toast y campos inyectables en Node
- Listo cuando automatizado: extraer `ErrorCode` del YAML vs claves del mapa, sin parser YAML de npm

**Non-Goals:**

- Importar `errors.js` desde `app.js` (no hay callers de negocio)
- Sincronizar `title_for_code` del crate `api` en Rust (la SPA copia la redacción; el Gateway no se toca)
- Match de campos anidados más allá de igualdad exacta de `name` (`attributes[1].value` espera un `vx-field` con ese `name` cuando un ABM lo ponga)

## Decisions

### 1. Mapa como fuente de verdad, no `err.title`

`messageFor(code)` lee `MESSAGES[code]`; si falta, `MESSAGES.internal`. `showError` **no** usa `err.title` ni `err.message` para el toast: T-01-06 pone `title: "network"` en fallos de `fetch`, y un `problem+json` puede repetir el `code` en `title`. HU-01-05 pide el texto asociado al `code`, sin técnicos.

Alternativa descartada: preferir `err.title` si parece español — frágil y rompería `network`.

Los strings de la tabla de la spec copian `title_for_code` salvo:

- `network` (solo cliente)
- `unsupported_media_type` (el Gateway hoy cae al fallback `"Error"`; la SPA muestra `"El tipo de contenido no está soportado"` sin cambiar OpenAPI ni Rust)

### 2. Firma del ticket + factory para tests

`errors.js` exporta `MESSAGES`, `messageFor`, `showError` y `createErrorUi({ toast, root })`. El `showError` de módulo usa `document.querySelector("vx-toast")` y `document` como root; si no hay toast y hay `document.body`, crea un `vx-toast` y lo appende. Los tests llaman `createErrorUi` con un toast y un root falsos (`setAttribute` / `textContent` / `querySelector`).

No importa `api.js`. `showError` duck-typea `{ code?, errors?, name? }`.

Alternativa descartada: tercer argumento `deps` en `showError` — ensucia la firma del ticket. Alternativa: evento global `vx-error` — los ABM tendrían que escuchar; el ticket pide una función.

### 3. Toast siempre + campos por `name` exacto

Tras un `ApiError` (no abort):

1. Toast `variant="error"` y `textContent = messageFor(code)`
2. Por cada `errors[]`, `root.querySelector(`vx-field[name="${CSS.escape(field)}"]`)` → `setAttribute("error", message)` (texto plano). Sin match: no-op.

Así el operador ve el mensaje del `code` aunque ningún campo coincida (p. ej. `attributes[1].value` antes de que el ABM exista). Cuando el formulario use `name="sku"`, el mensaje de campo aparece *además* en el `vx-field`.

Alternativa descartada: toast solo si no hubo match — el ticket dice que `showError` usa `vx-toast`; silenciarlo en validación retrasaría el feedback en pantallas sin campos.

Abort (`err?.name === "AbortError"`): return inmediato.

### 4. Cobertura OpenAPI sin librería YAML

Exportar `errorCodesFromOpenApi(yamlText)` (regex sobre el bloque `ErrorCode:` → `enum:` → líneas `- valor`) y `missingErrorCodes(codes, messages)`. `errors.test.mjs` lee `docs/openapi.yaml` con `fs` y aserta `missingErrorCodes(enum, MESSAGES) === []` más presencia de `network`. El unhappy path llama `missingErrorCodes` con un mapa al que le falta `timeout` (no se muta el módulo).

`check-vectra-web.sh` exige `vectra-web/js/errors.js` y corre `errors.test.mjs` junto al resto.

Alternativa descartada: `js-yaml` / Redocly en Node — viola el No hace de npm. Alternativa: duplicar la lista en el test — se desincroniza del esquema (el Listo cuando pide comparar *ambas* listas).

### 5. Tests con stub mínimo, no Playwright

Cobertura y `messageFor` no necesitan DOM. `showError` usa el mismo `dom-stub.mjs` que componentes *o* fakes duck-typed; no se registra el custom element si el fake basta. Sin `package.json`.

## Risks / Trade-offs

- **[Redacción SPA vs Gateway en `unsupported_media_type`]** → Aceptado: la SPA no debe mostrar `"Error"`. Un ticket futuro de API puede alinear `title_for_code`; este change no lo abre (alcance ~2 h, No hace OpenAPI).
- **[Match exacto de `field`]** → Los ABM (T-02-*) MUST poner `name` igual a la ruta que envía el Gateway. Hasta entonces los mensajes de campo solo tostean el `code`.
- **[Toast único]** → Un segundo `showError` pisa el texto; igual que T-01-04. Cola sería otro ticket.
- **[Regex YAML]** → Si alguien reformatea el bloque `ErrorCode` el chequeo falla en rojo (deseable: el Listo cuando se rompe a propósito).

## Migration Plan

Módulo nuevo. Rollback: borrar `js/errors.js`, revertir el harness y el fixture. Sin datos persistidos ni cambio de contrato HTTP.
