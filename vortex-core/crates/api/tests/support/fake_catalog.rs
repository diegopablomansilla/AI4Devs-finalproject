//! In-memory `CategoryRepo` for HTTP integration tests (not the `catalog` crate test module).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use catalog::stock::testing::InMemoryStockLedger;
use catalog::{
    AttributeDefGateway, AttributeDefService, BrandGateway, BrandService, CategoryGateway,
    CategoryService, EmptyAttributeUsage, PatchProducto, ProductGateway, ProductService,
    ProductionOccupancy, StockGateway, StockService, product_id_after,
    product_matches_categoria_branch,
};
use domain::AttributeUsage;
use domain::{
    AtributoDef, AtributoDefDraft, AtributoFicha, AttributeDefRepo, BrandRepo, Categoria,
    CategoriaDraft, CategoryRepo, ConflictCode, DomainError, FichaProducto, FiltroProducto, Id,
    InvalidInputCode, MAX_DEPTH, Marca, MarcaDraft, NewAtributoDef, NewCategoria, NewMarca,
    NewProducto, ProductRepo, Producto, ResumenProducto, Sku, Slug, Timestamp, TipoAtributo,
    UpdateAtributoDef, UpdateCategoria, UpdateMarca, UpdateProducto, ValorAtributo, reject_cycle,
};

const TS: &str = "2026-09-24T21:33:00Z";

fn ts() -> Timestamp {
    TS.parse().expect("timestamp")
}

#[derive(Clone)]
pub struct FakeCategoryRepo {
    inner: Arc<Mutex<HashMap<String, Categoria>>>,
}

impl FakeCategoryRepo {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn sibling_slug_taken(
        map: &HashMap<String, Categoria>,
        padre: Option<&Id<Categoria>>,
        slug: &str,
        except: Option<&Id<Categoria>>,
    ) -> bool {
        map.values().any(|c| {
            if except.is_some_and(|e| e == &c.id) {
                return false;
            }
            c.padre.as_ref() == padre && c.slug.as_str() == slug
        })
    }

    fn subtree_ids(map: &HashMap<String, Categoria>, root: &Id<Categoria>) -> Vec<Id<Categoria>> {
        let mut out = vec![root.clone()];
        let mut i = 0;
        while i < out.len() {
            let id = out[i].clone();
            for c in map.values() {
                if c.padre.as_ref() == Some(&id) && !out.iter().any(|x| x == &c.id) {
                    out.push(c.id.clone());
                }
            }
            i += 1;
        }
        out
    }
}

impl CategoryRepo for FakeCategoryRepo {
    async fn create(&self, input: NewCategoria) -> Result<Categoria, DomainError> {
        let slug = match input.slug {
            Some(s) => s,
            None => Slug::from_nombre(&input.nombre)?,
        };
        let map = self.inner.lock().expect("lock");
        let (ancestros, ruta, padre) = match &input.padre {
            None => (Vec::new(), vec![input.nombre.trim().to_string()], None),
            Some(padre_id) => {
                let parent = map
                    .get(&padre_id.to_string())
                    .ok_or(DomainError::not_found())?;
                if parent.depth() >= MAX_DEPTH {
                    return Err(DomainError::invalid_input(
                        InvalidInputCode::InvalidInput,
                        vec![],
                    ));
                }
                let mut ancestros = parent.ancestros.clone();
                ancestros.push(parent.id.clone());
                let mut ruta = parent.ruta.clone();
                ruta.push(input.nombre.trim().to_string());
                (ancestros, ruta, Some(padre_id.clone()))
            }
        };
        if Self::sibling_slug_taken(&map, padre.as_ref(), slug.as_str(), None) {
            return Err(DomainError::conflict(ConflictCode::DuplicateSlug));
        }
        drop(map);
        let id = Id::<Categoria>::generate();
        let cat = Categoria::try_new(CategoriaDraft {
            id,
            nombre: input.nombre.trim().to_string(),
            slug,
            padre,
            ancestros,
            ruta,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })?;
        self.inner
            .lock()
            .expect("lock")
            .insert(cat.id.to_string(), cat.clone());
        Ok(cat)
    }

    async fn get(&self, id: Id<Categoria>) -> Result<Categoria, DomainError> {
        self.inner
            .lock()
            .expect("lock")
            .get(&id.to_string())
            .cloned()
            .ok_or(DomainError::not_found())
    }

