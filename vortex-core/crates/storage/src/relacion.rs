//! SurrealDB adapter for `RelationRepo` (create T-04-03; update/delete/list T-04-04).

use std::future::Future;
use std::str::FromStr as _;
use std::sync::Arc;

use domain::{
    DestinoResumen, DomainError, GrupoBidireccional, Id, NewRelacion, Producto, Relacion,
    RelacionConDestino, RelacionDraft, RelacionesDeProducto, RelationRepo, Timestamp, TipoRelacion,
    UnavailableCode, UpdateRelacion,
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

    fn update(
        &self,
        input: UpdateRelacion,
    ) -> impl Future<Output = Result<Relacion, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { update_impl(db.as_ref(), input).await }
    }

    fn delete(
        &self,
        tipo: TipoRelacion,
        id: Id<domain::Relacion>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { delete_impl(db.as_ref(), tipo, id).await }
    }

    fn list_for_product(
        &self,
        producto: Id<Producto>,
    ) -> impl Future<Output = Result<RelacionesDeProducto, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { list_for_product_impl(db.as_ref(), producto).await }
    }

    fn list_incoming_for_product(
        &self,
        producto: Id<Producto>,
    ) -> impl Future<Output = Result<RelacionesDeProducto, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { list_incoming_for_product_impl(db.as_ref(), producto).await }
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

#[derive(Debug, Deserialize)]
struct OutProductoFields {
    sku: String,
    nombre: String,
    activo: bool,
}

#[derive(Debug, Deserialize)]
struct RelacionListRecord {
    id: RecordId,
    #[serde(rename = "in")]
    r#in: RecordId,
    out: RecordId,
    prioridad: i32,
    nota: Option<String>,
    grupo_bidireccional: Option<String>,
    creado_en: String,
    actualizado_en: String,
    out_sku: Option<String>,
    out_nombre: Option<String>,
    out_activo: Option<bool>,
    #[serde(default)]
    out_fetched: Option<OutProductoFields>,
    in_sku: Option<String>,
    in_nombre: Option<String>,
    in_activo: Option<bool>,
    #[serde(default)]
    in_fetched: Option<OutProductoFields>,
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

async fn update_impl(db: &Db, input: UpdateRelacion) -> Result<Relacion, DomainError> {
    let current = fetch_by_id(db, input.tipo, &input.id).await?;
    let merged_prioridad = input.prioridad.unwrap_or(current.prioridad);
    let merged_nota = match &input.nota {
        None => current.nota.clone(),
        Some(None) => None,
        Some(Some(s)) => Some(s.clone()),
    };

    Relacion::try_new(RelacionDraft {
        id: current.id.clone(),
        origen: current.origen.clone(),
        destino: current.destino.clone(),
        tipo: current.tipo,
        prioridad: merged_prioridad,
        nota: merged_nota.clone(),
        grupo_bidireccional: current.grupo_bidireccional,
        creado_en: current.creado_en,
        actualizado_en: current.actualizado_en,
    })?;

    let set_clause = build_update_set_clause(&input);
    if set_clause.is_empty() {
        return Ok(current);
    }

    let tabla = input.tipo.as_str();
    let mut tx = db.tx();

    let stmt = if let Some(grupo) = current.grupo_bidireccional {
        let g = grupo.to_string();
        format!(
            "UPDATE {tabla} SET {set_clause} WHERE grupo_bidireccional = {};",
            sql_string(&g)
        )
    } else {
        format!(
            "UPDATE type::thing('{tabla}', '{}') SET {set_clause};",
            input.id
        )
    };
    tx.run(stmt);
    tx.commit().await?;

    fetch_by_id(db, input.tipo, &input.id).await
}

async fn delete_impl(
    db: &Db,
    tipo: TipoRelacion,
    id: Id<domain::Relacion>,
) -> Result<(), DomainError> {
    let current = fetch_by_id(db, tipo, &id).await?;
    let tabla = tipo.as_str();
    let mut tx = db.tx();

    if let Some(grupo) = current.grupo_bidireccional {
        let g = grupo.to_string();
        tx.run(format!(
            "DELETE {tabla} WHERE grupo_bidireccional = {};",
            sql_string(&g)
        ));
    } else {
        tx.run(format!("DELETE type::thing('{tabla}', '{}');", id));
    }
    tx.commit().await
}

async fn list_incoming_for_product_impl(
    db: &Db,
    producto: Id<Producto>,
) -> Result<RelacionesDeProducto, DomainError> {
    let pid = producto.to_string();
    let mut similar_a = list_incoming_edges_for_table(db, TipoRelacion::SimilarA, &pid).await?;
    let mut sustituible_por =
        list_incoming_edges_for_table(db, TipoRelacion::SustituiblePor, &pid).await?;
    let mut complementa =
        list_incoming_edges_for_table(db, TipoRelacion::Complementa, &pid).await?;

    sort_relacion_items(&mut similar_a);
    sort_relacion_items(&mut sustituible_por);
    sort_relacion_items(&mut complementa);

    Ok(RelacionesDeProducto {
        similar_a,
        sustituible_por,
        complementa,
    })
}

async fn list_for_product_impl(
    db: &Db,
    producto: Id<Producto>,
) -> Result<RelacionesDeProducto, DomainError> {
    let pid = producto.to_string();
    let mut similar_a = list_edges_for_table(db, TipoRelacion::SimilarA, &pid).await?;
    let mut sustituible_por = list_edges_for_table(db, TipoRelacion::SustituiblePor, &pid).await?;
    let mut complementa = list_edges_for_table(db, TipoRelacion::Complementa, &pid).await?;

    sort_relacion_items(&mut similar_a);
    sort_relacion_items(&mut sustituible_por);
    sort_relacion_items(&mut complementa);

    Ok(RelacionesDeProducto {
        similar_a,
        sustituible_por,
        complementa,
    })
}

async fn list_edges_for_table(
    db: &Db,
    tipo: TipoRelacion,
    producto_id: &str,
) -> Result<Vec<RelacionConDestino>, DomainError> {
    let tabla = tipo.as_str();
    let stmt = format!(
        "SELECT id, in, out, prioridad, nota, grupo_bidireccional, creado_en, actualizado_en, \
         out.sku AS out_sku, out.nombre AS out_nombre, out.activo AS out_activo \
         FROM {tabla} \
         WHERE in = type::thing('producto', '{producto_id}');"
    );
    let rows = query_relacion_list(db, &stmt).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let relacion = record_to_relacion(tipo, row_to_edge_record(&row))?;
        let destino = destino_from_list_row(&row)?;
        out.push(RelacionConDestino { relacion, destino });
    }
    Ok(out)
}

