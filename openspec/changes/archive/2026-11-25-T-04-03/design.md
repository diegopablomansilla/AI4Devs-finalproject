# Design

## Context

En `develop`, `domain` expone `Relacion`, `NewRelacion`, `TipoRelacion`, `resolve_bidireccional`, `RelationRepo` (`create` / `update` / `delete` / `list_for_product` sin adaptador) y `DomainError` (`self_relation`, `duplicate_relation`, `not_found`). `storage` abre SurrealDB, aplica `0000`–`0007` (tablas `TYPE RELATION` `ENFORCED` + UNIQUE `(in, out)`), y ofrece `Tx` + mapper UNIQUE (`sku` → `duplicate_sku`; token `relation` → `duplicate_relation`). Los adaptadores de catálogo viven en `storage` como `*Repo { db: Arc<Db> }`. No hay adaptador de `RelationRepo`. `recommend` sigue vacío. Ver `proposal.md` para motivación y `specs/vortex-storage` para el contrato.

Constraints: sin edición ni listado (T-04-04); sin HTTP ni `recommend` (T-04-05); ~2 h; `domain` no cambia firmas; SurrealDB `=2.7.0`; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Adaptador hexagonal de `RelationRepo::create`: un `RELATE` o el par bidireccional atómico con `grupo_bidireccional` compartido
- UNIQUE `(in, out)` → `duplicate_relation` (Listo cuando)
- Tests que cierran el Listo cuando: "A similar B" crea A→B y B→A; repetir da `duplicate_relation`

**Non-Goals (diseño):**

- Implementar `update` / `delete` / `list_for_product` (stubs `Unavailable`)
- Outbox, Meilisearch, HTTP, `catalog`, `recommend`
- Cambiar `0007_relaciones.surql` o `DomainError`

## Decisions

### 1. Adaptador `RelacionRepo` en `storage/src/relacion.rs`

**Elección:** newtype `RelacionRepo { db: Arc<Db> }` que implementa `domain::RelationRepo`. `lib.rs` hace `mod relacion` y `pub use relacion::RelacionRepo`. Hidratar con DTO `serde` + strip de `similar_a:` / `sustituible_por:` / `complementa:` / `producto:` (reusar el patrón `record_id_to_domain_id` de `marca.rs` / `producto.rs`). Escrituras vía `Tx::run` + `commit` **sin** `emit`. Tras el commit, un `SELECT` de la arista saliente para timestamps del motor y `Relacion::try_new`.

`update` / `delete` / `list_for_product` devuelven `DomainError::unavailable(UnavailableCode::Unavailable)` sin I/O.

**Por qué:** mismo molde que T-02-08 / T-02-18. El adaptador pertenece a `storage`, no a `catalog` ni a `recommend` (DC-05 se cablea en T-04-05).

**Alternativa:** crate `recommend`. Se descartó: `recommend` sigue como lib vacía (`check-workspace.sh`) y T-04-05 es el dueño de los casos de uso. **Alternativa:** implementar `list_for_product` ahora para releer. Se descartó: T-04-04; un `SELECT` privado basta para el retorno de `create`.

### 2. Una `Tx`: RELATE simple o par con ULID de grupo

**Elección:**

```text
-- unidireccional
RELATE producto:{origen} -> {tabla}:{id} -> producto:{destino}
  SET prioridad = $p, nota = $n;

-- bidireccional (misma Tx)
LET $g = '{grupo_ulid}';
RELATE producto:{A} -> {tabla}:{id_ab} -> producto:{B}
  SET prioridad = $p, nota = $n, grupo_bidireccional = $g;
RELATE producto:{B} -> {tabla}:{id_ba} -> producto:{A}
  SET prioridad = $p, nota = $n, grupo_bidireccional = $g;
```

`tabla` = `tipo.as_str()` (`similar_a` / `sustituible_por` / `complementa`). Cada arista lleva un `Id<Relacion>::generate()` propio. El grupo es `Id<GrupoBidireccional>::generate()`. `nota` ausente no se setea (queda `NONE`). Bidireccionalidad: `input.resolved_bidireccional()`.

Antes de la `Tx`: construir un `RelacionDraft` / `Relacion::try_new` con ids provisionales para `self_relation`, `prioridad` y `nota`. Si falla, no hay `RELATE`.

**Por qué:** 4.6.1 muestra exactamente ese snippet. `Tx` ya envuelve `BEGIN`/`COMMIT`; si el segundo `RELATE` viola UNIQUE, el primero no queda.

**Alternativa:** dos commits. Se descartó: el Hace pide una transacción. **Alternativa:** ids asignados por el motor. Se descartó: `Id<Relacion>` es ULID de 26 caracteres; generarlos en Rust igual que `MarcaRepo`.

