# Spec Delta

Ticket padre: T-04-09. Change: `T-04-09`. HU: HU-04-03 (Listado; Edición y eliminación). Endpoints `GET /api/v1/products/{productId}/relations`, `PATCH`/`DELETE /api/v1/relations/{relationType}/{relationId}` = T-04-05 (fuera de este change). Alta de relaciones = T-04-10 (fuera de este change).

## ADDED Requirements

### Requirement: UI pestaña Relaciones (listado, edición y eliminación)

Cuando la **ficha de edición** está abierta (requirement «UI edición de producto (ficha Datos)»), la pestaña **Relaciones** MUST ser operable: MUST NOT tener `aria-disabled`. Activarla MUST seleccionarla (`aria-selected="true"` en Relaciones, `"false"` en Datos y Stock) y MUST mostrar el panel de relaciones **sin** cambiar el hash (sigue `#/admin/productos`). MUST NOT pintar pestaña Imágenes (v2, T-02-39).

En la ficha de **alta**, Relaciones MUST permanecer no operable (requirement «UI alta de producto (ficha Datos)»); MUST NOT habilitarse ni enviar `GET`/`PATCH`/`DELETE` de relaciones desde el alta.

Al activar la pestaña en edición, MUST:

1. Cancelar un `GET` de relaciones anterior con `AbortController` (`signal` reenviado al cliente HTTP, T-01-06).
2. Invocar `GET /api/v1/products/{productId}/relations` en el mismo origen (`encodeURIComponent` en el segmento). MUST NOT enviar `type`. MUST NOT enviar `direction` (el contrato default es `outgoing`) o MUST enviar `direction=outgoing`. MUST NOT llamar `GET .../recommendations` ni `POST /api/v1/relations`.
3. Si la ronda no fue abortada y la respuesta es 200 `{ items }` (`Relation[]` de [docs/openapi.yaml](../../../../../docs/openapi.yaml)): pintar el panel agrupado por `type`.

El panel MUST estar en español y operable por teclado (controles con tamaño táctil ≥ `--vx-touch-min`). MUST mostrar **tres grupos** en este orden, con títulos fijos aunque un grupo esté vacío:

| Título visible | `Relation.type` |
|---|---|
| Similares | `similar` |
| Sustitutos | `substitute` |
| Se usa junto con | `complement` |

Dentro de cada grupo, las filas MUST ordenarse por `priority` ascendente (empate: orden de `items` en la respuesta). Cada fila MUST mostrar:

- Destino: `target.sku` y `target.name` (si `target` falta, MUST mostrar `targetProductId`). Destinos **inactivos** MUST seguir visibles (el GET de administración no los filtra).
- Prioridad (`Relation.priority`, entero ≥ 1) en un control editable (`vx-field` `name="priority"` o equivalente).
- Nota (`Relation.note`; vacío si es `null`) en un control editable (`vx-field` `name="note"` o equivalente).
- Indicador bidireccional visible en español (p. ej. «Bidireccional») si y solo si `bidirectionalGroup` es un ULID no nulo. Si es `null` o falta, MUST NOT afirmar que es bidireccional.

Cada fila MUST tener `data-relation-id` (ULID) y `data-relation-type` (`similar` \| `substitute` \| `complement`). MUST NOT asignar JSON crudo a `innerHTML`.

**Edición en línea:** una acción de guardar por fila (p. ej. `data-action="save-relation"`) MUST enviar `PATCH /api/v1/relations/{relationType}/{relationId}` con cuerpo `RelationUpdate`: `priority` como **entero JSON** (MUST NOT string) y/o `note` (string o `null`). MUST incluir al menos un campo. Tras **200** `{ items }`, el panel MUST reflejar de inmediato la prioridad y nota nuevas (re-GET o pintar con el 200). MUST permanecer en la pestaña Relaciones. MUST NOT volver al listado de productos. MUST NOT cambiar la `version` de la ficha.

Si prioridad está vacía, no es un entero ≥ 1, o es menor que 1, MUST NOT enviar `PATCH` y MUST tratarlo como `invalid_input` sobre `priority`. Si no cambió ni prioridad ni nota respecto de lo pintado, MUST NOT enviar `PATCH`.

**Eliminar:** una acción por fila (p. ej. `data-action="delete-relation"`) MUST enviar `DELETE /api/v1/relations/{relationType}/{relationId}` sin cuerpo. MUST NOT pedir confirmación modal. Tras **204**, esa fila MUST desaparecer de inmediato (re-GET o quitarla del DOM). El backend elimina el grupo bidireccional; la UI MUST NOT emitir un segundo DELETE por la arista inversa.