    async fn list_all(&self) -> Result<Vec<Categoria>, DomainError> {
        Ok(self.inner.lock().expect("lock").values().cloned().collect())
    }

    async fn update(&self, input: UpdateCategoria) -> Result<Categoria, DomainError> {
        let slug = Slug::from_nombre(&input.nombre)?;
        let new_parent_ancestors: Vec<Id<Categoria>> = {
            let map = self.inner.lock().expect("lock");
            if !map.contains_key(&input.id.to_string()) {
                return Err(DomainError::not_found());
            }
            match &input.padre {
                None => Vec::new(),
                Some(pid) => {
                    let parent = map.get(&pid.to_string()).ok_or(DomainError::not_found())?;
                    let mut a = parent.ancestros.clone();
                    a.push(parent.id.clone());
                    a
                }
            }
        };
        reject_cycle(&input.id, input.padre.as_ref(), &new_parent_ancestors)?;

        let parent_snapshot = match &input.padre {
            None => None,
            Some(pid) => Some(self.get(pid.clone()).await?),
        };
        if let Some(ref parent) = parent_snapshot
            && parent.depth() >= MAX_DEPTH
        {
            return Err(DomainError::invalid_input(
                InvalidInputCode::InvalidInput,
                vec![],
            ));
        }

        let nombre = input.nombre.trim().to_string();
        {
            let mut map = self.inner.lock().expect("lock");
            if Self::sibling_slug_taken(&map, input.padre.as_ref(), slug.as_str(), Some(&input.id))
            {
                return Err(DomainError::conflict(ConflictCode::DuplicateSlug));
            }

            let mut subtree: Vec<Id<Categoria>> = Self::subtree_ids(&map, &input.id);
            subtree.sort_by_key(|id| {
                map.get(&id.to_string())
                    .map(|c| c.ancestros.len())
                    .unwrap_or(0)
            });

            let mut updated: Vec<Categoria> = Vec::with_capacity(subtree.len());
            for node_id in &subtree {
                let old = map.get(&node_id.to_string()).expect("subtree node").clone();
                let (new_padre, new_ancestros, new_ruta) = if node_id == &input.id {
                    match (&input.padre, &parent_snapshot) {
                        (None, _) => (None, Vec::new(), vec![nombre.clone()]),
                        (Some(_), Some(parent)) => {
                            let mut a = parent.ancestros.clone();
                            a.push(parent.id.clone());
                            let mut r = parent.ruta.clone();
                            r.push(nombre.clone());
                            (Some(parent.id.clone()), a, r)
                        }
                        (Some(_), None) => return Err(DomainError::not_found()),
                    }
                } else {
                    let parent_id = old.padre.as_ref().expect("child has padre");
                    let parent = updated
                        .iter()
                        .find(|p| &p.id == parent_id)
                        .expect("parent in updated");
                    let mut a = parent.ancestros.clone();
                    a.push(parent.id.clone());
                    let mut r = parent.ruta.clone();
                    r.push(old.nombre.clone());
                    (Some(parent.id.clone()), a, r)
                };
                let node_slug = if node_id == &input.id {
                    slug.clone()
                } else {
                    old.slug.clone()
                };
                let node_nombre = if node_id == &input.id {
                    nombre.clone()
                } else {
                    old.nombre.clone()
                };
                let built = Categoria::try_new(CategoriaDraft {
                    id: old.id.clone(),
                    nombre: node_nombre,
                    slug: node_slug,
                    padre: new_padre,
                    ancestros: new_ancestros,
                    ruta: new_ruta,
                    activo: old.activo,
                    creado_en: old.creado_en,
                    actualizado_en: old.actualizado_en,
                })?;
                updated.push(built);
            }

            for cat in updated {
                map.insert(cat.id.to_string(), cat);
            }
        }
        self.get(input.id).await
    }

    async fn set_activo(&self, id: Id<Categoria>, activo: bool) -> Result<Categoria, DomainError> {
        let mut map = self.inner.lock().expect("lock");
        let cat = map
            .get(&id.to_string())
            .cloned()
            .ok_or(DomainError::not_found())?;
        let draft = CategoriaDraft {
            id: cat.id.clone(),
            nombre: cat.nombre.clone(),
            slug: cat.slug.clone(),
            padre: cat.padre.clone(),
            ancestros: cat.ancestros.clone(),
            ruta: cat.ruta.clone(),
            activo,
            creado_en: cat.creado_en,
            actualizado_en: cat.actualizado_en,
        };
        let built = Categoria::try_new(draft)?;
        map.insert(id.to_string(), built.clone());
        Ok(built)
    }
}

