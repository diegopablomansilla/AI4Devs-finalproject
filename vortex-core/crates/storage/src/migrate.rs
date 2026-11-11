//! Embedded `.surql` migrations and checksum registry (T-00-09).

use std::collections::HashMap;

use sha2::{Digest, Sha256};
use surrealdb::Error as SurrealError;

use crate::{Db, OpenError};

/// SQL for migration `0000` (exported for tests).
pub const MIGRATION_0000_SQL: &str = include_str!("../migrations/0000_migracion.surql");

/// SQL for migration `0001` (exported for tests).
pub const MIGRATION_0001_SQL: &str = include_str!("../migrations/0001_outbox.surql");

/// One versioned migration script.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: i32,
    pub name: &'static str,
    pub sql: &'static str,
}

const EMBEDDED: &[Migration] = &[
    Migration {
        version: 0,
        name: "migracion",
        sql: MIGRATION_0000_SQL,
    },
    Migration {
        version: 1,
        name: "outbox",
        sql: MIGRATION_0001_SQL,
    },
];

/// Row stored in `migracion` (for tests and diagnostics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationRecord {
    pub version: i32,
    pub nombre: String,
    pub checksum: String,
}

/// Failure while applying migrations. Display MUST name the version on checksum drift.
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    #[error(
        "migración {version:04}: checksum distinto al registrado (registrado {stored}, actual {computed})"
    )]
    ChecksumMismatch {
        version: i32,
        stored: String,
        computed: String,
    },
    #[error("migración {version:04} ({name}): {reason}")]
    Failed {
        version: i32,
        name: &'static str,
        reason: String,
    },
}

impl Db {
    /// Applies all embedded migrations in version order.
    pub async fn migrate(&self) -> Result<(), MigrateError> {
        self.apply_migrations(EMBEDDED).await
    }

    /// Applies the given migrations (used by tests and future embed expansion).
    pub async fn apply_migrations(&self, migrations: &[Migration]) -> Result<(), MigrateError> {
        let mut sorted: Vec<Migration> = migrations.to_vec();
        sorted.sort_by_key(|m| m.version);

        let applied = self.load_applied_checksums().await?;

        for migration in &sorted {
            let computed = checksum_hex(migration.sql);
            if let Some(stored) = applied.get(&migration.version) {
                if stored == &computed {
                    continue;
                }
                return Err(MigrateError::ChecksumMismatch {
                    version: migration.version,
                    stored: stored.clone(),
                    computed,
                });
            }

            self.run_sql(migration)
                .await
                .map_err(|reason| MigrateError::Failed {
                    version: migration.version,
                    name: migration.name,
                    reason,
                })?;

            self.insert_migration_row(migration, &computed)
                .await
                .map_err(|reason| MigrateError::Failed {
                    version: migration.version,
                    name: migration.name,
                    reason,
                })?;
        }

        Ok(())
    }

    /// Returns applied migration rows (`version`, `nombre`, `checksum`).
    pub async fn list_migrations(&self) -> Result<Vec<MigrationRecord>, MigrateError> {
        let versions = self.query_versions().await?;
        let mut out = Vec::with_capacity(versions.len());
        for version in versions {
            let nombre = self
                .query_string_value(
                    "SELECT VALUE nombre FROM ONLY migracion WHERE version = $version LIMIT 1;",
                    version,
                )
                .await?;
            let checksum = self
                .query_string_value(
                    "SELECT VALUE checksum FROM ONLY migracion WHERE version = $version LIMIT 1;",
                    version,
                )
                .await?;
            out.push(MigrationRecord {
                version,
                nombre,
                checksum,
            });
        }
        Ok(out)
    }

    async fn query_versions(&self) -> Result<Vec<i32>, MigrateError> {
        let path = std::path::Path::new(".");
        let response = self
            .inner
            .query("SELECT VALUE version FROM migracion ORDER BY version ASC;")
            .await
            .map_err(map_list_err)?;
        let mut checked = response.check().map_err(|e| MigrateError::Failed {
            version: -1,
            name: "migracion",
            reason: OpenError::surreal(path, e).to_string(),
        })?;
        let values: Vec<i64> = checked.take(0).map_err(|e| MigrateError::Failed {
            version: -1,
            name: "migracion",
            reason: e.to_string(),
        })?;
        Ok(values.into_iter().map(|v| v as i32).collect())
    }

    async fn query_string_value(&self, stmt: &str, version: i32) -> Result<String, MigrateError> {
        let path = std::path::Path::new(".");
        let response = self
            .inner
            .query(stmt)
            .bind(("version", version))
            .await
            .map_err(map_list_err)?;
        let mut checked = response.check().map_err(|e| MigrateError::Failed {
            version: -1,
            name: "migracion",
            reason: OpenError::surreal(path, e).to_string(),
        })?;
        let value: Option<String> = checked.take(0).map_err(|e| MigrateError::Failed {
            version: -1,
            name: "migracion",
            reason: e.to_string(),
        })?;
        value.ok_or_else(|| MigrateError::Failed {
            version,
            name: "migracion",
            reason: "fila de migración incompleta".into(),
        })
    }

    async fn load_applied_checksums(&self) -> Result<HashMap<i32, String>, MigrateError> {
        let rows = match self.list_migrations().await {
            Ok(rows) => rows,
            Err(MigrateError::Failed { .. }) => return Ok(HashMap::new()),
            Err(e) => return Err(e),
        };
        Ok(rows.into_iter().map(|r| (r.version, r.checksum)).collect())
    }

    async fn run_sql(&self, migration: &Migration) -> Result<(), String> {
        let path = std::path::Path::new(".");
        let response = self
            .inner
            .query(migration.sql)
            .await
            .map_err(|e| OpenError::surreal(path, e).to_string())?;
        response
            .check()
            .map_err(|e| OpenError::surreal(path, e).to_string())?;
        Ok(())
    }

    async fn insert_migration_row(
        &self,
        migration: &Migration,
        checksum: &str,
    ) -> Result<(), String> {
        let path = std::path::Path::new(".");
        let stmt =
            "CREATE migracion CONTENT { version: $version, nombre: $nombre, checksum: $checksum };";
        let response = self
            .inner
            .query(stmt)
            .bind(("version", migration.version))
            .bind(("nombre", migration.name))
            .bind(("checksum", checksum.to_string()))
            .await
            .map_err(|e| OpenError::surreal(path, e).to_string())?;
        response
            .check()
            .map_err(|e| OpenError::surreal(path, e).to_string())?;
        Ok(())
    }
}

/// SHA-256 hex (lowercase) of migration SQL (for tests).
pub fn checksum_hex(sql: &str) -> String {
    let digest = Sha256::digest(sql.as_bytes());
    format!("{digest:x}")
}

fn map_list_err(err: SurrealError) -> MigrateError {
    MigrateError::Failed {
        version: -1,
        name: "migracion",
        reason: err.to_string(),
    }
}
