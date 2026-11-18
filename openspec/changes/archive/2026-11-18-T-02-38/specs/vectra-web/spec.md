# Spec Delta

Ticket padre: T-02-38. Change: T-02-38. HU: HU-02-07.

## ADDED Requirements

### Requirement: UI pestaña Stock (ingresos y ajustes)

Cuando la **ficha de edición** está abierta (requirement «UI edición de producto (ficha Datos)»), la pestaña **Stock** MUST ser operable: MUST NOT tener `aria-disabled`. Activarla MUST seleccionarla (`aria-selected="true"` en Stock, `"false"` en Datos) y MUST mostrar el panel de stock **sin** cambiar el hash (sigue `#/admin/productos`). MUST NOT enviar HTTP solo por cambiar de pestaña. La pestaña **Relaciones** MUST seguir no operable. MUST NOT pintar pestaña Imágenes (v2, T-02-39).

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
- **AND** Relaciones sigue no operable
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

### Requirement: Tests de pestaña Stock

Antes de archive, la suite MUST incluir:

| Test | Ubicación | Tipo |
|---|---|---|
| `sheet_edit_stock_tab_is_operable` | `scripts/fixtures/vectra-web/product-sheet.test.mjs` | happy |
| `sheet_stock_shows_form_without_http` | `product-sheet.test.mjs` | happy |
| `sheet_stock_inbound_posts_and_shows_resulting_stock` | `product-sheet.test.mjs` | happy (HU-02-07, Listo cuando) |
| `sheet_stock_adjustment_to_zero_with_reason` | `product-sheet.test.mjs` | happy (HU-02-07) |
| `sheet_stock_adjustment_without_reason_shows_invalid_input` | `product-sheet.test.mjs` | unhappy (HU-02-07, Listo cuando) |
| `sheet_stock_negative_keeps_stock_and_shows_negative_stock` | `product-sheet.test.mjs` | unhappy (HU-02-07, Listo cuando) |
| `sheet_stock_invalid_input_sets_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_not_found_shows_not_found` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_network_shows_network` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_empty_quantity_does_not_post` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_has_no_edit_or_delete_movement` | `product-sheet.test.mjs` | unhappy (Inmutabilidad) |
| `sheet_stock_does_not_get_or_export_history` | `product-sheet.test.mjs` | unhappy (No hace) |
| `sheet_create_stock_tab_stays_disabled` | `product-sheet.test.mjs` | happy / frontera |

`./scripts/check-vectra-web.sh` MUST ejecutarlos. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5). El Listo cuando del ticket (escenarios de HU-02-07 a mano) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable (la cabecera `X-Vectra-Terminal` queda cubierta por `api.js` / T-01-06).

#### Scenario: la suite de Stock corre en check (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** incluye `product-sheet.test.mjs` con los tests de la tabla de este requirement

#### Scenario: el Listo cuando está cubierto por tests nombrados (unhappy)

- **WHEN** se inspecciona `product-sheet.test.mjs`
- **THEN** existen `sheet_stock_inbound_posts_and_shows_resulting_stock`, `sheet_stock_adjustment_without_reason_shows_invalid_input` y `sheet_stock_negative_keeps_stock_and_shows_negative_stock`
