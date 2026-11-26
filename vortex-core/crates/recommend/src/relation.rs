//! Product relation use cases and HTTP-facing gateway (T-04-05).

use std::future::Future;
use std::pin::Pin;

use domain::{
    DomainError, Id, NewRelacion, ProductRepo, Producto, Relacion, RelacionConDestino,
    RelacionesDeProducto, RelationRepo, TipoRelacion, UpdateRelacion,
};

/// List query for relation use cases.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListRelationsParams {
    pub tipo: Option<TipoRelacion>,
    pub incoming: bool,
}

/// Object-safe facade for HTTP injection (`Arc<dyn RelationGateway>`).
pub trait RelationGateway: Send + Sync {
    fn create(
        &self,
        input: NewRelacion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>;

    fn update(
        &self,
        input: UpdateRelacion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>;

    fn delete(
        &self,
        tipo: TipoRelacion,
        id: Id<Relacion>,
    ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>>;

    fn list(
        &self,
        product: Id<Producto>,
        params: ListRelationsParams,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>;
}

/// Relation ABM over `RelationRepo` and `ProductRepo` (DC-05).
pub struct RelationService<R, P> {
    relations: R,
    products: P,
}

impl<R, P> RelationService<R, P> {
    pub fn new(relations: R, products: P) -> Self {
        Self {
            relations,
            products,
        }
    }
}

impl<R, P> RelationService<R, P>
where
    R: RelationRepo + Send + Sync,
    P: ProductRepo + Send + Sync,
{
    pub async fn create(&self, input: NewRelacion) -> Result<Vec<RelacionConDestino>, DomainError> {
        let created = self.relations.create(input).await?;
        items_after_write(self, &created).await
    }

    pub async fn update(
        &self,
        input: UpdateRelacion,
    ) -> Result<Vec<RelacionConDestino>, DomainError> {
        let updated = self.relations.update(input).await?;
        items_after_write(self, &updated).await
    }

    pub async fn delete(&self, tipo: TipoRelacion, id: Id<Relacion>) -> Result<(), DomainError> {
        self.relations.delete(tipo, id).await
    }

    pub async fn list(
        &self,
        product: Id<Producto>,
        params: ListRelationsParams,
    ) -> Result<Vec<RelacionConDestino>, DomainError> {
        self.products.get(product.clone()).await?;
        let grouped = if params.incoming {
            self.relations.list_incoming_for_product(product).await?
        } else {
            self.relations.list_for_product(product).await?
        };
        Ok(flatten_relaciones(grouped, params.tipo))
    }
}

impl<R, P> RelationGateway for RelationService<R, P>
where
    R: RelationRepo + Send + Sync,
    P: ProductRepo + Send + Sync,
{
    fn create(
        &self,
        input: NewRelacion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>
    {
        Box::pin(RelationService::create(self, input))
    }

    fn update(
        &self,
        input: UpdateRelacion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>
    {
        Box::pin(RelationService::update(self, input))
    }

    fn delete(
        &self,
        tipo: TipoRelacion,
        id: Id<Relacion>,
    ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
        Box::pin(RelationService::delete(self, tipo, id))
    }

    fn list(
        &self,
        product: Id<Producto>,
        params: ListRelationsParams,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>
    {
        Box::pin(RelationService::list(self, product, params))
    }
}

async fn items_after_write<R, P>(
    svc: &RelationService<R, P>,
    rel: &Relacion,
) -> Result<Vec<RelacionConDestino>, DomainError>
where
    R: RelationRepo + Send + Sync,
    P: ProductRepo + Send + Sync,
{
    if rel.grupo_bidireccional.is_some() {
        collect_bidirectional_group(svc, rel).await
    } else {
        find_edge_in_outgoing(svc, rel).await
    }
}

async fn find_edge_in_outgoing<R, P>(
    svc: &RelationService<R, P>,
    rel: &Relacion,
) -> Result<Vec<RelacionConDestino>, DomainError>
where
    R: RelationRepo + Send + Sync,
{
    let list = svc.relations.list_for_product(rel.origen.clone()).await?;
    Ok(find_edge_by_id(&list, rel.tipo, &rel.id))
}

fn find_edge_by_id(
    list: &RelacionesDeProducto,
    tipo: TipoRelacion,
    id: &Id<Relacion>,
) -> Vec<RelacionConDestino> {
    let bucket = match tipo {
        TipoRelacion::SimilarA => &list.similar_a,
        TipoRelacion::SustituiblePor => &list.sustituible_por,
        TipoRelacion::Complementa => &list.complementa,
    };
    bucket
        .iter()
        .filter(|i| i.relacion.id == *id)
        .cloned()
        .collect()
}

async fn collect_bidirectional_group<R, P>(
    svc: &RelationService<R, P>,
    rel: &Relacion,
) -> Result<Vec<RelacionConDestino>, DomainError>
where
    R: RelationRepo + Send + Sync,
{
    let grupo = rel.grupo_bidireccional.expect("grupo");
    let mut items = Vec::new();
    for origen in [rel.origen.clone(), rel.destino.clone()] {
        let list = svc.relations.list_for_product(origen).await?;
        for item in find_all_in_group(&list, rel.tipo, grupo) {
            if !items
                .iter()
                .any(|e: &RelacionConDestino| e.relacion.id == item.relacion.id)
            {
                items.push(item);
            }
        }
    }
    items.sort_by_key(|a| a.relacion.id.to_string());
    Ok(items)
}

fn find_all_in_group(
    list: &RelacionesDeProducto,
    tipo: TipoRelacion,
    grupo: domain::Id<domain::GrupoBidireccional>,
) -> Vec<RelacionConDestino> {
    let bucket = match tipo {
        TipoRelacion::SimilarA => &list.similar_a,
        TipoRelacion::SustituiblePor => &list.sustituible_por,
        TipoRelacion::Complementa => &list.complementa,
    };
    bucket
        .iter()
        .filter(|i| i.relacion.grupo_bidireccional == Some(grupo))
        .cloned()
        .collect()
}

fn flatten_relaciones(
    grouped: RelacionesDeProducto,
    tipo: Option<TipoRelacion>,
) -> Vec<RelacionConDestino> {
    let mut out = Vec::new();
    let mut push = |items: &[RelacionConDestino], filter: TipoRelacion| {
        if tipo.is_none() || tipo == Some(filter) {
            out.extend_from_slice(items);
        }
    };
    push(&grouped.similar_a, TipoRelacion::SimilarA);
    push(&grouped.sustituible_por, TipoRelacion::SustituiblePor);
    push(&grouped.complementa, TipoRelacion::Complementa);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use domain::error::{ConflictCode, DomainCode};
    use domain::{
        Categoria, Decimal, DestinoResumen, GrupoBidireccional, NewProducto, Producto,
        ProductoDraft, ProductoRecomendado, Recomendaciones, RelacionDraft, Timestamp,
        UnavailableCode, UnidadMedida,
    };

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    type ProductMap = Vec<(Id<Producto>, DestinoResumen)>;

    fn map_get(map: &ProductMap, id: &Id<Producto>) -> Option<DestinoResumen> {
        map.iter().find(|(k, _)| k == id).map(|(_, v)| v.clone())
    }

    fn map_has(map: &ProductMap, id: &Id<Producto>) -> bool {
        map.iter().any(|(k, _)| k == id)
    }

    type DecimalMap = Vec<(Id<Producto>, Decimal)>;
    type LocationMap = Vec<(Id<Producto>, Option<String>)>;

    fn map_decimal(map: &DecimalMap, id: &Id<Producto>) -> Option<Decimal> {
        map.iter().find(|(k, _)| k == id).map(|(_, v)| *v)
    }

    fn map_location(map: &LocationMap, id: &Id<Producto>) -> Option<Option<String>> {
        map.iter().find(|(k, _)| k == id).map(|(_, v)| v.clone())
    }

    struct InMemoryRelationRepo {
        items: Arc<Mutex<Vec<RelacionConDestino>>>,
        products: Arc<ProductMap>,
        stocks: Arc<DecimalMap>,
        precios: Arc<DecimalMap>,
        ubicaciones: Arc<LocationMap>,
    }

    impl InMemoryRelationRepo {
        fn new(products: ProductMap) -> Self {
            Self {
                items: Arc::new(Mutex::new(Vec::new())),
                products: Arc::new(products),
                stocks: Arc::new(Vec::new()),
                precios: Arc::new(Vec::new()),
                ubicaciones: Arc::new(Vec::new()),
            }
        }

        fn with_panel(
            mut self,
            stocks: DecimalMap,
            precios: DecimalMap,
            ubicaciones: LocationMap,
        ) -> Self {
            self.stocks = Arc::new(stocks);
            self.precios = Arc::new(precios);
            self.ubicaciones = Arc::new(ubicaciones);
            self
        }

        fn push_outgoing(
            &self,
            origen: Id<Producto>,
            destino: Id<Producto>,
            tipo: TipoRelacion,
            prioridad: i32,
        ) {
            let dest = map_get(self.products.as_ref(), &destino).expect("destino");
            let relacion = Relacion::try_new(RelacionDraft {
                id: Id::generate(),
                origen,
                destino: destino.clone(),
                tipo,
                prioridad,
                nota: None,
                grupo_bidireccional: None,
                creado_en: ts(),
                actualizado_en: ts(),
            })
            .expect("rel");
            self.items.lock().expect("lock").push(RelacionConDestino {
                relacion,
                destino: dest,
            });
        }

        fn bucket_mut(
            grouped: &mut RelacionesDeProducto,
            tipo: TipoRelacion,
        ) -> &mut Vec<RelacionConDestino> {
            match tipo {
                TipoRelacion::SimilarA => &mut grouped.similar_a,
                TipoRelacion::SustituiblePor => &mut grouped.sustituible_por,
                TipoRelacion::Complementa => &mut grouped.complementa,
            }
        }
    }

    impl RelationRepo for InMemoryRelationRepo {
        fn create(
            &self,
            input: NewRelacion,
        ) -> impl Future<Output = Result<Relacion, DomainError>> + Send {
            let items = Arc::clone(&self.items);
            let products = Arc::clone(&self.products);
            async move {
                if !map_has(&products, &input.origen) || !map_has(&products, &input.destino) {
                    return Err(DomainError::not_found());
                }
                let bidireccional = input.resolved_bidireccional();
                let grupo = if bidireccional {
                    Some(Id::<GrupoBidireccional>::generate())
                } else {
                    None
                };
                let edge_id = Id::<Relacion>::generate();
                let rel = Relacion::try_new(RelacionDraft {
                    id: edge_id,
                    origen: input.origen.clone(),
                    destino: input.destino.clone(),
                    tipo: input.tipo,
                    prioridad: input.prioridad,
                    nota: input.nota.clone(),
                    grupo_bidireccional: grupo,
                    creado_en: ts(),
                    actualizado_en: ts(),
                })?;

                let mut guard = items.lock().expect("lock");
                if guard.iter().any(|i| {
                    i.relacion.tipo == input.tipo
                        && i.relacion.origen == input.origen
                        && i.relacion.destino == input.destino
                }) {
                    return Err(DomainError::conflict(ConflictCode::DuplicateRelation));
                }

                let dest_resumen = map_get(&products, &input.destino).expect("destino");
                guard.push(RelacionConDestino {
                    relacion: rel.clone(),
                    destino: dest_resumen,
                });

                if bidireccional {
                    let rev_id = Id::<Relacion>::generate();
                    let rev = Relacion::try_new(RelacionDraft {
                        id: rev_id,
                        origen: input.destino.clone(),
                        destino: input.origen.clone(),
                        tipo: input.tipo,
                        prioridad: input.prioridad,
                        nota: input.nota,
                        grupo_bidireccional: grupo,
                        creado_en: ts(),
                        actualizado_en: ts(),
                    })?;
                    let dest_resumen = map_get(&products, &input.origen).expect("origen");
                    guard.push(RelacionConDestino {
                        relacion: rev,
                        destino: dest_resumen,
                    });
                }

                Ok(rel)
            }
        }

        fn update(
            &self,
            input: UpdateRelacion,
        ) -> impl Future<Output = Result<Relacion, DomainError>> + Send {
            let items = Arc::clone(&self.items);
            async move {
                let mut guard = items.lock().expect("lock");
                let pos = guard
                    .iter()
                    .position(|i| i.relacion.id == input.id && i.relacion.tipo == input.tipo)
                    .ok_or_else(DomainError::not_found)?;
                let current = guard[pos].relacion.clone();
                let merged_prioridad = input.prioridad.unwrap_or(current.prioridad);
                let merged_nota = match &input.nota {
                    None => current.nota.clone(),
                    Some(None) => None,
                    Some(Some(s)) => Some(s.clone()),
                };
                let updated = Relacion::try_new(RelacionDraft {
                    id: current.id.clone(),
                    origen: current.origen.clone(),
                    destino: current.destino.clone(),
                    tipo: current.tipo,
                    prioridad: merged_prioridad,
                    nota: merged_nota.clone(),
                    grupo_bidireccional: current.grupo_bidireccional,
                    creado_en: current.creado_en,
                    actualizado_en: ts(),
                })?;

                let grupo = current.grupo_bidireccional;
                for item in guard.iter_mut() {
                    let same_group = grupo.is_some()
                        && item.relacion.grupo_bidireccional == grupo
                        && item.relacion.tipo == input.tipo;
                    let is_self = item.relacion.id == input.id;
                    if is_self || same_group {
                        item.relacion = Relacion::try_new(RelacionDraft {
                            id: item.relacion.id.clone(),
                            origen: item.relacion.origen.clone(),
                            destino: item.relacion.destino.clone(),
                            tipo: item.relacion.tipo,
                            prioridad: merged_prioridad,
                            nota: merged_nota.clone(),
                            grupo_bidireccional: item.relacion.grupo_bidireccional,
                            creado_en: item.relacion.creado_en,
                            actualizado_en: ts(),
                        })?;
                    }
                }
                Ok(updated)
            }
        }

        fn delete(
            &self,
            tipo: TipoRelacion,
            id: Id<Relacion>,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            let items = Arc::clone(&self.items);
            async move {
                let mut guard = items.lock().expect("lock");
                let pos = guard
                    .iter()
                    .position(|i| i.relacion.id == id && i.relacion.tipo == tipo)
                    .ok_or_else(DomainError::not_found)?;
                let grupo = guard[pos].relacion.grupo_bidireccional;
                guard.retain(|i| {
                    if i.relacion.tipo != tipo {
                        return true;
                    }
                    if i.relacion.id == id {
                        return false;
                    }
                    if let Some(g) = grupo {
                        return i.relacion.grupo_bidireccional != Some(g);
                    }
                    true
                });
                Ok(())
            }
        }

        fn list_for_product(
            &self,
            producto: Id<Producto>,
        ) -> impl Future<Output = Result<RelacionesDeProducto, DomainError>> + Send {
            let items = Arc::clone(&self.items);
            async move {
                let guard = items.lock().expect("lock");
                let mut grouped = RelacionesDeProducto::empty();
                for item in guard.iter().filter(|i| i.relacion.origen == producto) {
                    Self::bucket_mut(&mut grouped, item.relacion.tipo).push(item.clone());
                }
                Ok(grouped)
            }
        }

        fn list_incoming_for_product(
            &self,
            producto: Id<Producto>,
        ) -> impl Future<Output = Result<RelacionesDeProducto, DomainError>> + Send {
            let items = Arc::clone(&self.items);
            let products = Arc::clone(&self.products);
            async move {
                let guard = items.lock().expect("lock");
                let mut grouped = RelacionesDeProducto::empty();
                for item in guard.iter().filter(|i| i.relacion.destino == producto) {
                    let origin = map_get(&products, &item.relacion.origen).expect("origin");
                    let mut incoming = item.clone();
                    incoming.destino = origin;
                    Self::bucket_mut(&mut grouped, item.relacion.tipo).push(incoming);
                }
                Ok(grouped)
            }
        }

        fn recommendations_for_product(
            &self,
            producto: Id<Producto>,
        ) -> impl Future<Output = Result<Recomendaciones, DomainError>> + Send {
            let items = Arc::clone(&self.items);
            let products = Arc::clone(&self.products);
            let stocks = Arc::clone(&self.stocks);
            let precios = Arc::clone(&self.precios);
            let ubicaciones = Arc::clone(&self.ubicaciones);
            async move {
                if !map_has(&products, &producto) {
                    return Err(DomainError::not_found());
                }
                let zero: Decimal = "0".parse().expect("zero");
                let source_stock = map_decimal(stocks.as_ref(), &producto).unwrap_or(zero);
                let destacar_sustitutos = source_stock == zero;

                let guard = items.lock().expect("lock");
                let mut similares = Vec::new();
                let mut sustitutos = Vec::new();
                let mut complementarios = Vec::new();

                for item in guard.iter().filter(|i| i.relacion.origen == producto) {
                    let dest_id = item.relacion.destino.clone();
                    let Some(resumen) = map_get(&products, &dest_id) else {
                        continue;
                    };
                    if !resumen.activo {
                        continue;
                    }
                    let dest_stock = map_decimal(stocks.as_ref(), &dest_id).unwrap_or(zero);
                    if item.relacion.tipo == TipoRelacion::SustituiblePor && dest_stock == zero {
                        continue;
                    }
                    let precio = map_decimal(precios.as_ref(), &dest_id)
                        .unwrap_or_else(|| "0".parse().expect("precio"));
                    let ubicacion = map_location(ubicaciones.as_ref(), &dest_id).flatten();
                    let panel_item = ProductoRecomendado {
                        relacion_id: item.relacion.id.clone(),
                        id: dest_id,
                        sku: resumen.sku,
                        nombre: resumen.nombre,
                        stock: dest_stock,
                        precio,
                        ubicacion,
                        prioridad: item.relacion.prioridad,
                        nota: item.relacion.nota.clone(),
                    };
                    match item.relacion.tipo {
                        TipoRelacion::SimilarA => similares.push(panel_item),
                        TipoRelacion::SustituiblePor => sustitutos.push(panel_item),
                        TipoRelacion::Complementa => complementarios.push(panel_item),
                    }
                }

                let sort = |v: &mut [ProductoRecomendado]| {
                    v.sort_by(|a, b| {
                        a.prioridad
                            .cmp(&b.prioridad)
                            .then_with(|| a.relacion_id.to_string().cmp(&b.relacion_id.to_string()))
                    });
                };
                sort(&mut similares);
                sort(&mut sustitutos);
                sort(&mut complementarios);

                Ok(Recomendaciones {
                    producto_id: producto,
                    destacar_sustitutos,
                    similares,
                    sustitutos,
                    complementarios,
                })
            }
        }
    }

    struct StubProductRepo {
        ids: Arc<Mutex<Vec<Id<Producto>>>>,
    }

    impl StubProductRepo {
        fn with_ids(ids: impl IntoIterator<Item = Id<Producto>>) -> Self {
            Self {
                ids: Arc::new(Mutex::new(ids.into_iter().collect())),
            }
        }
    }

    impl ProductRepo for StubProductRepo {
        fn create(
            &self,
            _input: NewProducto,
        ) -> impl Future<Output = Result<Producto, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn get(
            &self,
            id: Id<Producto>,
        ) -> impl Future<Output = Result<domain::FichaProducto, DomainError>> + Send {
            let ids = Arc::clone(&self.ids);
            async move {
                if ids.lock().expect("lock").iter().any(|k| k == &id) {
                    Ok(minimal_ficha(id))
                } else {
                    Err(DomainError::not_found())
                }
            }
        }

        fn update(
            &self,
            _input: domain::UpdateProducto,
        ) -> impl Future<Output = Result<domain::FichaProducto, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn set_activo(
            &self,
            _id: Id<Producto>,
            _activo: bool,
        ) -> impl Future<Output = Result<domain::FichaProducto, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn list(
            &self,
            _filtro: domain::FiltroProducto,
            _after: Option<Id<Producto>>,
            _limit: u32,
        ) -> impl Future<Output = Result<Vec<domain::ResumenProducto>, DomainError>> + Send
        {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn find_by_code(
            &self,
            _code: String,
        ) -> impl Future<Output = Result<Vec<domain::ResumenProducto>, DomainError>> + Send
        {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }
    }

    fn minimal_ficha(id: Id<Producto>) -> domain::FichaProducto {
        use domain::Slug;
        let cat_id = Id::generate();
        let slug = Slug::parse("cat").expect("slug");
        let cat = Categoria {
            id: cat_id.clone(),
            nombre: "Cat".into(),
            slug,
            padre: None,
            ancestros: vec![],
            ruta: vec!["Cat".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        };
        domain::FichaProducto {
            producto: Producto::try_new(
                ProductoDraft {
                    id: id.clone(),
                    sku: "SKU".into(),
                    nombre: "Nombre".into(),
                    tipo: "tipo".into(),
                    descripcion: None,
                    marca: None,
                    categoria: cat_id,
                    atributos: vec![],
                    codigos: vec![],
                    unidad_medida: UnidadMedida::Unidad,
                    precio: "0".parse().expect("dec"),
                    ubicacion: None,
                    stock: "0".parse().expect("stock"),
                    activo: true,
                    version: 1,
                    creado_en: ts(),
                    actualizado_en: ts(),
                },
                &[],
            )
            .expect("producto"),
            marca: None,
            categoria: cat,
            atributos: vec![],
        }
    }

    fn resumen(id: Id<Producto>, sku: &str, nombre: &str, activo: bool) -> DestinoResumen {
        DestinoResumen {
            id,
            sku: sku.into(),
            nombre: nombre.into(),
            activo,
        }
    }

    fn svc(
        products: ProductMap,
        known: Vec<Id<Producto>>,
    ) -> RelationService<InMemoryRelationRepo, StubProductRepo> {
        RelationService::new(
            InMemoryRelationRepo::new(products),
            StubProductRepo::with_ids(known),
        )
    }

    #[tokio::test]
    async fn recommend_create_complement_perno_arandela() {
        let perno = Id::<Producto>::generate();
        let arandela = Id::<Producto>::generate();
        let map = vec![
            (
                perno.clone(),
                resumen(
                    perno.clone(),
                    "PER-M8X40-ZN",
                    "Perno hexagonal M8 x 40 zincado",
                    true,
                ),
            ),
            (
                arandela.clone(),
                resumen(arandela.clone(), "ARA-M8-ZN", "Arandela M8 zincada", true),
            ),
        ];
        let service = svc(map, vec![perno.clone(), arandela.clone()]);

        let items = service
            .create(NewRelacion {
                origen: perno,
                destino: arandela.clone(),
                tipo: TipoRelacion::Complementa,
                prioridad: 1,
                nota: Some("Arandela plana M8".into()),
                bidireccional: None,
            })
            .await
            .expect("create");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].relacion.tipo, TipoRelacion::Complementa);
        assert_eq!(items[0].destino.id, arandela);
        assert_eq!(items[0].destino.sku, "ARA-M8-ZN");
        assert!(items[0].relacion.grupo_bidireccional.is_none());
    }

    #[tokio::test]
    async fn recommend_create_similar_default_bidirectional() {
        let a = Id::<Producto>::generate();
        let b = Id::<Producto>::generate();
        let map = vec![
            (a.clone(), resumen(a.clone(), "A", "Product A", true)),
            (b.clone(), resumen(b.clone(), "B", "Product B", true)),
        ];
        let service = svc(map, vec![a.clone(), b.clone()]);

        let items = service
            .create(NewRelacion {
                origen: a.clone(),
                destino: b.clone(),
                tipo: TipoRelacion::SimilarA,
                prioridad: 1,
                nota: None,
                bidireccional: None,
            })
            .await
            .expect("create");

        assert_eq!(items.len(), 2);
        let grupo = items[0].relacion.grupo_bidireccional;
        assert_eq!(items[1].relacion.grupo_bidireccional, grupo);

        let list_a = service
            .list(a.clone(), ListRelationsParams::default())
            .await
            .expect("list a");
        assert!(list_a.iter().any(|i| i.destino.id == b));
        let list_b = service
            .list(b, ListRelationsParams::default())
            .await
            .expect("list b");
        assert!(list_b.iter().any(|i| i.destino.id == a));
    }

    #[tokio::test]
    async fn recommend_update_priority_from_b_updates_both() {
        let a = Id::<Producto>::generate();
        let b = Id::<Producto>::generate();
        let map = vec![
            (a.clone(), resumen(a.clone(), "A", "A", true)),
            (b.clone(), resumen(b.clone(), "B", "B", true)),
        ];
        let service = svc(map, vec![a.clone(), b.clone()]);

        let created = service
            .create(NewRelacion {
                origen: a.clone(),
                destino: b.clone(),
                tipo: TipoRelacion::SimilarA,
                prioridad: 1,
                nota: None,
                bidireccional: None,
            })
            .await
            .expect("create");

        let b_to_a = created
            .iter()
            .find(|i| i.relacion.origen == b)
            .expect("b to a");

        let items = service
            .update(UpdateRelacion {
                id: b_to_a.relacion.id.clone(),
                tipo: TipoRelacion::SimilarA,
                prioridad: Some(3),
                nota: None,
            })
            .await
            .expect("update");

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.relacion.prioridad == 3));
    }

    #[tokio::test]
    async fn recommend_list_includes_inactive_target() {
        let src = Id::<Producto>::generate();
        let inactive = Id::<Producto>::generate();
        let map = vec![
            (src.clone(), resumen(src.clone(), "SRC", "Source", true)),
            (
                inactive.clone(),
                resumen(inactive.clone(), "OFF", "Inactive", false),
            ),
        ];
        let service = svc(map, vec![src.clone(), inactive.clone()]);

        for (tipo, dest) in [
            (TipoRelacion::SimilarA, inactive.clone()),
            (TipoRelacion::SustituiblePor, inactive.clone()),
            (TipoRelacion::Complementa, inactive.clone()),
        ] {
            service
                .create(NewRelacion {
                    origen: src.clone(),
                    destino: dest,
                    tipo,
                    prioridad: 1,
                    nota: None,
                    bidireccional: Some(false),
                })
                .await
                .expect("create");
        }

        let items = service
            .list(src, ListRelationsParams::default())
            .await
            .expect("list");
        assert_eq!(items.len(), 3);
        assert!(items.iter().all(|i| !i.destino.activo));
    }

    #[tokio::test]
    async fn recommend_create_self_relation() {
        let pid = Id::<Producto>::generate();
        let map = vec![(pid.clone(), resumen(pid.clone(), "X", "X", true))];
        let service = svc(map, vec![pid.clone()]);

        let err = service
            .create(NewRelacion {
                origen: pid.clone(),
                destino: pid,
                tipo: TipoRelacion::Complementa,
                prioridad: 1,
                nota: None,
                bidireccional: None,
            })
            .await
            .expect_err("self");

        assert_eq!(err.code(), DomainCode::SelfRelation);
    }

    #[tokio::test]
    async fn recommend_create_duplicate_relation() {
        let a = Id::<Producto>::generate();
        let b = Id::<Producto>::generate();
        let map = vec![
            (a.clone(), resumen(a.clone(), "A", "A", true)),
            (b.clone(), resumen(b.clone(), "B", "B", true)),
        ];
        let service = svc(map, vec![a.clone(), b.clone()]);

        service
            .create(NewRelacion {
                origen: a.clone(),
                destino: b.clone(),
                tipo: TipoRelacion::Complementa,
                prioridad: 1,
                nota: None,
                bidireccional: None,
            })
            .await
            .expect("first");

        let err = service
            .create(NewRelacion {
                origen: a,
                destino: b,
                tipo: TipoRelacion::Complementa,
                prioridad: 1,
                nota: None,
                bidireccional: None,
            })
            .await
            .expect_err("dup");

        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::DuplicateRelation,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn recommend_list_unknown_product_is_not_found() {
        let service = svc(vec![], vec![]);
        let err = service
            .list(Id::generate(), ListRelationsParams::default())
            .await
            .expect_err("missing product");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[test]
    fn relation_gateway_is_object_safe() {
        let a = Id::<Producto>::generate();
        let b = Id::<Producto>::generate();
        let map = vec![
            (a.clone(), resumen(a.clone(), "A", "A", true)),
            (b.clone(), resumen(b.clone(), "B", "B", true)),
        ];
        let gateway: Arc<dyn RelationGateway> = Arc::new(svc(map, vec![a, b]));
        let _ = gateway.clone();
    }

    #[tokio::test]
    async fn recommend_get_three_blocks_ordered() {
        let perno = Id::<Producto>::generate();
        let dest_a = Id::<Producto>::generate();
        let dest_b = Id::<Producto>::generate();
        let map = vec![
            (
                perno.clone(),
                resumen(perno.clone(), "PER-M8", "Perno M8", true),
            ),
            (dest_a.clone(), resumen(dest_a.clone(), "SKU-A", "A", true)),
            (dest_b.clone(), resumen(dest_b.clone(), "SKU-B", "B", true)),
        ];
        let stocks = vec![
            (dest_a.clone(), "1".parse().expect("stock")),
            (perno.clone(), "1".parse().expect("stock")),
        ];
        let repo = InMemoryRelationRepo::new(map).with_panel(stocks, vec![], vec![]);
        repo.push_outgoing(perno.clone(), dest_b.clone(), TipoRelacion::SimilarA, 2);
        repo.push_outgoing(perno.clone(), dest_a.clone(), TipoRelacion::SimilarA, 1);
        repo.push_outgoing(
            perno.clone(),
            dest_a.clone(),
            TipoRelacion::SustituiblePor,
            1,
        );
        repo.push_outgoing(perno.clone(), dest_a.clone(), TipoRelacion::Complementa, 1);

        let rec = crate::RecommendationService::new(repo);
        let panel = rec.get(perno).await.expect("get");

        assert_eq!(panel.similares.len(), 2);
        assert_eq!(panel.similares[0].prioridad, 1);
        assert_eq!(panel.similares[1].prioridad, 2);
        assert_eq!(panel.sustitutos.len(), 1);
        assert_eq!(panel.complementarios.len(), 1);
    }

    #[tokio::test]
    async fn recommend_get_excludes_inactive_target() {
        let perno = Id::<Producto>::generate();
        let activo = Id::<Producto>::generate();
        let inactivo = Id::<Producto>::generate();
        let map = vec![
            (perno.clone(), resumen(perno.clone(), "P", "Perno", true)),
            (activo.clone(), resumen(activo.clone(), "A", "Activo", true)),
            (
                inactivo.clone(),
                resumen(inactivo.clone(), "I", "Inactivo", false),
            ),
        ];
        let repo = InMemoryRelationRepo::new(map);
        repo.push_outgoing(perno.clone(), inactivo, TipoRelacion::SimilarA, 1);
        repo.push_outgoing(perno.clone(), activo, TipoRelacion::SimilarA, 1);

        let panel = crate::RecommendationService::new(repo)
            .get(perno)
            .await
            .expect("get");
        assert_eq!(panel.similares.len(), 1);
        assert_eq!(panel.similares[0].sku, "A");
    }

    #[tokio::test]
    async fn recommend_get_highlights_substitutes_when_stock_zero() {
        let perno = Id::<Producto>::generate();
        let sub_a = Id::<Producto>::generate();
        let sub_b = Id::<Producto>::generate();
        let map = vec![
            (
                perno.clone(),
                resumen(perno.clone(), "FIX", "Perno Fixser", true),
            ),
            (sub_a.clone(), resumen(sub_a.clone(), "SA", "Sub A", true)),
            (sub_b.clone(), resumen(sub_b.clone(), "SB", "Sub B", true)),
        ];
        let stocks = vec![
            (perno.clone(), "0".parse().expect("stock")),
            (sub_a.clone(), "12".parse().expect("stock")),
            (sub_b.clone(), "0".parse().expect("stock")),
        ];
        let repo = InMemoryRelationRepo::new(map).with_panel(stocks, vec![], vec![]);
        repo.push_outgoing(perno.clone(), sub_a, TipoRelacion::SustituiblePor, 1);
        repo.push_outgoing(perno.clone(), sub_b, TipoRelacion::SustituiblePor, 2);

        let panel = crate::RecommendationService::new(repo)
            .get(perno)
            .await
            .expect("get");
        assert!(panel.destacar_sustitutos);
        assert_eq!(panel.sustitutos.len(), 1);
        assert_eq!(panel.sustitutos[0].stock.to_string(), "12");
    }

    #[tokio::test]
    async fn recommend_get_does_not_highlight_when_in_stock() {
        let perno = Id::<Producto>::generate();
        let sub = Id::<Producto>::generate();
        let map = vec![
            (perno.clone(), resumen(perno.clone(), "P", "Perno", true)),
            (sub.clone(), resumen(sub.clone(), "S", "Sub", true)),
        ];
        let stocks = vec![
            (perno.clone(), "5".parse().expect("stock")),
            (sub.clone(), "12".parse().expect("stock")),
        ];
        let repo = InMemoryRelationRepo::new(map).with_panel(stocks, vec![], vec![]);
        repo.push_outgoing(perno.clone(), sub, TipoRelacion::SustituiblePor, 1);

        let panel = crate::RecommendationService::new(repo)
            .get(perno)
            .await
            .expect("get");
        assert!(!panel.destacar_sustitutos);
    }

    #[tokio::test]
    async fn recommend_get_unknown_product_is_not_found() {
        let repo = InMemoryRelationRepo::new(vec![]);
        let err = crate::RecommendationService::new(repo)
            .get(Id::generate())
            .await
            .expect_err("missing");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[test]
    fn recommendation_gateway_is_object_safe() {
        let perno = Id::<Producto>::generate();
        let map = vec![(perno.clone(), resumen(perno.clone(), "P", "P", true))];
        let gateway: Arc<dyn crate::RecommendationGateway> = Arc::new(
            crate::RecommendationService::new(InMemoryRelationRepo::new(map)),
        );
        let _ = gateway.clone();
    }
}
