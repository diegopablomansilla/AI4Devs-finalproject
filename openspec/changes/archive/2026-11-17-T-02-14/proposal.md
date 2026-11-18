# Proposal

## Ticket padre

- **Ticket:** [T-02-14](../../../docs/7-tickets.md) — Migración `producto`
- **Change:** `T-02-14`
- **HU:** [HU-02-04](../../../docs/6-historias.md#hu-02-04--alta-de-producto)
- **Depende de:** T-02-01, T-02-08, T-02-11 (en `develop`: tablas `categoria`, `marca` y `atributo_def` ya migradas)
- **Hace:** tabla, campos e índices de [4.4.4](../../../docs/4-modelo.md#444-producto), incluidos el analizador `an_respaldo`, el índice `ft_producto_nombre` y un índice nuevo `idx_producto_codigos` sobre `codigos[*].valor` (hallazgo H-05, se documenta en 4.4.4).
- **No hace:** lógica.
- **Listo cuando:** tests verifican `uq_producto_sku`, el `ASSERT` de `precio >= 0` y que `codigos[*].tipo` rechaza valores fuera del enum.

## Why

HU-02-04 necesita la tabla `producto` como fuente de verdad del catálogo (SKU, ficha, códigos, precio, proyección de stock). Categorías, marcas y definiciones de atributo ya existen; sin el esquema de 4.4.4 no se puede persistir un SKU, ni T-02-16/T-02-18 construir dominio y repositorio, ni T-03-22 buscar por código exacto cuando Meilisearch está caído (H-05).

## What Changes

- Migración embebida `0005_producto.surql`: tabla `SCHEMAFULL` `producto` según 4.4.4 (campos, `ASSERT` de `sku`/`nombre`/`precio`/`stock`/`unidad_medida`/`codigos[*].tipo`, FKs a `marca`/`categoria`/`atributo_def`, índices `uq_producto_sku`, `idx_producto_categoria`, `idx_producto_marca`, analizador `an_respaldo`, índice de búsqueda `ft_producto_nombre`), con `DEFINE … IF NOT EXISTS` (4.10)
- Índice **nuevo** `idx_producto_codigos` sobre `codigos[*].valor` (no UNIQUE: los OEM se pueden compartir) y su documentación en [4.4.4](../../../docs/4-modelo.md#444-producto) (H-05)
- Registro de la migración en `EMBEDDED` para que `Db::migrate()` / `storage::testing::test_db()` la apliquen
- Tests de integración en `crates/storage/tests/` (happy: esquema + alta mínima + OEM duplicable + `precio = 0`; unhappy: SKU duplicado, `precio < 0`, `codigos[*].tipo` fuera del enum)
- Ajuste de tests existentes (T-00-09 / T-09-02) que cuentan filas de `migracion` (`len() == 5`)
- TDD: red = tests de esquema sin `0005`; green = `.surql` + embed + nota en 4.4.4

## Non-goals

- Entidad `Producto`, value objects (`Sku`, `Codigo`, `Ubicacion`, `UnidadMedida`) y validación de atributos contra `AtributoDef` — **T-02-16** (No hace del ticket: lógica)
- Puerto `ProductRepo` y persistencia de alta/edición — **T-02-18**, **T-02-19**
- Tabla `movimiento_stock` y ledger — **T-02-15**, **T-02-17**, **T-02-24**
- Casos de uso, HTTP `POST/GET/PATCH /products`, UI de ficha — **T-02-20**, **T-02-21**, **T-02-35**
- Baja/reactivación, listado admin, `attribute_in_use` — **T-02-22**, **T-02-27**, **T-02-23**
- `imagen_producto` — **T-02-28** (v2)
- Búsqueda por código (`GET /search/by-code`) y búsqueda de respaldo en SurrealDB — **T-03-22**, **T-03-30**
- Relaciones `sustituible_por`, embeddings, notas técnicas — **T-04-01**, **T-05-03**, **T-05-25**
- Seed de productos — **T-02-43**
- Mapeo UNIQUE `sku` → `duplicate_sku` en `Tx` / HTTP — **T-02-18** / **T-02-21**
- HTTP, `openapi.yaml`, `tracing`, UI
- Ampliar `DomainError` o `storage::Tx` — ya cubiertos por **T-00-03** / **T-03-02**

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: migración `0005` con tabla `producto`, campos e índices de 4.4.4 más `idx_producto_codigos`; tests de `uq_producto_sku`, `precio >= 0` y enum de `codigos[*].tipo`

## Impact

- **Repo:** `vortex-core/crates/storage/migrations/0005_producto.surql`, `src/migrate.rs` (`EMBEDDED`), `src/lib.rs` (re-export de `MIGRATION_0005_SQL`), `tests/` (nuevo `producto.rs` + ajuste de `migrate.rs` / `harness.rs`); `docs/4-modelo.md` § 4.4.4 (`idx_producto_codigos` + H-05)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p storage --test producto`; `openspec validate T-02-14`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml`
- **Ticket:** cierra T-02-14 (Listo cuando: `uq_producto_sku`, `ASSERT precio >= 0`, `codigos[*].tipo` fuera del enum)
- **Ámbito:** migración de esquema + tests + nota en 4.4.4; cabe en ~2 h (mismo molde que T-02-01). No se divide el ticket.
