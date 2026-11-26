//! Outbox read model and `OutboxRepo` port for Index Sync (T-03-07).

use std::future::Future;

use crate::error::DomainError;
use crate::id::Id;
use crate::outbox::OutboxEvent;
use crate::timestamp::Timestamp;

/// Marker for an `outbox` row id (ULID without table prefix).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutboxRecord;

/// One persisted outbox row plus its domain event payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxRow {
    pub id: Id<OutboxRecord>,
    pub event: OutboxEvent,
    pub creado_en: Timestamp,
}

/// Reads the catalog outbox and advances `sync_cursor:meilisearch` (T-03-07).
pub trait OutboxRepo {
    /// Rows with id strictly greater than `after`, ascending, at most 200.
    fn list_after_meilisearch(
        &self,
        after: Option<Id<OutboxRecord>>,
    ) -> impl Future<Output = Result<Vec<OutboxRow>, DomainError>> + Send;

    fn meilisearch_cursor(
        &self,
    ) -> impl Future<Output = Result<Option<Id<OutboxRecord>>, DomainError>> + Send;

    fn advance_meilisearch_cursor(
        &self,
        event_id: Id<OutboxRecord>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Registers a permanent Meilisearch consumer failure for `event_id`.
    fn record_meilisearch_failure(
        &self,
        event_id: Id<OutboxRecord>,
        ultimo_error: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Pending `outbox` rows strictly after `sync_cursor:meilisearch`.
    fn meilisearch_lag(&self) -> impl Future<Output = Result<u64, DomainError>> + Send;

    /// Count of `outbox_fallo` with `consumidor` `meilisearch` and `estado` `fallido`.
    fn meilisearch_failed_count(&self) -> impl Future<Output = Result<u64, DomainError>> + Send;
}

/// Maximum outbox batch size ([4.8.1](../../../../../docs/4-modelo.md#481-outbox)).
pub const OUTBOX_BATCH_LIMIT: u32 = 200;

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::outbox::{OutboxEntity, OutboxOp};

    struct DummyOutboxRepo {
        inner: Arc<Mutex<DummyInner>>,
    }

    struct DummyInner {
        rows: Vec<OutboxRow>,
        cursor: Option<Id<OutboxRecord>>,
        fallido_ids: Vec<Id<OutboxRecord>>,
    }

    impl DummyOutboxRepo {
        fn new(rows: Vec<OutboxRow>) -> Self {
            Self {
                inner: Arc::new(Mutex::new(DummyInner {
                    rows,
                    cursor: None,
                    fallido_ids: vec![],
                })),
            }
        }

        fn list_after(
            &self,
            after: Option<Id<OutboxRecord>>,
        ) -> Result<Vec<OutboxRow>, DomainError> {
            let inner = self.inner.lock().expect("lock");
            let mut out: Vec<_> = inner
                .rows
                .iter()
                .filter(|row| match after {
                    None => true,
                    Some(cur) => row.id.inner_ulid() > cur.inner_ulid(),
                })
                .cloned()
                .collect();
            out.sort_by_key(|r| r.id.inner_ulid());
            out.truncate(OUTBOX_BATCH_LIMIT as usize);
            Ok(out)
        }

        fn advance(&self, event_id: Id<OutboxRecord>) -> Result<(), DomainError> {
            let mut inner = self.inner.lock().expect("lock");
            if !inner.rows.iter().any(|r| r.id == event_id) {
                return Err(DomainError::not_found());
            }
            inner.cursor = Some(event_id);
            Ok(())
        }

        fn record_failure(
            &self,
            event_id: Id<OutboxRecord>,
            _ultimo_error: &str,
        ) -> Result<(), DomainError> {
            let mut inner = self.inner.lock().expect("lock");
            if !inner.rows.iter().any(|r| r.id == event_id) {
                return Err(DomainError::not_found());
            }
            if !inner.fallido_ids.contains(&event_id) {
                inner.fallido_ids.push(event_id);
            }
            Ok(())
        }

        fn lag_pending(&self) -> u64 {
            let inner = self.inner.lock().expect("lock");
            let after = inner.cursor;
            inner
                .rows
                .iter()
                .filter(|row| match after {
                    None => true,
                    Some(cur) => row.id.inner_ulid() > cur.inner_ulid(),
                })
                .count() as u64
        }

        fn failed_count(&self) -> u64 {
            self.inner.lock().expect("lock").fallido_ids.len() as u64
        }
    }

    impl OutboxRepo for DummyOutboxRepo {
        fn list_after_meilisearch(
            &self,
            after: Option<Id<OutboxRecord>>,
        ) -> impl Future<Output = Result<Vec<OutboxRow>, DomainError>> + Send {
            let result = self.list_after(after);
            async move { result }
        }

        fn meilisearch_cursor(
            &self,
        ) -> impl Future<Output = Result<Option<Id<OutboxRecord>>, DomainError>> + Send {
            let inner = Arc::clone(&self.inner);
            async move {
                let guard = inner.lock().expect("lock");
                Ok(guard.cursor)
            }
        }

        fn advance_meilisearch_cursor(
            &self,
            event_id: Id<OutboxRecord>,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let result = self.advance(event_id);
            async move { result }
        }

        fn record_meilisearch_failure(
            &self,
            event_id: Id<OutboxRecord>,
            ultimo_error: &str,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let result = self.record_failure(event_id, ultimo_error);
            async move { result }
        }

        fn meilisearch_lag(&self) -> impl Future<Output = Result<u64, DomainError>> + Send {
            let lag = self.lag_pending();
            async move { Ok(lag) }
        }

        fn meilisearch_failed_count(
            &self,
        ) -> impl Future<Output = Result<u64, DomainError>> + Send {
            let count = self.failed_count();
            async move { Ok(count) }
        }
    }

    const REGISTRO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

    fn ts() -> Timestamp {
        "2026-01-01T00:00:00Z".parse().expect("ts")
    }

    fn row(id: Id<OutboxRecord>) -> OutboxRow {
        OutboxRow {
            id,
            event: OutboxEvent::try_new(OutboxEntity::Producto, OutboxOp::Upsert, REGISTRO, None)
                .expect("event"),
            creado_en: ts(),
        }
    }

    fn ordered_pair() -> (Id<OutboxRecord>, Id<OutboxRecord>) {
        let a = Id::<OutboxRecord>::generate();
        let b = Id::<OutboxRecord>::generate();
        if a.inner_ulid() < b.inner_ulid() {
            (a, b)
        } else {
            (b, a)
        }
    }

    #[test]
    fn outbox_repo_dummy_lists_after_cursor() {
        let (first, second) = ordered_pair();
        let repo = DummyOutboxRepo::new(vec![row(first), row(second)]);
        let batch = repo.list_after(Some(first)).expect("list");
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].id, second);
    }

    #[test]
    fn outbox_repo_dummy_advance_not_found() {
        let repo = DummyOutboxRepo::new(vec![]);
        let missing = Id::<OutboxRecord>::generate();
        let err = repo.advance(missing).expect_err("missing");
        assert_eq!(err.code(), crate::error::DomainCode::NotFound);
    }

    #[test]
    fn outbox_repo_dummy_records_fallido() {
        let (first, second) = ordered_pair();
        let repo = DummyOutboxRepo::new(vec![row(first), row(second)]);
        repo.record_failure(first, "bad doc").expect("record");
        assert_eq!(repo.failed_count(), 1);
        repo.record_failure(first, "again").expect("idempotent");
        assert_eq!(repo.failed_count(), 1);
        let _ = second;
    }

    #[test]
    fn outbox_repo_dummy_lag_counts_pending() {
        let (first, second) = ordered_pair();
        let repo = DummyOutboxRepo::new(vec![row(first), row(second)]);
        assert_eq!(repo.lag_pending(), 2);
        repo.advance(first).expect("advance");
        assert_eq!(repo.lag_pending(), 1);
    }

    #[test]
    fn outbox_repo_dummy_record_unknown_event_is_not_found() {
        let repo = DummyOutboxRepo::new(vec![]);
        let missing = Id::<OutboxRecord>::generate();
        let err = repo.record_failure(missing, "x").expect_err("missing");
        assert_eq!(err.code(), crate::error::DomainCode::NotFound);
    }
}
