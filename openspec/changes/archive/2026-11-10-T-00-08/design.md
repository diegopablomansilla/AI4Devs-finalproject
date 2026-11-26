# Design

## Context

En `develop`, `crates/storage` es un lib vacío (`src/lib.rs` placeholder, sin `[dependencies]`). `surrealdb = "=2.7.0"` ya está en `[workspace.dependencies]` pero **prohibido** en cualquier lib que no sea `domain` — y `domain` tampoco puede usarlo (`check-workspace.sh` + `check-domain-deps.sh`). `bin/vortex` no abre Storage (`serve` es stub, T-00-10 / T-00-20). `DomainError::Unavailable` no transporta un mensaje de motivo. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: no migraciones (T-00-09); no `kv-mem` / harness `test_db` (T-09-02); no cablear la CLI; ~2 h; MVP.

## Goals / Non-Goals

**Goals:**

- API `storage::Db::open(path)` con RocksDB embebido, `NS vectra` / `DB comercio`
- `OpenError` tipado con motivo en `Display`
- Allowlist de `storage` en `check-workspace.sh`; tests RocksDB en directorio temporal (happy + permiso/corrupción)

**Non-Goals (diseño):**

- Crate `tempfile` u otras deps fuera de `[workspace.dependencies]`
- Dependencia de `storage` → `domain` (el motivo vive en `OpenError`; el mapeo a `DomainError::Unavailable` es T-00-20 / T-00-11)
- Exponer el tipo `Surreal<…>` en la API pública más de lo necesario para T-00-09
- Segundo `open` concurrente sobre el mismo path (lock H-01 → T-00-22)
- `kv-mem`, `DEFINE` de tablas de negocio, `BEGIN`/`COMMIT`

## Decisions

### 1. `path` es el directorio RocksDB, no la raíz `<datos>`

**Elección:** `Db::open(path)` recibe el directorio del motor. Constante `storage::DB_DIR_NAME = "db"` documenta la convención `<paths.datos>/db` (T-00-05) para T-00-20. `storage` MUST NOT leer TOML ni `VORTEX_CONFIG`.

**Por qué:** el Listo cuando habla de «directorio temporal», no de un árbol de instalación. Inyectar el path hace testable el crate sin `Config`. El join `<datos>/db` es del composition root.

**Alternativa:** `open(datos: Path)` y concatenar `db` adentro. Se descartó: mezcla config con el motor y obliga a crear `<datos>` en cada test. **Alternativa:** leer `Config` en `storage`. Se descartó: viola PC-01 (storage no es el composition root) y el non-goal de T-00-05.

### 2. Motor `RocksDb` (`kv-rocksdb`); `tokio` solo en tests

**Elección:** `surrealdb = { workspace = true, default-features = false, features = ["kv-rocksdb"] }`. `Db::open` es `async` y usa `Surreal::new::<RocksDb>(path)` + `use_ns("vectra").use_db("comercio")`. Si `use_ns`/`use_db` no crean el namespace/base en 2.7, se ejecutan `DEFINE NAMESPACE IF NOT EXISTS vectra` y `DEFINE DATABASE IF NOT EXISTS comercio` como parte de `open` (no son archivos de migración). `[dev-dependencies] tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }` para `#[tokio::test]`. El crate no declara `tokio` en `[dependencies]`: el runtime lo aporta el caller.

**Por qué:** PM-03 y el Listo cuando (corrupción/permisos) exigen disco. `default-features = false` evita el cliente WS/HTTP que `storage` no usa. Pin `=2.7.0` ya cumple «versión exacta».

**Alternativa:** `kv-mem` en los tests happy. Se pospone a T-09-02: no cubre el Listo cuando de disco. **Alternativa:** `tokio` en `[dependencies]`. Se descartó: `open` no arranca el runtime; T-00-10 ya tendrá Tokio en el binario.

### 3. `OpenError` en `storage`, sin tocar `DomainError`

**Elección:** `OpenError` con `thiserror`, al menos una variante que conserva `path` y la fuente (`io` / error de SurrealDB). `Display` incluye path y motivo. Implementa `std::error::Error`. No hay `From<OpenError> for DomainError` ni dependencia `storage` → `domain`. Comentario (o método `#[allow(dead_code)]` no) documenta el mapeo futuro a `Unavailable` / `unavailable`.

**Por qué:** T-00-03 no tiene campo de detalle en `Unavailable`; meter el motivo ahí es otro ticket. El Listo cuando exige el motivo en el error que devuelve `open`. `check-domain-deps.sh` sigue verde porque `domain` no ve `surrealdb`.

