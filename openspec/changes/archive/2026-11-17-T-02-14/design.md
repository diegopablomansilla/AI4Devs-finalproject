# Design

## Context

En `develop`, `storage` embebe `0000`–`0004` (`atributo_def`). `Db::migrate()` / `storage::testing::test_db()` las aplican. `tests/migrate.rs` y `tests/harness.rs` asumen `list_migrations().len() == 5` (versiones 0–4). No hay tabla `producto`. Ver `proposal.md` para motivación y la spec `vortex-storage` para el contrato de esquema.

SurrealDB pinneado: `=2.7.0`. Convención 4.10: `DEFINE … IF NOT EXISTS`. 4.4.4 es el contrato de campos e índices; el ticket añade `idx_producto_codigos` (H-05) y pide documentarlo en 4.4.4. Constraints: sin lógica de dominio ni repositorio (T-02-16 / T-02-18); ~2 h; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Una migración `0005` fiel a 4.4.4 más `idx_producto_codigos` y `an_respaldo` / `ft_producto_nombre`
- Tests que cierran el Listo cuando: `uq_producto_sku`, `precio >= 0`, `codigos[*].tipo` fuera del enum
- Documentar el índice de códigos en 4.4.4
- Tests existentes de 0000–0004 siguen verdes tras aceptar `version` 5

**Non-Goals (diseño):**

- Entidad `Producto`, `ProductRepo`, mapeo UNIQUE → `duplicate_sku`
- Consultas `by-code` o búsqueda full-text de respaldo (solo se define el índice / analizador)
- HTTP, `openapi.yaml`, outbox al insertar (inserción cruda en tests)

## Decisions

### 1. Archivo `0005_producto.surql` y embed en `EMBEDDED`

**Elección:** siguiente versión libre después de `0004_atributo_def`. Exportar `MIGRATION_0005_SQL` (igual que 0000–0004) para checksum en tests. Registrar `{ version: 5, name: "producto", sql: MIGRATION_0005_SQL }` en `EMBEDDED`. Re-exportar desde `lib.rs`.

**Por qué:** el motor de migraciones (T-00-09) exige checksum y orden. No mezclar con `0004`.

**Alternativa:** meter `producto` en `0004`. Se descartó: checksum drift y mezcla de tickets.

### 2. Esquema 4.4.4 completo, con CHANGEFEED y `idx_producto_codigos`

**Elección:** copiar campos, `ASSERT`, FKs e índices de 4.4.4 (`uq_producto_sku`, `idx_producto_categoria`, `idx_producto_marca`, `an_respaldo`, `ft_producto_nombre`). Añadir `DEFINE INDEX IF NOT EXISTS idx_producto_codigos ON producto FIELDS <path-de-valor>`. Todas las sentencias con `DEFINE … IF NOT EXISTS`. Tabla maestra: `CHANGEFEED 3d`. No se siembran filas.

En `docs/4-modelo.md` § 4.4.4, agregar el `DEFINE INDEX idx_producto_codigos` y una nota de que no es UNIQUE (OEM compartible; H-05 / T-03-22).

**Por qué:** el ticket nombra el analizador, el índice de búsqueda y el índice de códigos. 4.10 exige `IF NOT EXISTS` aunque el snippet de 4.4.4 no lo muestre.

**Alternativa:** omitir `an_respaldo` / `ft_producto_nombre` “porque T-03-30 los usa”. Se descartó: están en el Hace. **Alternativa:** índice UNIQUE sobre códigos. Se descartó: 4.4.4 dice que los OEM no son únicos.

### 3. Path del índice de códigos: dialecto SurrealDB 2.7

**Elección:** el ticket y 4.4.4 documentan `codigos[*].valor`. En el `.surql`, usar la sintaxis que acepte SurrealDB `=2.7.0` para un campo de objeto dentro de un array (`codigos[*].valor` o, si el parser lo rechaza, `codigos.*.valor`). La spec de comportamiento no cambia: el índice cubre el valor de cada código y no es UNIQUE.

**Por qué:** el Listo cuando no verifica el nombre del path; T-03-22 sí lo usará. Fijar el path real en apply con un `CREATE` + (si hace falta) `INFO FOR INDEX`.

