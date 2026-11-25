# Design

## Context

En `develop`, `storage` embebe `0000`–`0006` (`movimiento_stock`). `Db::migrate()` / `storage::testing::test_db()` las aplican. `tests/migrate.rs` y `tests/harness.rs` asumen `list_migrations().len() == 7` (versiones 0–6). No hay tablas `TYPE RELATION`. Ver `proposal.md` para motivación y la spec `vortex-storage` para el contrato de esquema.

SurrealDB pinneado: `=2.7.0`. Convención 4.10: `DEFINE … IF NOT EXISTS`. 4.6.1 es el contrato de las tres tablas (PM-05: `sustituible_por`). Constraints: sin lógica de dominio ni repositorio (T-04-02 / T-04-03); ~2 h; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Una migración `0007` con las tres tablas de 4.6.1 (`similar_a`, `sustituible_por`, `complementa`), `ENFORCED`, índices únicos `(in, out)` y `CHANGEFEED 3d`
- Tests que cierran el Listo cuando: `ENFORCED` rechaza destino inexistente; UNIQUE `(in, out)` rechaza duplicados
- Tests existentes de 0000–0006 siguen verdes tras aceptar `version` 7

**Non-Goals (diseño):**

- Entidad Relación, `RelationRepo`, mapeo UNIQUE → `duplicate_relation`, rechazo de `in = out`
- Consulta 4.6.2, HTTP, `openapi.yaml`, outbox al `RELATE` (inserción cruda en tests)
- Documentar de nuevo PM-05 en 4.6.1 (el nombre `sustituible_por` ya está)

## Decisions

### 1. Archivo `0007_relaciones.surql` y embed en `EMBEDDED`

**Elección:** siguiente versión libre después de `0006_movimiento_stock`. Un solo archivo con las tres tablas (igual que `0001_outbox.surql` agrupa tablas técnicas). Nombre de migración `"relaciones"` (no `"sustituible_por"`): las tres tablas son pares. Exportar `MIGRATION_0007_SQL`. Registrar `{ version: 7, name: "relaciones", sql: MIGRATION_0007_SQL }` en `EMBEDDED`. Re-exportar desde `lib.rs`.

**Por qué:** el motor de migraciones (T-00-09) exige checksum y orden. No mezclar con `0006`. Un archivo por ticket.

**Alternativa:** tres migraciones `0007`/`0008`/`0009`. Se descartó: un ticket, un archivo, ~2 h. **Alternativa:** `0007_sustituible_por.surql`. Se descartó: oculta `similar_a` y `complementa`.

### 2. Esquema 4.6.1 completo, tres tablas idénticas

**Elección:** copiar el snippet de 4.6.1 para `sustituible_por` y repetirlo cambiando el nombre de tabla e índice (`uq_similar_a`, `uq_sustituible_por`, `uq_complementa`). Cada una: `SCHEMAFULL TYPE RELATION IN producto OUT producto ENFORCED`, campos `prioridad` / `nota` / `grupo_bidireccional` / `creado_en` / `actualizado_en`, `CHANGEFEED 3d`, `DEFINE … IF NOT EXISTS`. No se siembran aristas. No hay `ASSERT in != out` (4.6.1 lo deja en dominio).

El contrato HTTP `substitute` no se toca; solo existe la tabla `sustituible_por`.

**Por qué:** el Hace nombra las tres tablas y PM-05. 4.10 exige `IF NOT EXISTS` aunque el snippet de 4.6.1 no lo muestre.

**Alternativa:** una sola tabla con campo `tipo`. Se descartó: 4.6.1 y el grafo Surreal (`$p->similar_a`) exigen tres tablas.

### 3. Tests contra `test_db()`, `RELATE` crudo

**Elección:** `vortex-core/crates/storage/tests/relaciones.rs` (archivo nuevo). Harness `storage::testing::test_db()`. Aristas vía `Db::query` / `RELATE producto:… -> <tabla> -> producto:…`, no `storage::Tx` ni repositorio. Un `RELATE` mínimo necesita dos `producto` (FK `ENFORCED`). Antes: `CREATE categoria:…` y dos `CREATE producto:…` (mismo patrón que `tests/producto.rs`). Unhappy: el motor rechaza el `RELATE`; no se exige `DomainError` en este ticket.