Errores MUST usar `showError` / `messageFor(code)`: el toast usa el `code` (`invalid_input`, `not_found`, `unavailable`, `network`, …), no el `title` crudo. `invalid_input` MUST pintar `errors[].field` en el `vx-field` de esa fila cuyo `name` es `priority` o `note`. `AbortError` / ronda abortada del GET: MUST NOT pintar esa respuesta y MUST NOT invocar `showError`. 404/400/503/`network` que no sean abort: MUST invocar `showError` y MUST NOT reemplazar el panel con esa respuesta fallida. La ficha MUST permanecer abierta.

MUST NOT ofrecer alta (buscador, selector de tipo, casilla bidireccional, `POST /api/v1/relations`). MUST NOT editar `type`, `sourceProductId` ni `targetProductId`. MUST NOT listar aristas `direction=incoming`. MUST NOT abrir el `vx-side-panel` de recomendaciones desde esta pestaña. `base.css` MAY añadir clases de pestaña/panel Relaciones solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Textos visibles MUST estar en español.

#### Scenario: pestaña Relaciones operable en edición (happy)

- **WHEN** la ficha de edición está abierta
- **THEN** la pestaña Relaciones no tiene `aria-disabled`
- **AND** Datos está seleccionada
- **AND** no hay pestaña Imágenes

#### Scenario: activar Relaciones lista agrupada por tipo (happy, HU-04-03 Listado, Listo cuando)

- **WHEN** la ficha de edición está abierta para un producto con relaciones de los tres tipos
- **AND** el administrador activa la pestaña Relaciones
- **THEN** se envía `GET /api/v1/products/{productId}/relations` (sin `type`; sin `direction` o con `outgoing`)
- **AND** Relaciones queda `aria-selected="true"`
- **AND** se ven los títulos «Similares», «Sustitutos» y «Se usa junto con»
- **AND** cada grupo muestra SKU/nombre del destino, prioridad, nota y, si `bidirectionalGroup` no es nulo, el indicador bidireccional
- **AND** el hash sigue `#/admin/productos`
- **AND** no se envía `POST /api/v1/relations` ni `GET .../recommendations`

#### Scenario: grupos vacíos igual muestran los tres títulos (happy)

- **WHEN** el GET responde 200 con `items: []`
- **THEN** se ven los tres títulos
- **AND** no hay filas de relación

#### Scenario: destino inactivo sigue visible (happy, admin)

- **WHEN** un ítem trae `target.active === false`
- **THEN** esa fila se pinta (SKU/nombre visibles)
- **AND** no se oculta por inactivo

#### Scenario: cambiar prioridad actualiza de inmediato (happy, HU-04-03 Edición, Listo cuando)

- **WHEN** hay un sustituto pintado con `priority` 2
- **AND** el administrador guarda prioridad `1` en esa fila
- **THEN** se envía `PATCH /api/v1/relations/substitute/{relationId}` con `{ "priority": 1 }` (entero JSON)
- **AND** tras 200 la fila muestra prioridad 1
- **AND** la pestaña Relaciones permanece abierta

#### Scenario: cambiar nota actualiza de inmediato (happy)

- **WHEN** hay una relación con `note` `"vieja"`
- **AND** el administrador guarda nota `"Arandela plana M8"`
- **THEN** se envía `PATCH` con `{ "note": "Arandela plana M8" }`
- **AND** tras 200 la fila muestra esa nota

#### Scenario: eliminar quita la fila de inmediato (happy, HU-04-03 Eliminación, Listo cuando)

- **WHEN** hay una relación pintada
- **AND** el administrador activa eliminar en esa fila
- **THEN** se envía `DELETE /api/v1/relations/{relationType}/{relationId}`
- **AND** tras 204 esa fila ya no está
- **AND** no hay un segundo DELETE por una arista inversa
- **AND** no se abre un diálogo de confirmación

#### Scenario: prioridad menor que 1 no envía PATCH (unhappy)

- **WHEN** el administrador guarda prioridad `0` o vacía
- **THEN** no se envía `PATCH`
- **AND** el control `priority` de esa fila recibe un error de campo
- **AND** se muestra el mensaje en español de `invalid_input`

#### Scenario: invalid_input del PATCH pinta el campo (unhappy)

