# Proposal

## Ticket padre

- **Ticket:** [T-02-43](../../../docs/7-tickets.md) — `vortex seed`: productos y stock inicial
- **Change:** `T-02-43`
- **HU:** [HU-02-11](../../../docs/6-historias.md#hu-02-11--dataset-semilla) (MVP parcial: sin imágenes ni sinónimos; dataset de desarrollo ~200 SKUs)
- **Depende de:** T-02-42 (`vortex seed` maestros) y T-02-20 (casos de uso de producto), en `develop`
- **Hace:** cada producto en su propia transacción con su movimiento `inicial`; los registros inválidos se informan (archivo, línea, campo) y no dejan datos parciales; resumen final de importados y rechazados.
- **No hace:** imágenes, relaciones, sinónimos ni notas (tickets propios, porque dependen de tablas que se crean más adelante).
- **Listo cuando:** el dataset de desarrollo se importa completo y un registro inválido agregado a propósito se rechaza solo.

## Why

`vortex seed` ya carga categorías, marcas y `atributo_def`, pero ignora `productos.jsonl`. Sin productos ni movimiento `inicial` por SKU, HU-02-11 no deja un catálogo usable y T-04-11 / búsqueda no tienen fichas que referenciar. Ahora, porque T-02-42 y T-02-20 están en `develop` y el recableo MVP (§7.5.1) sigue siendo modo directo: servidor detenido, sin socket T-00-22.

## What Changes

- Orquestador de **productos** en `catalog`: lee `productos.jsonl` (mismo esquema que `seed-dataset`), resuelve `categoria`/`marca`/`atributos[].clave` a ids de maestros ya importados, y llama a `ProductGateway::create` (casos de uso de T-02-20) con `stock_inicial`
- Una transacción por producto: el alta de `ProductRepo::create` (T-02-18) ya persiste ficha + movimiento `inicial` (si `stock_inicial > 0`) en la misma `Tx`; el seed MUST NOT agrupar SKUs en una sola transacción ni usar el caso de uso de stock (que rechaza `tipo` `inicial`)
- Registros inválidos se informan con `archivo`/`linea`/`campo` y **no dejan datos parciales de ese SKU**; el resto de productos válidos SÍ se persiste (**se rechaza solo**, Listo cuando)
- Resumen final: conteo de importados y rechazados; `vortex seed` imprime ese resumen; exit 0 solo si no hay rechazados
- Idempotencia por SKU: si el SKU ya existe, no se crea otro producto ni otro movimiento `inicial` (se cuenta como importado)
- `vortex seed` pasa a importar maestros **y** productos; MUST NOT persistir imágenes, relaciones, sinónimos ni notas
- TDD: happy (dataset de desarrollo completo; movimiento `inicial`; segunda corrida sin duplicar SKUs ni ledger) + unhappy (precio no numérico u otro issue: solo ese registro se rechaza; el resto queda)

## Non-goals

- Importar imágenes — **T-02-44** (v2)
- Importar relaciones — **T-04-11**
- Importar sinónimos — **T-03-29** (v2)
- Importar notas técnicas — **T-05-29** (v2)
- Catálogo del comercio piloto — **T-02-46** (v2)
- Canal Unix socket / seed con `vortex serve` vivo — **T-00-22** (v2)
- Actualizar ficha de un SKU ya existente (precio, nombre, atributos) o reactivar inactivos — fuera del Hace; la segunda corrida solo evita duplicar
- `vortex reindex`, worker Meilisearch, HTTP, `openapi.yaml`, UI
- Cambiar el esquema JSONL o el contrato de `validate_dir`
- Crates nuevas o puertos `get_by_sku` en `domain`
- `vortex verify` / conciliación — **T-02-45** (v2)

## Capabilities

### New Capabilities

- (ninguna; se extiende `vortex-seed`)

### Modified Capabilities

- `vortex-seed`: `vortex seed` importa productos con movimiento `inicial` por transacción, aísla rechazos, resume importados/rechazados e idempotencia por SKU
- `seed-dataset`: `README.md` documenta que la importación MVP cubre maestros **y** productos (sin imágenes/relaciones/sinónimos/notas)

## Impact

- **Repo:** `vortex-core/crates/catalog/` (`import_products` o equivalente + tests in-memory), `vortex-core/bin/vortex/` (composition root de `seed`, tests CLI), `seed/README.md`
- **APIs / OpenAPI:** ninguna
- **CLI:** mismo `vortex seed [DIR]`; ahora persiste productos; stderr con issues y resumen
- **Verificación:** `cargo test -p catalog`, `cargo test -p vortex --test seed`, [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`, `openspec validate T-02-43`
- **Ticket:** cierra T-02-43 (Listo cuando: dataset de desarrollo completo + registro inválido aislado)
- **Ámbito:** ~2 h; reutiliza `import_masters`, parsers de `productos.jsonl` y `ProductGateway::create`. No se divide el ticket.
