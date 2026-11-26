//! Test harness: in-memory and temporary RocksDB with migrations applied (T-09-02).

use std::ops::Deref;
use std::path::PathBuf;

use crate::Db;

fn unique_rocks_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "vortex-storage-harness-{prefix}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Opens an in-memory SurrealDB with namespace `vectra`, database `comercio`, and embedded migrations applied.
pub async fn test_db() -> Db {
    let db = Db::open_mem()
        .await
        .expect("test_db: open in-memory engine");
    db.migrate()
        .await
        .expect("test_db: apply embedded migrations");
    db
}

/// RocksDB in a unique temporary directory; migrations applied. Directory removed on drop.
pub struct TestRocksDb {
    path: PathBuf,
    inner: Option<Db>,
}

impl TestRocksDb {
    /// Path to the RocksDB data directory (exists while this value is alive).
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Handle to the opened database.
    pub fn db(&self) -> &Db {
        self.inner.as_ref().expect("database handle")
    }
}

impl Deref for TestRocksDb {
    type Target = Db;

    fn deref(&self) -> &Self::Target {
        self.db()
    }
}

impl Drop for TestRocksDb {
    fn drop(&mut self) {
        self.inner = None;
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Opens RocksDB at a fresh temp path with migrations applied.
pub async fn test_db_rocks() -> TestRocksDb {
    let path = unique_rocks_dir("rocks");
    let db = Db::open(&path).await.expect("test_db_rocks: open rocksdb");
    db.migrate()
        .await
        .expect("test_db_rocks: apply embedded migrations");
    TestRocksDb {
        path,
        inner: Some(db),
    }
}
