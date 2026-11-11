//! SurrealDB adapter for `CategoryRepo` (T-02-03).

use std::future::Future;
use std::str::FromStr as _;

use domain::{
    Categoria, CategoriaDraft, CategoryRepo, ConflictCode, DomainError, Id, InvalidInputCode,
    MAX_DEPTH, NewCategoria, OutboxEntity, OutboxEvent, OutboxOp, Slug, Timestamp, UnavailableCode,
};
use serde::Deserialize;
use surrealdb::{RecordId, RecordIdKey};

use crate::Db;
use crate::tx::map_surreal_message;

/// Category persistence against table `categoria`.
pub struct CategoriaRepo<'a> {
    db: &'a Db,
}

impl<'a> CategoriaRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }
}

#[derive(Debug, Deserialize)]
struct CategoriaRecord {
    id: RecordId,
    nombre: String,
    slug: String,
    #[serde(default)]
    padre: Option<RecordId>,
    #[serde(default)]
    ancestros: Vec<RecordId>,
    ruta: Vec<String>,
    #[serde(default = "default_true")]
    activo: bool,
    creado_en: String,
    actualizado_en: String,
}

fn default_true() -> bool {
    true
}

impl<'a> CategoryRepo for CategoriaRepo<'a> {
    fn create(
        &self,
        input: NewCategoria,
    ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
        let db = self.db;
        async move { create_impl(db, input).await }
    }

    fn get(
        &self,
        id: Id<Categoria>,
    ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
        let db = self.db;
        async move { fetch_one(db, id).await }
    }

    fn list_all(&self) -> impl Future<Output = Result<Vec<Categoria>, DomainError>> + Send {
        let db = self.db;
        async move { fetch_all(db).await }
    }
}

async fn create_impl(db: &Db, input: NewCategoria) -> Result<Categoria, DomainError> {
    let slug = match input.slug {
        Some(s) => s,
        None => Slug::from_nombre(&input.nombre)?,
    };

    let (ancestros, ruta, padre) = match input.padre {
        None => {
            if root_slug_taken(db, slug.as_str()).await? {
                return Err(DomainError::conflict(ConflictCode::DuplicateSlug));
            }
            (Vec::new(), vec![input.nombre.trim().to_string()], None)
        }
        Some(padre_id) => {
            let parent = fetch_one(db, padre_id.clone()).await?;
            if parent.depth() >= MAX_DEPTH {
                return Err(DomainError::invalid_input(
                    InvalidInputCode::InvalidInput,
                    vec![],
                ));
            }
            let mut ancestros = parent.ancestros.clone();
            ancestros.push(parent.id);
            let mut ruta = parent.ruta.clone();
            ruta.push(input.nombre.trim().to_string());
            (ancestros, ruta, Some(padre_id))
        }
    };

    let id = Id::<Categoria>::generate();
    let id_str = id.to_string();
    let padre_sql = match padre {
        None => "NONE".to_string(),
        Some(p) => format!("type::thing('categoria', '{}')", p),
    };
    let slug_sql = sql_string(slug.as_str());
    let nombre_sql = sql_string(input.nombre.trim());
    let ruta_sql = sql_string_array(&ruta);
    let ancestros_sql = sql_record_array(&ancestros);

    let create_sql = format!(
        "CREATE type::thing('categoria', '{id_str}') CONTENT {{
            nombre: {nombre_sql},
            slug: {slug_sql},
            padre: {padre_sql},
            ancestros: {ancestros_sql},
            ruta: {ruta_sql},
            activo: true
        }};"
    );

    let event = OutboxEvent::try_new(OutboxEntity::Categoria, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.tx();
    tx.run(create_sql);
    tx.emit(event);
    tx.commit().await?;

    fetch_one(db, id).await
}

async fn root_slug_taken(db: &Db, slug: &str) -> Result<bool, DomainError> {
    let stmt = format!(
        "RETURN array::len((SELECT id FROM categoria WHERE padre IS NONE AND slug = {} LIMIT 1));",
        sql_string(slug)
    );
    let len = scalar_i64(db, &stmt).await?.unwrap_or(0);
    Ok(len > 0)
}

async fn fetch_one(db: &Db, id: Id<Categoria>) -> Result<Categoria, DomainError> {
    let id_str = id.to_string();
    let stmt = format!("SELECT * FROM type::thing('categoria', '{id_str}');");
    let rows = query_records(db, &stmt).await?;
    let row = rows.into_iter().next().ok_or(DomainError::not_found())?;
    record_to_categoria(Some(id), row)
}

async fn fetch_all(db: &Db) -> Result<Vec<Categoria>, DomainError> {
    let rows = query_records(db, "SELECT * FROM categoria;").await?;
    rows.into_iter()
        .map(|row| record_to_categoria(None, row))
        .collect()
}

async fn query_records(db: &Db, stmt: &str) -> Result<Vec<CategoriaRecord>, DomainError> {
    let response = db
        .inner
        .query(stmt)
        .await
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let mut checked = response
        .check()
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let rows: Vec<CategoriaRecord> = checked
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

fn record_to_categoria(
    known_id: Option<Id<Categoria>>,
    row: CategoriaRecord,
) -> Result<Categoria, DomainError> {
    let id = match known_id {
        Some(id) => id,
        None => record_id_to_domain_id(&row.id)?,
    };
    let slug = Slug::parse(&row.slug)?;
    let padre = row.padre.as_ref().map(record_id_to_domain_id).transpose()?;
    let ancestros: Vec<_> = row
        .ancestros
        .iter()
        .map(record_id_to_domain_id)
        .collect::<Result<Vec<_>, _>>()?;
    let creado_en = parse_storage_timestamp(&row.creado_en)?;
    let actualizado_en = parse_storage_timestamp(&row.actualizado_en)?;
    Categoria::try_new(CategoriaDraft {
        id,
        nombre: row.nombre,
        slug,
        padre,
        ancestros,
        ruta: row.ruta,
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

fn sql_string_array(items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let inner: Vec<String> = items.iter().map(|s| sql_string(s)).collect();
    format!("[{}]", inner.join(", "))
}

fn sql_record_array(ids: &[Id<Categoria>]) -> String {
    if ids.is_empty() {
        return "[]".to_string();
    }
    let inner: Vec<String> = ids
        .iter()
        .map(|id| format!("type::thing('categoria', '{}')", id))
        .collect();
    format!("[{}]", inner.join(", "))
}
