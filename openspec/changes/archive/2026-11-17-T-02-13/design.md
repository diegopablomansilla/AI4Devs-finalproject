# Design

## Context

`#/admin/atributos` pinta el shell de T-01-11 (`data-region="admin-content"` con `<h1>Atributos</h1>`). T-02-07 y T-02-10 ya dejaron el patrón: `startRouter` acepta `request` / `showError` / `mountCategories` / `mountBrands` y monta solo la sección activa. El cliente `api.js` (`request`) y `errors.js` (`showError` / `messageFor`) ya mapean `invalid_input`, `conflict`, `not_found`, `unavailable`, `network` y `attribute_in_use`. T-02-12 expone lista ordenada por `order`, alta (`AttributeDefinitionCreate`), PATCH y DELETE (baja lógica). El harness de `vectra-web` usa `node --test` con stubs de HTML (sin jsdom ni upgrade de custom elements). Ver proposal.md — Why.

## Goals / Non-Goals

**Goals:**

- Módulo de atributos testeable en Node con `request` / `showError` inyectados
- Listado en orden de la API + formulario tipado (opciones solo si `enum`) + alta / edición / baja / reactivación + toasts por `code`
- `startRouter` monta la vista en `atributos`; el resto del admin (salvo `categorias` y `marcas`) sigue sin ABM
- Listo cuando: crear "grado" enum 2/5/8 facetable a mano

**Non-Goals:**

- Extraer un *framework* de vistas admin (copia explícita del patrón de T-02-10)
- jsdom, Playwright, Web Component `vx-attribute-list`
- Vista previa de facetas, occupancy, `attribute_in_use` (T-02-23), query extra

## Decisions

### 1. Módulo `vectra-web/js/admin/attribute-definitions.js`; el router solo monta

Exportar `mountAttributeDefinitions(el, { request, showError, document })` → GET, pintar, bind de acciones.

`startRouter` acepta `mountAttributeDefinitions` opcional. Tras pintar admin, si `section === "atributos"` y `typeof request === "function"`, llama `mountAttributeDefinitions` sobre `[data-region="admin-content"]`. `categorias` y `marcas` no cambian. Sin `request` (tests actuales de `router.test.mjs`) no hay fetch: `router_admin_does_not_call_api` y `router_admin_content_region_has_no_business_ui` siguen verdes en `#/admin/productos` / `#/admin/stock`.

`app.js` pasa `request` de `api.js`, `showError` de `errors.js` y los tres `mount*`. Alternativa descartada: un `mountAdminSection` genérico — T-02-10 lo dejó para este ticket como copia del patrón, no un framework.

### 2. HTML del listado como string; sin reordenar en el cliente

Misma estrategia que T-02-10: el test lee `el.innerHTML` y dispara handlers sin custom elements reales. Recorrer `items` **en el orden recibido** (no `localeCompare` como marcas). Cada fila:

```html
<li data-attribute-id="…" data-active="true|false" data-key="grado">
  <span data-label>Grado</span>
  <span data-key>grado</span>
  <span data-type>enum</span>
  <button type="button" data-action="edit">Editar</button>
  <button type="button" data-action="deactivate">Dar de baja</button>
</li>
```

### 3. Formulario: `select` nativo de tipo; opciones como `vx-field` separado por comas

Estado `activeForm` incluye `type` (default `"text"`). El formulario:

- `vx-field name="key"` y `name="label"`
- `<select name="type">` con valores API `text|number|boolean|enum` y etiquetas en español (Texto, Número, Booleano, Enumerado). Listener `change`: actualiza `activeForm.type` y vuelve a pintar (así aparece/desaparece opciones).
- `vx-field name="unit"` opcional
- `vx-field name="options"` **solo** si `type === "enum"`: el valor es texto separado por comas (`"2, 5, 8"` → `["2","5","8"]` recortando espacios y descartando vacíos)
- `<input type="checkbox" name="facetable">` y `name="searchable"` (nativos; el stub de test lee `checked`)
- `vx-field name="order"` opcional; vacío → se omite en el POST (default servidor `0`)

Alternativa descartada: un input por opción con botones ± — peor de testear en el stub y se pasa de ~2 h. Alternativa descartada: `vx-modal` obligatorio — el formulario inline de marcas ya cubre `showError` con `errors[].field`.

Al `change` de tipo hacia no-enum, las opciones no se envían (`options` omitido o `null`). Hacia `enum`, el campo reaparece vacío (o con las opciones previas si se está editando).

### 4. Cliente HTTP: un `request` por operación; recargar con GET

Tras 2xx, `GET ?includeInactive=true` de nuevo. No *optimistic UI*. `DELETE` → 204 (`request` → `null`). Errores: `catch` → `showError(err)` y se conserva el listado previo.

`POST` envía `{ key, label, type, facetable, searchable }` más `unit` / `options` / `order` solo si aplican. `PATCH` de edición envía los campos del formulario (no `active`; la reactivación es acción aparte). `key`/`label` vacíos no disparan `POST`/`PATCH`. Enum con opciones vacías **sí** envía `POST` (para que el servidor responda `invalid_input`, escenario HU-02-03).

### 5. CSS `.vx-attr-*` con tokens existentes

Lista (`ul`/`li`), inactiva con `--vx-color-text-muted`. Botones ya tienen `--vx-touch-min`. Sin *tokens* nuevos. Sin estilos de “preview” de facetas.

### 6. Tests en `attribute-definitions.test.mjs`; harness del archivo

Fake `request(method, path, opts)` que graba llamadas y resuelve `{ items }` o lanza `{ code, name: "ApiError" }`. Fake `showError` que acumula `code`. Stub `{ innerHTML, querySelector, querySelectorAll, addEventListener }` al estilo de `brands.test.mjs`, extendido para `change` del `select` y checkboxes.

`check-vectra-web.sh`: exigir `vectra-web/js/admin/attribute-definitions.js` y añadir el archivo a `node --test`.

`router_admin_content_region_has_no_business_ui` MUST seguir usando `#/admin/productos` (no `atributos` / `marcas` / `categorias`).

### 7. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-13` y rename a `T-02-13` (igual que T-02-10 / T-02-12).

## Risks / Trade-offs

- [Stub sin shadow DOM] → los tests no cubren el *upgrade* de `vx-field`. Mitigación: igual que T-02-10; DoD de teclado/contraste se valida a mano en el Listo cuando.
- [Opciones por comas] → un valor que contenga coma no se puede expresar. Mitigación: las opciones del rubro (2/5/8, M8, UNC) no usan coma; lista dinámica queda para un ticket futuro si hace falta.
- [GET tras cada mutación] → un round-trip extra. Mitigación: el maestro de atributos es pequeño; evita estado divergente.
- [`attribute_in_use` aún no lo emite la API] → un PATCH de `key`/`type` no falla por occupancy. Mitigación: T-02-23; `messageFor` ya tiene el `code` para cuando exista.
- [HU-02-03 «aparece como filtro»] → fuera de MVP de facetas. Mitigación: T-03-18 / T-03-19.

## Migration Plan

Sin migración Surreal ni cambio de OpenAPI. Archivos nuevos bajo `vectra-web/js/admin/` y un test Node. Rollback: revertir el change; el shell admin vuelve al placeholder de atributos.

## Open Questions

Ninguna que altere specs o tareas.
