# Design

## Context

En `develop`, `catalog::validate_dir` parsea JSONL y no persiste; `vortex seed` es stub (T-00-07). Los casos de uso `CategoryService` / `BrandService` / `AttributeDefService` y las fachadas `*Gateway` ya existen; `bin/vortex` ya declara `catalog` y `storage` y `serve` abre `<paths.datos>/db` + migraciones. `catalog` MUST NOT depender de `storage` ni `surrealdb` (allowlist). Recableo MVP §7.5.1: seed abre la base con el servidor detenido (sin T-00-22). Ver `proposal.md` para motivación y `specs/` para el contrato.

Constraints: No hace = productos; ~2 h; members fijos; `openspec new change` exige kebab-case (`t-02-42` → rename `T-02-42`).

## Goals / Non-Goals

**Goals:**

- Orquestador de maestros en `catalog` inyectando gateways (tests in-memory = Listo cuando)
- `vortex seed [DIR]` composition root: config → Storage → importar maestros → exit 0
- Fail-closed: issues de maestros o lock de RocksDB → exit ≠ 0, sin `no implementado`

**Non-Goals (diseño):**

- Puerto `get_by_slug` / `get_by_clave` en `domain` (se resuelve con `list_all`)
- Transacción única que abarque las tres entidades (el crate `catalog` no abre `Tx`)
- `vortex seed --check` separado; el validador sigue siendo `validate_dir`
- Extraer un crate `boot` compartido con `serve`

## Decisions

### 1. Orquestador en `catalog` sobre `*Gateway`, persistencia en el binario

**Elección:** `catalog::seed` expone `import_masters(dir, categories, brands, attrs) -> Result<Summary, Vec<SeedIssue>>` (nombres MAY variar) que recibe `&dyn CategoryGateway`, `&dyn BrandGateway` y `&dyn AttributeDefGateway`. Tests en `seed.rs` usan `InMemory*Repo` ya existentes. `bin/vortex` abre `Db`, migra, construye los mismos servicios que `serve` (sin HTTP ni `ProductGateway`) y llama al orquestador.

**Por qué:** el Hace pide casos de uso de `catalog`; la allowlist impide que `catalog` toque SurrealDB. Las fachadas object-safe ya están; no hace falta un puerto nuevo.

**Alternativa:** lógica solo en `bin/vortex`. Se descartó: el Listo cuando (dos corridas) quedaría atado a RocksDB y duplicaría parseo. **Alternativa:** ampliar `CategoryRepo` con `upsert_by_slug`. Se descartó: cambia `domain` fuera de este ticket.

### 2. Parsear solo los tres JSONL de maestros; no llamar `validate_dir`

**Elección:** reutilizar los parsers de `categorias.jsonl` / `marcas.jsonl` / `atributos.jsonl` (ampliar las structs internas para conservar `nombre` y campos de `AtributoDef`). Si hay issues en esos tres archivos, return Err y cero escrituras. Ignorar `productos.jsonl` aunque exista o sea inválido.

**Por qué:** `validate_dir` exige productos válidos; mezclaría el unhappy de T-02-43 en este ticket. El No hace es productos, no «validar el dataset entero».

**Alternativa:** `validate_dir` primero. Se descartó: un precio `"barato"` bloquearía maestros. **Alternativa:** exigir los cuatro JSONL y validar productos sin persistirlos. Se descartó: fixtures de importación de maestros no deberían arrastrar un SKU dummy.

### 3. Upsert por `list_all` + create/update; reactivar; no borrar extras

**Elección:** una pasada `list_all` por tipo (o tras cada create, el mapa local). Match categoría/marca por `slug`, atributo por `clave`. Alta → `create` con slug/clave explícitos. Existente → `rename`/`move_category` o `update`; si `activo == false`, `reactivate`. No se eliminan filas ausentes del JSONL.

**Por qué:** slugs de categoría son únicos **en el archivo** (seed-dataset); en DB la UNIQUE es padre+slug, pero el dataset no repite slug. `list_all` de maestros cabe en memoria. Reactivar hace que re-seed restaure un catálogo usable.

