# Design

## Context

En `develop`, `domain` ya tiene `Producto`, `Decimal`, `Slug`, `AtributoDef` y `FieldError` (T-02-16). El crate `catalog` solo puede depender de `domain` (allowlist T-02-05). `vortex seed` es stub «no implementado» (T-00-07). No existe `seed/` en la raíz. Ver `proposal.md` para motivación y `specs/seed-dataset` / `specs/cargo-workspace` para el contrato.

Constraints: No hace = importar; ~2 h; members del workspace fijos; HU-02-11 MVP parcial (sin importar imágenes ni sinónimos). `openspec new change` exige kebab-case: scaffold `t-02-40` y rename a `T-02-40` (igual que T-02-38).

## Goals / Non-Goals

**Goals:**

- Contrato JSONL en `seed/` + README, con ejemplo mínimo (perno M8)
- Validador de directorio en `catalog` que cierra el Listo cuando (precio no numérico → archivo, línea, campo)
- Allowlist de `catalog` ampliada a `serde` / `serde_json` para parsear JSONL

**Non-Goals (diseño):**

- Enganchar `vortex seed` (sigue stub; T-02-42 llama al validador y luego importa)
- Dataset ~200 SKUs, importación, imágenes reales, CLI `--check`
- `tempfile` u otras crates nuevas en workspace.dependencies
- Reusar constructores de `Producto` para **todo** el JSONL (el informe es archivo/línea, no `DomainError` HTTP)

## Decisions

### 1. Biblioteca en `catalog`, no CLI

**Elección:** `catalog::seed::validate_dir(path) -> Vec<SeedIssue>` (nombres exactos MAY variar). `bin/vortex` no despacha `Commands::Seed`. Tests invocan la función; T-02-41 y T-02-42 la reutilizan.

**Por qué:** el ticket No hace importar; T-00-07 reserva `vortex seed` al comando de carga. Validar en biblioteca cumple Listo cuando sin romper el stub ni mezclar T-02-42.

**Alternativa:** `vortex seed --check`. Se descartó: cambia `vortex-cli` y adelanta el comando de T-02-42. **Alternativa:** script en `scripts/`. Se descartó: T-02-42/43 viven en Rust sobre casos de uso de `catalog`.

### 2. Claves naturales; `precio` string decimal

**Elección:** JSON con nombres de [4.4](../../docs/4-modelo.md#44-catálogo). Referencias: `categoria`/`marca` = slug, `atributos[].clave`, productos/relaciones/notas/imágenes = SKU. `precio` y `stock_inicial` son strings del wire `Decimal` (igual que OpenAPI), no números JSON.

**Por qué:** el ticket pide slug, clave y SKU; ULIDs no son estables entre seeds. El Listo cuando es un precio **no numérico**: un string `"barato"` solo es observable si el campo es string (un número JSON ya sería numérico).

**Alternativa:** IDs ULID en el JSONL. Se descartó: el Hace del ticket. **Alternativa:** aceptar número JSON en `precio`. Se descartó: no cubre «no numérico» y se aparta del `Decimal` de dominio.

### 3. Archivos v2 opcionales, esquema documentado

**Elección:** `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` e `imagenes/` son opcionales. Si están, se validan. README documenta el esquema completo (mismo formato que la importación futura). El ejemplo mínimo del repo no incluye filas v2.

**Por qué:** HU-02-11 MVP «sin imágenes ni sinónimos»; T-02-41 sí llenará esos archivos y el validador debe aceptarlos.

**Alternativa:** exigir los siete JSONL vacíos. Se descartó: ruido para el MVP y T-02-42 no los importa.

### 4. Allowlist `catalog` += serde + serde_json

**Elección:** ampliar `cargo-workspace` y `check-workspace.sh` (`catalog_deps_allowed` acepta `serde` y `serde_json` en `[dependencies]`). Fixture happy nuevo; `catalog` + `surrealdb` / `clap` sigue fallando.

**Por qué:** parsear JSONL sin `serde_json` en `catalog` fuerza o un crate nuevo (members fijos) o meter el validador en `api`/`vortex` (capa incorrecta). Pins ya existen en `[workspace.dependencies]`.

**Alternativa:** crate `seed`. Se descartó: viola members fijos. **Alternativa:** parseo a mano. Se descartó: ~2 h y peor informe de campo.

### 5. Issues acumulados, no fail-fast; tests con dirs temporales de `std`

**Elección:** el validador recorre todos los JSONL y devuelve la lista completa. Fixtures de test: directorio bajo `std::env::temp_dir()` + nombre único, o árbol en `crates/catalog/tests/fixtures/`. Sin crate `tempfile`.

**Por qué:** T-02-43 informará varios rechazos; un solo error escondido no sirve. `tempfile` no está pineado.

**Alternativa:** primer error y return. Se descartó: el Hace pide informar registro y campo; HU-02-11 no limita a uno.

### 6. Ejemplo mínimo en `seed/`, no 200 SKUs

**Elección:** una categoría (`fijaciones` / `pernos`), marca `fixser`, atributo `medida`, producto `PER-M8X40-ZN` con `precio` `"120.50"`. T-02-41 reemplaza/expande este árbol.

**Por qué:** el ticket es formato + validador; ~200 SKUs es T-02-41. El test `seed_repo_seed_dir_has_no_issues` ancla el Listo cuando de T-02-41.

## Risks / Trade-offs

- **[Allowlist de catalog más ancha]** → Mitigación: `serde`/`serde_json` ya pineados; `check-workspace.sh` sigue vetando `surrealdb`/`storage`/`clap`.
- **[Validador más laxo que el import de T-02-42]** → Mitigación: reutilizar `Decimal`/`Slug` de dominio para precio y slugs; T-02-42 puede endurecer (categoría inactiva, etc.) sin cambiar el JSONL.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-40` y rename a `T-02-40`.
- **[Tests que leen `seed/` desde `catalog`]** → Path `CARGO_MANIFEST_DIR/../../../seed`; si alguien mueve el crate, el test happy del repo falla de forma explícita.

## Migration Plan

No hay datos en producción. El change solo agrega archivos y una función. Rollback: revertir el PR. T-02-42 es el primer consumidor CLI.

## Open Questions

Ninguna que altere specs o tareas. El cableado `vortex seed` queda en T-02-42.