Listo cuando sobre `sustituible_por` (nombre del ticket). Happy cubre las tres tablas. El par A→B en `similar_a` y `sustituible_por` a la vez demuestra que UNIQUE es por tabla.

Nombres de test = tabla «Tests previstos» de la spec.

**Por qué:** T-02-14 / T-02-15 demostraron índices y `ASSERT` sin dominio. `producto` ya migrada habilita `ENFORCED`.

**Alternativa:** tests solo con `INFO FOR TABLE`. Se descartó: el Listo cuando pide violaciones observables de `ENFORCED` y UNIQUE.

### 4. Ajuste de tests que cuentan filas de `migracion`

**Elección:** `migrate.rs` y `harness.rs` MUST pasar de `len() == 7` a `len() == 8` y afirmar `version` 7 con checksum de `0007`. Checksums de 0000–0006 intactos. `test_db_has_no_business_tables` sigue válido (cuenta `categoria`); el THEN de relaciones ya cubre `record_count == 0`.

**Por qué:** si se olvida el conteo, CI de T-09-01 falla.

### 5. Sin mapeo a DomainError ni outbox

**Elección:** una violación `ENFORCED` / UNIQUE / `ASSERT` en `query` cruda es error del motor. T-04-03 usará `Tx` y mapeará UNIQUE → `duplicate_relation`. Este ticket solo demuestra que el esquema existe. No se emite outbox al `RELATE` en tests. `self_relation` y `nota` ≤ 500 son T-04-02.

**Por qué:** No hace = lógica.

## Risks / Trade-offs

- **[`ENFORCED` en SurrealDB 2.7 no rechaza un `out` inexistente]** → Mitigación: el test `relacion_enforced_rejects_missing_target` es el Listo cuando. Si apply confirma que el motor acepta el `RELATE`, **no** sustituir con chequeo en Rust (T-04-03). Escalar con el error observado; el dialecto 2.7.0 es el pin.
- **[`in` / `out` reservados en `DEFINE INDEX … FIELDS in, out`]** → Mitigación: probar el snippet de 4.6.1; si el parser rechaza, usar la forma que acepte 2.7 (`in`/`out` entre acentos graves u otra) sin cambiar el comportamiento UNIQUE `(in, out)`.
- **[`DEFINE TABLE … TYPE RELATION … IF NOT EXISTS` + `CHANGEFEED`]** → Mitigación: misma convención 4.10; si el dialecto no admite `IF NOT EXISTS` en RELATION, usar la forma idempotente que acepte 2.7 (`OVERWRITE`) sin cambiar campos ni `ENFORCED`.
- **[FK `ENFORCED` obliga a sembrar categoría + dos productos en cada test de `RELATE`]** → Mitigación: helper local de SQL en `relaciones.rs`; no se usa `ProductRepo`.
- **[Tests existentes `len() == 7`]** → Mitigación: tarea explícita de ajuste.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-04-01` y rename a `T-04-01` (igual que tickets previos).
- **[Ámbito ~2 h]** → Mitigación: mismo molde que T-02-14 / T-02-15 (solo esquema). No se divide el ticket.

## Migration Plan

1. Red: tests de `tests/relaciones.rs` que no compilan / fallan (sin `0007`).
2. Green: `0007_relaciones.surql` + embed; ajustar conteo de migraciones.
3. Cierre: § 5 en `vortex-core/`, `openspec validate T-04-01`.
4. Rollback: quitar migración `0007` y tests. Bases ya migradas a `0007` conservan las tablas vacías (idempotente). Sin API HTTP.

Instalaciones nuevas: `Db::migrate()` aplica `0000`–`0006`, luego `0007`. Bases locales que ya tienen `0006`: la siguiente `migrate()` aplica `0007` una vez. No hay rollback de esquema en el MVP; un checksum distinto en `0007` ya aplicado corta el arranque (T-00-09).

## Open Questions

Ninguna que cambie specs, enfoque o desglose. La sintaxis exacta de `FIELDS in, out` y `TYPE RELATION … IF NOT EXISTS` se confirma en apply contra SurrealDB 2.7.