- **WHEN** el PATCH responde `400` con `code` `invalid_input` y `errors[].field` `note`
- **THEN** el `vx-field` de nota de esa fila recibe el mensaje
- **AND** el toast muestra el mensaje en español de `invalid_input`
- **AND** la ficha permanece abierta en Relaciones

#### Scenario: not_found en GET o mutación (unhappy)

- **WHEN** el GET de relaciones o un PATCH/DELETE responde `404` con `code` `not_found`
- **THEN** se muestra el mensaje en español de `not_found`
- **AND** la ficha permanece abierta

#### Scenario: 503 o network (unhappy)

- **WHEN** el GET, PATCH o DELETE falla con `unavailable` o `network`
- **THEN** se muestra el mensaje en español de ese `code`
- **AND** la ficha permanece abierta

#### Scenario: AbortError del GET no llama a showError ni pinta (unhappy)

- **WHEN** el administrador activa Relaciones y luego Datos (o Relaciones de nuevo) antes de que responda el GET
- **THEN** la petición anterior queda abortada
- **AND** no se invoca `showError` por el abort
- **AND** no se pinta la respuesta abortada

#### Scenario: alta sigue con Relaciones no operable (happy / frontera, No hace T-04-10)

- **WHEN** la ficha de alta está abierta
- **THEN** la pestaña Relaciones tiene `aria-disabled="true"` o equivalente
- **AND** activarla no envía `GET`/`PATCH`/`DELETE`/`POST` de relaciones

#### Scenario: no hay alta de relaciones (unhappy, No hace T-04-10)

- **WHEN** se inspecciona el panel Relaciones en edición
- **THEN** no hay buscador de productos ni selector de tipo para crear
- **AND** no se envía `POST /api/v1/relations`

#### Scenario: no se listan aristas entrantes (unhappy)

- **WHEN** se activa la pestaña Relaciones
- **THEN** el GET no incluye `direction=incoming`

### Requirement: Tests de pestaña Relaciones

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm).

| Test | Ubicación | Tipo |
|---|---|---|
| `sheet_edit_relations_tab_is_operable` | `scripts/fixtures/vectra-web/product-sheet.test.mjs` | happy |
| `sheet_relations_lists_grouped_by_type` | `product-sheet.test.mjs` | happy (HU-04-03 Listado, Listo cuando) |
| `sheet_relations_empty_items_still_show_headings` | `product-sheet.test.mjs` | happy |
| `sheet_relations_shows_inactive_target` | `product-sheet.test.mjs` | happy |
| `sheet_relations_shows_bidirectional_indicator` | `product-sheet.test.mjs` | happy |
| `sheet_relations_patch_priority_updates_row` | `product-sheet.test.mjs` | happy (HU-04-03 Edición, Listo cuando) |
| `sheet_relations_patch_note_updates_row` | `product-sheet.test.mjs` | happy |
| `sheet_relations_delete_removes_row` | `product-sheet.test.mjs` | happy (HU-04-03 Eliminación, Listo cuando) |
| `sheet_relations_priority_below_one_does_not_patch` | `product-sheet.test.mjs` | unhappy |
| `sheet_relations_invalid_input_sets_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_relations_not_found_shows_not_found` | `product-sheet.test.mjs` | unhappy |
| `sheet_relations_unavailable_shows_unavailable` | `product-sheet.test.mjs` | unhappy |
| `sheet_relations_network_shows_network` | `product-sheet.test.mjs` | unhappy |
| `sheet_relations_abort_does_not_paint_stale` | `product-sheet.test.mjs` | unhappy |
| `sheet_create_relations_tab_stays_disabled` | `product-sheet.test.mjs` | happy / frontera |
| `sheet_relations_has_no_create_controls` | `product-sheet.test.mjs` | unhappy (No hace T-04-10) |
| `sheet_relations_does_not_get_incoming` | `product-sheet.test.mjs` | unhappy |

`./scripts/check-vectra-web.sh` MUST ejecutarlos. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5). El Listo cuando del ticket (escenarios de HU-04-03 a mano) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el listado/PATCH/DELETE observables. Que el panel operador de T-04-08 muestre el grafo actualizado al reabrirlo es verificación **a mano** (no hay caché de recomendaciones, T-04-07).

El test existente `sheet_edit_stock_tab_is_operable` MUST dejar de exigir `aria-disabled` en Relaciones (requirement «UI pestaña Stock» modificado).