pub fn test_category_gateway() -> Arc<dyn CategoryGateway> {
    Arc::new(CategoryService::new(
        FakeCategoryRepo::new(),
        ProductionOccupancy,
    ))
}

#[derive(Clone)]
pub struct FakeBrandRepo {
    inner: Arc<Mutex<HashMap<String, Marca>>>,
}

impl FakeBrandRepo {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn slug_taken(map: &HashMap<String, Marca>, slug: &str, except: Option<&Id<Marca>>) -> bool {
        map.values().any(|m| {
            if except.is_some_and(|e| e == &m.id) {
                return false;
            }
            m.slug.as_str() == slug
        })
    }
}

impl BrandRepo for FakeBrandRepo {
    fn create(
        &self,
        input: NewMarca,
    ) -> impl std::future::Future<Output = Result<Marca, DomainError>> + Send {
        async move {
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
            let map = self.inner.lock().expect("lock");
            if Self::slug_taken(&map, slug.as_str(), None) {
                return Err(DomainError::conflict(ConflictCode::DuplicateSlug));
            }
            drop(map);
            let id = Id::<Marca>::generate();
            let marca = Marca::try_new(MarcaDraft {
                id: id.clone(),
                nombre,
                slug,
                activo: true,
                creado_en: ts(),
                actualizado_en: ts(),
            })?;
            self.inner
                .lock()
                .expect("lock")
                .insert(id.to_string(), marca.clone());
            Ok(marca)
        }
    }

    fn get(
        &self,
        id: Id<Marca>,
    ) -> impl std::future::Future<Output = Result<Marca, DomainError>> + Send {
        async move {
            self.inner
                .lock()
                .expect("lock")
                .get(&id.to_string())
                .cloned()
                .ok_or(DomainError::not_found())
        }
    }

    fn list_all(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Marca>, DomainError>> + Send {
        async move { Ok(self.inner.lock().expect("lock").values().cloned().collect()) }
    }

    fn update(
        &self,
        input: UpdateMarca,
    ) -> impl std::future::Future<Output = Result<Marca, DomainError>> + Send {
        async move {
            let current = self.get(input.id.clone()).await?;
            let slug = match input.slug {
                Some(s) => s,
                None => Slug::from_nombre(&input.nombre)?,
            };
            let nombre = input.nombre.trim().to_string();
            {
                let map = self.inner.lock().expect("lock");
                if Self::slug_taken(&map, slug.as_str(), Some(&input.id)) {
                    return Err(DomainError::conflict(ConflictCode::DuplicateSlug));
                }
            }
            let marca = Marca::try_new(MarcaDraft {
                id: input.id.clone(),
                nombre,
                slug,
                activo: current.activo,
                creado_en: current.creado_en,
                actualizado_en: current.actualizado_en,
            })?;
            self.inner
                .lock()
                .expect("lock")
                .insert(input.id.to_string(), marca.clone());
            Ok(marca)
        }
    }

    fn set_activo(
        &self,
        id: Id<Marca>,
        activo: bool,
    ) -> impl std::future::Future<Output = Result<Marca, DomainError>> + Send {
        async move {
            let mut map = self.inner.lock().expect("lock");
            let current = map
                .get(&id.to_string())
                .cloned()
                .ok_or(DomainError::not_found())?;
            let marca = Marca::try_new(MarcaDraft {
                id: current.id.clone(),
                nombre: current.nombre.clone(),
                slug: current.slug.clone(),
                activo,
                creado_en: current.creado_en,
                actualizado_en: current.actualizado_en,
            })?;
            map.insert(id.to_string(), marca.clone());
            Ok(marca)
        }
    }
}

pub fn test_brand_gateway() -> Arc<dyn BrandGateway> {
    Arc::new(BrandService::new(FakeBrandRepo::new()))
}

#[derive(Clone)]
pub struct FakeAttributeDefRepo {
    inner: Arc<Mutex<HashMap<String, AtributoDef>>>,
}

impl FakeAttributeDefRepo {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn clave_taken(
        map: &HashMap<String, AtributoDef>,
        clave: &str,
        except: Option<&Id<AtributoDef>>,
    ) -> bool {
        map.values().any(|d| {
            if except.is_some_and(|e| e == &d.id) {
                return false;
            }
            d.clave == clave
        })
    }
}

impl AttributeDefRepo for FakeAttributeDefRepo {
    fn create(
        &self,
        input: NewAtributoDef,
    ) -> impl std::future::Future<Output = Result<AtributoDef, DomainError>> + Send {
        async move {
            input.validate()?;
            let map = self.inner.lock().expect("lock");
            if Self::clave_taken(&map, &input.clave, None) {
                return Err(DomainError::conflict(ConflictCode::Conflict));
            }
            drop(map);
            let id = Id::<AtributoDef>::generate();
            let def = AtributoDef::try_new(AtributoDefDraft {
                id: id.clone(),
                clave: input.clave,
                etiqueta: input.etiqueta,
                tipo: input.tipo,
                unidad: input.unidad,
                opciones: input.opciones,
                facetable: input.facetable,
                buscable: input.buscable,
                orden: input.orden,
                activo: true,
                creado_en: ts(),
                actualizado_en: ts(),
            })?;
            self.inner
                .lock()
                .expect("lock")
                .insert(id.to_string(), def.clone());
            Ok(def)
        }
    }