**Alternativa:** esperar a T-03-22 para el índice. Se descartó: el Hace lo exige ahora.

### 4. Tests contra `test_db()`, inserción cruda

**Elección:** `vortex-core/crates/storage/tests/producto.rs` (archivo nuevo). Harness `storage::testing::test_db()`. Inserción vía `Db::query` / `CREATE … CONTENT`, no `storage::Tx` ni repositorio. Un `CREATE` mínimo necesita `sku`, `nombre`, `tipo`, `categoria` (FK obligatoria) y `precio`. Antes, un `CREATE categoria:…` con `nombre`, `slug` y `ruta` (mismo patrón que `tests/categoria.rs`). `marca` y `atributos` se omiten. `precio` como decimal Surreal (`120.50dec` / `0dec` / `-1dec`). Unhappy: el motor rechaza el `CREATE`; no se exige `DomainError` en este ticket.

Nombres de test = tabla «Tests previstos» de la spec.

**Por qué:** T-02-01 demostró índices y `ASSERT` sin dominio. `categoria` ya migrada habilita la FK.

**Alternativa:** tests solo con `INFO FOR TABLE` sin insertar. Se descartó: el Listo cuando pide violaciones observables de UNIQUE/`ASSERT`.

### 5. Ajuste de tests que cuentan filas de `migracion`

**Elección:** `migrate.rs` y `harness.rs` MUST pasar de `len() == 5` a `len() == 6` y afirmar `version` 5 con checksum de `0005`. Checksums de 0000–0004 intactos. `test_db_has_no_business_tables` sigue válido si también aserta `record_count("producto") == 0` (opcional; el THEN de `producto` ya lo cubre).

**Por qué:** si se olvida el conteo, CI de T-09-01 falla.

### 6. Sin mapeo a DomainError ni outbox

**Elección:** una violación UNIQUE/`ASSERT` en `query` cruda es error del motor. T-02-18 usará `Tx` y mapeará `uq_producto_sku` → `duplicate_sku`. Este ticket solo demuestra que el índice y los `ASSERT` existen. No se emite outbox al insertar en tests.

**Por qué:** No hace = lógica. PC-02 / T-03-02 ya cubren el outbox para cuando exista el repositorio.

## Risks / Trade-offs

- **[Sintaxis `codigos[*].valor` vs `codigos.*.valor` en SurrealDB 2.7]** → Mitigación: decisión 3; probar el `DEFINE INDEX` en apply y documentar el path real en 4.4.4 si difiere del snippet del ticket.
- **[`DEFINE ANALYZER` / `SEARCH … BM25` con `IF NOT EXISTS`]** → Mitigación: misma convención 4.10; si el dialecto no admite `IF NOT EXISTS` en analyzer, usar la forma idempotente que acepte 2.7 (p. ej. `OVERWRITE`) sin cambiar el comportamiento.
- **[FK `record<categoria>` obliga a sembrar categoría en cada test de INSERT]** → Mitigación: helper local de SQL en `producto.rs`; no se usa `CategoryRepo`.
- **[Tests existentes `len() == 5`]** → Mitigación: tarea explícita de ajuste.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-14` y rename a `T-02-14` (igual que tickets previos).
- **[Ámbito ~2 h]** → Mitigación: mismo molde que T-02-01 (solo esquema). No se divide el ticket.

## Migration Plan

1. Red: tests de `tests/producto.rs` que no compilan / fallan (sin `0005`).
2. Green: `0005_producto.surql` + embed + nota en 4.4.4; ajustar conteo de migraciones.
3. Cierre: § 5 en `vortex-core/`, `openspec validate T-02-14`.
4. Rollback: quitar migración `0005`, tests y la línea de 4.4.4. Bases ya migradas a `0005` conservan la tabla vacía (idempotente). Sin API HTTP.

Instalaciones nuevas: `Db::migrate()` aplica `0000`–`0004`, luego `0005`. Bases locales que ya tienen `0004`: la siguiente `migrate()` aplica `0005` una vez. No hay rollback de esquema en el MVP; un checksum distinto en `0005` ya aplicado corta el arranque (T-00-09).

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El path exacto del índice de códigos se confirma en apply contra SurrealDB 2.7.