### 3. Mapper UNIQUE: tablas/índices de relación, no el token `relation`

**Elección:** extender `map_surreal_message` para `duplicate_relation` cuando, además del token `relation`:

- el mensaje contiene `uq_similar_a`, `uq_sustituible_por`, `uq_complementa`, `similar_a`, `sustituible_por` o `complementa`; o
- alguna sentencia de la `Tx` contiene `RELATE` y una de esas tablas

`sku` / `slug` siguen teniendo prioridad. Test `tx_unique_relation_is_duplicate_relation` en `tests/tx.rs` (seed dos productos + `RELATE` duplicado en `sustituible_por`, igual que T-04-01).

**Por qué:** los índices `uq_similar_a` etc. no contienen la subcadena `relation`; sin este cambio el mapper devolvería `conflict` genérico y el Listo cuando fallaría.

**Alternativa:** pre-check `SELECT` por `(in, out)`. Se descartó: race y duplica el índice. **Alternativa:** `THROW "duplicate_relation"` en SurrealQL. Se descartó: el índice ya existe; el mapper debe honrarlo.

### 4. `ENFORCED` → `not_found`; `self_relation` no llega al motor

**Elección:** `origen == destino` se rechaza en `Relacion::try_new` (`self_relation`). Destino u origen inexistente: el motor rechaza por `ENFORCED`; el mapper clasifica el mensaje como `not_found` si habla de record/table inexistente, o se añade un token reconocible. Si apply confirma que 2.7.0 no incluye `not_found` en el error, mapear el fallo de `RELATE` + `ENFORCED` desde las sentencias (tabla de relación + producto ausente) a `not_found` — no inventar un `code` nuevo.

**Por qué:** OpenAPI (T-04-05) mapea producto inexistente a 404; el repo debe devolver `not_found`. HU-04-01 nombra `self_relation` en dominio.

**Alternativa:** dejar `ENFORCED` como `unavailable`. Se descartó: T-04-05 no tendría 404.

### 5. Tests en archivo nuevo `tests/relacion_repo.rs`

**Elección:** no mezclar con `tests/relaciones.rs` (esquema T-04-01, ids cortos `prdA`/`prdB` que no son ULID de dominio). Seed con `ProductoRepo::create` (categoría + dos productos, SKUs del perno/arandela) para ids válidos. Verificar aristas con `SELECT` SQL sobre `in`/`out` (sin `list_for_product`). Nombres de test = tabla «Tests previstos» de la spec.

**Por qué:** `Id<Producto>` exige ULID de 26 caracteres; reusar el seed de T-04-01 rompería `Relacion::try_new`. Un archivo nuevo deja verdes los tests de esquema.

**Alternativa:** extender `relaciones.rs`. Se descartó: mezcla esquema crudo con el puerto.

### 6. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-04-03` y rename a `T-04-03` (igual que tickets previos). `status` / `validate` aceptan el ID de proyecto.

## Risks / Trade-offs

- **[El mensaje UNIQUE de Surreal 2.7 no incluye el nombre del índice]** → Mitigación: el mapper también mira las sentencias (`RELATE` + tabla); el test `tx_unique_relation_is_duplicate_relation` es el contrato.
- **[`ENFORCED` no produce un string mapeable a `not_found`]** → Mitigación: el escenario `relation_repo_missing_destino_is_not_found` es obligatorio; si apply ve otro texto, ajustar el mapper sin cambiar el `code` expuesto.
- **[Sintaxis `RELATE in -> tabla:id -> out` en 2.7]** → Mitigación: confirmar en apply; si el parser rechaza el id explícito, usar `RELATE … SET id = …` o la forma que acepte 2.7, conservando ULID sin prefijo.
- **[`in` / `out` reservados al hidratar DTO]** → Mitigación: alias serde (`r#in` / rename) o `SELECT in AS origen, out AS destino`.
- **[Ámbito ~2 h]** → Mitigación: solo `create` + mapper; stubs `Unavailable`; sin outbox. No se divide el ticket.

## Migration Plan

Sin migración de esquema (`0007` ya está). Despliegue = merge a `develop`. Rollback = revertir el adaptador y el mapper; las tablas no cambian.

1. Red: tests de `tests/relacion_repo.rs` (y `tx_unique_relation_is_duplicate_relation`) que no compilan / fallan.
2. Green: `relacion.rs` + reexport + mapper; stubs `Unavailable`.
3. Cierre: § 5 en `vortex-core/`, `openspec validate T-04-03`.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. La sintaxis exacta de `RELATE` con id explícito y el texto de `ENFORCED` se confirman en apply contra SurrealDB 2.7.
