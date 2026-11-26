# Design

## Context

En `develop`, `domain` expone `TextSearch` (RPITIT, nueve métodos, `SearchDocument` solo con `id`, `IndexSettings` vacío) y `DomainError` (`Unavailable` / `Timeout` / `InvalidInput`). El crate `search` es un placeholder más el probe TCP `#[ignore = "meilisearch"]` de T-09-05. `meilisearch-sdk = "=0.33.0"` está pinneado en el workspace pero **ningún crate lo declara**. `scripts/check-workspace.sh` falla si `search` tiene `[dependencies]` (allowlist solo para `domain`, `storage`, `api`, `supervisor`, `catalog`). El job CI `meilisearch` ya corre `cargo test --workspace -- --include-ignored meilisearch` contra `127.0.0.1:7700` sin master key. Ver `proposal.md` para motivación y `specs/vortex-search/spec.md` / `specs/cargo-workspace/spec.md` para el contrato.

Constraints: sin 4.11.2 (T-03-05); sin documento 4.11.1 (T-03-06); sin wiring en `bin/vortex` (T-00-27); MVP del curso; ~2 h. CLI OpenSpec kebab-case (`t-03-04` → carpeta `T-03-04`).

## Goals / Non-Goals

**Goals:**

- `MeiliTextSearch` (nombre tentativo) en `search` que implementa `TextSearch`
- Espera de *tasks* con *timeout* (defecto 5 s)
- Mapper de errores SDK → `DomainError`
- Allowlist de `search` en `check-workspace.sh` + fixtures
- Tests ignore contra el motor real (alta / consulta / borrado / health) y unhappy de URL muerta + mapper

**Non-Goals (diseño):**

- `async-trait` / `dyn TextSearch` (el puerto es RPITIT)
- Traducir `SearchFilters` / `SearchOrden` a sintaxis Meilisearch
- Serializar campos 4.11.1 en `SearchDocument` (DTO local `{ id }` basta)
- Relajar `check-domain-deps.sh` ni declarar `reqwest` directo en `search`
- Cambiar el probe TCP de T-09-05 ni el workflow CI

## Decisions

### 1. El adaptador vive en `search`, no en `domain` ni en `bin/vortex`

**Elección:** `vortex-core/crates/search/src/meilisearch.rs` (o `lib.rs` si cabe) + `impl TextSearch for MeiliTextSearch`. Tests de integración en `crates/search/tests/meilisearch_adapter.rs`. Constructor: URL + `Option<String>` api_key + `Duration` de *timeout* (defecto 5 s, alineado al *timeout* de gateway T-00-12).

