//! SurrealDB adapter for `ProductRepo` (T-02-18, T-02-19).

use std::future::Future;
use std::str::FromStr as _;
use std::sync::Arc;

use domain::{
    AtributoDef, AtributoFicha, AtributoValor, AttributeDefRepo, BrandRepo, Categoria,
    CategoryRepo, Codigo, ConflictCode, Decimal, DomainError, FichaProducto, FiltroProducto, Id,
    InvalidInputCode, Marca, MovimientoStock, NewProducto, OutboxEntity, OutboxEvent, OutboxOp,
    ProductRepo, Producto, ProductoDraft, ResumenProducto, Timestamp, TipoCodigo, Ubicacion,
    UnavailableCode, UnidadMedida, UpdateProducto, ValorAtributo,
};
use serde::Deserialize;
use surrealdb::{RecordId, RecordIdKey};

use crate::Db;
use crate::atributo_def::AtributoDefRepo;
use crate::categoria::CategoriaRepo;
use crate::marca::MarcaRepo;
use crate::tx::map_surreal_message;

/// Product persistence against table `producto`.
pub struct ProductoRepo {
    db: Arc<Db>,
}

impl ProductoRepo {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }
}

#[derive(Debug, Deserialize)]
struct ProductoRecord {
    id: RecordId,
    sku: String,
    nombre: String,
    tipo: String,
    descripcion: Option<String>,
    marca: Option<RecordId>,
    categoria: RecordId,
    #[serde(default)]
    atributos: Vec<AtributoValorRecord>,
    #[serde(default)]
    codigos: Vec<CodigoRecord>,
    unidad_medida: String,
    precio: SurrealDecimal,
    ubicacion: Option<UbicacionRecord>,
    stock: SurrealDecimal,
    #[serde(default = "default_true")]
    activo: bool,
    version: i32,
    creado_en: String,
    actualizado_en: String,
}

#[derive(Debug, Deserialize)]
struct AtributoValorRecord {
    def: RecordId,
    valor: SurrealValor,
}

#[derive(Debug, Deserialize)]
struct CodigoRecord {
    tipo: String,
    valor: String,
}

#[derive(Debug, Deserialize)]
struct UbicacionRecord {
    pasillo: Option<String>,
    estante: Option<String>,
    posicion: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SurrealDecimal {
    Text(String),
    Number(f64),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum SurrealValor {
    Text(String),
    Number(f64),
    Bool(bool),
}

fn default_true() -> bool {
    true
}

impl ProductRepo for ProductoRepo {
    fn create(
        &self,
        input: NewProducto,
    ) -> impl Future<Output = Result<Producto, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { create_impl(db, input).await }
    }

    fn get(
        &self,
        id: Id<domain::Producto>,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { fetch_ficha(db, id).await }
    }

    fn update(
        &self,
        input: UpdateProducto,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { update_impl(db, input).await }
    }

    fn set_activo(
        &self,
        id: Id<domain::Producto>,
        activo: bool,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { set_activo_impl(db, id, activo).await }
    }

    fn list(
        &self,
        filtro: FiltroProducto,
        after: Option<Id<domain::Producto>>,
        take: u32,
    ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
        let db = Arc::clone(&self.db);
        async move { list_impl(db, filtro, after, take).await }
    }
}

async fn list_impl(
    db: Arc<Db>,
    filtro: FiltroProducto,
    after: Option<Id<Producto>>,
    take: u32,
) -> Result<Vec<ResumenProducto>, DomainError> {
    let mut conditions = vec![if filtro.activo {
        "activo = true".to_string()
    } else {
        "activo = false".to_string()
    }];
    if let Some(marca) = &filtro.marca {
        conditions.push(format!("marca = type::thing('marca', '{marca}')"));
    }
    if let Some(categoria) = &filtro.categoria {
        let c = format!("type::thing('categoria', '{categoria}')");
        conditions.push(format!(
            "(categoria = {c} OR categoria.ancestros CONTAINS {c})"
        ));
    }
    let where_clause = conditions.join(" AND ");
    let stmt = format!("SELECT * FROM producto WHERE {where_clause};");
    let mut rows = query_product_records(db.as_ref(), &stmt).await?;
    rows.sort_by(|a, b| {
        let a_id = record_id_to_domain_id::<Producto>(&a.id).expect("product id");
        let b_id = record_id_to_domain_id::<Producto>(&b.id).expect("product id");
        a_id.to_string().cmp(&b_id.to_string())
    });
    if let Some(after_id) = after {
        rows.retain(|row| {
            let id = record_id_to_domain_id::<Producto>(&row.id).expect("product id");
            id.to_string() > after_id.to_string()
        });
    }
    rows.truncate(take as usize);
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(record_to_resumen(Arc::clone(&db), row).await?);
    }
    Ok(out)
}