    fn get(
        &self,
        id: Id<AtributoDef>,
    ) -> impl std::future::Future<Output = Result<AtributoDef, DomainError>> + Send {
        async move {
            self.inner
                .lock()
                .expect("lock")
                .get(&id.to_string())
                .cloned()
                .ok_or(DomainError::not_found())
        }
    }

    fn list_all(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send {
        async move { Ok(self.inner.lock().expect("lock").values().cloned().collect()) }
    }

    fn update(
        &self,
        input: UpdateAtributoDef,
    ) -> impl std::future::Future<Output = Result<AtributoDef, DomainError>> + Send {
        async move {
            input.validate()?;
            let current = self.get(input.id.clone()).await?;
            {
                let map = self.inner.lock().expect("lock");
                if Self::clave_taken(&map, &input.clave, Some(&input.id)) {
                    return Err(DomainError::conflict(ConflictCode::Conflict));
                }
            }
            let def = AtributoDef::try_new(AtributoDefDraft {
                id: input.id.clone(),
                clave: input.clave,
                etiqueta: input.etiqueta,
                tipo: input.tipo,
                unidad: input.unidad,
                opciones: input.opciones,
                facetable: input.facetable,
                buscable: input.buscable,
                orden: input.orden,
                activo: current.activo,
                creado_en: current.creado_en,
                actualizado_en: current.actualizado_en,
            })?;
            self.inner
                .lock()
                .expect("lock")
                .insert(input.id.to_string(), def.clone());
            Ok(def)
        }
    }

    fn set_activo(
        &self,
        id: Id<AtributoDef>,
        activo: bool,
    ) -> impl std::future::Future<Output = Result<AtributoDef, DomainError>> + Send {
        async move {
            let mut map = self.inner.lock().expect("lock");
            let current = map
                .get(&id.to_string())
                .cloned()
                .ok_or(DomainError::not_found())?;
            let def = AtributoDef::try_new(AtributoDefDraft {
                id: current.id.clone(),
                clave: current.clave.clone(),
                etiqueta: current.etiqueta.clone(),
                tipo: current.tipo,
                unidad: current.unidad.clone(),
                opciones: current.opciones.clone(),
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
                activo,
                creado_en: current.creado_en,
                actualizado_en: current.actualizado_en,
            })?;
            map.insert(id.to_string(), def.clone());
            Ok(def)
        }
    }
}

#[derive(Clone, Default)]
pub struct FakeAttributeUsage {
    inner: Arc<Mutex<HashMap<String, Vec<ValorAtributo>>>>,
}

impl FakeAttributeUsage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, id: Id<AtributoDef>, valores: Vec<ValorAtributo>) {
        self.inner
            .lock()
            .expect("lock")
            .insert(id.to_string(), valores);
    }
}

impl AttributeUsage for FakeAttributeUsage {
    fn valores(
        &self,
        id: Id<AtributoDef>,
    ) -> impl std::future::Future<Output = Result<Vec<ValorAtributo>, DomainError>> + Send {
        let inner = Arc::clone(&self.inner);
        async move {
            Ok(inner
                .lock()
                .expect("lock")
                .get(&id.to_string())
                .cloned()
                .unwrap_or_default())
        }
    }
}

pub fn test_attribute_def_gateway() -> Arc<dyn AttributeDefGateway> {
    Arc::new(AttributeDefService::new(
        FakeAttributeDefRepo::new(),
        EmptyAttributeUsage,
    ))
}

