//! SurrealDB adapter for `AttributeDefRepo` (T-02-11).

use std::future::Future;
use std::str::FromStr as _;
use std::sync::Arc;

use domain::{
    AtributoDef, AtributoDefDraft, AttributeDefRepo, ConflictCode, DomainError, Id, NewAtributoDef,
    OutboxEntity, OutboxEvent, OutboxOp, Timestamp, TipoAtributo, UnavailableCode,
    UpdateAtributoDef,
};
use serde::Deserialize;
use surrealdb::{RecordId, RecordIdKey};

use crate::Db;
use crate::tx::map_surreal_message;

/// Attribute definition persistence against table `atributo_def`.
pub struct AtributoDefRepo {
    db: Arc<Db>,
}

impl AtributoDefRepo {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Deserialize)]
struct AtributoDefRecord {
    id: RecordId,
    clave: String,
    etiqueta: String,
    tipo: String,
    unidad: Option<String>,
    opciones: Option<Vec<String>>,
    #[serde(default)]
    facetable: bool,
    #[serde(default = "default_true")]
    buscable: bool,
    #[serde(default)]
    orden: i32,
    #[serde(default = "default_true")]
    activo: bool,
    creado_en: String,
    actualizado_en: String,
}

fn default_true() -> bool {
    true
}

impl AttributeDefRepo for AtributoDefRepo {
    fn create(
        &self,
        input: NewAtributoDef,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { create_impl(db.as_ref(), input).await }
    }

    fn get(
        &self,
        id: Id<AtributoDef>,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { fetch_one(db.as_ref(), id).await }
    }

    fn list_all(&self) -> impl Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { fetch_all(db.as_ref()).await }
    }

    fn update(
        &self,
        input: UpdateAtributoDef,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { update_impl(db.as_ref(), input).await }
    }

    fn set_activo(
        &self,
        id: Id<AtributoDef>,
        activo: bool,
    ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { set_activo_impl(db.as_ref(), id, activo).await }
    }
}

#[derive(Clone)]
struct PersistFields {
    clave: String,
    etiqueta: String,
    tipo: TipoAtributo,
    unidad: Option<String>,
    opciones: Option<Vec<String>>,
    facetable: bool,
    buscable: bool,
    orden: i32,
    activo: bool,
}

impl From<NewAtributoDef> for PersistFields {
    fn from(input: NewAtributoDef) -> Self {
        Self {
            clave: input.clave,
            etiqueta: input.etiqueta,
            tipo: input.tipo,
            unidad: input.unidad,
            opciones: input.opciones,
            facetable: input.facetable,
            buscable: input.buscable,
            orden: input.orden,
            activo: true,
        }
    }
}

impl From<UpdateAtributoDef> for PersistFields {
    fn from(input: UpdateAtributoDef) -> Self {
        Self {
            clave: input.clave,
            etiqueta: input.etiqueta,
            tipo: input.tipo,
            unidad: input.unidad,
            opciones: input.opciones,
            facetable: input.facetable,
            buscable: input.buscable,
            orden: input.orden,
            activo: true,
        }
    }
}