#### Scenario: la suite de Relaciones corre en check (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** incluye `product-sheet.test.mjs` con los tests de la tabla de este requirement

#### Scenario: el Listo cuando está cubierto por tests nombrados (unhappy)

- **WHEN** se inspecciona `product-sheet.test.mjs`
- **THEN** existen `sheet_relations_lists_grouped_by_type`, `sheet_relations_patch_priority_updates_row` y `sheet_relations_delete_removes_row`

## MODIFIED Requirements

### Requirement: UI pestaña Stock (ingresos y ajustes)

Cuando la **ficha de edición** está abierta (requirement «UI edición de producto (ficha Datos)»), la pestaña **Stock** MUST ser operable: MUST NOT tener `aria-disabled`. Activarla MUST seleccionarla (`aria-selected="true"` en Stock, `"false"` en Datos) y MUST mostrar el panel de stock **sin** cambiar el hash (sigue `#/admin/productos`). MUST NOT enviar HTTP solo por cambiar de pestaña. MUST NOT pintar pestaña Imágenes (v2, T-02-39). La pestaña **Relaciones** en edición se rige por el requirement «UI pestaña Relaciones (listado, edición y eliminación)».

En la ficha de **alta**, Stock MUST permanecer no operable (requirement «UI alta de producto (ficha Datos)»); MUST NOT habilitarse ni enviar `POST .../stock-movements` desde el alta.

El panel Stock MUST estar en español y operable por teclado (controles con tamaño táctil ≥ `--vx-touch-min`). MUST incluir:

| Campo visible | Control | `name` (JSON / `vx-field`) | Obligatorio |
|---|---|---|---|
| Tipo | `<select>` | `type` | sí (`inbound` o `adjustment`; etiquetas Ingreso / Ajuste) |
| Cantidad | `vx-field` | `quantity` | sí (string decimal con signo, ≠ 0; `inbound` > 0) |
| Motivo | `vx-field` | `reason` | sí si `type` es `adjustment`; no si `inbound` |
| Stock actual | solo lectura | — | — (valor de la ficha; MUST NOT ser `<input>` editable) |

MUST haber una acción de registrar (p. ej. «Registrar movimiento») en un `<form data-form="stock-movement">`. MUST NOT ofrecer editar ni borrar un movimiento (ni `data-action` de update/delete sobre un id de movimiento). MUST NOT pintar un listado paginado de historial ni una acción de exportar. MUST NOT llamar `GET /api/v1/products/{productId}/stock-movements`.

Al registrar, si faltan tipo o cantidad (vacíos / solo espacios), o si el tipo es `adjustment` y el motivo está vacío, MUST NOT enviar `POST`. Si el tipo es `adjustment` y el motivo falta, MUST tratarlo como `invalid_input` sobre `reason` (HU-02-07 «Ajuste sin motivo»).

Si el formulario es válido, MUST enviar `POST /api/v1/products/{productId}/stock-movements` con cuerpo `StockMovementCreate` alineado con [docs/openapi.yaml](../../../../../docs/openapi.yaml): `type` (`inbound` \| `adjustment`), `quantity` como **string** decimal, `reason` omitido o `null` en ingreso vacío, `reason` string en ajuste. MUST NOT enviar `initial` ni `sale`. MUST NOT enviar `version` ni `stock` ni PATCH de ficha. `api.request` ya adjunta `X-Vectra-Terminal` si hay nombre de terminal (T-01-06); esta vista MUST NOT duplicar esa cabecera.

Tras **201** `StockMovement`, MUST permanecer en la pestaña Stock y MUST mostrar el `resultingStock` (p. ej. `role="status"`). MUST actualizar el stock de solo lectura de la ficha a ese valor. MUST NOT volver al listado. MUST NOT cambiar la `version` visible de la ficha.

Errores MUST usar `showError` / `messageFor(code)`: el toast usa el `code` (`invalid_input`, `negative_stock`, `not_found`, `unavailable`, `network`, …), no el `title` crudo. `invalid_input` MUST pintar `errors[].field` en el `vx-field` cuyo `name` es `quantity` o `reason`. `negative_stock` MUST mostrar el mensaje en español de `errors.js` y MUST dejar el stock de solo lectura **igual que antes del POST**. La ficha MUST permanecer abierta si el POST falla. `base.css` MAY añadir clases de pestaña/panel Stock solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Textos visibles MUST estar en español.

#### Scenario: pestaña Stock operable en edición (happy)

