//! Atomic data + outbox writes and post-commit notify (T-03-02).

use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Notify;

use domain::{
    ConflictCode, DomainCode, DomainError, InvalidInputCode, OutboxEvent, UnavailableCode,
};
use surrealdb::Error as SurrealError;

use crate::Db;

/// Wakes the Index Sync Worker after a successful outbox commit.
#[derive(Debug, Clone)]
pub struct OutboxNotifier {
    inner: Arc<Notify>,
}

impl OutboxNotifier {
    /// Returns a future that completes when [`Db::tx`] commits successfully.
    pub fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.inner.notified()
    }

    /// Owned wait handle for use inside `tokio::spawn` ( `'static` ).
    pub fn wait_for_commit(self) -> impl std::future::Future<Output = ()> + Send + 'static {
        let inner = self.inner;
        async move {
            inner.notified().await;
        }
    }

    pub(crate) fn signal(&self) {
        self.inner.notify_waiters();
    }
}

/// Transaction builder: business SQL + outbox events in one SurrealDB transaction.
pub struct Tx<'a> {
    db: &'a Db,
    statements: Vec<String>,
    events: Vec<OutboxEvent>,
}

impl<'a> Tx<'a> {
    pub(crate) fn new(db: &'a Db) -> Self {
        Self {
            db,
            statements: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Appends a SurrealQL statement (no bindings); executed inside `commit`.
    pub fn run(&mut self, sql: impl Into<String>) {
        self.statements.push(sql.into());
    }

    /// Queues an outbox row created on `commit`.
    pub fn emit(&mut self, event: OutboxEvent) {
        self.events.push(event);
    }

    /// Runs `BEGIN` … business SQL … `CREATE outbox` … `COMMIT`.
    pub async fn commit(self) -> Result<(), DomainError> {
        if self.statements.is_empty() && self.events.is_empty() {
            return Ok(());
        }

        let mut script = String::from("BEGIN TRANSACTION;\n");
        for stmt in &self.statements {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            script.push_str(trimmed);
            if !trimmed.ends_with(';') {
                script.push(';');
            }
            script.push('\n');
        }

        let events = self.events;
        for (i, _ev) in events.iter().enumerate() {
            script.push_str(&format!(
                "CREATE type::thing('outbox', rand::ulid()) CONTENT {{ \
                 entidad: $entidad_{i}, \
                 registro: type::thing($entidad_{i}, $registro_{i}), \
                 operacion: $operacion_{i}, \
                 motivo: $motivo_{i} \
                 }};\n"
            ));
        }
        script.push_str("COMMIT TRANSACTION;");

        let mut query = self.db.inner.query(script);
        for (i, ev) in events.into_iter().enumerate() {
            let entidad = ev.entidad.as_str().to_string();
            let registro = ev.registro;
            let operacion = ev.operacion.as_str().to_string();
            let motivo = ev.motivo;
            query = query
                .bind((format!("entidad_{i}"), entidad))
                .bind((format!("registro_{i}"), registro))
                .bind((format!("operacion_{i}"), operacion))
                .bind((format!("motivo_{i}"), motivo));
        }

        let statements = self.statements;
        let response = query.await.map_err(|e| map_surreal_error(e, &statements))?;
        response
            .check()
            .map_err(|e| map_surreal_error(e, &statements))?;

        self.db.outbox_notifier().signal();
        Ok(())
    }
}

impl Db {
    /// Shared notifier for outbox commits on this database.
    pub fn outbox_notifier(&self) -> OutboxNotifier {
        OutboxNotifier {
            inner: Arc::clone(&self.outbox_notify),
        }
    }

    /// Starts a transaction bound to this database.
    pub fn tx(&self) -> Tx<'_> {
        Tx::new(self)
    }
}

fn map_surreal_error(err: SurrealError, statements: &[String]) -> DomainError {
    map_surreal_message(&err.to_string(), statements)
}

fn map_surreal_message(msg: &str, statements: &[String]) -> DomainError {
    let lower = msg.to_lowercase();

    if lower.contains("stock_negativo") {
        return DomainError::invalid_input(InvalidInputCode::NegativeStock, vec![]);
    }

    if lower.contains("failed transaction")
        && statements.iter().any(|s| s.contains("stock_negativo"))
    {
        return DomainError::invalid_input(InvalidInputCode::NegativeStock, vec![]);
    }

    for token in msg.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
        if let Ok(code) = DomainCode::from_str(token) {
            return DomainError::from_domain_code(code, vec![]);
        }
    }

    if lower.contains("unique")
        || lower.contains("already exists")
        || lower.contains("already contains")
        || lower.contains("duplicate")
    {
        let conflict = if lower.contains("sku") {
            ConflictCode::DuplicateSku
        } else if lower.contains("slug") {
            ConflictCode::DuplicateSlug
        } else if lower.contains("relation") {
            ConflictCode::DuplicateRelation
        } else {
            ConflictCode::Conflict
        };
        return DomainError::conflict(conflict);
    }

    DomainError::unavailable(UnavailableCode::Unavailable)
}
