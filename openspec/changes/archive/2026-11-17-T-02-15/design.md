# Design

## Context

En `develop`, `storage` embebe `0000`–`0005` (`producto`). `Db::migrate()` / `storage::testing::test_db()` las aplican. `tests/migrate.rs` y `tests/harness.rs` asumen `list_migrations().len() == 6` (versiones 0–5). No hay tabla `movimiento_stock`. Ver `proposal.md` para motivación y la spec `vortex-storage` para el contrato de esquema.

SurrealDB pinneado: `=2.7.0`. Convención 4.10: `DEFINE … IF NOT EXISTS`. 4.5.1 es el contrato de campos e índice; el ticket relaja `venta` a `option<record>` hasta T-08-01 y pide documentarlo en 4.5.1. Constraints: sin lógica de dominio ni repositorio (T-02-17 / T-02-24); ~2 h; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Una migración `0006` fiel a 4.5.1 con todos los campos `READONLY`, `CHANGEFEED 3d` e `idx_mov_producto`
- `venta` como `option<record>` genérico (la tabla `venta` no existe en el MVP)
- Tests que cierran el Listo cuando: un `UPDATE` sobre un movimiento falla
- Documentar la desviación de `venta` en 4.5.1
- Tests existentes de 0000–0005 siguen verdes tras aceptar `version` 6

**Non-Goals (diseño):**

- Entidad `MovimientoStock`, signo por `tipo`, `motivo` obligatorio en `ajuste`, `StockLedger`
- `PERMISSIONS` que nieguen `DELETE` (READONLY cubre el Listo cuando: `UPDATE`)
- HTTP, `openapi.yaml`, outbox al insertar (inserción cruda en tests)

## Decisions

### 1. Archivo `0006_movimiento_stock.surql` y embed en `EMBEDDED`

**Elección:** siguiente versión libre después de `0005_producto`. Exportar `MIGRATION_0006_SQL` (igual que 0000–0005) para checksum en tests. Registrar `{ version: 6, name: "movimiento_stock", sql: MIGRATION_0006_SQL }` en `EMBEDDED`. Re-exportar desde `lib.rs`.

**Por qué:** el motor de migraciones (T-00-09) exige checksum y orden. No mezclar con `0005`.

**Alternativa:** meter `movimiento_stock` en `0005`. Se descartó: checksum drift y mezcla de tickets.

### 2. Esquema 4.5.1 completo, con CHANGEFEED y `venta` genérico

**Elección:** copiar campos, `ASSERT` e índice de 4.5.1 (`idx_mov_producto` sobre `producto, creado_en`). Todos los campos `READONLY`. `CHANGEFEED 3d` (ledger, 4.10). `DEFINE … IF NOT EXISTS`. No se siembran filas.

`venta` MUST ser `TYPE option<record>` **sin** tabla destino. El snippet de 4.5.1 muestra `option<record<venta>>`; esa tabla llega en T-08-01 (v2). En `docs/4-modelo.md` § 4.5.1, dejar el snippet objetivo y agregar una nota: hasta T-08-01 la migración usa `option<record>`.

**Por qué:** el Hace del ticket lo exige. Definir `record<venta>` fallaría o crearía una dependencia falsa a una tabla que no existe.

**Alternativa:** omitir el campo `venta` hasta T-08-01. Se descartó: 4.5.1 lo declara y T-08-01 lo *ajusta*, no lo crea.

### 3. Inmutabilidad vía `READONLY`, no vía `PERMISSIONS` de `DELETE`

**Elección:** el Listo cuando es un `UPDATE` que falla. Los campos `READONLY` de 4.5.1 son el mecanismo. El test hace `UPDATE movimiento_stock:… SET motivo = 'hack'` (u otro campo) y espera error; los valores originales se conservan.

No se añaden `PERMISSIONS FOR DELETE NONE`. Un `DELETE` SQL puede seguir existiendo a nivel motor; T-02-17 / T-02-24 no exponen borrar.

**Por qué:** el ticket nombra `UPDATE` y `READONLY`, no permisos de tabla.

**Alternativa:** `PERMISSIONS FOR UPDATE NONE` a nivel tabla. Se reserva como fallback (riesgo 1) si `READONLY` en 2.7 no rechaza el `UPDATE` con error.