async fn record_to_resumen(
    db: Arc<Db>,
    row: ProductoRecord,
) -> Result<ResumenProducto, DomainError> {
    let id = record_id_to_domain_id(&row.id)?;
    let categoria_id = record_id_to_domain_id::<Categoria>(&row.categoria)?;
    let categoria = CategoriaRepo::new(Arc::clone(&db))
        .get(categoria_id)
        .await?;
    let marca_nombre = match &row.marca {
        None => None,
        Some(rid) => {
            let marca_id = record_id_to_domain_id::<Marca>(rid)?;
            Some(MarcaRepo::new(Arc::clone(&db)).get(marca_id).await?.nombre)
        }
    };
    let precio = surreal_decimal_to_domain(&row.precio)?;
    let stock = surreal_decimal_to_domain(&row.stock)?;
    Ok(ResumenProducto {
        id,
        sku: row.sku,
        nombre: row.nombre,
        tipo: row.tipo,
        precio,
        stock,
        activo: row.activo,
        marca_nombre,
        categoria_ruta: categoria.ruta,
    })
}

async fn set_activo_impl(
    db: Arc<Db>,
    id: Id<domain::Producto>,
    activo: bool,
) -> Result<FichaProducto, DomainError> {
    let _ = fetch_producto(Arc::clone(&db), id.clone()).await?;
    let id_str = id.to_string();
    let activo_sql = if activo { "true" } else { "false" };
    let producto_thing = product_thing_ref(&id_str);
    let stmt = format!("UPDATE {producto_thing} SET activo = {activo_sql};");
    let event = OutboxEvent::try_new(OutboxEntity::Producto, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;
    let mut tx = db.as_ref().tx();
    tx.run(stmt);
    tx.emit(event);
    tx.commit().await?;
    fetch_ficha(db, id).await
}

async fn create_impl(db: Arc<Db>, input: NewProducto) -> Result<Producto, DomainError> {
    validate_create_input(&input)?;

    let id = Id::<Producto>::generate();
    let id_str = id.to_string();
    let producto_thing = product_thing_ref(&id_str);
    let content = build_create_content(&input)?;
    let create_sql = format!("CREATE {producto_thing} CONTENT {{ {content} }};");

    let zero = Decimal::from_str("0").map_err(|_| invalid_create())?;
    let has_stock = input.stock_inicial > zero;

    let motivo = if has_stock {
        Some("stock".to_string())
    } else {
        None
    };
    let event = OutboxEvent::try_new(OutboxEntity::Producto, OutboxOp::Upsert, &id_str, motivo)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.as_ref().tx();
    tx.run(create_sql);
    if has_stock {
        let mov_id = Id::<MovimientoStock>::generate().to_string();
        let stock_lit = sql_decimal(input.stock_inicial);
        let terminal_sql = match &input.terminal {
            Some(t) => format!(", terminal: {}", sql_string(t)),
            None => String::new(),
        };
        let mov_sql = format!(
            "CREATE type::thing('movimiento_stock', '{mov_id}') CONTENT {{ \
             producto: {producto_thing}, \
             tipo: 'inicial', \
             cantidad: {stock_lit}, \
             stock_resultante: {stock_lit}{terminal_sql} \
             }};"
        );
        tx.run(mov_sql);
        tx.run(format!("UPDATE {producto_thing} SET stock = {stock_lit};"));
    }
    tx.emit(event);
    tx.commit().await?;

    fetch_producto(db, id).await
}

async fn update_impl(db: Arc<Db>, input: UpdateProducto) -> Result<FichaProducto, DomainError> {
    validate_update_input(&input)?;
    let current = fetch_producto(Arc::clone(&db), input.id.clone()).await?;
    if current.version != input.version {
        return Err(DomainError::conflict(ConflictCode::VersionConflict));
    }

    let id_str = input.id.to_string();
    let producto_thing = product_thing_ref(&id_str);
    let set_body = build_update_set(&input)?;
    let version = input.version;
    let update_sql = format!(
        "UPDATE {producto_thing} \
         SET {set_body}, version += 1 \
         WHERE version = {version};"
    );

    let event = OutboxEvent::try_new(OutboxEntity::Producto, OutboxOp::Upsert, &id_str, None)
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))?;

    let mut tx = db.as_ref().tx();
    tx.run(update_sql);
    tx.emit(event);
    tx.commit().await?;

    let ficha = fetch_ficha(db, input.id).await?;
    let p = &ficha.producto;
    if p.version != input.version + 1
        || p.sku.as_str() != input.sku.trim()
        || p.nombre.trim() != input.nombre.trim()
        || p.precio != input.precio
    {
        return Err(DomainError::conflict(ConflictCode::VersionConflict));
    }
    Ok(ficha)
}

