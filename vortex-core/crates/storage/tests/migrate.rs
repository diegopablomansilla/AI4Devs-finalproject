//! Integration tests for schema migrations (T-00-09).

use std::path::PathBuf;

use storage::{Db, MIGRATION_0000_SQL, MIGRATION_0001_SQL, Migration, checksum_hex};

fn unique_db_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vortex-migrate-{prefix}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

async fn open_db(path: &PathBuf) -> Db {
    Db::open(path).await.expect("open db for migrate tests")
}

#[tokio::test]
async fn migrate_first_install_records_0000() {
    let path = unique_db_dir("first");
    let db = open_db(&path).await;
    db.migrate().await.expect("first migrate");

    let rows = db.list_migrations().await.expect("list migrations");
    assert_eq!(rows.len(), 2, "expected migrations 0000 and 0001");
    let row0 = rows.iter().find(|r| r.version == 0).expect("version 0");
    assert_eq!(row0.nombre, "migracion");
    assert_eq!(row0.checksum, checksum_hex(MIGRATION_0000_SQL));
    let row1 = rows.iter().find(|r| r.version == 1).expect("version 1");
    assert_eq!(row1.nombre, "outbox");
    assert_eq!(row1.checksum, checksum_hex(MIGRATION_0001_SQL));

    drop(db);
    let _ = std::fs::remove_dir_all(&path);
}

#[tokio::test]
async fn migrate_rerun_is_noop() {
    let path = unique_db_dir("rerun");
    let db = open_db(&path).await;
    db.migrate().await.expect("first migrate");
    db.migrate().await.expect("second migrate");

    let rows = db.list_migrations().await.expect("list migrations");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|r| r.version == 0));
    assert!(rows.iter().any(|r| r.version == 1));

    drop(db);
    let _ = std::fs::remove_dir_all(&path);
}

#[tokio::test]
async fn migrate_rejects_altered_checksum() {
    let path = unique_db_dir("altered");
    let db = open_db(&path).await;
    db.migrate().await.expect("baseline migrate");

    let tampered = Migration {
        version: 0,
        name: "migracion",
        sql: "-- altered\nDEFINE TABLE IF NOT EXISTS tampered SCHEMAFULL;",
    };

    let err = db
        .apply_migrations(&[tampered])
        .await
        .expect_err("checksum drift must fail");
    let msg = err.to_string();
    assert!(
        msg.contains('0') || msg.contains("0000"),
        "error must name version, got: {msg}"
    );

    drop(db);
    let _ = std::fs::remove_dir_all(&path);
}
