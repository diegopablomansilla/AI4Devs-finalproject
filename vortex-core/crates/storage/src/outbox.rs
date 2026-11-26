//! SurrealDB adapter for `OutboxRepo` (T-03-07).

use std::future::Future;
use std::sync::Arc;

use domain::{
    DomainError, Id, OUTBOX_BATCH_LIMIT, OutboxEntity, OutboxEvent, OutboxOp, OutboxRecord,
    OutboxRepo, OutboxRow, Timestamp,
};
use serde::Deserialize;
use surrealdb::{RecordId, RecordIdKey};

use crate::Db;
use crate::tx::map_surreal_message;

/// Reads `outbox` rows and updates `sync_cursor:meilisearch`.
#[derive(Clone)]
pub struct OutboxStore {
    db: Arc<Db>,
}

impl OutboxStore {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Deserialize)]
struct OutboxRowRecord {
    id: RecordId,
    entidad: String,
    registro: RecordId,
    operacion: String,
    motivo: Option<String>,
    creado_en: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CursorRecord {
    ultimo_evento: Option<RecordId>,
}

impl OutboxRepo for OutboxStore {
    fn list_after_meilisearch(
        &self,
        after: Option<Id<OutboxRecord>>,
    ) -> impl Future<Output = Result<Vec<OutboxRow>, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move {
            let (stmt, bind_after) = match after {
                None => (
                    format!(
                        "SELECT id, entidad, registro, operacion, motivo, creado_en FROM outbox \
                         ORDER BY id LIMIT {OUTBOX_BATCH_LIMIT};"
                    ),
                    false,
                ),
                Some(_cursor) => (
                    "SELECT id, entidad, registro, operacion, motivo, creado_en FROM outbox \
                     WHERE id > type::thing('outbox', $cursor) \
                     ORDER BY id LIMIT $limit;"
                        .to_string(),
                    true,
                ),
            };

            let response = if bind_after {
                let cursor = after.expect("cursor").to_string();
                db.inner
                    .query(&stmt)
                    .bind(("cursor", cursor))
                    .bind(("limit", OUTBOX_BATCH_LIMIT))
                    .await
            } else {
                db.inner.query(&stmt).await
            }
            .map_err(|e| map_surreal_message(&e.to_string(), std::slice::from_ref(&stmt)))?;

            let mut checked = response
                .check()
                .map_err(|e| map_surreal_message(&e.to_string(), &[stmt]))?;
            let rows: Vec<OutboxRowRecord> = checked
                .take(0)
                .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[]))?;