**Alternativa:** devolver `DomainError::Unavailable`. Se descartó: pierde el motivo. **Alternativa:** ampliar `Unavailable { code, message }`. Se descartó: cambia el contrato de T-00-03 y T-00-11.

### 4. Allowlist `storage` en `check-workspace.sh`; fixture unhappy = `axum`

**Elección:** extraer `storage_deps_allowed` análogo a `domain_deps_allowed` / `vortex_deps_allowed`. Acepta `{surrealdb, thiserror}` en `[dependencies]` (`workspace = true`; `surrealdb` MAY `default-features = false` y MUST `features` que incluyan `kv-rocksdb`) y `tokio` en `[dev-dependencies]`. El loop de libs trata `storage` como caso especial junto a `domain`. Fixture unhappy: `storage` declara `axum` (próxima crate del stack, T-00-10, y sigue fuera de `storage`). El caso `api` con deps se conserva.

**Por qué:** el mismo patrón que T-00-04…T-00-07. `axum` no se habilita en ningún lib todavía; falla con needle `storage`.

**Alternativa:** permitir cualquier crate en `storage`. Se descartó: rompería PC-01 (storage no es un cajón de SDKs). **Alternativa:** `tempfile` en allowlist. Se descartó: no está en `[workspace.dependencies]`; los tests usan `std::env::temp_dir` + id único y `Drop` para limpiar.

### 5. Tests de integración `tests/open.rs`; TDD red → green

**Elección:** `vortex-core/crates/storage/tests/open.rs`. Happy: directorio único bajo temp, `open`, comprobar NS/DB con una consulta de sesión (`RETURN $session` o equivalente 2.7); cerrar y reabrir el mismo path. Unhappy: (1) `path` es un archivo, o un directorio con basura de RocksDB (`CURRENT` / manifiesto inválido); (2) `#[cfg(unix)]` `chmod 0o000` sobre el directorio (restaurar en `Drop`; no corre si el proceso es root). Verificación: [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) en `vortex-core/`. Para leer NS/DB, `Db` expone un helper de test o un método de consulta mínimo (`query` interno / `pub(crate)` no alcanza desde `tests/`); un método `async fn session_ns_db(&self) -> Result<(String, String), OpenError>` (o query pública acotada) es suficiente y T-00-09 lo reutilizará.

**Por qué:** el Listo cuando es observable contra el motor real, no contra un mock. Tests de integración en `tests/` es la convención que T-09-02 formaliza.

**Alternativa:** unitarios con un fake. Se descartó: no cubre RocksDB ni el motivo del motor. **Alternativa:** abrir desde `bin/vortex`. Se descartó: `serve` sigue stub.

## Risks / Trade-offs

- **[Compilación nativa de `librocksdb-sys` lenta / necesita C++ y CMake en CI macos-14]** → Mitigación: el runner de T-09-01 ya es `macos-14` con Xcode; cache de `vortex-core/target` por `Cargo.lock`. Si el job `test` no encuentra CMake, documentar el paquete en el PR; no añadir job nuevo.
- **[Surreal 2.7: `use_ns`/`use_db` vs `DEFINE NAMESPACE`]** → Mitigación: implementar `use_*` primero; si el test de NS/DB falla, añadir los `DEFINE … IF NOT EXISTS` en `open` (sigue fuera de T-00-09: no hay `.surql` ni tabla `migracion`).
- **[Unhappy de permisos en macOS: chmod 000 a veces sigue permitiendo al dueño]** → Mitigación: si `open` no falla, usar un path bajo un directorio padre sin bit de ejecución (`search`), o `path` cuyo padre es 0o000. Si el entorno no puede denegar (root), `#[ignore]` documentado no: preferir path-es-archivo como corrupto y un subdir no escribible. CI macos-14 no es root.
- **[Lock exclusivo: un segundo `open` concurrente falla]** → Mitigación: los tests happy cierran la primera instancia antes de reabrir. No se afirma el lock (T-00-22).
- **[`openspec new change` exige kebab-case]** → Mitigación: scaffold `t-00-08` y rename a `T-00-08` (igual que T-00-05…T-00-07).

## Migration Plan

1. Red: `tests/open.rs` no compila o falla; fixtures de allowlist de `storage` aún no existen.
2. Green: allowlist + `Cargo.toml` de `storage` + `Db::open` + `OpenError` + `Cargo.lock`.
3. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` verde, `openspec validate T-00-08`.
4. Rollback: revertir crate `storage` a placeholder, allowlist y fixtures. Sin datos de usuario ni API.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. T-00-09 decide el handle concreto para ejecutar `.surql` sobre `Db`. T-00-20 decide el join `Path::new(&config.paths.datos).join(storage::DB_DIR_NAME)`.