fn validate_update_input(input: &UpdateProducto) -> Result<(), DomainError> {
    if input.sku.trim().is_empty() || input.nombre.trim().is_empty() {
        return Err(invalid_create());
    }
    if input.version < 1 {
        return Err(invalid_create());
    }
    let zero = Decimal::from_str("0").map_err(|_| invalid_create())?;
    if input.precio < zero {
        return Err(invalid_create());
    }
    Ok(())
}

fn build_update_set(input: &UpdateProducto) -> Result<String, DomainError> {
    let sku = sql_string(input.sku.trim());
    let nombre = sql_string(input.nombre.trim());
    let tipo = sql_string(input.tipo.trim());
    let descripcion = match &input.descripcion {
        None => "NONE".to_string(),
        Some(d) => sql_string(d),
    };
    let marca = match &input.marca {
        None => "NONE".to_string(),
        Some(id) => format!("type::thing('marca', '{}')", id),
    };
    let categoria = format!("type::thing('categoria', '{}')", input.categoria);
    let atributos = build_atributos_sql(&input.atributos)?;
    let codigos = build_codigos_sql(&input.codigos);
    let unidad = sql_string(input.unidad_medida.as_str());
    let precio = sql_decimal(input.precio);
    let ubicacion = build_ubicacion_sql(&input.ubicacion);

    Ok(format!(
        "sku = {sku}, \
         nombre = {nombre}, \
         tipo = {tipo}, \
         descripcion = {descripcion}, \
         marca = {marca}, \
         categoria = {categoria}, \
         atributos = {atributos}, \
         codigos = {codigos}, \
         unidad_medida = {unidad}, \
         precio = {precio}, \
         ubicacion = {ubicacion}"
    ))
}

fn validate_create_input(input: &NewProducto) -> Result<(), DomainError> {
    if input.sku.trim().is_empty() || input.nombre.trim().is_empty() {
        return Err(invalid_create());
    }
    let zero = Decimal::from_str("0").map_err(|_| invalid_create())?;
    if input.stock_inicial < zero {
        return Err(invalid_create());
    }
    if input.precio < zero {
        return Err(invalid_create());
    }
    Ok(())
}

fn invalid_create() -> DomainError {
    DomainError::invalid_input(InvalidInputCode::InvalidInput, vec![])
}

fn build_create_content(input: &NewProducto) -> Result<String, DomainError> {
    let sku = sql_string(input.sku.trim());
    let nombre = sql_string(input.nombre.trim());
    let tipo = sql_string(input.tipo.trim());
    let descripcion = match &input.descripcion {
        None => "NONE".to_string(),
        Some(d) => sql_string(d),
    };
    let marca = match &input.marca {
        None => "NONE".to_string(),
        Some(id) => format!("type::thing('marca', '{}')", id),
    };
    let categoria = format!("type::thing('categoria', '{}')", input.categoria);
    let atributos = build_atributos_sql(&input.atributos)?;
    let codigos = build_codigos_sql(&input.codigos);
    let unidad = sql_string(input.unidad_medida.as_str());
    let precio = sql_decimal(input.precio);
    let ubicacion = build_ubicacion_sql(&input.ubicacion);

    Ok(format!(
        "sku: {sku}, \
         nombre: {nombre}, \
         tipo: {tipo}, \
         descripcion: {descripcion}, \
         marca: {marca}, \
         categoria: {categoria}, \
         atributos: {atributos}, \
         codigos: {codigos}, \
         unidad_medida: {unidad}, \
         precio: {precio}, \
         ubicacion: {ubicacion}, \
         activo: true"
    ))
}

fn build_atributos_sql(atributos: &[AtributoValor]) -> Result<String, DomainError> {
    if atributos.is_empty() {
        return Ok("[]".to_string());
    }
    let mut parts = Vec::with_capacity(atributos.len());
    for attr in atributos {
        let def = format!("type::thing('atributo_def', '{}')", attr.def);
        let valor = valor_to_sql(&attr.valor)?;
        parts.push(format!("{{ def: {def}, valor: {valor} }}"));
    }
    Ok(format!("[{}]", parts.join(", ")))
}

