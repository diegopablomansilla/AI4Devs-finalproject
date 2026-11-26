//! In-memory `RelationGateway` for HTTP integration tests (T-04-05).

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use domain::error::{ConflictCode, DomainError};
use domain::{
    Categoria, DestinoResumen, FichaProducto, FiltroProducto, GrupoBidireccional, Id, NewProducto,
    NewRelacion, ProductRepo, Producto, ProductoDraft, Relacion, RelacionConDestino,
    RelacionesDeProducto, RelationRepo, ResumenProducto, Slug, Timestamp, TipoRelacion,
    UnavailableCode, UnidadMedida, UpdateProducto, UpdateRelacion,
};
use recommend::{ListRelationsParams, RelationGateway, RelationService};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FakeRelationMode {
    Normal,
    Unavailable,
}

pub struct FakeRelationGateway {
    mode: Arc<Mutex<FakeRelationMode>>,
    service: RelationService<InMemoryRelationRepo, StubProductRepo>,
}

impl FakeRelationGateway {
    pub fn with_products(products: ProductMap) -> Self {
        let known: Vec<Id<Producto>> = products.iter().map(|(id, _)| id.clone()).collect();
        Self {
            mode: Arc::new(Mutex::new(FakeRelationMode::Normal)),
            service: RelationService::new(
                InMemoryRelationRepo::new(products),
                StubProductRepo::with_ids(known),
            ),
        }
    }

    pub fn unavailable() -> Self {
        let gw = Self::with_products(vec![]);
        gw.set_mode(FakeRelationMode::Unavailable);
        gw
    }

    pub fn set_mode(&self, mode: FakeRelationMode) {
        *self.mode.lock().expect("lock") = mode;
    }

    pub async fn seed_create(&self, input: NewRelacion) -> Result<(), DomainError> {
        RelationService::create(&self.service, input).await?;
        Ok(())
    }
}

impl RelationGateway for FakeRelationGateway {
    fn create(
        &self,
        input: NewRelacion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>
    {
        let mode = *self.mode.lock().expect("lock");
        Box::pin(async move {
            if mode == FakeRelationMode::Unavailable {
                return Err(DomainError::unavailable(UnavailableCode::Unavailable));
            }
            RelationService::create(&self.service, input).await
        })
    }

    fn update(
        &self,
        input: UpdateRelacion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>
    {
        let mode = *self.mode.lock().expect("lock");
        Box::pin(async move {
            if mode == FakeRelationMode::Unavailable {
                return Err(DomainError::unavailable(UnavailableCode::Unavailable));
            }
            RelationService::update(&self.service, input).await
        })
    }

    fn delete(
        &self,
        tipo: TipoRelacion,
        id: Id<Relacion>,
    ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + '_>> {
        let mode = *self.mode.lock().expect("lock");
        Box::pin(async move {
            if mode == FakeRelationMode::Unavailable {
                return Err(DomainError::unavailable(UnavailableCode::Unavailable));
            }
            RelationService::delete(&self.service, tipo, id).await
        })
    }

    fn list(
        &self,
        product: Id<Producto>,
        params: ListRelationsParams,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<RelacionConDestino>, DomainError>> + Send + '_>>
    {
        let mode = *self.mode.lock().expect("lock");
        Box::pin(async move {
            if mode == FakeRelationMode::Unavailable {
                return Err(DomainError::unavailable(UnavailableCode::Unavailable));
            }
            RelationService::list(&self.service, product, params).await
        })
    }
}

pub fn resumen(id: Id<Producto>, sku: &str, nombre: &str, activo: bool) -> DestinoResumen {
    DestinoResumen {
        id,
        sku: sku.into(),
        nombre: nombre.into(),
        activo,
    }
}

pub struct PernoArandelaFixtures {
    pub perno: Id<Producto>,
    pub arandela: Id<Producto>,
}

pub fn perno_arandela_gateway() -> (Arc<dyn RelationGateway>, PernoArandelaFixtures) {
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
    let gateway: Arc<dyn RelationGateway> = Arc::new(FakeRelationGateway::with_products(map));
    (gateway, PernoArandelaFixtures { perno, arandela })
}

pub fn pair_gateway() -> (Arc<dyn RelationGateway>, Id<Producto>, Id<Producto>) {
    let a = Id::<Producto>::generate();
    let b = Id::<Producto>::generate();
    let map = vec![
        (a.clone(), resumen(a.clone(), "A-SKU", "Product A", true)),
        (b.clone(), resumen(b.clone(), "B-SKU", "Product B", true)),
    ];
    let gateway: Arc<dyn RelationGateway> = Arc::new(FakeRelationGateway::with_products(map));
    (gateway, a, b)
}

pub fn empty_known_gateway() -> Arc<dyn RelationGateway> {
    Arc::new(FakeRelationGateway::with_products(vec![]))
}

struct InMemoryRelationRepo {
    items: Arc<Mutex<Vec<RelacionConDestino>>>,
    products: Arc<ProductMap>,
}

impl InMemoryRelationRepo {
    fn new(products: ProductMap) -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            products: Arc::new(products),
        }
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
            let rel = Relacion::try_new(domain::RelacionDraft {
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
                let rev = Relacion::try_new(domain::RelacionDraft {
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
            let updated = Relacion::try_new(domain::RelacionDraft {
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
                    item.relacion = Relacion::try_new(domain::RelacionDraft {
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
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
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
        _input: UpdateProducto,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
        async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
    }

    fn set_activo(
        &self,
        _id: Id<Producto>,
        _activo: bool,
    ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
        async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
    }

    fn list(
        &self,
        _filtro: FiltroProducto,
        _after: Option<Id<Producto>>,
        _limit: u32,
    ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
        async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
    }

    fn find_by_code(
        &self,
        _code: String,
    ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
        async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
    }
}

fn minimal_ficha(id: Id<Producto>) -> FichaProducto {
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
    FichaProducto {
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
