# Design

## Context

`Db::migrate()` embebe `0000_migracion.surql` y `0001_outbox.surql`. El harness `storage::testing::test_db()` aplica todas las embebidas. Los tests `migrate.rs` y `harness.rs` asumen `list_migrations().len() == 2`. No hay tabla `categoria`. Ver `proposal.md` para motivación y alcance.

SurrealDB pinneado: `=2.7.0`. Convención 4.10: `DEFINE … IF NOT EXISTS`. 4.4.1 es el contrato de campos e índices; 4.10 añade `IF NOT EXISTS` y `CHANGEFEED 3d` en maestras.

## Goals / Non-Goals

**Goals:** una migración `0002` fiel a 4.4.1; índice único `(padre, slug)` observable con `test_db()`; tests existentes de 0000/0001 siguen verdes tras aceptar `version` 2.

**Non-Goals:** lógica de árbol, mapeo UNIQUE → `DomainError::Conflict`/`duplicate_slug` (T-02-03 usa `Tx`), HTTP. Ver Non-goals del proposal.

## Decisions

### 1. Archivo `0002_categoria.surql` y embed en `EMBEDDED`

Siguiente versión libre después de `0001`. Exportar `MIGRATION_0002_SQL` (igual que 0000/0001) para checksum en tests. Alternativa descartada: alterar `0001_outbox` (rompe checksum de bases ya migradas).

### 2. Esquema 4.4.1 completo, con CHANGEFEED

Tabla maestra: `CHANGEFEED 3d` según 4.10. Campos e índices copiados de 4.4.1. `DEFINE … IF NOT EXISTS` aunque el snippet de 4.4.1 no lo muestre (la convención operativa manda). Alternativa: omitir changefeed “porque el MVP usa outbox”. Se descarta: el ticket pide 4.4.1 y 4.10 lo exige en maestras; el outbox sigue siendo la vía de sync (PC-02).

No se siembran filas. `ancestros` y `ruta` existen como columnas; quién las calcula es T-02-03.

### 3. Tests contra `test_db()`, inserción cruda

T-09-02 deja el harness para tablas concretas. Un archivo `tests/categoria.rs` usa `storage::testing::test_db()`. Inserción vía `Db::query` / `CREATE … CONTENT` (como `tests/outbox.rs`), no `storage::Tx` ni repositorio. Un `CREATE` mínimo incluye `nombre`, `slug` y `ruta` (`ruta` no tiene `DEFAULT`). `padre` omitido = raíz (`NONE`). Unhappy: el motor rechaza el `CREATE`; no se exige `DomainError` en este ticket.

Happy de unicidad: dos padres distintos, mismo `slug` en las hijas. Unhappy **Listo cuando**: mismo `padre` (record id) y mismo `slug`.

### 4. UNIQUE con `padre` NONE

El índice de 4.4.1 es `(padre, slug)` también para raíces (`padre` ausente). Este change MUST incluir el caso “mismo padre concreto + mismo slug”. Dos raíces con el mismo `slug` se ejercitan en apply: si SurrealDB 2.7.0 trata `NONE` como igual en UNIQUE, el segundo `CREATE` falla (deseable para HU-02-01); si el motor las admite (semántica tipo SQL NULL), se deja constancia en el test o en un comentario y T-02-03 cubre unicidad de raíces en aplicación. No se cambia el índice ni se agrega un índice extra en este ticket.

### 5. Ajuste de tests que cuentan filas de `migracion`

`migrate_first_install_records_0000`, `migrate_rerun_is_noop` y los happy del harness MUST seguir exigiendo `version` 0 y 1, y MUST afirmar también `version` 2. No se relaja el checksum de `0000`/`0001`.

`test_db_has_no_business_tables` sigue válido: no hay seed, `record_count("categoria") == 0`. No se reescribe el escenario del harness (“tabla no migrada”); el THEN de conteo 0 se mantiene.

### 6. Sin mapeo a DomainError

Una violación UNIQUE en `query` cruda es error del motor. T-03-02 ya traduce UNIQUE `slug` → `duplicate_slug` en `Tx::commit`. T-02-03 usará `Tx` para el alta. Este ticket solo demuestra que el índice existe.

## Risks / Trade-offs

- **[UNIQUE con `padre` NONE en SurrealDB 2.7.0]** → Mitigación: decisión 4; el **Listo cuando** cubre el par `(padre, slug)` con padre concreto.
- **[`ASSERT string::len($value) > 0` vs string vacío]** → Mitigación: test unhappy `categoria_rejects_empty_nombre`; si el dialecto exige `NONE` vs `""`, ajustar el `.surql` sin cambiar la spec.
- **[Tests existentes `len() == 2`]** → Mitigación: tarea explícita de ajuste; si se olvida, CI de T-09-01 falla.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-01` y rename a `T-02-01` (igual que T-03-01).

## Migration Plan

Instalaciones nuevas: `Db::migrate()` aplica `0000`, `0001`, luego `0002`. Bases locales que ya tienen `0001`: la siguiente `migrate()` (p. ej. `vortex serve`, T-00-20) aplica `0002` una vez. No hay rollback de esquema en el MVP; un checksum distinto en `0002` ya aplicado corta el arranque (T-00-09).