- **WHEN** la ficha de edición está abierta
- **THEN** la pestaña Stock no tiene `aria-disabled`
- **AND** Datos está seleccionada
- **AND** no hay pestaña Imágenes

#### Scenario: activar Stock muestra el formulario (happy)

- **WHEN** la ficha de edición está abierta
- **AND** el administrador activa la pestaña Stock
- **THEN** Stock queda `aria-selected="true"`
- **AND** hay `data-form="stock-movement"` con controles `type`, `quantity` y `reason`
- **AND** el stock actual de la ficha es visible en solo lectura
- **AND** no se envía HTTP por el cambio de pestaña
- **AND** el hash sigue `#/admin/productos`

#### Scenario: Ingreso actualiza stock resultante (happy, HU-02-07, Listo cuando)

- **WHEN** el producto tiene stock `"10"`
- **AND** el administrador registra un ingreso de cantidad `"20"`
- **THEN** se envía `POST /api/v1/products/{id}/stock-movements` con `{ "type": "inbound", "quantity": "20" }` y sin `reason` vacío como obligatorio
- **AND** `quantity` viaja como string (MUST NOT número JSON)
- **AND** el cuerpo no incluye `version` ni `stock`
- **AND** tras 201 con `resultingStock` `"30"` se muestra ese valor
- **AND** el stock de solo lectura pasa a `"30"`
- **AND** la pestaña Stock permanece abierta

#### Scenario: Ajuste a cero con motivo (happy, HU-02-07)

- **WHEN** el producto tiene stock `"5"`
- **AND** el administrador registra un ajuste de `"-5"` con motivo `"Conteo físico"`
- **THEN** se envía `POST` con `{ "type": "adjustment", "quantity": "-5", "reason": "Conteo físico" }`
- **AND** tras 201 con `resultingStock` `"0"` se muestra ese valor

#### Scenario: Ajuste sin motivo no envía POST (unhappy, HU-02-07, Listo cuando)

- **WHEN** el administrador elige tipo Ajuste y cantidad `"-1"` sin motivo
- **THEN** no se envía `POST /api/v1/products/{id}/stock-movements`
- **AND** el control `reason` recibe un error de campo
- **AND** se muestra el mensaje en español de `invalid_input`

#### Scenario: Stock negativo no cambia el stock (unhappy, HU-02-07, Listo cuando)

- **WHEN** el producto tiene stock `"5"`
- **AND** el `POST` de ajuste `"-8"` con motivo responde `400` con `code` `negative_stock`
- **THEN** se muestra el mensaje en español de `negative_stock`
- **AND** el stock de solo lectura sigue `"5"`
- **AND** la ficha permanece abierta en Stock

#### Scenario: invalid_input pinta quantity (unhappy)

- **WHEN** el POST responde `400` con `code` `invalid_input` y `errors[].field` `quantity`
- **THEN** el `vx-field` de cantidad recibe el mensaje
- **AND** el toast muestra el mensaje en español de `invalid_input`
- **AND** la ficha permanece abierta

#### Scenario: not_found en POST de movimiento (unhappy)

- **WHEN** el POST responde `404` con `code` `not_found`
- **THEN** se muestra el mensaje en español de `not_found`
- **AND** la ficha permanece abierta

#### Scenario: red en POST de movimiento (unhappy)

- **WHEN** el POST falla con error `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** la ficha permanece abierta

#### Scenario: no hay edición ni borrado de movimientos (unhappy, HU-02-07 Inmutabilidad)

- **WHEN** se inspecciona el panel Stock
- **THEN** no hay acción de editar ni borrar un movimiento
- **AND** no hay `PATCH` ni `DELETE` hacia `stock-movements`

#### Scenario: no hay historial ni exportar (unhappy, No hace)

- **WHEN** se inspecciona el panel Stock o se registra un movimiento
- **THEN** no se envía `GET /api/v1/products/{id}/stock-movements`
- **AND** no hay acción de exportar el historial
- **AND** no hay tabla paginada de movimientos

#### Scenario: cantidad vacía no envía POST (unhappy)

- **WHEN** el tipo es Ingreso y la cantidad está vacía
- **AND** el administrador intenta registrar
- **THEN** no se envía `POST`

#### Scenario: alta sigue con Stock no operable (happy / frontera)

- **WHEN** la ficha de alta está abierta
- **THEN** la pestaña Stock tiene `aria-disabled="true"` o equivalente
- **AND** activarla no envía `POST .../stock-movements`