**Por qué:** [3.3](../../../docs/3-componentes.md#33-puertos-y-adaptadores) y [3.4.4](../../../docs/3-componentes.md#344-search-service) ponen el adaptador en `search`. T-00-02 prohíbe el SDK en `domain`. T-00-27 cableará el composition root.

**Alternativa:** cliente en `bin/vortex`. Se descartó: el worker (T-03-07) y el caso de uso (T-03-12) vivirían en crates que no deben conocer la URL. **Alternativa:** crate nuevo. Se descartó: el member `search` ya existe para esto.

### 2. Abrir allowlist de `search` (mismo molde que `catalog` / `storage`)

**Elección:** `search_deps_allowed()` en `scripts/check-workspace.sh`:

| Sección | Crates |
|---|---|
| `[dependencies]` | `domain` (path), `meilisearch-sdk`, `tokio`, `serde`, `serde_json` (`workspace = true`; `serde` MAY `derive`; `tokio` MAY `time`) |
| `[dev-dependencies]` | solo `tokio` (`macros`, `rt-multi-thread`) |

Fixtures en `scripts/fixtures/workspace/run-script-tests.sh`: happy allowlist + unhappy `surrealdb`/`reqwest`/`clap`. El escenario «lib vacío» pasa a usar `recommend` (ya no `search`).

**Por qué:** sin esto `check-workspace.sh` bloquea el ticket. `reqwest` llega **transitivo** por el SDK; no se declara para no ampliar la allowlist ni duplicar el pin.

**Alternativa:** exceptuar `search` de la regla «sin dependencies» sin allowlist. Se descartó: el resto de crates sí tienen allowlist explícita.

### 3. DTO `{ id }` en el adaptador, no `Serialize` en `domain::SearchDocument`

**Elección:** struct privado `MeiliDocument { id: String }` (`serde`). Alta: `id = document.id.to_string()` (ULID sin `producto:`). Hits: parsear `id` del JSON del motor a `Id<Producto>` → `SearchHit`. Primary key Meilisearch: `"id"`.

**Por qué:** `SearchDocument` no deriva `Serialize` (T-03-03 evitó `serde_json` en `[dependencies]` de `domain`). T-03-06 añadirá campos; el DTO crece ahí o se pasa a serializar el tipo de dominio. Tocar `domain` en este ticket duplicaría alcance.

**Alternativa:** `#[derive(Serialize)]` en `SearchDocument`. Se descartó: cambia el crate del puerto sin ganancia ahora.

### 4. `wait_for_task` del SDK envuelto en `tokio::time::timeout`

**Elección:** tras cada operación de escritura, `client.wait_for_task(...)` dentro de `tokio::time::timeout(self.task_timeout, ...)`. Si el *timeout* de Tokio gana → `DomainError::timeout()`. Defecto 5 s; tests del mapper no esperan 5 s.

**Por qué:** el ticket pide espera de *tasks* con *timeout*. El SDK 0.33 ya espera *tasks*; el wrapper garantiza `Timeout` de dominio aunque el SDK bloquee.

**Alternativa:** solo el *timeout* interno del SDK. Se descartó: el mapeo a `DomainError::Timeout` quedaría implícito. **Alternativa:** 2 s (T-03-13). Se descartó: esa cifra es del gateway HTTP; indexar un lote puede ser más lento.

### 5. Mapper de errores, no *mock* del SDK

**Elección:** función `pub(crate) fn map_meili_error(err: &meilisearch_sdk::errors::Error) -> DomainError` (firma exacta al aplicar). Clasificación:

| Señal | `DomainError` |
|---|---|
| conexión, `reqwest` connect, I/O, HTTP 5xx / unreachable | `unavailable` |
| *timeout* (SDK o wrapper Tokio) | `timeout` |
| HTTP 400 / error de documento / payload inválido | `invalid_input` |
| resto | `unavailable` (motor no usable; no se inventan `code`) |

Tests unitarios construyen o simulan errores del SDK (o el wrapper de *timeout*) sin red. El Listo cuando «Meilisearch detenido» es un cliente a `http://127.0.0.1:1` (o puerto cerrado): corre **sin** `#[ignore]`, no mata el sidecar de CI.

**Por qué:** CI T-09-05 **siempre** tiene el motor arriba; un test que exige el proceso caído fallaría el job. URL muerta = «detenido» observable y estable. Timeout / invalid_input contra motor real son flaky o imposibles con `SearchDocument` siempre válido.

**Alternativa:** `#[ignore]` que apaga Meilisearch. Se descartó: rompe el job paralelo. **Alternativa:** *wiremock*. Se descartó: otra crate y ~2 h.

### 6. Índice de prueba efímero; `apply_settings` no-op; `facet_search` delega

**Elección:** tests de integración `create_index` + uid `t0304-<pid>-<n>` (nunca `productos`), `upsert` / `search` / `delete`, `delete_index` al final. `apply_settings(&IndexSettings)` → `Ok(())` sin HTTP mientras el tipo sea unidad. `facet_search` llama a `search` (sin `facetas`). `swap_indexes` usa `validate_index_swap` y luego el SDK.

**Por qué:** T-03-05 es dueño de 4.11.2; T-03-18 de facetas. Implementar los nueve métodos es obligatorio para `impl TextSearch`; el comportamiento extra se deja fino.

**Alternativa:** no implementar `create_index` / `swap`. Se descartó: el *trait* no compilaba. **Alternativa:** auto-create de Meilisearch al primer `add_documents`. Se usa igual, pero `create_index` explícito aísla tests.

### 7. Tests `#[ignore = "meilisearch"]` + probe T-09-05 intacto

**Elección:** nombres de la tabla en `specs/vortex-search/spec.md`. Happy path ignore; unhappy URL muerta y mapper **no** ignore. `MEILISEARCH_URL` igual que el probe. No se toca `.github/workflows/ci.yml`.

**Por qué:** el filtro `meilisearch` del job ya recogerá `meilisearch_adapter_*`. `cargo test --workspace` sin ignore sigue verde en laptops sin sidecar.

## Risks / Trade-offs

- [`SearchDocument` solo tiene `id`; `search` por ULID depende de que el campo sea buscable por defecto] → Mitigation: Meilisearch indexa todos los atributos hasta T-03-05; si un default cambiara, el test usa `q` vacía + `limit` y filtra el hit por `id`.
- [Timeout de 5 s en un test ignore si el motor cuelga] → Mitigation: CI mata el job; local el *timeout* corta. No se bajan a 50 ms en producción.
- [Allowlist de `search` desvía el escenario «lib vacío» a `recommend`] → Mitigation: fixtures nuevas; `recommend`/`ai`/… siguen sin deps.
- [T-03-06 tendrá que extender el DTO] → Mitigation: un struct privado; el *trait* no cambia.

## Migration Plan

Cambio aditivo. Sin datos que migrar. Rollback: revertir el crate `search` a placeholder + allowlist. `cargo test --workspace` (sin ignore) y `./scripts/check-workspace.sh` bastan.

## Open Questions

Ninguna que altere spec, enfoque o tasks. El nombre `MeiliTextSearch` vs `MeilisearchAdapter` se decide en apply (público del crate `search`).
