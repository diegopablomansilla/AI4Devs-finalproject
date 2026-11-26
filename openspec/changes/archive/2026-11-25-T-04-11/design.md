# Design

## Context

En `develop`, `catalog::import_masters` + `import_products` y `vortex seed` abren SurrealDB en modo directo (T-02-42 / T-02-43). `RelationService::create` (T-04-05) valida y delega en `RelationRepo::create` (T-04-03): un `RELATE` o el par bidireccional atómico; UNIQUE `(in, out)` → `duplicate_relation`. `catalog::validate_dir` ya parsea `relaciones.jsonl` si existe (`origen`, `destino`, `tipo`; SKU contra `productos.jsonl`). `catalog` MUST NOT depender de `recommend` ni de `storage` (DC-05: el ABM vive en `recommend`). `seed/relaciones.jsonl` tiene tres aristas del dataset de desarrollo (complementario Perno→Arandela, sustituto `PER-ACERO-01`→`PER-ACERO-02`, similar `TOR-AUTO-10X1`→`TOR-4-16-AC-001`). Ver `proposal.md` para motivación y `specs/` para el contrato.

Constraints: No hace = imágenes/sinónimos/notas; ~2 h; members fijos; `openspec new change` exige kebab-case (`t-04-11` → carpeta `T-04-11`); MVP del curso.

## Goals / Non-Goals

**Goals:**

- `import_relations` en `catalog` sobre un puerto de alta de relaciones + mapa SKU→id de `ProductGateway`
- Omitir `duplicate_relation`; aislar rechazos por línea; resumen `importados` / `omitidas` / `rechazados`
- `vortex seed` encadena maestros → productos → relaciones; exit 0 solo si ambos `rechazados == 0`
- Tras seed del `seed/` del repo, `PER-M8X40-ZN` tiene complementario `ARA-PERNO-01` (Listo cuando del panel)

**Non-Goals (diseño):**

- Dependencia `catalog` → `recommend` o `catalog` → `storage`
- Upsert de `prioridad`/`nota` en re-seed
- Invocar `validate_dir` en el comando (un `sinonimos.jsonl` inválido no debe bloquear relaciones)
- Extraer crate `boot` compartida con `serve`
- Automatizar el panel en Playwright (T-09-11)

## Decisions

### 1. `import_relations` en `catalog` con puerto fino; ABM sigue en `recommend`

**Elección:** `catalog::import_relations(dir, products, relations) -> SeedRelationSummary`. `relations` es un trait object-safe local (`create(NewRelacion) -> Result<(), DomainError>`), no `recommend::RelationGateway`. El binario, tras `import_products`, construye `RelationService` / `RelacionRepo` (igual que `serve`) y adapta `create` al puerto. `catalog` no implementa bidireccionalidad ni UNIQUE.

**Por qué:** T-02-43 dejó el orquestador JSONL en `catalog` (parsers, `SeedIssue`, mapa SKU). DC-05 prohíbe el ABM en `catalog`, no leer JSONL. `recommend` solo depende de `domain`; no arrastrar `catalog` ahí.

**Alternativa:** `import_relations` en `recommend`. Se descartó: duplicaría parsers/`SeedIssue` o añadiría `catalog` a `recommend`. **Alternativa:** que el binario parsee JSONL. Se descartó: el Listo cuando de aislamiento vive en `catalog` con fake in-memory.

### 2. No llamar `validate_dir` en el comando

**Elección:** parsear solo `relaciones.jsonl`. Issues de imágenes/sinónimos/notas no abortan ni se reportan aquí. Ausencia del archivo = no-op.

**Por qué:** `validate_dir` exige coherencia de archivos v2 (T-02-44 / T-03-29 / T-05-29). El No hace es no importarlos, no validar el árbol entero. T-02-43 ya tomó esta decisión para productos.

**Alternativa:** `validate_dir` primero. Se descartó: un SKU de imagen desconocido bloquearía relaciones.

### 3. Un `create` por línea; `duplicate_relation` = omitir

