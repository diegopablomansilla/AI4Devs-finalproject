//! Integration tests for storage::Tx and OutboxNotifier (T-03-02).

use std::time::Duration;

use domain::{
    ConflictCode, DomainCode, DomainError, Id, InvalidInputCode, OutboxEntity, OutboxEvent,
    OutboxOp, UnavailableCode,
};

#[derive(Debug, Clone, Copy)]
struct RecordMarker;
use storage::testing::test_db;

const REGISTRO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";
const SCRATCH_ROW: &str = "01J0000000000000000000000";

async fn define_scratch(db: &storage::Db) {
    db.query(
        "REMOVE TABLE IF EXISTS scratch; \
         DEFINE TABLE scratch SCHEMAFULL; \
         DEFINE FIELD sku ON scratch TYPE string; \
         DEFINE INDEX uq_scratch_sku ON scratch FIELDS sku UNIQUE;",
    )
    .await
    .expect("define scratch");
}

fn sample_event() -> OutboxEvent {
    OutboxEvent::try_new(
        OutboxEntity::Producto,
        OutboxOp::Upsert,
        REGISTRO,
        Some("stock".into()),
    )
    .expect("event")
}

#[tokio::test]
async fn tx_commit_writes_data_and_outbox() {
    let db = test_db().await;
    define_scratch(&db).await;

    let mut tx = db.tx();
    tx.run(format!(
        "CREATE scratch:{SCRATCH_ROW} CONTENT {{ sku: 'A' }};"
    ));
    tx.emit(sample_event());
    tx.commit().await.expect("commit");

    assert_eq!(db.record_count("scratch").await.expect("count"), 1);
    assert_eq!(db.record_count("outbox").await.expect("count"), 1);
}

#[tokio::test]
async fn tx_commit_writes_multiple_outbox_events() {
    let db = test_db().await;
    let second_registro = Id::<RecordMarker>::generate().to_string();
    let second = OutboxEvent::try_new(
        OutboxEntity::Marca,
        OutboxOp::Delete,
        &second_registro,
        None,
    )
    .expect("second");

    let mut tx = db.tx();
    tx.emit(sample_event());
    tx.emit(second);
    tx.commit().await.expect("commit");

    assert_eq!(db.record_count("outbox").await.expect("count"), 2);
}

#[tokio::test]
async fn tx_commit_notifies_waiter() {
    let db = test_db().await;
    define_scratch(&db).await;

    let waiter = tokio::spawn(db.outbox_notifier().wait_for_commit());

    let mut tx = db.tx();
    tx.run(format!(
        "CREATE scratch:{SCRATCH_ROW} CONTENT {{ sku: 'notify' }};"
    ));
    tx.emit(sample_event());
    tx.commit().await.expect("commit");

    tokio::time::timeout(Duration::from_secs(2), waiter)
        .await
        .expect("waiter timeout")
        .expect("join");
}

#[tokio::test]
async fn tx_throw_rolls_back_data_and_outbox() {
    let db = test_db().await;
    define_scratch(&db).await;

    let mut tx = db.tx();
    tx.run(format!(
        "CREATE scratch:{SCRATCH_ROW} CONTENT {{ sku: 'rollback' }};"
    ));
    tx.emit(sample_event());
    tx.run(r#"THROW "stock_negativo";"#);
    let err = tx.commit().await.expect_err("must fail");
    assert!(matches!(
        err,
        DomainError::InvalidInput {
            code: InvalidInputCode::NegativeStock,
            ..
        }
    ));

    assert_eq!(db.record_count("scratch").await.expect("count"), 0);
    assert_eq!(db.record_count("outbox").await.expect("count"), 0);
}

#[tokio::test]
async fn tx_failed_does_not_notify() {
    let db = test_db().await;
    define_scratch(&db).await;

    let mut wait = std::pin::pin!(db.outbox_notifier().wait_for_commit());

    let mut tx = db.tx();
    tx.run(format!(
        "CREATE scratch:{SCRATCH_ROW} CONTENT {{ sku: 'no-notify' }};"
    ));
    tx.emit(sample_event());
    tx.run(r#"THROW "stock_negativo";"#);
    let _ = tx.commit().await.expect_err("fail");

    let wait = tokio::time::timeout(Duration::from_millis(150), wait.as_mut()).await;
    assert!(wait.is_err(), "notifier must not fire on failed commit");
}

#[tokio::test]
async fn tx_throw_stock_negativo_is_negative_stock() {
    let db = test_db().await;

    let mut tx = db.tx();
    tx.run(r#"THROW "stock_negativo";"#);
    let err = tx.commit().await.expect_err("throw");
    assert_eq!(err.code(), DomainCode::NegativeStock);
}

#[tokio::test]
async fn tx_unique_sku_is_duplicate_sku() {
    let db = test_db().await;
    define_scratch(&db).await;

    db.query(&format!(
        "CREATE scratch:{SCRATCH_ROW} CONTENT {{ sku: 'dup' }};"
    ))
    .await
    .expect("seed");

    let mut tx = db.tx();
    tx.run(format!(
        "CREATE scratch:01J0000000000000000000001 CONTENT {{ sku: 'dup' }};"
    ));
    tx.emit(sample_event());
    let err = tx.commit().await.expect_err("unique");
    assert!(matches!(
        err,
        DomainError::Conflict {
            code: ConflictCode::DuplicateSku,
        }
    ));
    assert_eq!(db.record_count("outbox").await.expect("count"), 0);
}

#[tokio::test]
async fn tx_unclassified_error_is_unavailable() {
    let db = test_db().await;

    let mut tx = db.tx();
    tx.run("SELECT * FROM;");
    let err = tx.commit().await.expect_err("bad table must fail commit");
    assert!(matches!(
        err,
        DomainError::Unavailable {
            code: UnavailableCode::Unavailable
        }
    ));
}
