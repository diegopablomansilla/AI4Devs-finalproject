# Proposal

## Ticket padre

- **Ticket:** [T-02-38](../../../docs/7-tickets.md) — UI stock: movimientos
- **Change:** `T-02-38`
- **HU:** [HU-02-07](../../../docs/6-historias.md#hu-02-07--ingresos-y-ajustes-de-stock)
- **Depende de:** T-02-25 (`POST .../stock-movements` en `develop`), T-02-35 (ficha con pestaña Stock visible y no operable). La pestaña se habilita en la **ficha de edición** de T-02-37 (PR #149); esta rama sale de `T-02-37` para no reimplementar Editar.
- **Hace:** pestaña "Stock" con formulario de ingreso y ajuste (motivo obligatorio en ajuste) y el stock resultante; no hay opción de editar ni borrar movimientos.
- **No hace:** historial paginado (v2, con T-02-26 y HU-02-08) ni exportar el historial.
- **Listo cuando:** escenarios de HU-02-07 ejecutados a mano.

## Why

HU-02-07 exige registrar un ingreso (stock 10 + 20 → 30, terminal `caja-1`), rechazar un ajuste sin motivo (`invalid_input`), rechazar un ajuste que deje stock negativo (`negative_stock`, stock intacto) y no ofrecer edición ni borrado de un movimiento. T-02-25 ya expone `POST /api/v1/products/{productId}/stock-movements`; la SPA muestra la pestaña Stock **deshabilitada**. Sin este ticket el administrador no puede ajustar el ledger desde la GUI (criterio MVP: dejar un producto en 0 y ver sustitutos).

## What Changes

- En la ficha de **edición**, la pestaña **Stock** pasa a operable (sin `aria-disabled`). La ficha de **alta** sigue con Stock no operable (no hay `productId`).
- Formulario de ingreso (`inbound`) y ajuste (`adjustment`): cantidad decimal en string; **motivo obligatorio** en ajuste (guardia local + error de API `reason`).
- Tras `201`, mostrar el **stock resultante** del `StockMovement` y actualizar el stock de solo lectura de la ficha. MUST NOT cambiar `version` ni enviar PATCH de ficha.
- `X-Vectra-Terminal` lo adjunta `api.js` (T-01-06); MUST NOT duplicar la cabecera.
- Inmutabilidad: MUST NOT pintar acciones de editar/borrar movimientos ni llamar `GET .../stock-movements`.
- TDD Node: happy (ingreso, ajuste a 0, stock resultante) + unhappy (ajuste sin motivo, stock negativo, invalid_input, not_found, red, inmutabilidad / no historial).
- Supersede, solo en edición, el hueco «activar Stock no opera» de T-02-35.

## Non-goals

- Historial paginado `GET .../stock-movements` y exportar — **T-02-26** / HU-02-08 (v2; **No hace** del ticket y recableo MVP §7.5.1)
- Pestaña Imágenes (T-02-39, v2) y Relaciones (T-04-09)
- Tipos `initial` / `sale` desde esta UI (alta T-02-21, POS T-08-03)
- Merge de ficha, deep-link `#/admin/productos/{id}`, cambios a `openapi.yaml` o al crate `api`
- Playwright, jsdom, `package.json`, crates Rust nuevos

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `vectra-web`: pestaña Stock operable en la ficha de edición; formulario ingreso/ajuste; stock resultante; inmutabilidad (sin editar/borrar/historial); suite Node ampliada (happy + unhappy de HU-02-07).

## Impact

- **Repo:** `vectra-web/js/admin/product-sheet.js`, `vectra-web/js/admin/products.js`, `vectra-web/css/base.css`, `scripts/fixtures/vectra-web/product-sheet.test.mjs` (MAY tocar `products.test.mjs` si hace falta un helper de tabs)
- **APIs:** consumo existente de `POST /api/v1/products/{productId}/stock-movements` según [docs/openapi.yaml](../../../docs/openapi.yaml) (`registerStockMovement`); MUST NOT agregar paths ni implementar GET de historial
- **Verificación:** `./scripts/check-vectra-web.sh`, `openspec validate T-02-38`, regresión cargo en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ámbito:** ~2 h de UI sobre contrato ya existente; no se divide el ticket
