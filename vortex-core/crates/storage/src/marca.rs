//! SurrealDB adapter for `BrandRepo` (T-02-08).

use std::future::Future;
use std::str::FromStr as _;
use std::sync::Arc;

use domain::{
    BrandRepo, ConflictCode, DomainError, Id, InvalidInputCode, Marca, MarcaDraft, NewMarca,
    OutboxEntity, OutboxEvent, OutboxOp, Slug, Timestamp, UnavailableCode, UpdateMarca,
};
use serde::Deserialize;
use surrealdb::{RecordId, RecordIdKey};

use crate::Db;
use crate::tx::map_surreal_message;

/// Brand persistence against table `marca`.
pub struct MarcaRepo {
    db: Arc<Db>,
}

impl MarcaRepo {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Deserialize)]
struct MarcaRecord {
    id: RecordId,
    nombre: String,
    slug: String,
    #[serde(default = "default_true")]
    activo: bool,
    creado_en: String,
    actualizado_en: String,
}

fn default_true() -> bool {
    true
}

impl BrandRepo for MarcaRepo {
    fn create(&self, input: NewMarca) -> impl Future<Output = Result<Marca, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { create_impl(db.as_ref(), input).await }
    }

    fn get(&self, id: Id<Marca>) -> impl Future<Output = Result<Marca, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { fetch_one(db.as_ref(), id).await }
    }

    fn list_all(&self) -> impl Future<Output = Result<Vec<Marca>, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { fetch_all(db.as_ref()).await }
    }

    fn update(
        &self,
        input: UpdateMarca,
    ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { update_impl(db.as_ref(), input).await }
    }

    fn set_activo(
        &self,
        id: Id<Marca>,
        activo: bool,
    ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { set_activo_impl(db.as_ref(), id, activo).await }
    }
}

async fn create_impl(db: &Db, input: NewMarca) -> Result<Marca, DomainError> {
    let slug = match input.slug {
        Some(s) => s,
        None => Slug::from_nombre(&input.nombre)?,
    };
    let nombre = input.nombre.trim().to_string();
    if nombre.is_empty() {
        return Err(DomainError::invalid_input(
            InvalidInputCode::InvalidInput,
            vec![],
        ));
    }

    let id = Id::<Marca>::generate();
    let id_str = id.to_string();
    let slug_sql = sql_string(slug.as_str());
    let nombre_sql = sql_string(&nombre);

    let create_sql = format!(
        "CREATE type::thing('marca', '{id_str}') CONTENT {{
            nombre: {nombre_sql},
            slug: {slug_sql},
            activo: true
        }};"
    );

    let event = OutboxEvent::try_new(OutboxEntity::Marca, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.tx();
    tx.run(create_sql);
    tx.emit(event);
    tx.commit().await?;

    fetch_one(db, id).await
}

async fn update_impl(db: &Db, input: UpdateMarca) -> Result<Marca, DomainError> {
    let current = fetch_one(db, input.id.clone()).await?;
    let slug = match input.slug {
        Some(s) => s,
        None => Slug::from_nombre(&input.nombre)?,
    };
    let nombre = input.nombre.trim().to_string();

    if slug.as_str() != current.slug.as_str()
        && slug_taken_by_other(db, slug.as_str(), &input.id).await?
    {
        return Err(DomainError::conflict(ConflictCode::DuplicateSlug));
    }

    let id_str = input.id.to_string();
    let slug_sql = sql_string(slug.as_str());
    let nombre_sql = sql_string(&nombre);
    let stmt = format!(
        "UPDATE type::thing('marca', '{id_str}') SET nombre = {nombre_sql}, slug = {slug_sql};"
    );

    let event = OutboxEvent::try_new(OutboxEntity::Marca, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.tx();
    tx.run(stmt);
    tx.emit(event);
    tx.commit().await?;

    fetch_one(db, input.id).await
}

async fn set_activo_impl(db: &Db, id: Id<Marca>, activo: bool) -> Result<Marca, DomainError> {
    let _ = fetch_one(db, id.clone()).await?;
    let id_str = id.to_string();
    let activo_sql = if activo { "true" } else { "false" };
    let stmt = format!("UPDATE type::thing('marca', '{id_str}') SET activo = {activo_sql};");
    let event = OutboxEvent::try_new(OutboxEntity::Marca, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let mut tx = db.tx();
    tx.run(stmt);
    tx.emit(event);
    tx.commit().await?;
    fetch_one(db, id).await
}

async fn slug_taken_by_other(
    db: &Db,
    slug: &str,
    exclude: &Id<Marca>,
) -> Result<bool, DomainError> {
    let stmt = format!(
        "RETURN array::len((SELECT id FROM marca WHERE slug = {} AND id != type::thing('marca', '{}') LIMIT 1));",
        sql_string(slug),
        exclude
    );
    let len = scalar_i64(db, &stmt).await?.unwrap_or(0);
    Ok(len > 0)
}

async fn fetch_one(db: &Db, id: Id<Marca>) -> Result<Marca, DomainError> {
    let id_str = id.to_string();
    let stmt = format!("SELECT * FROM type::thing('marca', '{id_str}');");
    let rows = query_records(db, &stmt).await?;
    let row = rows.into_iter().next().ok_or(DomainError::not_found())?;
    record_to_marca(Some(id), row)
}

async fn fetch_all(db: &Db) -> Result<Vec<Marca>, DomainError> {
    let rows = query_records(db, "SELECT * FROM marca;").await?;
    rows.into_iter()
        .map(|row| record_to_marca(None, row))
        .collect()
}

async fn query_records(db: &Db, stmt: &str) -> Result<Vec<MarcaRecord>, DomainError> {
    let response = db
        .inner
        .query(stmt)
        .await
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let mut checked = response
        .check()
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let rows: Vec<MarcaRecord> = checked
        .take(0)
        .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[stmt.to_string()]))?;
    Ok(rows)
}

async fn scalar_i64(db: &Db, stmt: &str) -> Result<Option<i64>, DomainError> {
    let response = db
        .inner
        .query(stmt)
        .await
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let mut checked = response
        .check()
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    checked
        .take(0)
        .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[stmt.to_string()]))
}

fn record_to_marca(known_id: Option<Id<Marca>>, row: MarcaRecord) -> Result<Marca, DomainError> {
    let id = match known_id {
        Some(id) => id,
        None => record_id_to_domain_id(&row.id)?,
    };
    let slug = Slug::parse(&row.slug)?;
    let creado_en = parse_storage_timestamp(&row.creado_en)?;
    let actualizado_en = parse_storage_timestamp(&row.actualizado_en)?;
    Marca::try_new(MarcaDraft {
        id,
        nombre: row.nombre,
        slug,
        activo: row.activo,
        creado_en,
        actualizado_en,
    })
}

fn record_id_to_domain_id<T>(rid: &RecordId) -> Result<Id<T>, DomainError> {
    let key: RecordIdKey = rid.key().clone();
    let s: String = key
        .try_into()
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
    Id::from_str(&s).map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))
}

fn parse_storage_timestamp(s: &str) -> Result<Timestamp, DomainError> {
    if s.is_empty() {
        return Err(DomainError::unavailable(UnavailableCode::Unavailable));
    }
    Timestamp::from_str(s)
        .or_else(|_| Timestamp::from_str(&format!("{s}Z")))
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))
}

fn sql_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}
