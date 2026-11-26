# Proposal

## Ticket padre

- **Ticket:** [T-02-36](../../../docs/7-tickets.md) — UI atributos y códigos en la ficha
- **Change:** `T-02-36`
- **HU:** [HU-02-04](../../../docs/6-historias.md#hu-02-04--alta-de-producto)
- **Depende de:** T-02-35 (ficha de alta, pestaña Datos, en `develop`); T-02-12 (`GET /api/v1/attribute-definitions` y contrato de `AttributeDefinition`, en `develop`)
- **Hace:** editor de atributos que muestra el control según el tipo (`text`, `number`, `boolean`, `enum`) y la unidad; editor de códigos (fabricante, OEM).
- **No hace:** código de barras (post-MVP).
- **Listo cuando:** el error de "largo = cuarenta" se muestra sobre ese campo.

## Why

HU-02-04 pide un alta con ficha completa: atributos tipados y códigos. T-02-35 ya crea el producto desde la pestaña Datos, pero **omite** `attributes` y `codes` (No hace de ese ticket). Sin el editor, el administrador no puede cargar medida/largo/grado ni fabricante/OEM, y el escenario «Atributo con tipo inválido» (`largo` = `"cuarenta"` → `invalid_input` en `attributes[n].value`) no tiene superficie en la SPA.

## What Changes

- En la pestaña **Datos** de la ficha de alta (`#/admin/productos`), pintar un **editor de atributos** hidratado desde `GET /api/v1/attribute-definitions` (solo definiciones **activas**, orden de la API). Cada fila MUST mostrar `label` y `unit` (si hay) y un control según `type`: `text` → `vx-field`; `number` → `vx-field` de texto (para poder escribir `"cuarenta"`); `boolean` → `<select>` Sí/No; `enum` → `<select>` con `options`.
- Pintar un **editor de códigos** con fabricante (`manufacturer`) y OEM (`oem`). MUST NOT ofrecer `barcode`.
- Al guardar, `POST /api/v1/products` MUST incluir `attributes` (`AttributeValueInput`: `definitionId` + `value` tipado) y `codes` (`ProductCode`) con los valores no vacíos. Número válido → JSON number; `"cuarenta"` → string (el backend responde `400` `invalid_input` `attributes[n].value`).
- `invalid_input` con `errors[].field` `attributes[n].value` MUST pintarse **sobre el control de ese atributo** (Listo cuando: largo = cuarenta). Toast por `code` vía `showError` / `messageFor` (T-01-07).
- TDD Node (`node --test`, sin jsdom ni npm): happy = controles por tipo + unidad, POST con atributos/códigos del perno; unhappy = cuarenta sobre largo, sin barcode, definiciones inactivas ausentes, red en el lookup de atributos.
- El escenario T-02-35 «la ficha no ofrece atributos ni códigos» queda **superseded**. MUST NOT agregar *tokens* CSS nuevos ni Web Component nuevo.

## Non-goals

- Código de barras / `barcode` — **No hace** del ticket (post-MVP)
- Edición de ficha existente / `PATCH` / conflicto de `version` — **T-02-37**
- Pestaña Stock (ingresos/ajustes) — **T-02-38**
- Pestaña Imágenes / `multipart` — **T-02-39** (v2)
- Pestaña Relaciones — **T-04-09**
- Alta de definiciones de atributo (ABM `#/admin/atributos`) — ya cubierto por **T-02-13**
- Hash profundo `#/admin/productos/nuevo` — `parseHash` sigue en dos segmentos (T-01-01)
- Atributos filtrados por categoría (el maestro es global; [docs/4-modelo.md](../../../docs/4-modelo.md) § 4.4.3)
- Endpoints nuevos o cambios a `docs/openapi.yaml` — contrato cerrado en **T-02-21** / **T-02-12**
- Playwright / e2e de alta — **T-09-09** / **T-09-11**
- jsdom, `package.json`, dependencias npm, job nuevo de CI, crates Rust
- Autenticación o roles (MVP: red local de confianza)

## Capabilities

### New Capabilities

- Ninguna. El editor de atributos y códigos es contenido de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: la ficha de alta (pestaña Datos) incluye editor de atributos por tipo/unidad y editor de códigos fabricante/OEM; `POST ProductCreate` envía `attributes` y `codes`; el error `attributes[n].value` se pinta sobre el campo (Listo cuando); ampliar la suite Node (happy + unhappy). El requisito «la ficha no ofrece atributos ni códigos» de T-02-35 queda superseded.

## Impact

- **Repo:** `vectra-web/js/admin/product-sheet.js` (controles y parseo); `vectra-web/js/admin/products.js` (lookup de definiciones al abrir la ficha + POST); `vectra-web/css/base.css` (clases del editor con *tokens* `--vx-*` existentes); `scripts/fixtures/vectra-web/product-sheet.test.mjs` (reemplazar `sheet_no_attributes_or_codes`; nuevos casos); comentario de `scripts/check-vectra-web.sh` si hace falta
- **APIs / OpenAPI:** consumo de `GET /api/v1/attribute-definitions` y de `ProductCreate.attributes` / `codes` según [docs/openapi.yaml](../../../docs/openapi.yaml); MUST NOT agregar paths ni DTOs
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-02-36`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando del ticket: escribir `"cuarenta"` en largo (number) y ver el error sobre ese campo (contra `vortex serve`)
- **Ticket:** cierra T-02-36
- **Alcance:** cabe en ~2 h (extiende la ficha de T-02-35; contrato HTTP ya existe); no se divide