async fn list_incoming_edges_for_table(
    db: &Db,
    tipo: TipoRelacion,
    producto_id: &str,
) -> Result<Vec<RelacionConDestino>, DomainError> {
    let tabla = tipo.as_str();
    let stmt = format!(
        "SELECT id, in, out, prioridad, nota, grupo_bidireccional, creado_en, actualizado_en, \
         in.sku AS in_sku, in.nombre AS in_nombre, in.activo AS in_activo \
         FROM {tabla} \
         WHERE out = type::thing('producto', '{producto_id}');"
    );
    let rows = query_relacion_list(db, &stmt).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let relacion = record_to_relacion(tipo, row_to_edge_record(&row))?;
        let destino = destino_from_incoming_list_row(&row)?;
        out.push(RelacionConDestino { relacion, destino });
    }
    Ok(out)
}

fn destino_from_incoming_list_row(row: &RelacionListRecord) -> Result<DestinoResumen, DomainError> {
    let id = record_id_to_domain_id::<Producto>(&row.r#in)?;
    if let Some(fields) = &row.in_fetched {
        return Ok(DestinoResumen {
            id,
            sku: fields.sku.clone(),
            nombre: fields.nombre.clone(),
            activo: fields.activo,
        });
    }
    let sku = row
        .in_sku
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let nombre = row
        .in_nombre
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let activo = row.in_activo.unwrap_or(true);
    Ok(DestinoResumen {
        id,
        sku,
        nombre,
        activo,
    })
}

fn row_to_edge_record(row: &RelacionListRecord) -> RelacionRecord {
    RelacionRecord {
        id: row.id.clone(),
        r#in: row.r#in.clone(),
        out: row.out.clone(),
        prioridad: row.prioridad,
        nota: row.nota.clone(),
        grupo_bidireccional: row.grupo_bidireccional.clone(),
        creado_en: row.creado_en.clone(),
        actualizado_en: row.actualizado_en.clone(),
    }
}

fn destino_from_list_row(row: &RelacionListRecord) -> Result<DestinoResumen, DomainError> {
    let id = record_id_to_domain_id::<Producto>(&row.out)?;
    if let Some(fields) = &row.out_fetched {
        return Ok(DestinoResumen {
            id,
            sku: fields.sku.clone(),
            nombre: fields.nombre.clone(),
            activo: fields.activo,
        });
    }
    let sku = row
        .out_sku
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let nombre = row
        .out_nombre
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let activo = row.out_activo.unwrap_or(true);
    Ok(DestinoResumen {
        id,
        sku,
        nombre,
        activo,
    })
}

fn sort_relacion_items(items: &mut [RelacionConDestino]) {
    items.sort_by(|a, b| {
        a.relacion
            .prioridad
            .cmp(&b.relacion.prioridad)
            .then_with(|| a.relacion.id.to_string().cmp(&b.relacion.id.to_string()))
    });
}

fn build_update_set_clause(input: &UpdateRelacion) -> String {
    let mut parts = Vec::new();
    if let Some(prioridad) = input.prioridad {
        parts.push(format!("prioridad = {prioridad}"));
    }
    if let Some(nota_patch) = &input.nota {
        match nota_patch {
            None => parts.push("nota = NONE".into()),
            Some(s) => parts.push(format!("nota = {}", sql_string(s))),
        }
    }
    parts.join(", ")
}

async fn fetch_by_id(
    db: &Db,
    tipo: TipoRelacion,
    id: &Id<domain::Relacion>,
) -> Result<Relacion, DomainError> {
    let tabla = tipo.as_str();
    let stmt = format!(
        "SELECT * FROM {tabla} WHERE id = type::thing('{tabla}', '{}') LIMIT 1;",
        id
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
    rows.into_iter()
        .next()
        .map(|row| record_to_relacion(tipo, row))
        .transpose()?
        .ok_or_else(DomainError::not_found)
}

async fn query_relacion_list(db: &Db, stmt: &str) -> Result<Vec<RelacionListRecord>, DomainError> {
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
