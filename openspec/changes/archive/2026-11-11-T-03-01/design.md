# Design

## Context

`Db::migrate()` (T-00-09) embebe un único script `0000_migracion.surql`. El harness `storage::testing::test_db()` (T-09-02) aplica todas las migraciones embebidas. Los tests `migrate.rs` y `harness.rs` asumen `list_migrations().len() == 1`. No hay tablas de negocio ni outbox. Ver `proposal.md` para motivación y alcance.

SurrealDB pinneado: `=2.7.0`. Convención 4.10: `DEFINE … IF NOT EXISTS` u `OVERWRITE`.

## Goals / Non-Goals

**Goals:** una migración `0001` fiel a 4.8; seed de los dos cursores; `ASSERT` verificables con `test_db()`; tests existentes de 0000 siguen verdes tras aceptar `version` 1.

**Non-Goals:** API de escritura (`Tx`, notifier), tipos de dominio, worker, métricas, HTTP. Ver Non-goals del proposal.

## Decisions

### 1. Archivo `0001_outbox.surql` y embed en `EMBEDDED`

Siguiente versión libre después de `0000`. Exportar `MIGRATION_0001_SQL` (igual que `MIGRATION_0000_SQL`) para checksum en tests. Alternativa descartada: meter el SQL en `0000` (rompe checksum de instalaciones ya migradas).

### 2. Esquema 4.8 completo, incluidos valores de entidad v2

El `ASSERT` de `entidad` incluye `sinonimo`, `nota_tecnica` e `imagen_producto` aunque sus tickets sean v2. Alternativa: lista solo MVP (`producto`, `categoria`, `marca`, `atributo_def`) y un `ALTER` después. Se descarta: el worker no corre en este ticket; ampliar el `ASSERT` ahora evita una migración extra y el modelo publicado ya lista esos valores.

`registro` queda como `record` sin tabla (un evento apunta a distintas maestras). `ultimo_evento` sí se tipa `option<record<outbox>>`.

### 3. Seed de `sync_cursor:vectores` en el MVP del curso

El ticket pide ambos ids. El consumidor de vectores es T-05-08 (v2); sembrar el cursor ahora no habilita el worker y deja el esquema estable para catálogo. Alternativa: solo `meilisearch`. Se descarta por el **Hace** explícito.

Seed: `CREATE sync_cursor:meilisearch CONTENT { } IF NOT EXISTS` (y análogo para `vectores`). `ultimo_evento` es `option` y queda `NONE`. `actualizado_en` lo rellena `VALUE time::now()`. El motor de migraciones no reejecuta SQL de una versión ya registrada; `IF NOT EXISTS` cubre 4.10 y re-aplicación manual.

### 4. Sin CHANGEFEED en tablas técnicas

4.10 habilita `CHANGEFEED 3d` en maestras y ledger. Outbox es la vía MVP (PC-02). No se agrega changefeed aquí.

### 5. Tests contra `test_db()`, no RocksDB a mano

T-09-02 deja el harness para tablas concretas. Un archivo `tests/outbox.rs` usa `storage::testing::test_db()`. Happy: insertar `CREATE outbox:ulid() CONTENT { entidad: 'producto', registro: producto:seed, operacion: 'upsert' }` (el `record` no exige que exista la tabla destino). Unhappy: el mismo `CREATE` con `entidad`/`operacion`/`consumidor` fuera de lista; `Db::query` ya falla vía `.check()`.

Para comprobar ids de cursor sin ampliar `Db`: SurrealQL con `THROW` si falta el registro, ejecutado por `query`.

### 6. Ajuste de tests que cuentan filas de `migracion`

`migrate_first_install_records_0000`, `migrate_rerun_is_noop` y los happy del harness MUST seguir exigiendo `version` 0, y MUST aceptar (o afirmar) también `version` 1. No se relaja el checksum de `0000`.

### 7. Índice único de `outbox_fallo`

Se define en el `.surql` según 4.8. No se exige un test de colisión en este ticket (el **Listo cuando** cubre `entidad` y `operacion`); el unhappy de `consumidor` cubre el otro `ASSERT` de 4.8.2.

## Risks / Trade-offs

- **[SurrealQL `ASSERT` / `CREATE IF NOT EXISTS` en 2.7.0]** → Mitigación: el test unhappy es la prueba; si el dialecto difiere (`INSIDE` vs `IN`, `NONE` vs omitir campo), ajustar el `.surql` sin cambiar la spec.
- **[`producto:seed` sin tabla `producto`]** → Mitigación: `TYPE record` no ancla tabla; si el motor rechaza el id, usar `outbox:ulid()` como `registro` de prueba (sigue siendo `record`).
- **[Tests existentes `len() == 1`]** → Mitigación: tarea explícita de ajuste en el mismo change; si se olvida, CI de T-09-01 falla.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-03-01` y rename a `T-03-01` (igual que T-00-09 / T-01-15).

## Migration Plan

Instalaciones nuevas: `Db::migrate()` aplica `0000` luego `0001`. Bases locales que ya tienen `0000`: la siguiente `migrate()` (p. ej. `vortex serve`, T-00-20) aplica `0001` una vez. No hay rollback de esquema en el MVP; un checksum distinto en `0001` ya aplicado corta el arranque (T-00-09).