fn build_codigos_sql(codigos: &[Codigo]) -> String {
    if codigos.is_empty() {
        return "[]".to_string();
    }
    let parts: Vec<String> = codigos
        .iter()
        .map(|c| {
            format!(
                "{{ tipo: {}, valor: {} }}",
                sql_string(c.tipo.as_str()),
                sql_string(&c.valor)
            )
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

fn build_ubicacion_sql(ubicacion: &Option<Ubicacion>) -> String {
    match ubicacion {
        None => "NONE".to_string(),
        Some(u) => {
            let pasillo = option_sql_string(&u.pasillo);
            let estante = option_sql_string(&u.estante);
            let posicion = option_sql_string(&u.posicion);
            format!("{{ pasillo: {pasillo}, estante: {estante}, posicion: {posicion} }}")
        }
    }
}

fn option_sql_string(value: &Option<String>) -> String {
    match value {
        None => "NONE".to_string(),
        Some(s) => sql_string(s),
    }
}

fn valor_to_sql(valor: &ValorAtributo) -> Result<String, DomainError> {
    Ok(match valor {
        ValorAtributo::Texto(s) => sql_string(s),
        ValorAtributo::Numero(d) => sql_decimal(*d),
        ValorAtributo::Booleano(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
    })
}

async fn fetch_producto(db: Arc<Db>, id: Id<Producto>) -> Result<Producto, DomainError> {
    let row = fetch_product_record(Arc::clone(&db), &id).await?;
    record_to_producto(db, Some(id), row).await
}

async fn fetch_product_record(
    db: Arc<Db>,
    id: &Id<Producto>,
) -> Result<ProductoRecord, DomainError> {
    let id_str = id.to_string();
    let stmt = format!("SELECT * FROM type::thing('producto', '{id_str}');");
    query_product_records(db.as_ref(), &stmt)
        .await?
        .into_iter()
        .next()
        .ok_or(DomainError::not_found())
}

async fn fetch_ficha(db: Arc<Db>, id: Id<Producto>) -> Result<FichaProducto, DomainError> {
    let row = fetch_product_record(Arc::clone(&db), &id).await?;
    let producto = record_to_producto(Arc::clone(&db), Some(id.clone()), row).await?;
    hydrate_ficha(db, producto).await
}

async fn hydrate_ficha(db: Arc<Db>, producto: Producto) -> Result<FichaProducto, DomainError> {
    let categoria = CategoriaRepo::new(Arc::clone(&db))
        .get(producto.categoria.clone())
        .await?;
    let marca = match &producto.marca {
        None => None,
        Some(marca_id) => Some(
            MarcaRepo::new(Arc::clone(&db))
                .get(marca_id.clone())
                .await?,
        ),
    };

    let defs = load_atributo_defs_for_product(Arc::clone(&db), &producto.atributos).await?;
    let atributos = producto
        .atributos
        .iter()
        .map(|av| {
            let def = defs
                .iter()
                .find(|d| d.id == av.def)
                .ok_or(DomainError::unavailable(UnavailableCode::Unavailable))?;
            Ok(AtributoFicha {
                def: av.def.clone(),
                clave: def.clave.clone(),
                etiqueta: def.etiqueta.clone(),
                unidad: def.unidad.clone(),
                valor: av.valor.clone(),
            })
        })
        .collect::<Result<Vec<_>, DomainError>>()?;

    Ok(FichaProducto {
        producto,
        marca,
        categoria,
        atributos,
    })
}

async fn load_atributo_defs_for_product(
    db: Arc<Db>,
    attrs: &[AtributoValor],
) -> Result<Vec<AtributoDef>, DomainError> {
    if attrs.is_empty() {
        return Ok(vec![]);
    }
    let repo = AtributoDefRepo::new(db);
    let mut defs = Vec::new();
    let mut seen: Vec<Id<AtributoDef>> = Vec::new();
    for attr in attrs {
        if seen.contains(&attr.def) {
            continue;
        }
        seen.push(attr.def.clone());
        defs.push(repo.get(attr.def.clone()).await?);
    }
    Ok(defs)
}

async fn record_to_producto(
    db: Arc<Db>,
    known_id: Option<Id<Producto>>,
    row: ProductoRecord,
) -> Result<Producto, DomainError> {
    let id = match known_id {
        Some(id) => id,
        None => record_id_to_domain_id(&row.id)?,
    };
    let defs = load_atributo_defs(db, &row.atributos).await?;
    let atributos = map_atributos(&row.atributos)?;
    let codigos = row
        .codigos
        .iter()
        .map(|c| {
            let tipo = TipoCodigo::parse_wire(&c.tipo)?;
            Codigo::try_new(tipo, c.valor.clone())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let unidad_medida = UnidadMedida::parse_wire(&row.unidad_medida)?;
    let precio = surreal_decimal_to_domain(&row.precio)?;
    let stock = surreal_decimal_to_domain(&row.stock)?;
    let marca = match row.marca {
        None => None,
        Some(rid) => Some(record_id_to_domain_id::<Marca>(&rid)?),
    };
    let categoria = record_id_to_domain_id(&row.categoria)?;
    let ubicacion = row.ubicacion.map(|u| Ubicacion {
        pasillo: u.pasillo,
        estante: u.estante,
        posicion: u.posicion,
    });
    let creado_en = parse_storage_timestamp(&row.creado_en)?;
    let actualizado_en = parse_storage_timestamp(&row.actualizado_en)?;

    Producto::try_new(
        ProductoDraft {
            id,
            sku: row.sku,
            nombre: row.nombre,
            tipo: row.tipo,
            descripcion: row.descripcion,
            marca,
            categoria,
            atributos,
            codigos,
            unidad_medida,
            precio,
            ubicacion,
            stock,
            activo: row.activo,
            version: row.version,
            creado_en,
            actualizado_en,
        },
        &defs,
    )
}

async fn load_atributo_defs(
    db: Arc<Db>,
    attrs: &[AtributoValorRecord],
) -> Result<Vec<AtributoDef>, DomainError> {
    if attrs.is_empty() {
        return Ok(vec![]);
    }
    let repo = AtributoDefRepo::new(db);
    let mut defs = Vec::new();
    let mut seen: Vec<Id<AtributoDef>> = Vec::new();
    for attr in attrs {
        let def_id = record_id_to_domain_id(&attr.def)?;
        if seen.contains(&def_id) {
            continue;
        }
        seen.push(def_id.clone());
        defs.push(repo.get(def_id).await?);
    }
    Ok(defs)
}

fn map_atributos(attrs: &[AtributoValorRecord]) -> Result<Vec<AtributoValor>, DomainError> {
    attrs
        .iter()
        .map(|a| {
            Ok(AtributoValor {
                def: record_id_to_domain_id(&a.def)?,
                valor: surreal_valor_to_domain(&a.valor)?,
            })
        })
        .collect()
}

pub(crate) fn surreal_valor_to_domain(v: &SurrealValor) -> Result<ValorAtributo, DomainError> {
    Ok(match v {
        SurrealValor::Text(s) => ValorAtributo::Texto(s.clone()),
        SurrealValor::Number(n) => ValorAtributo::Numero(decimal_from_f64(*n)?),
        SurrealValor::Bool(b) => ValorAtributo::Booleano(*b),
    })
}

fn surreal_decimal_to_domain(v: &SurrealDecimal) -> Result<Decimal, DomainError> {
    match v {
        SurrealDecimal::Text(s) => {
            Decimal::from_str(s).map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))
        }
        SurrealDecimal::Number(n) => decimal_from_f64(*n),
    }
}

fn decimal_from_f64(n: f64) -> Result<Decimal, DomainError> {
    Decimal::from_str(&format!("{n}"))
        .map_err(|_| DomainError::unavailable(UnavailableCode::Unavailable))
}

async fn query_product_records(db: &Db, stmt: &str) -> Result<Vec<ProductoRecord>, DomainError> {
    let response = db
        .inner
        .query(stmt)
        .await
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let mut checked = response
        .check()
        .map_err(|e| map_surreal_message(&e.to_string(), &[stmt.to_string()]))?;
    let rows: Vec<ProductoRecord> = checked
        .take(0)
        .map_err(|e| map_surreal_message(&format!("deserialize: {e}"), &[stmt.to_string()]))?;
    Ok(rows)
}

pub(crate) fn record_id_to_domain_id<T>(rid: &RecordId) -> Result<Id<T>, DomainError> {
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

fn product_thing_ref(id_str: &str) -> String {
    format!("type::thing('producto', '{id_str}')")
}

fn sql_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

fn sql_decimal(d: Decimal) -> String {
    format!("{}dec", d)
}