pub fn test_attribute_def_gateway_with_usage(
    usage: FakeAttributeUsage,
) -> Arc<dyn AttributeDefGateway> {
    Arc::new(AttributeDefService::new(FakeAttributeDefRepo::new(), usage))
}

#[derive(Clone)]
pub struct FakeProductRepo {
    inner: Arc<Mutex<HashMap<String, Producto>>>,
    categories: FakeCategoryRepo,
    brands: FakeBrandRepo,
    attrs: FakeAttributeDefRepo,
}

impl FakeProductRepo {
    pub fn new(
        categories: FakeCategoryRepo,
        brands: FakeBrandRepo,
        attrs: FakeAttributeDefRepo,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            categories,
            brands,
            attrs,
        }
    }

    pub fn product_store(&self) -> Arc<Mutex<HashMap<String, Producto>>> {
        Arc::clone(&self.inner)
    }

    fn sku_taken(
        map: &HashMap<String, Producto>,
        sku: &str,
        except: Option<&Id<Producto>>,
    ) -> bool {
        map.values().any(|p| {
            if except.is_some_and(|e| e == &p.id) {
                return false;
            }
            p.sku.as_str() == sku
        })
    }

    async fn producto_to_resumen(
        &self,
        producto: Producto,
    ) -> Result<ResumenProducto, DomainError> {
        let categoria = self.categories.get(producto.categoria.clone()).await?;
        let marca_nombre = match producto.marca {
            None => None,
            Some(ref id) => Some(self.brands.get(id.clone()).await?.nombre),
        };
        Ok(ResumenProducto {
            id: producto.id,
            sku: producto.sku.as_str().to_string(),
            nombre: producto.nombre,
            tipo: producto.tipo,
            precio: producto.precio,
            stock: producto.stock,
            activo: producto.activo,
            marca_nombre,
            categoria_ruta: categoria.ruta,
        })
    }

    async fn hydrate_ficha(&self, producto: Producto) -> Result<FichaProducto, DomainError> {
        let categoria = self.categories.get(producto.categoria.clone()).await?;
        let marca = match producto.marca {
            None => None,
            Some(ref id) => Some(self.brands.get(id.clone()).await?),
        };
        let mut ficha_attrs = Vec::with_capacity(producto.atributos.len());
        for attr in &producto.atributos {
            let def = self.attrs.get(attr.def.clone()).await?;
            ficha_attrs.push(AtributoFicha {
                def: attr.def.clone(),
                clave: def.clave,
                etiqueta: def.etiqueta,
                unidad: def.unidad,
                valor: attr.valor.clone(),
            });
        }
        Ok(FichaProducto {
            producto,
            marca,
            categoria,
            atributos: ficha_attrs,
        })
    }
}

impl ProductRepo for FakeProductRepo {
    fn create(
        &self,
        input: NewProducto,
    ) -> impl std::future::Future<Output = Result<Producto, DomainError>> + Send {
        async move {
            let sku = Sku::try_new(&input.sku)?;
            let map = self.inner.lock().expect("lock");
            if Self::sku_taken(&map, sku.as_str(), None) {
                return Err(DomainError::conflict(ConflictCode::DuplicateSku));
            }
            drop(map);
            let id = Id::<Producto>::generate();
            let producto = Producto {
                id: id.clone(),
                sku,
                nombre: input.nombre.trim().to_string(),
                tipo: input.tipo.trim().to_string(),
                descripcion: input.descripcion,
                marca: input.marca,
                categoria: input.categoria,
                atributos: input.atributos,
                codigos: input.codigos,
                unidad_medida: input.unidad_medida,
                precio: input.precio,
                ubicacion: input.ubicacion,
                stock: input.stock_inicial,
                activo: true,
                version: 1,
                creado_en: ts(),
                actualizado_en: ts(),
            };
            self.inner
                .lock()
                .expect("lock")
                .insert(id.to_string(), producto.clone());
            Ok(producto)
        }
    }

    fn get(
        &self,
        id: Id<Producto>,
    ) -> impl std::future::Future<Output = Result<FichaProducto, DomainError>> + Send {
        async move {
            let producto = self
                .inner
                .lock()
                .expect("lock")
                .get(&id.to_string())
                .cloned()
                .ok_or(DomainError::not_found())?;
            self.hydrate_ficha(producto).await
        }
    }

