# Proposal

## Ticket padre

- **Ticket:** [T-02-16](../../../docs/7-tickets.md) — Dominio Producto y validación de atributos
- **Change:** `T-02-16`
- **HU:** [HU-02-04](../../../docs/6-historias.md#hu-02-04--alta-de-producto)
- **Depende de:** T-02-02, T-02-08, T-02-11 (en `develop`: `Categoria`/`CategoryRepo`, `Marca`/`BrandRepo`, `AtributoDef`/`TipoAtributo`/`AttributeDefRepo`; T-02-14 ya dejó la tabla `producto`)
- **Hace:** entidad `Producto` y *value objects* (`Sku`, `Codigo`, `Ubicacion`, `UnidadMedida`); validación de cada atributo contra su `AtributoDef` (tipo, opciones del enum, sin repetir `def`), con errores por ruta de campo (`attributes[n].value`); puerto `ProductRepo`.
- **No hace:** persistencia.
- **Listo cuando:** tests unitarios cubren "cuarenta" en un atributo `number` y un `def` repetido.

## Why

HU-02-04 exige dar de alta un producto con ficha completa y rechazar un valor de atributo incompatible (p. ej. `"cuarenta"` en un `number`) señalando `attributes[n].value`. La tabla `producto` ya existe (T-02-14) y el maestro `AtributoDef` está en `domain` (T-02-11); sin entidad, value objects, validación pura contra definiciones y el puerto `ProductRepo`, T-02-18 no tiene contrato hexagonal que persistir.

## What Changes

- Entidad `Producto` en el crate `domain` (campos de [4.4.4](../../../docs/4-modelo.md#444-producto) con nombres en español: `id`, `sku`, `nombre`, `tipo`, `descripcion`, `marca`, `categoria`, `atributos`, `codigos`, `unidad_medida`, `precio`, `ubicacion`, `stock`, `activo`, `version`, `creado_en`, `actualizado_en`) con `Id<Producto>`
- Value objects `Sku`, `Codigo` (`tipo` ∈ {`fabricante`, `oem`, `barras`}), `Ubicacion`, `UnidadMedida` (`unidad`/`par`/`caja`/`metro`/`kilogramo`/`litro`) y `ValorAtributo` (`texto` | `numero` | `booleano`)
- Validación pura de `atributos` contra un conjunto de `AtributoDef`: coincidencia de tipo, `opciones` si el def es `enum`, y `def` no repetido; mismatch de tipo o valor → `invalid_input` con `FieldError.field` `attributes[n].value` (índice 0-based)
- Puerto `ProductRepo` (`create`, `get`) en `domain`; sin adaptador ni SurrealDB
- Tests unitarios en `crates/domain` (happy de ficha válida + unhappy Listo cuando: `"cuarenta"` en `number` y `def` duplicado)
- TDD: red = tests que no compilan / fallan; green = entidad, VOs, validación y trait

## Non-goals

- Persistencia `ProductRepo::create` con stock inicial, UNIQUE → `duplicate_sku`, movimiento `inicial` y outbox — **T-02-18** (No hace del ticket)
- `get` hidratado (marca, ruta de categoría, atributos resueltos) y `update` con `version` — **T-02-19**
- Casos de uso en `catalog` (existencia/activo de categoría y marca) — **T-02-20**
- HTTP `POST`/`GET`/`PATCH /api/v1/products` y mapeo inglés ↔ español — **T-02-21**
- Baja/reactivación (`set_activo`) — **T-02-22**
- Listado admin paginado — **T-02-27** / **T-00-21**
- Imágenes de producto — **T-02-28** (v2)
- `StockLedger` y movimientos — **T-02-17**, **T-02-24**
- Dataset semilla / validador — **T-02-40**
- Ampliar `DomainError`, `Id<T>`, `Decimal` o `AtributoDef` — ya cubiertos por **T-00-03**, **T-00-04**, **T-02-11** (`duplicate_sku` ya existe; no se toca `error.rs`)
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas crates en `domain` — la allowlist de T-00-04 se mantiene

## Capabilities

### New Capabilities

- `domain-producto`: entidad `Producto`, value objects (`Sku`, `Codigo`, `Ubicacion`, `UnidadMedida`, `ValorAtributo`), validación de atributos contra `AtributoDef` y puerto `ProductRepo` (`create`, `get`) en el crate `domain`

### Modified Capabilities

- (ninguna; se reutilizan `domain-types`, `domain-error` y `domain-atributo-def` sin cambiar requisitos)

## Impact

- **Repo:** `vortex-core/crates/domain/` (`src/lib.rs`, módulo `producto`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `openspec validate T-02-16`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`; `check-domain-deps.sh` y `check-workspace.sh` siguen en verde (sin deps nuevas)
- **Sin** cambios de API HTTP ni de `openapi.yaml` ni de `storage`
- **Ticket:** cierra T-02-16 (Listo cuando: tests unitarios de `"cuarenta"` en atributo `number` y `def` repetido)
- **Ámbito:** dominio puro como T-02-02; cabe en ~2 h. No se divide el ticket.
