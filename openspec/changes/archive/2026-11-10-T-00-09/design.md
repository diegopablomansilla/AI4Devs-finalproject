# Design

## Context

`Db::open` (T-00-08) bootstrap NS/DB sin `.surql` ni tabla `migracion`. El ticket pide motor con archivos `NNNN_nombre.surql` embebidos, registro y validación de checksum. T-00-20 llamará `open` + `migrate`; este change solo expone `migrate` en el crate.

## Goals / Non-Goals

**Goals:** `migrate()` idempotente; checksum SHA-256 del contenido UTF-8 del `.surql`; error con versión en drift; solo `0000` en MVP de este ticket.

**Non-Goals:** harness mem, CLI, tablas de negocio, serde en `storage` más allá de allowlist.

## Decisions

### 1. `migrate()` separado de `open`

`open` no ejecuta migraciones. `Db::migrate()` aplica el set embebido tras `open`.

### 2. Embedd y orden

`const EMBEDDED: &[Migration]` con `version`, `name`, `sql` desde `include_str!`. Orden ascendente por `version`. Solo `0000_migracion.surql` en este ticket.

### 3. Checksum y registro

SHA-256 lowercase hex del archivo completo. Tras ejecutar el SQL con éxito, `CREATE migracion CONTENT { version, nombre, checksum }` (`aplicada_en` por DEFAULT). Si ya existe la `version` con el mismo checksum, skip. Si existe con checksum distinto, `MigrateError` que nombra la versión.

### 4. Lectura de aplicadas

`SELECT version, checksum FROM migracion`. Si la tabla no existe aún, tratar lista vacía (primera corrida aplica `0000`).

### 5. Tests de drift

`Db::apply_migrations(&[Migration])` público para integración: segunda pasada con SQL distinto en misma versión sin tocar filas a mano.

### 6. `sha2` en allowlist

Pin `sha2 = "=0.10.9"` (o versión acordada en workspace) y actualizar `storage_deps_allowed` + spec `cargo-workspace`.

## Risks

- **SurrealQL de DEFINE:** usar `IF NOT EXISTS` según 4.10; validar con tests.
- **Tipado de SELECT:** parsear con `take` del SDK sin añadir `serde` al crate.
