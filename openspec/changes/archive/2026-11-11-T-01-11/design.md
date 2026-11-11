# Design

## Context

`develop` pinta administración como `<h1>Administración: ${section}</h1>` + enlace `#/` (T-01-01). El operador ya tiene layout (T-01-08) y enlace a `#/admin/productos`. `vx-field` existe (T-01-03); `api.js` lee `localStorage["vectra.terminal"]` (T-01-06). El stub del *router* inspecciona `innerHTML` (no hay nodos reales ni upgrade de custom elements). Ver proposal.md para el porqué; spec para menú, `aria-current`, región vacía y persistencia ≤ 64.

## Goals / Non-Goals

**Goals:**

- Shell admin testeable en Node sobre el HTML string + funciones puras de terminal
- Listo cuando automatizado: enlace `#/` y cambio de *hash* sin recarga
- Misma clave `vectra.terminal` que T-01-06, validada al guardar

**Non-Goals:**

- Extraer un *router* de vistas por archivo (`views/admin-productos.js`) — T-02-07 en adelante pinta `admin-content`
- Cambiar `parseHash`: cualquier slug `[a-z0-9-]+` sigue siendo administración
- Extender el stub del *router* a jsdom / custom elements

## Decisions

### 1. HTML del shell en `viewHtml`; constante `ADMIN_SECTIONS` en `router.js`

`viewHtml({ view: "admin", section })` deja de ser un heading suelto. Estructura:

```html
<main class="vx-admin" data-view="admin" data-section="…">
  <header class="vx-admin-header">
    <p><a href="#/">Volver al operador</a></p>
    <vx-field name="terminal" label="Nombre de esta terminal" value="…"></vx-field>
  </header>
  <nav class="vx-admin-nav" aria-label="Administración">…</nav>
  <div data-region="admin-content" class="vx-admin-content vx-surface">
    <h1>…</h1>
  </div>
</main>
```

`ADMIN_SECTIONS` es un array `{ slug, label }[]` en el orden de la spec. El `<nav>` se genera de esa lista. `aria-current="page"` solo si `section === slug`. Título de `admin-content`: `label` si el slug está en la lista; si no, el slug crudo (texto, no HTML). Sin `<form>`, sin `vx-product-card`.

No se crea `js/admin.js`: T-02-07 puede extraerlo cuando pinte contenido. Alternativa descartada: un custom element `vx-admin-shell` — obliga a stub de upgrade y no está en el ticket.

`parseHash` no cambia. `#/admin/foo` sigue siendo `data-view="admin"`; el menú se pinta igual y ningún ítem lleva `aria-current`.

### 2. `startRouter` acepta `storage` opcional

```js
startRouter({ window, document, outlet, storage })
```

`storage` default: `window.localStorage` si existe. Si falta o `getItem`/`setItem`/`removeItem` lanzan, se trata como vacío (spec unhappy). Tras cada `paint`, si la ruta es admin, se llama `bindTerminalName` sobre el `vx-field[name="terminal"]`.

`app.js` no cambia de contrato (sigue pasando `{ window, document, outlet }`).

### 3. Terminal: funciones puras + `change` en el host

Exportar desde `router.js` (o un bloque interno testeable):

- `readTerminalName(storage)` → string recortado o `""`; catch → `""`
- `writeTerminalName(storage, raw)` → `{ ok: true, value }` o `{ ok: false, error }` (español, p. ej. "Usá como máximo 64 caracteres"). `ok: true` y `value === ""` hace `removeItem`. `ok: false` no toca storage.

Al pintar, el atributo `value` del `vx-field` se rellena con `readTerminalName` (escape de `"`, `&`, `<` en el HTML). Tests de carga asertan el string en `innerHTML`.

`vx-field` hoy no reexpone `change` fuera del shadow. Este ticket MUST agregar en `vx-field` un listener del `<input>` que:

1. copia `input.value` al atributo `value` del host;
2. despacha `change` en el host (`bubbles: true`; `composed` irrelevante si el listener está en el host).

El *router* escucha `change` en el host (no entra al shadow). Alternativa descartada: `field.shadowRoot.querySelector("input")` desde el *router* — rompe el encapsulamiento y no se puede testear con el stub de HTML.

Los tests de terminal (`admin.test.mjs` o `router.test.mjs`) llaman `writeTerminalName` / `readTerminalName` con un fake `{ getItem, setItem, removeItem }` y, para el binder, un `{ addEventListener, getAttribute, setAttribute }` mínimo. Spy de `globalThis.fetch` en `router_admin_does_not_call_api`.

`maxlength="64"` en el input MAY usarse como ayuda de UX; la fuente de verdad es `writeTerminalName` (el atributo HTML no cubre pegar texto largo en todos los stubs).

### 4. CSS `.vx-admin*` con tokens existentes

`base.css` añade `.vx-admin` (flex column, `min-height: 100vh`, gap/padding `--vx-space-*`), `.vx-admin-nav` (flex wrap, gap), `.vx-admin-content` (flex 1). Los `<a>` ya tienen `--vx-touch-min`. Sin *tokens* nuevos. El ítem actual MAY usar color de acento vía `[aria-current="page"]`.

### 5. Tests: layout en `router.test.mjs`; terminal en el mismo archivo o `admin.test.mjs`

Layout: ampliar el stub actual (`innerHTML` + `querySelector` de `data-view` / `data-section`). Nuevos asserts por regex sobre el HTML (ocho `href`, `aria-current`, `data-region="admin-content"`, `<vx-field` `name="terminal"`, ausencia de `<form>` / `vx-product-card` / `vx-omnibox` / password).

Si se crea `admin.test.mjs`, `check-vectra-web.sh` MUST incluirlo en `node --test` (spec). Si los tests viven en `router.test.mjs`, no hace falta archivo nuevo.

Extender `querySelector` del stub **solo** si un assert no se puede expresar con `innerHTML` (p. ej. `aria-current` en el enlace activo). No jsdom.

## Risks / Trade-offs

- **[HTML injection en `value` del vx-field]** → escape al interpolar; el slug del *hash* ya está acotado a `[a-z0-9-]+`.
- **[vx-field `change` vs tests existentes]** → el retarget es aditivo; `vx_field_error_without_aria_invalid_fails` no dispara `change`. Si algún test escribe el atributo `value` a cada tecla, `_sync` ya pisa el input: el listener copia input → atributo, no al revés en cada `input`.
- **[repintar admin al cambiar el hash pierde el foco del campo]** → aceptable en este ticket (navegación por *hash* ya reemplaza el *outlet*); T-02-07 puede preservar estado de sección.
- **[macOS: `openspec new change T-01-11`]** → el CLI exige kebab-case minúsculas; scaffold `t-01-11` y rename a `T-01-11` (igual que T-01-10).

## Migration Plan

N/A (SPA local). Rollback = revertir el change. `vectra.terminal` ya era la clave de T-01-06; no hay migración de datos.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
