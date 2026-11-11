//! Integration tests for `storage::testing` harness (T-09-02).

use std::io::Write as _;
use std::path::PathBuf;

use storage::testing::{test_db, test_db_rocks};
use storage::{Db, MIGRATION_0000_SQL, MIGRATION_0001_SQL, MIGRATION_0002_SQL, checksum_hex};

fn unique_file_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vortex-harness-{prefix}-{}", std::process::id()))
}

#[tokio::test]
async fn test_db_applies_embedded_migrations() {
    let db = test_db().await;
    let rows = db.list_migrations().await.expect("list migrations");
    assert_eq!(rows.len(), 3);
    let row0 = rows.iter().find(|r| r.version == 0).expect("version 0");
    assert_eq!(row0.checksum, checksum_hex(MIGRATION_0000_SQL));
    let row1 = rows.iter().find(|r| r.version == 1).expect("version 1");
    assert_eq!(row1.checksum, checksum_hex(MIGRATION_0001_SQL));
    let row2 = rows.iter().find(|r| r.version == 2).expect("version 2");
    assert_eq!(row2.checksum, checksum_hex(MIGRATION_0002_SQL));

    let (ns, db_name) = db.session_ns_db().await.expect("session");
    assert_eq!(ns, "vectra");
    assert_eq!(db_name, "comercio");
}

#[tokio::test]
async fn test_db_rocks_applies_embedded_migrations() {
    let wrapped = test_db_rocks().await;
    assert!(wrapped.path().exists());
    let rows = wrapped.list_migrations().await.expect("list migrations");
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().any(|r| r.version == 0));
    assert!(rows.iter().any(|r| r.version == 1));
    assert!(rows.iter().any(|r| r.version == 2));
}

#[tokio::test]
async fn test_db_instances_are_isolated() {
    let db_a = test_db().await;
    db_a.query(
        "DEFINE TABLE IF NOT EXISTS harness_probe SCHEMAFULL; \
             DEFINE FIELD tag ON harness_probe TYPE string; \
             CREATE harness_probe:1 CONTENT { tag: 'a' };",
    )
    .await
    .expect("seed probe on A");

    let db_b = test_db().await;
    assert_eq!(
        db_a.record_count("harness_probe")
            .await
            .expect("count on A"),
        1,
        "A must hold the seeded record"
    );
    assert_eq!(
        db_b.record_count("harness_probe")
            .await
            .expect("count on B"),
        0,
        "B must not see A's data"
    );
}

#[tokio::test]
async fn test_db_has_no_business_tables() {
    let db = test_db().await;
    assert_eq!(
        db.record_count("categoria").await.expect("categoria count"),
        0,
        "harness must not seed business tables"
    );
}

#[tokio::test]
async fn open_still_rejects_file_path() {
    let path = unique_file_path("file");
    let mut file = std::fs::File::create(&path).expect("create temp file");
    writeln!(file, "not a db dir").expect("write");
    drop(file);

    let err = match Db::open(&path).await {
        Err(e) => e,
        Ok(_) => panic!("file path must fail"),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("archivo") || msg.contains("directorio"),
        "expected file/dir reason, got: {msg}"
    );
    let _: &dyn std::error::Error = &err;
    let _ = std::fs::remove_file(&path);
}