    fn update(
        &self,
        input: UpdateProducto,
    ) -> impl std::future::Future<Output = Result<FichaProducto, DomainError>> + Send {
        async move {
            let sku = Sku::try_new(&input.sku)?;
            let current = {
                let map = self.inner.lock().expect("lock");
                map.get(&input.id.to_string()).cloned()
            };
            let Some(current) = current else {
                return Err(DomainError::not_found());
            };
            if current.version != input.version {
                return Err(DomainError::conflict(ConflictCode::VersionConflict));
            }
            {
                let map = self.inner.lock().expect("lock");
                if Self::sku_taken(&map, sku.as_str(), Some(&input.id)) {
                    return Err(DomainError::conflict(ConflictCode::DuplicateSku));
                }
            }
            let built = Producto {
                id: input.id.clone(),
                sku,
                nombre: input.nombre.trim().to_string(),
                tipo: input.tipo.trim().to_string(),
                descripcion: input.descripcion.clone(),
                marca: input.marca.clone(),
                categoria: input.categoria.clone(),
                atributos: input.atributos.clone(),
                codigos: input.codigos.clone(),
                unidad_medida: input.unidad_medida,
                precio: input.precio,
                ubicacion: input.ubicacion.clone(),
                stock: current.stock,
                activo: current.activo,
                version: input.version + 1,
                creado_en: current.creado_en,
                actualizado_en: ts(),
            };
            self.inner
                .lock()
                .expect("lock")
                .insert(input.id.to_string(), built.clone());
            self.hydrate_ficha(built).await
        }
    }

    fn set_activo(
        &self,
        id: Id<Producto>,
        activo: bool,
    ) -> impl std::future::Future<Output = Result<FichaProducto, DomainError>> + Send {
        async move {
            let updated = {
                let mut map = self.inner.lock().expect("lock");
                let producto = map
                    .get(&id.to_string())
                    .cloned()
                    .ok_or(DomainError::not_found())?;
                let mut updated = producto;
                updated.activo = activo;
                updated.actualizado_en = ts();
                map.insert(id.to_string(), updated.clone());
                updated
            };
            self.hydrate_ficha(updated).await
        }
    }

    fn list(
        &self,
        filtro: FiltroProducto,
        after: Option<Id<Producto>>,
        take: u32,
    ) -> impl std::future::Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
        async move {
            let mut items: Vec<Producto> = {
                let map = self.inner.lock().expect("lock");
                map.values().cloned().collect()
            };
            items.retain(|p| p.activo == filtro.activo);
            if let Some(ref marca) = filtro.marca {
                items.retain(|p| p.marca.as_ref() == Some(marca));
            }
            if let Some(ref cat_filter) = filtro.categoria {
                let mut matched = Vec::with_capacity(items.len());
                for p in items {
                    let cat = self.categories.get(p.categoria.clone()).await?;
                    if product_matches_categoria_branch(&p.categoria, cat_filter, &cat) {
                        matched.push(p);
                    }
                }
                items = matched;
            }
            if let Some(ref after_id) = after {
                items.retain(|p| product_id_after(after_id, &p.id));
            }
            items.sort_by(|a, b| a.id.to_string().cmp(&b.id.to_string()));
            items.truncate(take as usize);
            let mut out = Vec::with_capacity(items.len());
            for p in items {
                out.push(self.producto_to_resumen(p).await?);
            }
            Ok(out)
        }
    }

    fn find_by_code(
        &self,
        code: String,
    ) -> impl std::future::Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
        async move {
            let items: Vec<Producto> = {
                let map = self.inner.lock().expect("lock");
                map.values()
                    .filter(|p| p.activo)
                    .filter(|p| p.sku.as_str() == code || p.codigos.iter().any(|c| c.valor == code))
                    .cloned()
                    .collect()
            };
            let mut sorted = items;
            sorted.sort_by(|a, b| a.sku.as_str().cmp(b.sku.as_str()));
            let mut out = Vec::with_capacity(sorted.len());
            for p in sorted {
                out.push(self.producto_to_resumen(p).await?);
            }
            Ok(out)
        }
    }
}

pub struct ProductTestSeeds {
    pub fijaciones_id: Id<Categoria>,
    pub category_id: Id<Categoria>,
    pub brand_id: Id<Marca>,
    pub medida_id: Id<AtributoDef>,
    pub largo_id: Id<AtributoDef>,
    pub grado_id: Id<AtributoDef>,
    pub categories: FakeCategoryRepo,
}