### 4. Tests contra `test_db()`, inserción cruda

**Elección:** `vortex-core/crates/storage/tests/movimiento_stock.rs` (archivo nuevo). Harness `storage::testing::test_db()`. Inserción vía `Db::query` / `CREATE … CONTENT`, no `storage::Tx` ni repositorio. Un `CREATE` mínimo necesita un `producto` (FK). Antes: `CREATE categoria:…` y `CREATE producto:…` (mismo patrón que `tests/producto.rs`). `cantidad` y `stock_resultante` como decimal Surreal (`20dec` / `0dec` / `-1dec`). Unhappy: el motor rechaza el `CREATE`/`UPDATE`; no se exige `DomainError` en este ticket.

Nombres de test = tabla «Tests previstos» de la spec.

**Por qué:** T-02-14 demostró índices y `ASSERT` sin dominio. `producto` ya migrada habilita la FK.

**Alternativa:** tests solo con `INFO FOR TABLE` sin insertar. Se descartó: el Listo cuando pide un `UPDATE` observable sobre una fila.

### 5. Ajuste de tests que cuentan filas de `migracion`

**Elección:** `migrate.rs` y `harness.rs` MUST pasar de `len() == 6` a `len() == 7` y afirmar `version` 6 con checksum de `0006`. Checksums de 0000–0005 intactos. `test_db_has_no_business_tables` sigue válido (cuenta `categoria`); el THEN de `movimiento_stock` ya cubre `record_count == 0`.

**Por qué:** si se olvida el conteo, CI de T-09-01 falla.

### 6. Sin reglas de signo, sin mapeo a DomainError ni outbox

**Elección:** el esquema acepta `cantidad` distinta de cero de cualquier signo en cualquier `tipo`. Signo por `tipo` y `motivo` obligatorio en `ajuste` son T-02-17. Una violación `ASSERT`/`READONLY` en `query` cruda es error del motor. No se emite outbox al insertar en tests.

**Por qué:** No hace = lógica. PC-02 / T-03-02 ya cubren el outbox para cuando exista el ledger.

## Risks / Trade-offs

- **[`READONLY` en SurrealDB 2.7 no falla el `UPDATE` (no-op silencioso)]** → Mitigación: decisión 3; si apply confirma no-op, añadir denegación de `UPDATE` a nivel tabla (`PERMISSIONS` o equivalente del dialecto) para que el test pueda `expect_err` sin cambiar CREATE/SELECT.
- **[`TYPE option<record>` vs `record | none` en 2.7]** → Mitigación: probar el `DEFINE FIELD` en apply; el comportamiento (campo opcional, sin FK a `venta`) no cambia.
- **[FK `record<producto>` obliga a sembrar categoría + producto en cada test de INSERT]** → Mitigación: helper local de SQL en `movimiento_stock.rs`; no se usa `ProductRepo`.
- **[Tests existentes `len() == 6`]** → Mitigación: tarea explícita de ajuste.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-15` y rename a `T-02-15` (igual que tickets previos).
- **[Ámbito ~2 h]** → Mitigación: mismo molde que T-02-14 (solo esquema). No se divide el ticket.

## Migration Plan

1. Red: tests de `tests/movimiento_stock.rs` que no compilan / fallan (sin `0006`).
2. Green: `0006_movimiento_stock.surql` + embed + nota en 4.5.1; ajustar conteo de migraciones.
3. Cierre: § 5 en `vortex-core/`, `openspec validate T-02-15`.
4. Rollback: quitar migración `0006`, tests y la nota de 4.5.1. Bases ya migradas a `0006` conservan la tabla vacía (idempotente). Sin API HTTP.

Instalaciones nuevas: `Db::migrate()` aplica `0000`–`0005`, luego `0006`. Bases locales que ya tienen `0005`: la siguiente `migrate()` aplica `0006` una vez. No hay rollback de esquema en el MVP; un checksum distinto en `0006` ya aplicado corta el arranque (T-00-09).

## Open Questions

Ninguna que cambie specs, enfoque o desglose. Si `READONLY` no rechaza `UPDATE` con error, se usa el fallback de la decisión 3 sin ampliar el alcance del ticket.
