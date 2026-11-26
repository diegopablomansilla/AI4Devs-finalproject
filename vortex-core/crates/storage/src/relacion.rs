//! SurrealDB adapter for `RelationRepo::create` (T-04-03).

use std::future::Future;
use std::str::FromStr as _;
use std::sync::Arc;

use domain::{
    DomainError, GrupoBidireccional, Id, NewRelacion, Producto, Relacion, RelacionDraft,
    RelationRepo, Timestamp, TipoRelacion, UnavailableCode, UpdateRelacion,
};
use serde::Deserialize;
use surrealdb::{RecordId, RecordIdKey};

use crate::Db;
use crate::tx::map_surreal_message;

/// Product relation persistence (`similar_a`, `sustituible_por`, `complementa`).
pub struct RelacionRepo {
    db: Arc<Db>,
}

impl RelacionRepo {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

impl RelationRepo for RelacionRepo {
    fn create(
        &self,
        input: NewRelacion,
    ) -> impl Future<Output = Result<Relacion, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { create_impl(db.as_ref(), input).await }
    }

    async fn update(&self, _input: UpdateRelacion) -> Result<Relacion, DomainError> {
        Err(DomainError::unavailable(UnavailableCode::Unavailable))
    }

    async fn delete(
        &self,
        _tipo: TipoRelacion,
        _id: Id<domain::Relacion>,
    ) -> Result<(), DomainError> {
        Err(DomainError::unavailable(UnavailableCode::Unavailable))
    }

    async fn list_for_product(
        &self,
        _producto: Id<Producto>,
    ) -> Result<Vec<Relacion>, DomainError> {
        Err(DomainError::unavailable(UnavailableCode::Unavailable))
    }
}

#[derive(Debug, Deserialize)]
struct RelacionRecord {
    id: RecordId,
    #[serde(rename = "in")]
    r#in: RecordId,
    out: RecordId,
    prioridad: i32,
    nota: Option<String>,
    grupo_bidireccional: Option<String>,
    creado_en: String,
    actualizado_en: String,
}

async fn create_impl(db: &Db, input: NewRelacion) -> Result<Relacion, DomainError> {
    let bidireccional = input.resolved_bidireccional();
    let tabla = input.tipo.as_str();
    let edge_id = Id::<domain::Relacion>::generate();
    let grupo = if bidireccional {
        Some(Id::<GrupoBidireccional>::generate())
    } else {
        None
    };

    const PLACEHOLDER_TS: &str = "2026-09-24T21:33:00Z";
    let ts: Timestamp = PLACEHOLDER_TS
        .parse()
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    Relacion::try_new(RelacionDraft {
        id: edge_id.clone(),
        origen: input.origen.clone(),
        destino: input.destino.clone(),
        tipo: input.tipo,
        prioridad: input.prioridad,
        nota: input.nota.clone(),
        grupo_bidireccional: grupo,
        creado_en: ts,
        actualizado_en: ts,
    })?;

    let origen = input.origen.to_string();
    let destino = input.destino.to_string();
    let edge = edge_id.to_string();
    let prioridad = input.prioridad;
    let nota_sql = optional_nota_sql(input.nota.as_deref());

    let mut tx = db.tx();

    if bidireccional {
        let g = grupo.expect("grupo when bidireccional");
        let g_str = g.to_string();
        let edge_rev = Id::<domain::Relacion>::generate().to_string();
        let stmt_ab = relate_stmt(
            tabla,
            &edge,
            &origen,
            &destino,
            prioridad,
            &nota_sql,
            Some(&g_str),
        );
        let stmt_ba = relate_stmt(
            tabla,
            &edge_rev,
            &destino,
            &origen,
            prioridad,
            &nota_sql,
            Some(&g_str),
        );
        tx.run(stmt_ab);
        tx.run(stmt_ba);
    } else {
        let stmt = relate_stmt(tabla, &edge, &origen, &destino, prioridad, &nota_sql, None);
        tx.run(stmt);
    }

    tx.commit().await?;

    fetch_outgoing(db, input.tipo, input.origen, input.destino).await
}

fn relate_stmt(
    tabla: &str,
    edge_id: &str,
    origen: &str,
    destino: &str,
    prioridad: i32,
    nota_sql: &str,
    grupo: Option<&str>,
) -> String {
    let mut set = format!("prioridad = {prioridad}");
    if !nota_sql.is_empty() {
        set.push_str(nota_sql);
    }
    if let Some(g) = grupo {
        set.push_str(&format!(", grupo_bidireccional = {}", sql_string(g)));
    }
    format!("RELATE producto:{origen} -> {tabla}:{edge_id} -> producto:{destino} SET {set};")
}

fn optional_nota_sql(nota: Option<&str>) -> String {
    match nota {
        Some(n) if !n.is_empty() => format!(", nota = {}", sql_string(n)),
        _ => String::new(),
    }
}

async fn fetch_outgoing(
    db: &Db,
    tipo: TipoRelacion,
    origen: Id<Producto>,
    destino: Id<Producto>,
) -> Result<Relacion, DomainError> {
    let tabla = tipo.as_str();
    let stmt = format!(
        "SELECT * FROM {tabla} \
         WHERE in = type::thing('producto', '{}') \
         AND out = type::thing('producto', '{}') LIMIT 1;",
        origen, destino
    );
    let response = db
        .inner
        .query(&stmt)
        .await
        .map_err(|e| map_surreal_message(&e.to_string(), std::slice::from_ref(&stmt)))?;
    let mut checked = response
        .check()
        .map_err(|e| map_surreal_message(&e.to_string(), std::slice::from_ref(&stmt)))?;
    let rows: Vec<RelacionRecord> = checked
        .take(0)
        .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[stmt]))?;
    let row = rows.into_iter().next().ok_or_else(DomainError::not_found)?;
    record_to_relacion(tipo, row)
}

fn record_to_relacion(tipo: TipoRelacion, row: RelacionRecord) -> Result<Relacion, DomainError> {
    let id = record_id_to_domain_id::<domain::Relacion>(&row.id)?;
    let origen = record_id_to_domain_id::<Producto>(&row.r#in)?;
    let destino = record_id_to_domain_id::<Producto>(&row.out)?;
    let grupo_bidireccional = match row.grupo_bidireccional {
        Some(s) if !s.is_empty() => Some(
            Id::<GrupoBidireccional>::from_str(&s)
                .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?,
        ),
        _ => None,
    };
    let creado_en = parse_storage_timestamp(&row.creado_en)?;
    let actualizado_en = parse_storage_timestamp(&row.actualizado_en)?;

    Relacion::try_new(RelacionDraft {
        id,
        origen,
        destino,
        tipo,
        prioridad: row.prioridad,
        nota: row.nota,
        grupo_bidireccional,
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
