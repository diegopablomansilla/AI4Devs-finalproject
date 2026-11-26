//! SurrealDB adapter for `StockLedger` (T-02-24).

use std::future::Future;
use std::str::FromStr as _;
use std::sync::Arc;

use tokio::sync::Mutex;

use domain::{
    Decimal, DomainError, Id, MovimientoStock, MovimientoStockDraft, NewMovimientoStock,
    OutboxEntity, OutboxEvent, OutboxOp, ProductRepo, Producto, StockLedger, Timestamp,
    TipoMovimiento, UnavailableCode, Venta,
};
use serde::Deserialize;
use surrealdb::RecordId;

use crate::Db;
use crate::producto::{ProductoRepo, record_id_to_domain_id};
use crate::tx::map_surreal_message;

/// Stock ledger persistence against `movimiento_stock` and `producto.stock`.
pub struct MovimientoStockRepo {
    db: Arc<Db>,
    /// Serializes ledger commits on the embedded engine (kv-mem races on concurrent RMW).
    ledger_lock: Arc<Mutex<()>>,
}

impl MovimientoStockRepo {
    pub fn new(db: Arc<Db>) -> Self {
        Self {
            db,
            ledger_lock: Arc::new(Mutex::new(())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MovimientoStockRecord {
    id: RecordId,
    producto: RecordId,
    tipo: String,
    cantidad: SurrealDecimal,
    stock_resultante: SurrealDecimal,
    motivo: Option<String>,
    venta: Option<RecordId>,
    terminal: Option<String>,
    creado_en: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SurrealDecimal {
    Text(String),
    Number(f64),
}

impl StockLedger for MovimientoStockRepo {
    fn append(
        &self,
        input: NewMovimientoStock,
    ) -> impl Future<Output = Result<MovimientoStock, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        let lock = Arc::clone(&self.ledger_lock);
        async move {
            let _guard = lock.lock().await;
            append_impl(db, input).await
        }
    }

    fn list_by_producto(
        &self,
        producto: Id<Producto>,
    ) -> impl Future<Output = Result<Vec<MovimientoStock>, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { list_by_producto_impl(db, producto).await }
    }
}

async fn append_impl(
    db: Arc<Db>,
    input: NewMovimientoStock,
) -> Result<MovimientoStock, DomainError> {
    let zero = Decimal::from_str("0")
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let placeholder_ts = Timestamp::from_str("2026-01-01T00:00:00Z")
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
    MovimientoStock::try_new(MovimientoStockDraft {
        id: Id::generate(),
        producto: input.producto.clone(),
        tipo: input.tipo,
        cantidad: input.cantidad,
        stock_resultante: zero,
        motivo: input.motivo.clone(),
        venta: input.venta,
        terminal: input.terminal.clone(),
        creado_en: placeholder_ts,
    })?;

    let _ = ProductoRepo::new(Arc::clone(&db))
        .get(input.producto.clone())
        .await?;

    let mov_id = Id::<MovimientoStock>::generate().to_string();
    let producto_id = input.producto.to_string();
    let producto_thing = product_thing_ref(&producto_id);
    let cantidad_lit = sql_decimal(input.cantidad);
    let tipo = input.tipo.as_str();

    let mut optional_fields = String::new();
    if let Some(ref m) = input.motivo {
        optional_fields.push_str(&format!(", motivo: {}", sql_string(m)));
    }
    if let Some(ref v) = input.venta {
        optional_fields.push_str(&format!(", venta: type::thing('venta', '{v}')"));
    }
    if let Some(ref t) = input.terminal {
        optional_fields.push_str(&format!(", terminal: {}", sql_string(t)));
    }

    let append_sql = format!(
        "LET $producto = {producto_thing}; \
         LET $cantidad = {cantidad_lit}; \
         LET $stock_actual = (SELECT VALUE stock FROM ONLY $producto); \
         LET $nuevo = <decimal>$stock_actual + <decimal>$cantidad; \
         IF $nuevo < 0dec {{ THROW \"stock_negativo\"; }}; \
         CREATE type::thing('movimiento_stock', '{mov_id}') CONTENT {{ \
           producto: $producto, \
           tipo: '{tipo}', \
           cantidad: $cantidad, \
           stock_resultante: $nuevo{optional_fields} \
         }}; \
         UPDATE ONLY $producto SET stock = $nuevo;"
    );

    let event = OutboxEvent::try_new(
        OutboxEntity::Producto,
        OutboxOp::Upsert,
        &producto_id,
        Some("stock".to_string()),
    )
    .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.as_ref().tx();
    tx.run(append_sql);
    tx.emit(event);
    tx.commit().await?;

    fetch_movimiento(db, mov_id).await
}

async fn list_by_producto_impl(
    db: Arc<Db>,
    producto: Id<Producto>,
) -> Result<Vec<MovimientoStock>, DomainError> {
    let id_str = producto.to_string();
    let stmt = format!(
        "SELECT * FROM movimiento_stock WHERE producto = type::thing('producto', '{id_str}') ORDER BY creado_en DESC;"
    );
    let rows = query_movimiento_records(db.as_ref(), &stmt).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(record_to_movimiento(row)?);
    }
    Ok(out)
}

async fn fetch_movimiento(db: Arc<Db>, mov_id: String) -> Result<MovimientoStock, DomainError> {
    let stmt = format!("SELECT * FROM type::thing('movimiento_stock', '{mov_id}');");
    let row = query_movimiento_records(db.as_ref(), &stmt)
        .await?
        .into_iter()
        .next()
        .ok_or(DomainError::unavailable(UnavailableCode::Unavailable))?;
    record_to_movimiento(row)
}

fn record_to_movimiento(row: MovimientoStockRecord) -> Result<MovimientoStock, DomainError> {
    let id = record_id_to_domain_id(&row.id)?;
    let producto = record_id_to_domain_id(&row.producto)?;
    let tipo = TipoMovimiento::parse_wire(&row.tipo)?;
    let cantidad = surreal_decimal_to_domain(&row.cantidad)?;
    let stock_resultante = surreal_decimal_to_domain(&row.stock_resultante)?;
    let venta = match row.venta {
        None => None,
        Some(rid) => Some(record_id_to_domain_id::<Venta>(&rid)?),
    };
    let creado_en = parse_storage_timestamp(&row.creado_en)?;

    MovimientoStock::try_new(MovimientoStockDraft {
        id,
        producto,
        tipo,
        cantidad,
        stock_resultante,
        motivo: row.motivo,
        venta,
        terminal: row.terminal,
        creado_en,
    })
}

async fn query_movimiento_records(
    db: &Db,
    stmt: &str,
) -> Result<Vec<MovimientoStockRecord>, DomainError> {
    let response = db
        .inner
        .query(stmt)
        .await
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let mut checked = response
        .check()
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let rows: Vec<MovimientoStockRecord> = checked
        .take(0)
        .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[stmt.to_string()]))?;
    Ok(rows)
}

fn surreal_decimal_to_domain(v: &SurrealDecimal) -> Result<Decimal, DomainError> {
    match v {
        SurrealDecimal::Text(s) => {
            Decimal::from_str(s).map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))
        }
        SurrealDecimal::Number(n) => Decimal::from_str(&format!("{n}"))
            .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable)),
    }
}

fn parse_storage_timestamp(s: &str) -> Result<Timestamp, DomainError> {
    if s.is_empty() {
        return Err(DomainError::unavailable(UnavailableCode::Unavailable));
    }
    Timestamp::from_str(s)
        .or_else(|_| Timestamp::from_str(&format!("{s}Z")))
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))
}

fn product_thing_ref(id_str: &str) -> String {
    format!("type::thing('producto', '{id_str}')")
}

fn sql_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

fn sql_decimal(d: Decimal) -> String {
    format!("{d}dec")
}
