# Design

## Context

`Db::open` (RocksDB) y `Db::migrate()` están en `develop` con tests en `tests/open.rs` y `tests/migrate.rs` que duplican helpers de directorio temporal. T-09-02 formaliza el harness prometido en T-00-08/T-00-09 non-goals. PM-03: RocksDB en producción, `kv-mem` solo en tests.

## Goals / Non-Goals

**Goals:** `test_db()` rápido con migraciones embebidas; `test_db_rocks()` para casos que necesiten disco; test de ejemplo en CI; `Db::open` sin cambio de contrato.

**Non-Goals:** feature flag Cargo `testing`; serde/tempfile nuevos; repositorios de negocio; HTTP.

## Decisions

### 1. `kv-mem` en `[dependencies]` de `storage`

Ambas features en la misma línea `surrealdb`. Otros crates llaman `storage::testing::test_db()` desde sus tests sin depender de un feature opcional del crate.

### 2. Bootstrap compartido

Extraer lógica NS/DB/`DEFINE` de `Db::open` a función interna reutilizada por `open_mem()` (usado solo desde `testing`). `Surreal::new::<Mem>(())` según docs 2.7.

### 3. API `storage::testing`

- `test_db() -> Db`: `open_mem` + `migrate().expect(...)` (helper de test; falla con panic si el harness está roto).
- `test_db_rocks() -> TestRocksDb`: path único bajo `std::env::temp_dir()`, `Db::open` + `migrate`, `Drop` cierra y borra el directorio.
- `TestRocksDb` expone `db: Db` y guarda el path.

### 4. Consultas desde tests

`Db::query(&str) -> Result<(), OpenError>` público acotado a SurrealQL con `.check()` — usado por `harness.rs` para tablas inexistentes y aislamiento.

### 5. Tests `harness.rs`

Happy: migración 0000, NS/DB, aislamiento entre dos `test_db()`, rocks migrado. Unhappy: `categoria` no existe; archivo como path sigue fallando en `Db::open`.

## Risks

- **Compile time** al enlazar `kv-mem` y `kv-rocksdb` → aceptable; CI ya compila ambos.
- **Lock RocksDB** en `test_db_rocks` → un solo handle por path; tests serializados por proceso como `open.rs`.