**Elección:** (1) parsear líneas; issue de esquema/SKU/`self_relation` → `rechazados++`, no se intenta `create`. (2) Mapa SKU→id vía `ProductGateway::list` (activo true y false), igual que productos. (3) `create`; si `ConflictCode::DuplicateRelation`, `omitidas++`. (4) Cualquier otro error de dominio → issue, `rechazados++`.

**Por qué:** el Hace pide omitir existentes, no fallar. Tratar el duplicado como rechazo rompería la segunda corrida (exit ≠ 0). No actualizar ficha de relación: el ticket no pide upsert.

**Alternativa:** `list_for_product` previo para skip. Se descartó: N queries vs un `create` que ya es idempotente por UNIQUE. **Alternativa:** fail-closed de todo el archivo. Se descartó: contradice el aislamiento de T-02-43 / HU-02-11.

### 4. Defaults de fila y conteo por línea JSONL

**Elección:** `prioridad` omitida → 1; `bidireccional` omitido → `None` (el caso de uso aplica `default_bidireccional`); `nota` omitida → `None`. `importados` cuenta **líneas** que crearon aristas, no el número de aristas (`similar_a` bidireccional = 1 importado, 2 aristas).

**Por qué:** el esquema `seed-dataset` marca esos campos opcionales. Contar líneas alinea el resumen con `archivo`/`linea` de los issues.

**Alternativa:** exigir `prioridad` y `bidireccional` en seed. Se descartó: el dataset de desarrollo omite ambos en la línea `similar_a`.

### 5. CLI: relaciones después de productos; rechazos de producto no saltan el grafo

**Elección:** si maestros Ok, siempre llamar `import_relations` aunque `productos.rechazados > 0`. Exit ≠ 0 si cualquiera de los dos `rechazados ≥ 1`. Fixture CLI mínimo de T-02-43 **sin** `relaciones.jsonl` sigue saliendo 0 (archivo opcional).

**Por qué:** un SKU inválido no debe impedir sembrar el grafo de los productos válidos. El Listo cuando del dataset (exit 0) exige ambas listas limpias.

**Alternativa:** abortar relaciones si hay rechazos de producto. Se descartó: pierde aristas válidas en el mismo unhappy path de HU-02-11.

### 6. Tests: in-memory = grafo fake; CLI = `RelacionRepo`

**Elección:** tests de `catalog` usan un fake que guarda aristas y responde `duplicate_relation`. Tests de `vortex` (`tests/seed.rs`) reabren RocksDB y `RelacionRepo::list_for_product` / `RecommendationService` para afirmar `PER-M8X40-ZN` → `ARA-PERNO-01`. El panel UI no se automatiza (T-04-08 ya existe; Listo cuando humano).

**Por qué:** allowlist de `catalog`. Las tablas `TYPE RELATION` solo existen en `storage`.

**Alternativa:** test e2e del panel. Se descartó: T-09-11; el ticket es el comando.

## Risks / Trade-offs

- **[In-memory no prueba UNIQUE de SurrealDB]** → Mitigación: escenario de omitir existentes obligatorio en `tests/seed.rs` con `RelacionRepo`.
- **[SKU inactivo no se resuelve si `list` filtra mal]** → Mitigación: dos filtros `activo: true` y `false`; paginar (igual que T-02-43).
- **[Create a mitad del JSONL deja N aristas si luego hay issues]** → Aceptado: es el Hace (rechazo aislado).
- **[Re-seed no refresca `prioridad`/`nota`]** → Aceptado: Non-goal; documentado en proposal.
- **[Listo cuando es el panel]** → Mitigación: test de grafo con el SKU ancla del dataset; checklist manual del panel en cierre.

## Migration Plan

No hay datos de producción. Quien ya corrió T-02-43 tiene catálogo sin grafo: la primera corrida de T-04-11 completa las relaciones. Rollback: revertir el PR; las aristas sembradas permanecen. T-02-44 / T-03-29 / T-05-29 reutilizarán el mismo comando para imágenes, sinónimos y notas.

## Open Questions

Ninguna que altere specs o tareas. `duplicate_relation` = omitir (sin update) y archivo `relaciones.jsonl` opcional quedan fijados arriba.
