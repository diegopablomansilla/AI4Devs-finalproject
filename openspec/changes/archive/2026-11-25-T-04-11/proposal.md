# Proposal

## Ticket padre

- **Ticket:** [T-04-11](../../../docs/7-tickets.md) — `vortex seed`: relaciones
- **Change:** `T-04-11`
- **HU:** [HU-02-11](../../../docs/6-historias.md#hu-02-11--dataset-semilla) (MVP parcial: importar `relaciones.jsonl`; sin imágenes, sinónimos ni notas)
- **Depende de:** T-04-03 (repositorio: crear relación simple y bidireccional) y T-02-43 (`vortex seed`: productos), en `develop`
- **Hace:** importa `relaciones.jsonl` (origen y destino por SKU, tipo, prioridad, nota, bidireccional), omitiendo las existentes.
- **No hace:** — (el ticket no declara exclusiones; el recorte es el de otros tickets del backlog, abajo).
- **Listo cuando:** el panel de recomendaciones muestra las relaciones del dataset de desarrollo.

## Why

`vortex seed` ya carga maestros y productos, pero ignora `relaciones.jsonl`. Sin esas aristas el dataset de desarrollo no alimenta el panel de T-04-08 y HU-02-11 queda a medias. Ahora, porque T-04-03 y T-02-43 están en `develop`, el grafo `0007` existe y el recableo MVP (§7.5.1) sigue siendo modo directo: servidor detenido, sin socket T-00-22.

## What Changes

- Orquestador de **relaciones** en `catalog`: lee `relaciones.jsonl` (mismo esquema que `seed-dataset`), resuelve `origen`/`destino` por SKU a ids de productos ya importados, y llama al alta de relaciones (`RelationService::create` / T-04-05) con `tipo`, `prioridad`, `nota` y `bidireccional`
- Idempotencia: si la arista ya existe (`duplicate_relation`), se **omite** (no se duplica ni se trata como rechazo)
- Registros inválidos se informan con `archivo`/`linea`/`campo` y **no dejan aristas de esa línea**; el resto de relaciones válidas SÍ se persiste
- Resumen: `importados`, `omitidas`, `rechazados`; `vortex seed` lo imprime; exit 0 solo si no hay rechazados de productos ni de relaciones
- `vortex seed` pasa a importar maestros, productos **y** relaciones; MUST NOT persistir imágenes, sinónimos ni notas
- TDD: happy (dataset de desarrollo: las tres aristas de `seed/relaciones.jsonl` quedan en el grafo; segunda corrida las omite) + unhappy (SKU desconocido / JSON malformado / `self_relation`: solo esa línea se rechaza)
- Listo cuando: tras el seed, `list_for_product` / recomendaciones de `PER-M8X40-ZN` incluyen el complementario `ARA-PERNO-01` (el panel de T-04-08 las muestra; verificación UI a mano)

## Non-goals

- Importar imágenes — **T-02-44** (v2)
- Importar sinónimos — **T-03-29** (v2)
- Importar notas técnicas — **T-05-29** (v2)
- Catálogo del comercio piloto — **T-02-46** (v2)
- Canal Unix socket / seed con `vortex serve` vivo — **T-00-22** (v2)
- UI carga rápida de relaciones — **T-04-10** (v2)
- e2e Playwright del panel — **T-09-11**
- Editar o borrar aristas existentes; actualizar `prioridad`/`nota` en re-seed
- `vortex reindex`, worker Meilisearch, HTTP, `openapi.yaml`, cambios de UI
- Cambiar el esquema JSONL o el contrato de `validate_dir`
- Casos de uso de ABM de relaciones en `catalog` (siguen en `recommend`, DC-05)
- Nuevos tickets: el alcance cabe en ~2 h (mismo patrón que T-02-43)

## Capabilities

### New Capabilities

- (ninguna; se extiende `vortex-seed` y la documentación de `seed-dataset`)

### Modified Capabilities

- `vortex-seed`: `vortex seed` importa `relaciones.jsonl` tras productos, omite aristas existentes y aísla rechazos por línea
- `seed-dataset`: `README.md` documenta que la importación MVP cubre maestros, productos **y** relaciones (sin imágenes/sinónimos/notas)

## Impact

- **Repo:** `vortex-core/crates/catalog/` (`import_relations` + tests in-memory), `vortex-core/bin/vortex/` (composition root de `seed` con `RelationService`/`RelacionRepo`, tests CLI), `seed/README.md`
- **APIs / OpenAPI:** ninguna
- **CLI:** mismo `vortex seed [DIR]`; ahora persiste relaciones; stderr con issues y resumen de relaciones
- **Verificación:** `cargo test -p catalog`, `cargo test -p vortex --test seed`, [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`, `openspec validate T-04-11`
- **Ticket:** cierra T-04-11 (Listo cuando: panel con relaciones del dataset; cubierto en tests por grafo persistido + verificación UI a mano)
- **Ámbito:** ~2 h; reutiliza parsers de `relaciones.jsonl`, `ProductGateway::list` y `RelationService::create`. No se divide el ticket.