async fn create_impl(db: &Db, input: NewAtributoDef) -> Result<AtributoDef, DomainError> {
    input.validate()?;

    let id = Id::<AtributoDef>::generate();
    let id_str = id.to_string();
    let fields = PersistFields::from(input);
    let content = build_content_fields(&fields);
    let create_sql =
        format!("CREATE type::thing('atributo_def', '{id_str}') CONTENT {{ {content} }};");

    let event = OutboxEvent::try_new(OutboxEntity::AtributoDef, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.tx();
    tx.run(create_sql);
    tx.emit(event);
    tx.commit().await?;

    fetch_one(db, id).await
}

async fn update_impl(db: &Db, input: UpdateAtributoDef) -> Result<AtributoDef, DomainError> {
    input.validate()?;
    let target_id = input.id.clone();
    let current = fetch_one(db, target_id.clone()).await?;

    if input.clave != current.clave && clave_taken_by_other(db, &input.clave, &target_id).await? {
        return Err(DomainError::conflict(ConflictCode::Conflict));
    }

    let id_str = target_id.to_string();
    let mut fields = PersistFields::from(input);
    fields.activo = current.activo;
    let set_clause = build_set_clause(&fields);
    let stmt = format!("UPDATE type::thing('atributo_def', '{id_str}') SET {set_clause};");

    let event = OutboxEvent::try_new(OutboxEntity::AtributoDef, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.tx();
    tx.run(stmt);
    tx.emit(event);
    tx.commit().await?;

    fetch_one(db, target_id).await
}

async fn set_activo_impl(
    db: &Db,
    id: Id<AtributoDef>,
    activo: bool,
) -> Result<AtributoDef, DomainError> {
    let _ = fetch_one(db, id.clone()).await?;
    let id_str = id.to_string();
    let activo_sql = if activo { "true" } else { "false" };
    let stmt = format!("UPDATE type::thing('atributo_def', '{id_str}') SET activo = {activo_sql};");
    let event = OutboxEvent::try_new(OutboxEntity::AtributoDef, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let mut tx = db.tx();
    tx.run(stmt);
    tx.emit(event);
    tx.commit().await?;
    fetch_one(db, id).await
}

async fn clave_taken_by_other(
    db: &Db,
    clave: &str,
    exclude: &Id<AtributoDef>,
) -> Result<bool, DomainError> {
    let stmt = format!(
        "RETURN array::len((SELECT id FROM atributo_def WHERE clave = {} AND id != type::thing('atributo_def', '{}') LIMIT 1));",
        sql_string(clave),
        exclude
    );
    let len = scalar_i64(db, &stmt).await?.unwrap_or(0);
    Ok(len > 0)
}

async fn fetch_one(db: &Db, id: Id<AtributoDef>) -> Result<AtributoDef, DomainError> {
    let id_str = id.to_string();
    let stmt = format!("SELECT * FROM type::thing('atributo_def', '{id_str}');");
    let rows = query_records(db, &stmt).await?;
    let row = rows.into_iter().next().ok_or(DomainError::not_found())?;
    record_to_def(Some(id), row)
}

async fn fetch_all(db: &Db) -> Result<Vec<AtributoDef>, DomainError> {
    let rows = query_records(
        db,
        "SELECT * FROM atributo_def ORDER BY orden ASC, clave ASC;",
    )
    .await?;
    rows.into_iter()
        .map(|row| record_to_def(None, row))
        .collect()
}

async fn query_records(db: &Db, stmt: &str) -> Result<Vec<AtributoDefRecord>, DomainError> {
    let response = db
        .inner
        .query(stmt)
        .await
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let mut checked = response
        .check()
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let rows: Vec<AtributoDefRecord> = checked
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

fn record_to_def(
    known_id: Option<Id<AtributoDef>>,
    row: AtributoDefRecord,
) -> Result<AtributoDef, DomainError> {
    let id = match known_id {
        Some(id) => id,
        None => record_id_to_domain_id(&row.id)?,
    };
    let tipo = TipoAtributo::parse_wire(&row.tipo)?;
    let creado_en = parse_storage_timestamp(&row.creado_en)?;
    let actualizado_en = parse_storage_timestamp(&row.actualizado_en)?;
    AtributoDef::try_new(AtributoDefDraft {
        id,
        clave: row.clave,
        etiqueta: row.etiqueta,
        tipo,
        unidad: row.unidad,
        opciones: row.opciones,
        facetable: row.facetable,
        buscable: row.buscable,
        orden: row.orden,
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

fn build_content_fields(fields: &PersistFields) -> String {
    let etiqueta_trimmed = fields.etiqueta.trim();
    format!(
        "clave: {}, etiqueta: {}, tipo: {}, unidad: {}, opciones: {}, facetable: {}, buscable: {}, orden: {}, activo: {}",
        sql_string(&fields.clave),
        sql_string(etiqueta_trimmed),
        sql_string(fields.tipo.as_str()),
        sql_option_string(fields.unidad.as_ref()),
        sql_opciones(&fields.opciones),
        fields.facetable,
        fields.buscable,
        fields.orden,
        fields.activo
    )
}

fn build_set_clause(fields: &PersistFields) -> String {
    let etiqueta_trimmed = fields.etiqueta.trim();
    format!(
        "clave = {}, etiqueta = {}, tipo = {}, unidad = {}, opciones = {}, facetable = {}, buscable = {}, orden = {}, activo = {}",
        sql_string(&fields.clave),
        sql_string(etiqueta_trimmed),
        sql_string(fields.tipo.as_str()),
        sql_option_string(fields.unidad.as_ref()),
        sql_opciones(&fields.opciones),
        fields.facetable,
        fields.buscable,
        fields.orden,
        fields.activo
    )
}

fn sql_opciones(opciones: &Option<Vec<String>>) -> String {
    match opciones {
        None => "NONE".into(),
        Some(opts) => {
            let inner: Vec<String> = opts.iter().map(|o| sql_string(o)).collect();
            format!("[{}]", inner.join(", "))
        }
    }
}

fn sql_option_string(s: Option<&String>) -> String {
    match s {
        None => "NONE".into(),
        Some(v) => {
            let t = v.trim();
            if t.is_empty() {
                "NONE".into()
            } else {
                sql_string(t)
            }
        }
    }
}

fn sql_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}