**Alternativa:** dejar inactivos. Se descartó: un maestro dado de baja a mano desaparecería del seed «aplicado» sin duplicar, pero el operador no recuperaría la fila. **Alternativa:** prune de filas no listadas. Se descartó: destructivo y fuera del ticket.

### 4. Categorías en orden topológico

**Elección:** ordenar el JSONL para persistir raíces primero, luego hijas (Kahn o DFS sobre `padre`). Resolver `padre` al `id` ya upsertado. Ciclos: el validador de archivo no los detecta hoy; si `create`/`move_category` devuelve `category_cycle`, se informa como issue de esa línea y se detiene el resto (las filas previas pueden haber quedado).

**Por qué:** `NewCategoria.padre` es `Id`, no slug. Un create de hija antes que el padre fallaría.

**Alternativa:** dos pasadas (crear todas sin padre, luego mover). Se descartó: más writes y peor para UNIQUE (padre, slug).

### 5. CLI: `seed [DIR]` default `seed/`; config como `serve`; sin HTTP

**Elección:** argumento posicional opcional; default `seed` relativo al CWD. Reusar `serve::resolve_config_path` + `Config::from_path`. Tokio runtime, `Db::open` + `migrate`, import, `eprintln` de issues, exit 0/1. No `init_logging` obligatorio (MAY loguear); no bind TCP. Extraer helpers de config/open DB a un módulo `seed.rs` del binario (o `cli.rs`) para no inflar `main`.

**Por qué:** T-00-20 ya resolvió config y lock. El recableo MVP es «servidor detenido». Tests de `serve.rs` ya saben armar TOML + `datos/` temporales.

**Alternativa:** `--dir`. Se descartó: el ticket no pide flags nuevos; un posicional cubre fixtures. **Alternativa:** default al `seed/` del repo vía `CARGO_MANIFEST_DIR`. Se descartó: el binario instalado no tiene el crate; CWD `seed/` es el contrato de HU-02-11.

### 6. Tests CLI con RocksDB; lock = segundo `Db::open`

**Elección:** `vortex-core/bin/vortex/tests/seed.rs` (o ampliar `cli.rs`): temp dir + TOML como `serve.rs`. Happy: `vortex seed DIR` dos veces, reabrir `Db` y `list_all` sin duplicados. Unhappy lock: el test mantiene `Db::open` en el mismo `datos/db` y spawnea el binario. Quitar `seed` del loop de stubs en `cli.rs`.

**Por qué:** el Listo cuando de CLI es observable en proceso real (H-01). In-memory cubre el orquestador; RocksDB cubre modo directo.

**Alternativa:** solo tests de `catalog`. Se descartó: no prueba config/lock/**BREAKING** del stub. **Alternativa:** `kv-mem` en el binario. Se descartó: producción es RocksDB; el lock no existiría.

## Risks / Trade-offs

- **[Escrituras parciales si falla a mitad de persistencia]** → Mitigación: parseo completo primero; fallos de dominio post-validación son raros. T-02-43 hará transacciones por producto; aquí no hay `Tx` en `catalog`.
- **[UNIQUE padre+slug vs match global por slug]** → Mitigación: el JSONL exige slug único en archivo; no se importan dos `pernos` bajo padres distintos.
- **[Test de lock flaky en macOS (H-01)]** → Mitigación: mismo patrón que `storage/tests/open.rs` (`open_after_close` reintenta si el mensaje contiene `lock`); el unhappy del binario acepta cualquier `OpenError` con mensaje de apertura.
- **[ProductionOccupancy en seed]** → Mitigación: seed no da de baja; occupancy no se consulta en create/update/reactivate.
- **[kebab-case del CLI OpenSpec]** → Scaffold `t-02-42` y rename (ya hecho).

## Migration Plan

No hay datos de producción. Rollback: revertir el PR; `vortex seed` vuelve a stub. Quien tenga maestros sembrados los conserva (idempotente). T-02-43 reutilizará `import_masters` y el mismo comando.

## Open Questions

Ninguna que altere specs o tareas. Default `DIR=seed` y reactivar inactivos quedan fijados arriba.
