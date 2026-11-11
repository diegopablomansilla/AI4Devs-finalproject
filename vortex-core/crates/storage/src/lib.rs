//! Crate `storage`: SurrealDB embebido (apertura RocksDB, T-00-08; migraciones T-00-09; harness T-09-02).

mod migrate;
pub mod testing;

pub use migrate::{
    MIGRATION_0000_SQL, MIGRATION_0001_SQL, MigrateError, Migration, MigrationRecord, checksum_hex,
};

use std::path::{Path, PathBuf};

use surrealdb::engine::local::{Db as SurrealEngine, Mem, RocksDb};
use surrealdb::{Error as SurrealError, Surreal};

/// Subdirectorio del motor bajo `<paths.datos>` (T-00-05 / T-00-20).
pub const DB_DIR_NAME: &str = "db";

const NS: &str = "vectra";
const DB: &str = "comercio";

/// Embedded SurrealDB handle. Production opens RocksDB via [`Db::open`]; tests may use [`testing::test_db`].
pub struct Db {
    inner: Surreal<SurrealEngine>,
}

/// Failure to open the embedded database. Maps conceptually to `DomainError::Unavailable` / `unavailable`.
#[derive(Debug, thiserror::Error)]
#[error("no se pudo abrir la base en {path}: {reason}")]
pub struct OpenError {
    path: PathBuf,
    reason: String,
}

impl OpenError {
    fn new(path: impl AsRef<Path>, reason: impl Into<String>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            reason: reason.into(),
        }
    }

    fn io(path: impl AsRef<Path>, err: std::io::Error) -> Self {
        let reason = if err.kind() == std::io::ErrorKind::PermissionDenied {
            format!("permiso denegado: {err}")
        } else {
            format!("error de IO: {err}")
        };
        Self::new(path, reason)
    }

    fn surreal(path: impl AsRef<Path>, err: SurrealError) -> Self {
        Self::new(path, err.to_string())
    }
}

async fn bootstrap_session(
    inner: Surreal<SurrealEngine>,
    path_for_errors: &Path,
) -> Result<Db, OpenError> {
    inner
        .use_ns(NS)
        .use_db(DB)
        .await
        .map_err(|e| OpenError::surreal(path_for_errors, e))?;

    let bootstrap = format!(
        "DEFINE NAMESPACE IF NOT EXISTS {NS}; \
         USE NS {NS}; \
         DEFINE DATABASE IF NOT EXISTS {DB}; \
         USE DB {DB};"
    );
    inner
        .query(bootstrap)
        .await
        .map_err(|e| OpenError::surreal(path_for_errors, e))?
        .check()
        .map_err(|e| OpenError::surreal(path_for_errors, e))?;

    inner
        .use_ns(NS)
        .use_db(DB)
        .await
        .map_err(|e| OpenError::surreal(path_for_errors, e))?;

    Ok(Db { inner })
}

impl Db {
    /// Opens the in-memory engine (`kv-mem`). For integration tests only — see [`testing::test_db`].
    pub(crate) async fn open_mem() -> Result<Self, OpenError> {
        let label = Path::new(":memory:");
        let inner = Surreal::new::<Mem>(())
            .await
            .map_err(|e| OpenError::surreal(label, e))?;
        bootstrap_session(inner, label).await
    }

    /// Opens RocksDB at `path` and selects namespace `vectra` and database `comercio`.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, OpenError> {
        let path = path.as_ref();

        if path.is_file() {
            return Err(OpenError::new(
                path,
                "la ruta es un archivo, no un directorio de base",
            ));
        }

        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| OpenError::io(path, e))?;
        }
        std::fs::create_dir_all(path).map_err(|e| OpenError::io(path, e))?;

        let inner = Surreal::new::<RocksDb>(path)
            .await
            .map_err(|e| OpenError::surreal(path, e))?;

        bootstrap_session(inner, path).await
    }

    /// Runs SurrealQL and returns an error if the engine reports failure (integration tests and harness).
    pub async fn query(&self, sql: &str) -> Result<(), OpenError> {
        let path = Path::new(".");
        let response = self
            .inner
            .query(sql)
            .await
            .map_err(|e| OpenError::surreal(path, e))?;
        response.check().map_err(|e| OpenError::surreal(path, e))?;
        Ok(())
    }

    /// Record count for a table (`0` when the table is unknown or empty).
    pub async fn record_count(&self, table: &str) -> Result<usize, OpenError> {
        let path = Path::new(".");
        let stmt = format!("RETURN array::len((SELECT id FROM {table}));");
        let response = self
            .inner
            .query(&stmt)
            .await
            .map_err(|e| OpenError::surreal(path, e))?;
        let mut checked = match response.check() {
            Ok(r) => r,
            Err(err) => {
                let msg = err.to_string().to_lowercase();
                if msg.contains("not found")
                    || msg.contains("doesn't exist")
                    || msg.contains("does not exist")
                    || msg.contains("no table")
                    || msg.contains("unknown table")
                {
                    return Ok(0);
                }
                return Err(OpenError::surreal(path, err));
            }
        };
        let len: Option<i64> = checked.take(0).map_err(|e| OpenError::surreal(path, e))?;
        Ok(len.unwrap_or(0).max(0) as usize)
    }

    /// Returns the active namespace and database for the session.
    pub async fn session_ns_db(&self) -> Result<(String, String), OpenError> {
        let path = Path::new(".");
        let ns_response = self
            .inner
            .query("RETURN $session.ns;")
            .await
            .map_err(|e| OpenError::surreal(path, e))?;
        let mut ns_response = ns_response
            .check()
            .map_err(|e| OpenError::surreal(path, e))?;
        let ns: String = ns_response
            .take::<Option<String>>(0)
            .map_err(|e| OpenError::surreal(path, e))?
            .ok_or_else(|| OpenError::new(path, "sesión sin namespace"))?;

        let db_response = self
            .inner
            .query("RETURN $session.db;")
            .await
            .map_err(|e| OpenError::surreal(path, e))?;
        let mut db_response = db_response
            .check()
            .map_err(|e| OpenError::surreal(path, e))?;
        let db: String = db_response
            .take::<Option<String>>(0)
            .map_err(|e| OpenError::surreal(path, e))?
            .ok_or_else(|| OpenError::new(path, "sesión sin base"))?;

        Ok((ns, db))
    }
}