            rows.into_iter().map(row_record_to_domain).collect()
        }
    }

    fn meilisearch_cursor(
        &self,
    ) -> impl Future<Output = Result<Option<Id<OutboxRecord>>, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move {
            let stmt = "SELECT ultimo_evento FROM sync_cursor:meilisearch;";
            let response = db
                .inner
                .query(stmt)
                .await
                .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
            let mut checked = response
                .check()
                .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
            let rows: Vec<CursorRecord> = checked
                .take(0)
                .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[]))?;
            let cursor = rows
                .into_iter()
                .next()
                .and_then(|r| r.ultimo_evento)
                .map(|rid| record_id_to_outbox_id(&rid))
                .transpose()?;
            Ok(cursor)
        }
    }

    fn advance_meilisearch_cursor(
        &self,
        event_id: Id<OutboxRecord>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move {
            let id_str = event_id.to_string();
            let exists_stmt =
                "IF !record::exists(type::thing('outbox', $id)) { THROW 'not_found'; };"
                    .to_string();
            let update_stmt =
                "UPDATE sync_cursor:meilisearch SET ultimo_evento = type::thing('outbox', $id);";
            let script =
                format!("BEGIN TRANSACTION;\n{exists_stmt}\n{update_stmt}\nCOMMIT TRANSACTION;");

            let response = db
                .inner
                .query(script)
                .bind(("id", id_str))
                .await
                .map_err(|e| map_surreal_message(&e.to_string(), &[]))?;
            response
                .check()
                .map_err(|e| map_surreal_message(&e.to_string(), &[]))?;
            Ok(())
        }
    }

    fn record_meilisearch_failure(
        &self,
        event_id: Id<OutboxRecord>,
        ultimo_error: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let db = Arc::clone(&self.db);
        let ultimo_error = ultimo_error.to_string();
        async move {
            let id_str = event_id.to_string();
            let script = "BEGIN TRANSACTION;\n\
                 IF !record::exists(type::thing('outbox', $id)) { THROW 'not_found'; };\n\
                 LET $ev = type::thing('outbox', $id);\n\
                 LET $existing = (SELECT id FROM outbox_fallo \
                   WHERE evento = $ev AND consumidor = 'meilisearch' LIMIT 1);\n\
                 IF array::len($existing) > 0 {\n\
                   UPDATE $existing[0].id SET ultimo_error = $err, estado = 'fallido', actualizado_en = time::now();\n\
                 } ELSE {\n\
                   CREATE outbox_fallo CONTENT { evento: $ev, consumidor: 'meilisearch', estado: 'fallido', \
                   ultimo_error: $err, intentos: 1 };\n\
                 };\n\
                 COMMIT TRANSACTION;"
                .to_string();
            let response = db
                .inner
                .query(script)
                .bind(("id", id_str))
                .bind(("err", ultimo_error))
                .await
                .map_err(|e| map_surreal_message(&e.to_string(), &[]))?;
            response
                .check()
                .map_err(|e| map_surreal_message(&e.to_string(), &[]))?;
            Ok(())
        }
    }

    fn meilisearch_lag(&self) -> impl Future<Output = Result<u64, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move {
            let cursor_stmt = "SELECT ultimo_evento FROM sync_cursor:meilisearch;";
            let response = db
                .inner
                .query(cursor_stmt)
                .await
                .map_err(|e| map_surreal_message(&e.to_string(), &[cursor_stmt.to_string()]))?;
            let mut checked = response
                .check()
                .map_err(|e| map_surreal_message(&e.to_string(), &[cursor_stmt.to_string()]))?;
            let cursors: Vec<CursorRecord> = checked
                .take(0)
                .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[]))?;
            let after = cursors
                .into_iter()
                .next()
                .and_then(|r| r.ultimo_evento)
                .map(|rid| record_id_to_outbox_id(&rid))
                .transpose()?;

            let (stmt, bind) = match after {
                None => (
                    "SELECT count() AS c FROM outbox GROUP ALL;".to_string(),
                    false,
                ),
                Some(_) => (
                    "SELECT count() AS c FROM outbox WHERE id > type::thing('outbox', $cursor) GROUP ALL;"
                        .to_string(),
                    true,
                ),
            };

            let response = if bind {
                db.inner
                    .query(&stmt)
                    .bind(("cursor", after.expect("cursor").to_string()))
                    .await
            } else {
                db.inner.query(&stmt).await
            }
            .map_err(|e| map_surreal_message(&e.to_string(), std::slice::from_ref(&stmt)))?;
            let mut checked = response
                .check()
                .map_err(|e| map_surreal_message(&e.to_string(), &[stmt]))?;
            let rows: Vec<CountRow> = checked
                .take(0)
                .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[]))?;
            Ok(rows.into_iter().next().map(|r| r.c).unwrap_or(0))
        }
    }

    fn meilisearch_failed_count(&self) -> impl Future<Output = Result<u64, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move {
            let stmt = "SELECT count() AS c FROM outbox_fallo \
                WHERE consumidor = 'meilisearch' AND estado = 'fallido' GROUP ALL;";
            let response = db
                .inner
                .query(stmt)
                .await
                .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
            let mut checked = response
                .check()
                .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
            let rows: Vec<CountRow> = checked
                .take(0)
                .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[]))?;
            Ok(rows.into_iter().next().map(|r| r.c).unwrap_or(0))
        }
    }
}

#[derive(Debug, Deserialize)]
struct CountRow {
    c: u64,
}

fn row_record_to_domain(row: OutboxRowRecord) -> Result<OutboxRow, DomainError> {
    let id = record_id_to_outbox_id(&row.id)?;
    let entidad = OutboxEntity::try_from_str(&row.entidad).map_err(|_| {
        DomainError::invalid_input(domain::error::InvalidInputCode::InvalidInput, vec![])
    })?;
    let operacion = OutboxOp::try_from_str(&row.operacion).map_err(|_| {
        DomainError::invalid_input(domain::error::InvalidInputCode::InvalidInput, vec![])
    })?;
    let registro = record_id_to_ulid_string(&row.registro)?;
    let event = OutboxEvent::try_new(entidad, operacion, registro, row.motivo).map_err(|_| {
        DomainError::invalid_input(domain::error::InvalidInputCode::InvalidInput, vec![])
    })?;
    let creado_en = row
        .creado_en
        .as_deref()
        .unwrap_or("1970-01-01T00:00:00Z")
        .parse::<Timestamp>()
        .map_err(|_| {
            DomainError::invalid_input(domain::error::InvalidInputCode::InvalidInput, vec![])
        })?;
    Ok(OutboxRow {
        id,
        event,
        creado_en,
    })
}

fn record_id_to_outbox_id(rid: &RecordId) -> Result<Id<OutboxRecord>, DomainError> {
    let ulid = record_id_to_ulid_string(rid)?;
    ulid.parse().map_err(|_| {
        DomainError::invalid_input(domain::error::InvalidInputCode::InvalidInput, vec![])
    })
}

fn record_id_to_ulid_string(rid: &RecordId) -> Result<String, DomainError> {
    let key: RecordIdKey = rid.key().clone();
    let s: String = key
        .try_into()
        .map_err(|_| DomainError::unavailable(domain::error::UnavailableCode::Unavailable))?;
    if s.contains(':') {
        return Err(DomainError::invalid_input(
            domain::error::InvalidInputCode::InvalidInput,
            vec![],
        ));
    }
    Ok(s)
}
