//! Outbox read model and `OutboxRepo` port for Index Sync (T-03-07).

use std::future::Future;

use crate::error::DomainError;
use crate::id::Id;
use crate::outbox::OutboxEvent;

/// Marker for an `outbox` row id (ULID without table prefix).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutboxRecord;

/// One persisted outbox row plus its domain event payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxRow {
    pub id: Id<OutboxRecord>,
    pub event: OutboxEvent,
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
}

/// Maximum outbox batch size ([4.8.1](../../../../../docs/4-modelo.md#481-outbox)).
pub const OUTBOX_BATCH_LIMIT: u32 = 200;

#[cfg(test)]
mod tests {
    use std::future::Future;

    use super::*;
    use crate::outbox::{OutboxEntity, OutboxOp};

    struct DummyOutboxRepo {
        rows: Vec<OutboxRow>,
        cursor: Option<Id<OutboxRecord>>,
    }

    impl DummyOutboxRepo {
        fn new(rows: Vec<OutboxRow>) -> Self {
            Self { rows, cursor: None }
        }

        fn list_after(
            &self,
            after: Option<Id<OutboxRecord>>,
        ) -> Result<Vec<OutboxRow>, DomainError> {
            let mut out: Vec<_> = self
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
            if !self.rows.iter().any(|r| r.id == event_id) {
                return Err(DomainError::not_found());
            }
            let _ = event_id;
            Ok(())
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
            let cursor = self.cursor;
            async move { Ok(cursor) }
        }

        fn advance_meilisearch_cursor(
            &self,
            event_id: Id<OutboxRecord>,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let result = self.advance(event_id);
            async move { result }
        }
    }

    const REGISTRO: &str = "01J9ZKA7M3Q8R2T5V9X1Z4B6D8";

    fn row(id: Id<OutboxRecord>) -> OutboxRow {
        OutboxRow {
            id,
            event: OutboxEvent::try_new(OutboxEntity::Producto, OutboxOp::Upsert, REGISTRO, None)
                .expect("event"),
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
}
