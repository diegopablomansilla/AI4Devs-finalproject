//! Integration tests for embedded SurrealDB open (T-00-08).

use std::error::Error as StdError;
use std::path::{Path, PathBuf};

use storage::{Db, OpenError};

fn unique_db_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vortex-storage-{prefix}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

async fn open_after_close(path: &Path) -> Db {
    for attempt in 0..20 {
        match Db::open(path).await {
            Ok(db) => return db,
            Err(e) if attempt + 1 < 20 && e.to_string().to_lowercase().contains("lock") => {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Err(e) => panic!("second open failed: {e}"),
        }
    }
    unreachable!("open_after_close exhausted retries");
}

fn assert_open_error_has_reason(err: &OpenError) {
    let msg = err.to_string();
    assert!(
        !msg.trim().is_empty(),
        "OpenError display must not be empty"
    );
    assert!(
        msg.len() > 8,
        "expected a legible reason in OpenError, got: {msg}"
    );
}

#[tokio::test]
async fn open_empty_dir_selects_ns_and_db() {
    let path = unique_db_dir("empty");
    let db = Db::open(&path).await.expect("open empty temp dir");
    let (ns, db_name) = db.session_ns_db().await.expect("session");
    assert_eq!(ns, "vectra");
    assert_eq!(db_name, "comercio");
    drop(db);
    let _ = std::fs::remove_dir_all(&path);
}

#[tokio::test]
async fn reopen_same_dir_keeps_ns_and_db() {
    let path = unique_db_dir("reopen");
    {
        let db = Db::open(&path).await.expect("first open");
        let (ns, db_name) = db.session_ns_db().await.expect("session");
        assert_eq!(ns, "vectra");
        assert_eq!(db_name, "comercio");
    }
    let db = open_after_close(&path).await;
    let (ns, db_name) = db.session_ns_db().await.expect("session");
    assert_eq!(ns, "vectra");
    assert_eq!(db_name, "comercio");
    drop(db);
    let _ = std::fs::remove_dir_all(&path);
}

#[tokio::test]
async fn open_rejects_corrupt_directory() {
    let path = unique_db_dir("corrupt");
    std::fs::create_dir_all(&path).expect("mkdir");
    std::fs::write(path.join("CURRENT"), b"not-a-valid-rocksdb-manifest").expect("write garbage");
    let err = match Db::open(&path).await {
        Err(e) => e,
        Ok(_) => panic!("corrupt path must fail"),
    };
    assert_open_error_has_reason(&err);
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("corrupt")
            || msg.contains("invalid")
            || msg.contains("parse")
            || msg.contains("rocksdb")
            || msg.contains("abrir")
            || msg.contains("formato")
            || msg.contains("io"),
        "expected corruption-related reason, got: {msg}"
    );
    let _: &dyn StdError = &err;
    let _ = std::fs::remove_dir_all(&path);
}

#[cfg(unix)]
#[tokio::test]
async fn open_rejects_unwritable_directory() {
    use std::os::unix::fs::PermissionsExt;

    let path = unique_db_dir("unwritable");
    std::fs::create_dir_all(&path).expect("mkdir");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).expect("chmod");

    let result = Db::open(&path).await;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).expect("restore");

    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("unwritable dir must fail"),
    };
    assert_open_error_has_reason(&err);
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("permiso")
            || msg.contains("permission")
            || msg.contains("denied")
            || msg.contains("denegad")
            || msg.contains("abrir")
            || msg.contains("escribir")
            || msg.contains("io"),
        "expected permission-related reason, got: {msg}"
    );
    let _: &dyn StdError = &err;
    let _ = std::fs::remove_dir_all(&path);
}
