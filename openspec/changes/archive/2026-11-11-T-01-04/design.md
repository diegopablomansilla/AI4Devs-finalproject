# Design

## Context

`develop` ya tiene el kit T-01-03 (`shared.js`, Shadow DOM, `adoptedStyleSheets`, `dev/components.html`, harness `check-vectra-web.sh` + `dom-stub.mjs`). ADR-06: ES Modules, sin *build*. Ver proposal.md para el porqué; la spec para el contrato de `vx-closed`, trampa de foco y `aria-live`.

El stub actual no modela `document.activeElement`, `focus()` ni `keydown`. Hay que extenderlo: los tests de overlay no pueden depender de jsdom ni del navegador.

## Goals / Non-Goals

**Goals:**

- Un helper compartido de trampa de foco + restore para modal y panel, testeable en Node
- Toast como live region, sin robar el contrato `vx-closed` que T-01-08 usará para el Omnibox
- Catálogo de muestra que cumple el Listo cuando (cerrar panel → foco al abridor)

**Non-Goals:**

- `inert` nativo en el resto del documento (el trap cubre el teclado; el stub no simula `inert`)
- Auto-dismiss, cola o apilado de toasts (T-01-07 decide *cuándo* mostrar)
- Click en backdrop como requisito (el ticket solo exige Escape + restore; el control de cierre basta)

## Decisions

### 1. overlay.js para trap y restore

`vectra-web/js/components/overlay.js` exporta abrir/cerrar: recuerda el abridor (`document.activeElement` o el nodo pasado), mueve el foco al primer control del shadow, escucha `keydown` (Tab cicla, Escape cierra) y al cerrar restaura. Modal y panel lo usan; toast no.

Alternativa descartada: duplicar el trap en cada custom element — el ticket trata modal y panel como el mismo contrato.

### 2. API `open` + `show(opener)` / `hide()`

Atributo `open` para estado observable. `show(opener)` / `hide()` para el catálogo y para T-01-07/T-01-08. Si `show()` no recibe abridor, se usa `document.activeElement`. Evento `vx-closed` solo desde `hide()` tras haber estado abierto (`CustomEvent`, `bubbles: true`, `composed: true`).

### 3. Toast no emite `vx-closed`

T-01-08 devolverá el foco al Omnibox al oír `vx-closed`. Un toast de error no debe disparar eso. Variante inválida → `info`, sin throw.

### 4. Stub: foco y teclado mínimos

Extender `dom-stub.mjs` con `focus()`, `document.activeElement`, `KeyboardEvent` (`key`, `shiftKey`) y `querySelector` para el botón de cierre y un par de controles. Los tests despachan `keydown` sobre el host. Sin jsdom.

### 5. Estilos locales, sin tokens nuevos

z-index y layout (modal centrado, panel al borde) en la hoja local del componente con `var(--vx-*)`. No se toca `tokens.css`.

## Risks / Trade-offs

- **[Trap incompleto en el stub vs navegador]** → Tests cubren Tab/Escape/restore; la muestra `dev/components.html` es el oráculo manual del Listo cuando.
- **[Foco inicial en el shadow]** → Primer foco al control de cierre si no hay otro tabbable; evita foco perdido.
- **[Un toast a la vez]** → Suficiente para T-01-07; apilar sería otro ticket.