struct RecordingProductGateway {
    inner: ProductService<FakeProductRepo, FakeCategoryRepo, FakeBrandRepo, FakeAttributeDefRepo>,
    last_create: Arc<Mutex<Option<NewProducto>>>,
}

impl ProductGateway for RecordingProductGateway {
    fn create(
        &self,
        input: NewProducto,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FichaProducto, DomainError>> + Send + '_>,
    > {
        let last = Arc::clone(&self.last_create);
        let input_clone = input.clone();
        Box::pin(async move {
            *last.lock().expect("lock") = Some(input_clone);
            self.inner.create(input).await
        })
    }

    fn get(
        &self,
        id: Id<Producto>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FichaProducto, DomainError>> + Send + '_>,
    > {
        Box::pin(self.inner.get(id))
    }

    fn update(
        &self,
        patch: PatchProducto,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FichaProducto, DomainError>> + Send + '_>,
    > {
        Box::pin(self.inner.update(patch))
    }

    fn deactivate(
        &self,
        id: Id<Producto>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FichaProducto, DomainError>> + Send + '_>,
    > {
        Box::pin(self.inner.deactivate(id))
    }

    fn reactivate(
        &self,
        id: Id<Producto>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FichaProducto, DomainError>> + Send + '_>,
    > {
        Box::pin(self.inner.reactivate(id))
    }

    fn list(
        &self,
        filtro: FiltroProducto,
        after: Option<Id<Producto>>,
        take: u32,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send + '_,
        >,
    > {
        Box::pin(self.inner.list(filtro, after, take))
    }

    fn find_by_code(
        &self,
        code: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send + '_,
        >,
    > {
        Box::pin(self.inner.find_by_code(code))
    }
}

pub async fn test_product_gateway() -> (
    Arc<dyn ProductGateway>,
    Arc<Mutex<Option<NewProducto>>>,
    ProductTestSeeds,
) {
    let categories = FakeCategoryRepo::new();
    let fijaciones = categories
        .create(NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("fijaciones");
    let fijaciones_id = fijaciones.id.clone();
    let pernos = categories
        .create(NewCategoria {
            nombre: "Pernos".into(),
            slug: None,
            padre: Some(fijaciones.id),
        })
        .await
        .expect("pernos");
    let brands = FakeBrandRepo::new();
    let marca = brands
        .create(NewMarca {
            nombre: "Fixser".into(),
            slug: None,
        })
        .await
        .expect("marca");
    let attrs = FakeAttributeDefRepo::new();
    let medida = attrs
        .create(NewAtributoDef {
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect("medida");
    let largo = attrs
        .create(NewAtributoDef {
            clave: "largo".into(),
            etiqueta: "Largo".into(),
            tipo: TipoAtributo::Numero,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 1,
        })
        .await
        .expect("largo");
    let grado = attrs
        .create(NewAtributoDef {
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: Some(vec!["2".into(), "5".into(), "8".into()]),
            facetable: true,
            buscable: true,
            orden: 2,
        })
        .await
        .expect("grado");

    let categories_for_seeds = categories.clone();
    let product_repo = FakeProductRepo::new(categories.clone(), brands.clone(), attrs.clone());
    let service = ProductService::new(product_repo, categories, brands, attrs);
    let last_create = Arc::new(Mutex::new(None));
    let gateway = Arc::new(RecordingProductGateway {
        inner: service,
        last_create: Arc::clone(&last_create),
    });
    let seeds = ProductTestSeeds {
        fijaciones_id,
        category_id: pernos.id,
        brand_id: marca.id,
        medida_id: medida.id,
        largo_id: largo.id,
        grado_id: grado.id,
        categories: categories_for_seeds,
    };
    (gateway, last_create, seeds)
}

pub async fn test_stock_gateways() -> (
    Arc<dyn ProductGateway>,
    Arc<dyn StockGateway>,
    ProductTestSeeds,
) {
    let categories = FakeCategoryRepo::new();
    let fijaciones = categories
        .create(NewCategoria {
            nombre: "Fijaciones".into(),
            slug: None,
            padre: None,
        })
        .await
        .expect("fijaciones");
    let fijaciones_id = fijaciones.id.clone();
    let pernos = categories
        .create(NewCategoria {
            nombre: "Pernos".into(),
            slug: None,
            padre: Some(fijaciones.id),
        })
        .await
        .expect("pernos");
    let brands = FakeBrandRepo::new();
    let marca = brands
        .create(NewMarca {
            nombre: "Fixser".into(),
            slug: None,
        })
        .await
        .expect("marca");
    let attrs = FakeAttributeDefRepo::new();
    let medida = attrs
        .create(NewAtributoDef {
            clave: "medida".into(),
            etiqueta: "Medida".into(),
            tipo: TipoAtributo::Texto,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 0,
        })
        .await
        .expect("medida");
    let largo = attrs
        .create(NewAtributoDef {
            clave: "largo".into(),
            etiqueta: "Largo".into(),
            tipo: TipoAtributo::Numero,
            unidad: Some("mm".into()),
            opciones: None,
            facetable: false,
            buscable: true,
            orden: 1,
        })
        .await
        .expect("largo");
    let grado = attrs
        .create(NewAtributoDef {
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: Some(vec!["2".into(), "5".into(), "8".into()]),
            facetable: true,
            buscable: true,
            orden: 2,
        })
        .await
        .expect("grado");

    let categories_for_seeds = categories.clone();
    let product_repo = FakeProductRepo::new(categories.clone(), brands.clone(), attrs.clone());
    let product_store = product_repo.product_store();
    let service = ProductService::new(product_repo, categories, brands, attrs);
    let last_create = Arc::new(Mutex::new(None));
    let product_gateway: Arc<dyn ProductGateway> = Arc::new(RecordingProductGateway {
        inner: service,
        last_create: Arc::clone(&last_create),
    });
    let stock: Arc<dyn StockGateway> =
        Arc::new(StockService::new(InMemoryStockLedger::new(product_store)));
    let seeds = ProductTestSeeds {
        fijaciones_id,
        category_id: pernos.id,
        brand_id: marca.id,
        medida_id: medida.id,
        largo_id: largo.id,
        grado_id: grado.id,
        categories: categories_for_seeds,
    };
    (product_gateway, stock, seeds)
}

pub fn product_create_json_stock(
    seeds: &ProductTestSeeds,
    sku: &str,
    initial_stock: &str,
) -> String {
    format!(
        r#"{{
            "sku": "{sku}",
            "name": "Producto stock test",
            "productType": "Perno",
            "brandId": "{}",
            "categoryId": "{}",
            "attributes": [
                {{ "definitionId": "{}", "value": "M8" }},
                {{ "definitionId": "{}", "value": 40 }},
                {{ "definitionId": "{}", "value": "5" }}
            ],
            "unitOfMeasure": "unit",
            "price": "10",
            "initialStock": "{initial_stock}"
        }}"#,
        seeds.brand_id, seeds.category_id, seeds.medida_id, seeds.largo_id, seeds.grado_id
    )
}

pub fn product_create_json(
    seeds: &ProductTestSeeds,
    sku: &str,
    name: &str,
    category_id: &Id<Categoria>,
) -> String {
    format!(
        r#"{{
            "sku": "{sku}",
            "name": "{name}",
            "productType": "Perno",
            "brandId": "{}",
            "categoryId": "{}",
            "attributes": [
                {{ "definitionId": "{}", "value": "M8" }},
                {{ "definitionId": "{}", "value": 40 }},
                {{ "definitionId": "{}", "value": "5" }}
            ],
            "unitOfMeasure": "unit",
            "price": "10",
            "initialStock": "0"
        }}"#,
        seeds.brand_id, category_id, seeds.medida_id, seeds.largo_id, seeds.grado_id
    )
}

pub fn perno_create_json(seeds: &ProductTestSeeds) -> String {
    format!(
        r#"{{
            "sku": "PER-M8X40-ZN",
            "name": "Perno hexagonal M8 x 40 zincado",
            "productType": "Perno",
            "brandId": "{}",
            "categoryId": "{}",
            "attributes": [
                {{ "definitionId": "{}", "value": "M8" }},
                {{ "definitionId": "{}", "value": 40 }},
                {{ "definitionId": "{}", "value": "5" }}
            ],
            "codes": [{{ "type": "manufacturer", "value": "HX-0840Z" }}],
            "unitOfMeasure": "unit",
            "price": "120.50",
            "location": {{ "aisle": "3", "shelf": "B", "position": "12" }},
            "initialStock": "50"
        }}"#,
        seeds.brand_id, seeds.category_id, seeds.medida_id, seeds.largo_id, seeds.grado_id
    )
}
