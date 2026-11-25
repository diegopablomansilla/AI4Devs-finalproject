//! Product use cases (T-02-20).

use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

use domain::{
    AtributoDef, AtributoValor, AttributeDefRepo, BrandRepo, Categoria, CategoryRepo, Codigo,
    DomainError, FichaProducto, FieldError, FiltroProducto, Id, InvalidInputCode, Marca,
    NewProducto, ProductRepo, Producto, ProductoDraft, ResumenProducto, Timestamp, UnidadMedida,
    UpdateProducto,
};

/// Partial product sheet update (application layer; HTTP merge in T-02-21).
#[derive(Debug, Clone)]
pub struct PatchProducto {
    pub id: Id<domain::Producto>,
    pub version: i32,
    pub sku: Option<String>,
    pub nombre: Option<String>,
    pub tipo: Option<String>,
    pub descripcion: Option<Option<String>>,
    pub marca: Option<Option<Id<Marca>>>,
    pub categoria: Option<Id<Categoria>>,
    pub atributos: Option<Vec<AtributoValor>>,
    pub codigos: Option<Vec<Codigo>>,
    pub unidad_medida: Option<UnidadMedida>,
    pub precio: Option<domain::Decimal>,
    pub ubicacion: Option<Option<domain::Ubicacion>>,
}

pub struct ProductService<P, C, B, A> {
    pub products: P,
    pub categories: C,
    pub brands: B,
    pub attrs: A,
}

impl<P, C, B, A> ProductService<P, C, B, A>
where
    P: ProductRepo,
    C: CategoryRepo,
    B: BrandRepo,
    A: AttributeDefRepo,
{
    pub fn new(products: P, categories: C, brands: B, attrs: A) -> Self {
        Self {
            products,
            categories,
            brands,
            attrs,
        }
    }

    pub async fn create(&self, input: NewProducto) -> Result<FichaProducto, DomainError> {
        let defs = self
            .load_and_validate_refs(&input.categoria, input.marca.as_ref(), &input.atributos)
            .await?;
        let _ = preview_producto(&input, &defs)?;
        let created = self.products.create(input).await?;
        self.products.get(created.id).await
    }

    pub async fn get(&self, id: Id<domain::Producto>) -> Result<FichaProducto, DomainError> {
        self.products.get(id).await
    }

    pub async fn update(&self, patch: PatchProducto) -> Result<FichaProducto, DomainError> {
        if !patch_has_body(&patch) {
            return Err(DomainError::invalid_input(
                InvalidInputCode::InvalidInput,
                vec![FieldError::try_new("patch", "no fields to update").expect("field")],
            ));
        }
        let current = self.products.get(patch.id.clone()).await?;
        let snapshot = merge_patch(&current, &patch);
        let defs = self
            .load_and_validate_refs(
                &snapshot.categoria,
                snapshot.marca.as_ref(),
                &snapshot.atributos,
            )
            .await?;
        let _ = preview_update(&current.producto, &snapshot, &defs)?;
        self.products.update(snapshot).await
    }

    pub async fn deactivate(&self, id: Id<domain::Producto>) -> Result<FichaProducto, DomainError> {
        let current = self.products.get(id.clone()).await?;
        if !current.producto.activo {
            return Ok(current);
        }
        self.products.set_activo(id, false).await
    }

    pub async fn reactivate(&self, id: Id<domain::Producto>) -> Result<FichaProducto, DomainError> {
        let current = self.products.get(id.clone()).await?;
        if current.producto.activo {
            return Ok(current);
        }
        self.products.set_activo(id, true).await
    }

    pub async fn list(
        &self,
        filtro: FiltroProducto,
        after: Option<Id<domain::Producto>>,
        take: u32,
    ) -> Result<Vec<ResumenProducto>, DomainError> {
        self.products.list(filtro, after, take).await
    }

    pub async fn find_by_code(&self, code: String) -> Result<Vec<ResumenProducto>, DomainError> {
        self.products.find_by_code(code).await
    }

    async fn load_and_validate_refs(
        &self,
        categoria: &Id<Categoria>,
        marca: Option<&Id<Marca>>,
        atributos: &[AtributoValor],
    ) -> Result<Vec<AtributoDef>, DomainError> {
        require_active_categoria(&self.categories, categoria.clone()).await?;
        if let Some(brand_id) = marca {
            require_active_marca(&self.brands, brand_id.clone()).await?;
        }
        load_attribute_defs(&self.attrs, atributos).await
    }
}

fn patch_has_body(patch: &PatchProducto) -> bool {
    patch.sku.is_some()
        || patch.nombre.is_some()
        || patch.tipo.is_some()
        || patch.descripcion.is_some()
        || patch.marca.is_some()
        || patch.categoria.is_some()
        || patch.atributos.is_some()
        || patch.codigos.is_some()
        || patch.unidad_medida.is_some()
        || patch.precio.is_some()
        || patch.ubicacion.is_some()
}

fn merge_patch(current: &FichaProducto, patch: &PatchProducto) -> UpdateProducto {
    let p = &current.producto;
    UpdateProducto {
        id: patch.id.clone(),
        version: patch.version,
        sku: patch
            .sku
            .clone()
            .unwrap_or_else(|| p.sku.as_str().to_string()),
        nombre: patch.nombre.clone().unwrap_or_else(|| p.nombre.clone()),
        tipo: patch.tipo.clone().unwrap_or_else(|| p.tipo.clone()),
        descripcion: patch.descripcion.clone().unwrap_or(p.descripcion.clone()),
        marca: patch.marca.clone().unwrap_or(p.marca.clone()),
        categoria: patch
            .categoria
            .clone()
            .unwrap_or_else(|| p.categoria.clone()),
        atributos: patch
            .atributos
            .clone()
            .unwrap_or_else(|| p.atributos.clone()),
        codigos: patch.codigos.clone().unwrap_or_else(|| p.codigos.clone()),
        unidad_medida: patch.unidad_medida.unwrap_or(p.unidad_medida),
        precio: patch.precio.unwrap_or(p.precio),
        ubicacion: patch.ubicacion.clone().unwrap_or(p.ubicacion.clone()),
    }
}

async fn require_active_categoria<R: CategoryRepo>(
    repo: &R,
    id: Id<Categoria>,
) -> Result<(), DomainError> {
    match repo.get(id).await {
        Ok(c) if c.activo => Ok(()),
        Ok(_) => Err(field_invalid_input("categoria", "category must be active")),
        Err(DomainError::NotFound) => Err(field_invalid_input("categoria", "unknown category")),
        Err(e) => Err(e),
    }
}

async fn require_active_marca<R: BrandRepo>(repo: &R, id: Id<Marca>) -> Result<(), DomainError> {
    match repo.get(id).await {
        Ok(m) if m.activo => Ok(()),
        Ok(_) => Err(field_invalid_input("marca", "brand must be active")),
        Err(DomainError::NotFound) => Err(field_invalid_input("marca", "unknown brand")),
        Err(e) => Err(e),
    }
}

async fn load_attribute_defs<R: AttributeDefRepo>(
    repo: &R,
    atributos: &[AtributoValor],
) -> Result<Vec<AtributoDef>, DomainError> {
    let mut defs = Vec::with_capacity(atributos.len());
    for (n, attr) in atributos.iter().enumerate() {
        match repo.get(attr.def.clone()).await {
            Ok(d) => defs.push(d),
            Err(DomainError::NotFound) => {
                return Err(field_invalid_input(
                    &format!("attributes[{n}].definitionId"),
                    "unknown attribute definition",
                ));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(defs)
}

fn preview_producto(input: &NewProducto, defs: &[AtributoDef]) -> Result<Producto, DomainError> {
    let id = Id::<domain::Producto>::generate();
    let ts = Timestamp::from_str("2026-09-24T21:33:00Z").expect("ts");
    Producto::try_new(
        ProductoDraft {
            id,
            sku: input.sku.clone(),
            nombre: input.nombre.clone(),
            tipo: input.tipo.clone(),
            descripcion: input.descripcion.clone(),
            marca: input.marca.clone(),
            categoria: input.categoria.clone(),
            atributos: input.atributos.clone(),
            codigos: input.codigos.clone(),
            unidad_medida: input.unidad_medida,
            precio: input.precio,
            ubicacion: input.ubicacion.clone(),
            stock: input.stock_inicial,
            activo: true,
            version: 1,
            creado_en: ts,
            actualizado_en: ts,
        },
        defs,
    )
}

fn preview_update(
    current: &Producto,
    update: &UpdateProducto,
    defs: &[AtributoDef],
) -> Result<Producto, DomainError> {
    Producto::try_new(
        ProductoDraft {
            id: current.id.clone(),
            sku: update.sku.clone(),
            nombre: update.nombre.clone(),
            tipo: update.tipo.clone(),
            descripcion: update.descripcion.clone(),
            marca: update.marca.clone(),
            categoria: update.categoria.clone(),
            atributos: update.atributos.clone(),
            codigos: update.codigos.clone(),
            unidad_medida: update.unidad_medida,
            precio: update.precio,
            ubicacion: update.ubicacion.clone(),
            stock: current.stock,
            activo: current.activo,
            version: update.version,
            creado_en: current.creado_en,
            actualizado_en: current.actualizado_en,
        },
        defs,
    )
}

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

pub fn product_matches_categoria_branch(
    product_categoria: &Id<Categoria>,
    filter_categoria: &Id<Categoria>,
    categoria: &Categoria,
) -> bool {
    product_categoria == filter_categoria || categoria.ancestros.contains(filter_categoria)
}

pub fn product_id_after(after: &Id<domain::Producto>, id: &Id<domain::Producto>) -> bool {
    id.to_string() > after.to_string()
}

/// Object-safe facade for HTTP injection (`Arc<dyn ProductGateway>`).
pub trait ProductGateway: Send + Sync {
    fn create(
        &self,
        input: NewProducto,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>>;

    fn get(
        &self,
        id: Id<domain::Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>>;

    fn update(
        &self,
        patch: PatchProducto,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>>;

    fn deactivate(
        &self,
        id: Id<domain::Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>>;

    fn reactivate(
        &self,
        id: Id<domain::Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>>;

    fn list(
        &self,
        filtro: FiltroProducto,
        after: Option<Id<domain::Producto>>,
        take: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send + '_>>;

    fn find_by_code(
        &self,
        code: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send + '_>>;
}

impl<P, C, B, A> ProductGateway for ProductService<P, C, B, A>
where
    P: ProductRepo + Send + Sync,
    C: CategoryRepo + Send + Sync,
    B: BrandRepo + Send + Sync,
    A: AttributeDefRepo + Send + Sync,
{
    fn create(
        &self,
        input: NewProducto,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>> {
        Box::pin(ProductService::create(self, input))
    }

    fn get(
        &self,
        id: Id<domain::Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>> {
        Box::pin(ProductService::get(self, id))
    }

    fn update(
        &self,
        patch: PatchProducto,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>> {
        Box::pin(ProductService::update(self, patch))
    }

    fn deactivate(
        &self,
        id: Id<domain::Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>> {
        Box::pin(ProductService::deactivate(self, id))
    }

    fn reactivate(
        &self,
        id: Id<domain::Producto>,
    ) -> Pin<Box<dyn Future<Output = Result<FichaProducto, DomainError>> + Send + '_>> {
        Box::pin(ProductService::reactivate(self, id))
    }

    fn list(
        &self,
        filtro: FiltroProducto,
        after: Option<Id<domain::Producto>>,
        take: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send + '_>> {
        Box::pin(ProductService::list(self, filtro, after, take))
    }

    fn find_by_code(
        &self,
        code: String,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send + '_>> {
        Box::pin(ProductService::find_by_code(self, code))
    }
}

#[cfg(test)]
pub mod testing {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use domain::{ConflictCode, Sku};

    use super::*;

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    pub struct InMemoryProductRepo<C, B, A> {
        inner: Mutex<HashMap<String, Producto>>,
        categories: C,
        brands: B,
        attrs: A,
    }

    impl<C, B, A> InMemoryProductRepo<C, B, A> {
        pub fn new(categories: C, brands: B, attrs: A) -> Self {
            Self {
                inner: Mutex::new(HashMap::new()),
                categories,
                brands,
                attrs,
            }
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
    }

    impl<C, B, A> InMemoryProductRepo<C, B, A>
    where
        C: CategoryRepo,
        B: BrandRepo,
        A: AttributeDefRepo,
    {
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
                ficha_attrs.push(domain::AtributoFicha {
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

    impl<C, B, A> ProductRepo for InMemoryProductRepo<C, B, A>
    where
        C: CategoryRepo + Send + Sync,
        B: BrandRepo + Send + Sync,
        A: AttributeDefRepo + Send + Sync,
    {
        fn create(
            &self,
            input: NewProducto,
        ) -> impl Future<Output = Result<Producto, DomainError>> + Send {
            async move {
                let sku = Sku::try_new(&input.sku)?;
                let map = self.inner.lock().expect("lock");
                if Self::sku_taken(&map, sku.as_str(), None) {
                    return Err(DomainError::conflict(ConflictCode::DuplicateSku));
                }
                drop(map);

                let id = Id::<domain::Producto>::generate();
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
            id: Id<domain::Producto>,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
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
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
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
                {
                    let mut map = self.inner.lock().expect("lock");
                    if Self::sku_taken(&map, built.sku.as_str(), Some(&input.id)) {
                        return Err(DomainError::conflict(ConflictCode::DuplicateSku));
                    }
                    map.insert(input.id.to_string(), built.clone());
                }
                self.hydrate_ficha(built).await
            }
        }

        fn set_activo(
            &self,
            id: Id<domain::Producto>,
            activo: bool,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
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
            after: Option<Id<domain::Producto>>,
            take: u32,
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
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
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
            async move {
                let items: Vec<Producto> = {
                    let map = self.inner.lock().expect("lock");
                    map.values()
                        .filter(|p| p.activo)
                        .filter(|p| {
                            p.sku.as_str() == code || p.codigos.iter().any(|c| c.valor == code)
                        })
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

    use crate::atributo_def::testing::InMemoryAttributeDefRepo;
    use crate::categoria::testing::InMemoryCategoryRepo;
    use crate::marca::testing::InMemoryBrandRepo;

    #[derive(Clone)]
    pub struct SharedCategoryRepo(pub Arc<InMemoryCategoryRepo>);

    #[derive(Clone)]
    pub struct SharedBrandRepo(pub Arc<InMemoryBrandRepo>);

    #[derive(Clone)]
    pub struct SharedAttributeDefRepo(pub Arc<InMemoryAttributeDefRepo>);

    use domain::{
        NewAtributoDef, NewCategoria, NewMarca, UpdateAtributoDef, UpdateCategoria, UpdateMarca,
    };

    impl CategoryRepo for SharedCategoryRepo {
        fn create(
            &self,
            input: NewCategoria,
        ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
            self.0.create(input)
        }

        fn get(
            &self,
            id: Id<Categoria>,
        ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
            self.0.get(id)
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<Categoria>, DomainError>> + Send {
            self.0.list_all()
        }

        fn update(
            &self,
            input: UpdateCategoria,
        ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
            self.0.update(input)
        }

        fn set_activo(
            &self,
            id: Id<Categoria>,
            activo: bool,
        ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
            self.0.set_activo(id, activo)
        }
    }

    impl BrandRepo for SharedBrandRepo {
        fn create(
            &self,
            input: NewMarca,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            self.0.create(input)
        }

        fn get(&self, id: Id<Marca>) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            self.0.get(id)
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<Marca>, DomainError>> + Send {
            self.0.list_all()
        }

        fn update(
            &self,
            input: UpdateMarca,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            self.0.update(input)
        }

        fn set_activo(
            &self,
            id: Id<Marca>,
            activo: bool,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            self.0.set_activo(id, activo)
        }
    }

    impl AttributeDefRepo for SharedAttributeDefRepo {
        fn create(
            &self,
            input: NewAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            self.0.create(input)
        }

        fn get(
            &self,
            id: Id<AtributoDef>,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            self.0.get(id)
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send {
            self.0.list_all()
        }

        fn update(
            &self,
            input: UpdateAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            self.0.update(input)
        }

        fn set_activo(
            &self,
            id: Id<AtributoDef>,
            activo: bool,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            self.0.set_activo(id, activo)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{
        InMemoryProductRepo, SharedAttributeDefRepo, SharedBrandRepo, SharedCategoryRepo,
    };
    use super::*;
    use crate::atributo_def::testing::InMemoryAttributeDefRepo;
    use crate::categoria::testing::InMemoryCategoryRepo;
    use crate::marca::testing::InMemoryBrandRepo;
    use domain::{
        ConflictCode, DomainCode, FieldError, NewAtributoDef, NewCategoria, NewMarca, TipoAtributo,
        Ubicacion, ValorAtributo,
    };
    use std::sync::{Arc, Mutex};

    struct Seeds {
        fijaciones_id: Id<Categoria>,
        categoria_id: Id<Categoria>,
        marca_id: Id<Marca>,
        medida_id: Id<AtributoDef>,
        largo_id: Id<AtributoDef>,
        grado_id: Id<AtributoDef>,
        categories: Arc<InMemoryCategoryRepo>,
        brands: Arc<InMemoryBrandRepo>,
    }

    type TestService = ProductService<
        InMemoryProductRepo<SharedCategoryRepo, SharedBrandRepo, SharedAttributeDefRepo>,
        SharedCategoryRepo,
        SharedBrandRepo,
        SharedAttributeDefRepo,
    >;

    fn patch_precio(
        id: Id<domain::Producto>,
        version: i32,
        precio: domain::Decimal,
    ) -> PatchProducto {
        PatchProducto {
            id,
            version,
            precio: Some(precio),
            sku: None,
            nombre: None,
            tipo: None,
            descripcion: None,
            marca: None,
            categoria: None,
            atributos: None,
            codigos: None,
            unidad_medida: None,
            ubicacion: None,
        }
    }

    fn patch_categoria(
        id: Id<domain::Producto>,
        version: i32,
        categoria: Id<Categoria>,
    ) -> PatchProducto {
        PatchProducto {
            id,
            version,
            categoria: Some(categoria),
            sku: None,
            nombre: None,
            tipo: None,
            descripcion: None,
            marca: None,
            atributos: None,
            codigos: None,
            unidad_medida: None,
            precio: None,
            ubicacion: None,
        }
    }

    fn invalid_fields(err: &DomainError) -> &[FieldError] {
        match err {
            DomainError::InvalidInput { errors, .. } => errors,
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    async fn seed_active_catalog() -> (TestService, Seeds) {
        let categories = Arc::new(InMemoryCategoryRepo::new());
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
        let brands = Arc::new(InMemoryBrandRepo::new());
        let marca = brands
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("marca");
        let attrs = Arc::new(InMemoryAttributeDefRepo::new());
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
        let cat_shared = SharedCategoryRepo(Arc::clone(&categories));
        let brand_shared = SharedBrandRepo(Arc::clone(&brands));
        let attr_shared = SharedAttributeDefRepo(Arc::clone(&attrs));
        let products = InMemoryProductRepo::new(
            cat_shared.clone(),
            brand_shared.clone(),
            attr_shared.clone(),
        );
        let service = ProductService::new(products, cat_shared, brand_shared, attr_shared);
        let seeds = Seeds {
            fijaciones_id,
            categoria_id: pernos.id,
            marca_id: marca.id,
            medida_id: medida.id,
            largo_id: largo.id,
            grado_id: grado.id,
            categories,
            brands,
        };
        (service, seeds)
    }

    fn new_producto_for_category(
        seeds: &Seeds,
        sku: &str,
        categoria: Id<Categoria>,
    ) -> NewProducto {
        let mut input = perno_new_producto(seeds);
        input.sku = sku.into();
        input.nombre = format!("Producto {sku}");
        input.categoria = categoria;
        input
    }

    fn perno_new_producto(seeds: &Seeds) -> NewProducto {
        NewProducto {
            sku: "PER-M8X40-ZN".into(),
            nombre: "Perno hexagonal M8 x 40 zincado".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: Some(seeds.marca_id.clone()),
            categoria: seeds.categoria_id.clone(),
            atributos: vec![
                AtributoValor {
                    def: seeds.medida_id.clone(),
                    valor: ValorAtributo::Texto("M8".into()),
                },
                AtributoValor {
                    def: seeds.largo_id.clone(),
                    valor: ValorAtributo::Numero("40".parse().expect("40")),
                },
                AtributoValor {
                    def: seeds.grado_id.clone(),
                    valor: ValorAtributo::Texto("5".into()),
                },
            ],
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "120.50".parse().expect("precio"),
            ubicacion: Some(Ubicacion {
                pasillo: Some("3".into()),
                estante: Some("B".into()),
                posicion: Some("12".into()),
            }),
            stock_inicial: "50".parse().expect("stock"),
            terminal: None,
        }
    }

    #[tokio::test]
    async fn catalog_create_perno_m8x40_sets_version_1() {
        let (service, seeds) = seed_active_catalog().await;
        let ficha = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        assert!(ficha.producto.activo);
        assert_eq!(ficha.producto.version, 1);
        assert_eq!(ficha.producto.stock.to_string(), "50");
        assert_eq!(ficha.categoria.ruta, vec!["Fijaciones", "Pernos"]);
        assert!(ficha.marca.is_some());
        assert_eq!(ficha.atributos.len(), 3);
        assert_eq!(ficha.atributos[0].clave, "medida");
    }

    #[tokio::test]
    async fn catalog_get_returns_ficha_with_marca_ruta_and_attributes() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let ficha = service.get(created.producto.id.clone()).await.expect("get");
        assert_eq!(ficha.categoria.ruta, vec!["Fijaciones", "Pernos"]);
        assert_eq!(ficha.atributos[0].unidad.as_deref(), Some("mm"));
    }

    #[tokio::test]
    async fn catalog_create_duplicate_sku_is_duplicate_sku() {
        let (service, seeds) = seed_active_catalog().await;
        service
            .create(perno_new_producto(&seeds))
            .await
            .expect("first");
        let err = service
            .create(perno_new_producto(&seeds))
            .await
            .expect_err("duplicate");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::DuplicateSku
            }
        ));
    }

    #[tokio::test]
    async fn catalog_create_cuarenta_on_number_attribute_is_invalid_input() {
        let (service, seeds) = seed_active_catalog().await;
        let mut input = perno_new_producto(&seeds);
        input.atributos[1].valor = ValorAtributo::Texto("cuarenta".into());
        let err = service.create(input).await.expect_err("cuarenta");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "attributes[1].value");
    }

    #[tokio::test]
    async fn catalog_create_inactive_categoria_is_invalid_input() {
        let (service, seeds) = seed_active_catalog().await;
        seeds
            .categories
            .set_activo(seeds.categoria_id.clone(), false)
            .await
            .expect("deactivate");
        let err = service
            .create(perno_new_producto(&seeds))
            .await
            .expect_err("inactive cat");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "categoria");
    }

    #[tokio::test]
    async fn catalog_create_inactive_marca_is_invalid_input() {
        let (service, seeds) = seed_active_catalog().await;
        service
            .brands
            .set_activo(seeds.marca_id.clone(), false)
            .await
            .expect("deactivate");
        let err = service
            .create(perno_new_producto(&seeds))
            .await
            .expect_err("inactive brand");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "marca");
    }

    #[tokio::test]
    async fn catalog_create_unknown_categoria_is_invalid_input() {
        let (service, seeds) = seed_active_catalog().await;
        let mut input = perno_new_producto(&seeds);
        input.categoria = Id::generate();
        let err = service.create(input).await.expect_err("unknown cat");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "categoria");
    }

    #[tokio::test]
    async fn catalog_get_unknown_id_is_not_found() {
        let (service, _seeds) = seed_active_catalog().await;
        let err = service.get(Id::generate()).await.expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_update_price_increments_version() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let stock_before = created.producto.stock;
        let updated = service
            .update(patch_precio(
                created.producto.id.clone(),
                1,
                "130.00".parse().expect("precio"),
            ))
            .await
            .expect("update");
        assert_eq!(updated.producto.precio, "130.00".parse().expect("precio"));
        assert_eq!(updated.producto.version, 2);
        assert_eq!(updated.producto.stock, stock_before);
    }

    #[tokio::test]
    async fn catalog_update_partial_price_keeps_other_fields() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let updated = service
            .update(patch_precio(
                created.producto.id.clone(),
                1,
                "130.00".parse().expect("precio"),
            ))
            .await
            .expect("update");
        assert_eq!(updated.producto.nombre, "Perno hexagonal M8 x 40 zincado");
        assert_eq!(updated.producto.sku.as_str(), "PER-M8X40-ZN");
        assert_eq!(updated.producto.categoria, seeds.categoria_id);
    }

    #[tokio::test]
    async fn catalog_update_stale_version_is_version_conflict() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let product_id = created.producto.id.clone();
        service
            .update(patch_precio(
                product_id.clone(),
                1,
                "130.00".parse().expect("precio"),
            ))
            .await
            .expect("first update");
        let err = service
            .update(patch_precio(
                product_id.clone(),
                1,
                "140.00".parse().expect("precio"),
            ))
            .await
            .expect_err("stale");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::VersionConflict
            }
        ));
        let current = service.products.get(product_id).await.expect("get");
        assert_eq!(current.producto.precio, "130.00".parse().expect("precio"));
    }

    #[tokio::test]
    async fn catalog_update_unknown_id_is_not_found() {
        let (service, _seeds) = seed_active_catalog().await;
        let err = service
            .update(patch_precio(Id::generate(), 1, "1".parse().expect("p")))
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_update_inactive_categoria_is_invalid_input() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let fijaciones = seeds
            .categories
            .list_all()
            .await
            .expect("list")
            .into_iter()
            .find(|c| c.nombre == "Fijaciones")
            .expect("fijaciones");
        let fijaciones_id = fijaciones.id.clone();
        seeds
            .categories
            .set_activo(fijaciones_id.clone(), false)
            .await
            .expect("deactivate root");
        let product_id = created.producto.id.clone();
        let err = service
            .update(patch_categoria(product_id.clone(), 1, fijaciones_id))
            .await
            .expect_err("inactive");
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "categoria");
        let still = service.get(product_id).await.expect("get");
        assert_eq!(still.producto.version, 1);
    }

    #[tokio::test]
    async fn product_gateway_is_object_safe() {
        let (service, seeds) = seed_active_catalog().await;
        let gateway: Arc<dyn ProductGateway> = Arc::new(service);
        let ficha = gateway
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        assert_eq!(ficha.producto.sku.as_str(), "PER-M8X40-ZN");
        let listed = gateway
            .list(
                FiltroProducto {
                    categoria: None,
                    marca: None,
                    activo: true,
                },
                None,
                10,
            )
            .await
            .expect("list");
        assert!(!listed.is_empty());
    }

    struct RecordingProductRepo<C, B, A> {
        inner: InMemoryProductRepo<C, B, A>,
        last_create: Arc<Mutex<Option<NewProducto>>>,
    }

    impl<C, B, A> RecordingProductRepo<C, B, A> {
        fn new(inner: InMemoryProductRepo<C, B, A>) -> Self {
            Self {
                inner,
                last_create: Arc::new(Mutex::new(None)),
            }
        }

        fn last_create(&self) -> Option<NewProducto> {
            self.last_create.lock().expect("lock").clone()
        }
    }

    impl<C, B, A> ProductRepo for RecordingProductRepo<C, B, A>
    where
        C: CategoryRepo + Send + Sync,
        B: BrandRepo + Send + Sync,
        A: AttributeDefRepo + Send + Sync,
    {
        fn create(
            &self,
            input: NewProducto,
        ) -> impl Future<Output = Result<Producto, DomainError>> + Send {
            let inner = &self.inner;
            let last = Arc::clone(&self.last_create);
            async move {
                *last.lock().expect("lock") = Some(input.clone());
                inner.create(input).await
            }
        }

        fn get(
            &self,
            id: Id<domain::Producto>,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            self.inner.get(id)
        }

        fn update(
            &self,
            input: UpdateProducto,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            self.inner.update(input)
        }

        fn set_activo(
            &self,
            id: Id<domain::Producto>,
            activo: bool,
        ) -> impl Future<Output = Result<FichaProducto, DomainError>> + Send {
            self.inner.set_activo(id, activo)
        }

        fn list(
            &self,
            filtro: FiltroProducto,
            after: Option<Id<domain::Producto>>,
            take: u32,
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
            self.inner.list(filtro, after, take)
        }

        fn find_by_code(
            &self,
            code: String,
        ) -> impl Future<Output = Result<Vec<ResumenProducto>, DomainError>> + Send {
            self.inner.find_by_code(code)
        }
    }

    #[tokio::test]
    async fn product_service_find_by_code_delegates() {
        let (service, seeds) = seed_active_catalog().await;
        let mut input = perno_new_producto(&seeds);
        input.codigos =
            vec![Codigo::try_new(domain::TipoCodigo::Fabricante, "HX-0840Z").expect("fabricante")];
        service.create(input).await.expect("create");
        let rows = service
            .find_by_code("HX-0840Z".into())
            .await
            .expect("by code");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sku, "PER-M8X40-ZN");
        assert!(
            service
                .find_by_code("HX-0841Z".into())
                .await
                .expect("miss")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn catalog_list_branch_includes_descendants() {
        let (service, seeds) = seed_active_catalog().await;
        let other = seeds
            .categories
            .create(NewCategoria {
                nombre: "Tornillería".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("other cat");
        service
            .create(new_producto_for_category(
                &seeds,
                "SKU-FIJ",
                seeds.fijaciones_id.clone(),
            ))
            .await
            .expect("fijaciones");
        service
            .create(new_producto_for_category(
                &seeds,
                "SKU-PER",
                seeds.categoria_id.clone(),
            ))
            .await
            .expect("pernos");
        service
            .create(new_producto_for_category(&seeds, "SKU-OTH", other.id))
            .await
            .expect("other");
        let filtro = FiltroProducto {
            categoria: Some(seeds.fijaciones_id.clone()),
            marca: None,
            activo: true,
        };
        let rows = service.list(filtro, None, 50).await.expect("list");
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .all(|r| r.categoria_ruta.first().map(|s| s.as_str()) == Some("Fijaciones"))
        );
        let skus: Vec<_> = rows.iter().map(|r| r.sku.as_str()).collect();
        assert!(skus.contains(&"SKU-FIJ"));
        assert!(skus.contains(&"SKU-PER"));
        assert!(!skus.contains(&"SKU-OTH"));
    }

    #[tokio::test]
    async fn catalog_list_inactive_only() {
        let (service, seeds) = seed_active_catalog().await;
        let active = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("active");
        let inactive = service
            .create(new_producto_for_category(
                &seeds,
                "SKU-INACT",
                seeds.categoria_id.clone(),
            ))
            .await
            .expect("inactive");
        let inactive_id = inactive.producto.id.clone();
        service
            .deactivate(inactive_id.clone())
            .await
            .expect("deactivate");
        let filtro = FiltroProducto {
            categoria: None,
            marca: None,
            activo: false,
        };
        let rows = service.list(filtro, None, 50).await.expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, inactive_id);
        assert!(!rows[0].activo);
        assert_ne!(rows[0].id, active.producto.id);
    }

    #[tokio::test]
    async fn catalog_list_filters_by_brand() {
        let (service, seeds) = seed_active_catalog().await;
        let other_marca = seeds
            .brands
            .create(NewMarca {
                nombre: "Otra".into(),
                slug: None,
            })
            .await
            .expect("other brand");
        let mut a = perno_new_producto(&seeds);
        a.sku = "SKU-A".into();
        service.create(a).await.expect("a");
        let mut b = perno_new_producto(&seeds);
        b.sku = "SKU-B".into();
        service.create(b).await.expect("b");
        let mut c = perno_new_producto(&seeds);
        c.sku = "SKU-C".into();
        c.marca = Some(other_marca.id);
        service.create(c).await.expect("c");
        let filtro = FiltroProducto {
            categoria: None,
            marca: Some(seeds.marca_id.clone()),
            activo: true,
        };
        let rows = service.list(filtro, None, 50).await.expect("list");
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .all(|r| r.marca_nombre.as_deref() == Some("Fixser"))
        );
        let skus: Vec<_> = rows.iter().map(|r| r.sku.as_str()).collect();
        assert!(skus.contains(&"SKU-A"));
        assert!(skus.contains(&"SKU-B"));
        assert!(!skus.contains(&"SKU-C"));
    }

    #[tokio::test]
    async fn catalog_list_orders_by_id_and_after() {
        let (service, seeds) = seed_active_catalog().await;
        let a = service
            .create(new_producto_for_category(
                &seeds,
                "SKU-A",
                seeds.categoria_id.clone(),
            ))
            .await
            .expect("a");
        let b = service
            .create(new_producto_for_category(
                &seeds,
                "SKU-B",
                seeds.categoria_id.clone(),
            ))
            .await
            .expect("b");
        let c = service
            .create(new_producto_for_category(
                &seeds,
                "SKU-C",
                seeds.categoria_id.clone(),
            ))
            .await
            .expect("c");
        let filtro = FiltroProducto {
            categoria: None,
            marca: None,
            activo: true,
        };
        let page1 = service.list(filtro.clone(), None, 2).await.expect("page1");
        assert_eq!(page1.len(), 2);
        assert!(
            page1[0].id.to_string() < page1[1].id.to_string(),
            "list must be ordered by id ascending"
        );
        let after_id = page1[1].id.clone();
        let page2 = service
            .list(filtro, Some(after_id), 2)
            .await
            .expect("page2");
        assert_eq!(page2.len(), 1);
        let mut seen: std::collections::HashSet<_> = page1
            .iter()
            .chain(page2.iter())
            .map(|r| r.id.to_string())
            .collect();
        assert_eq!(seen.len(), 3);
        for id in [a.producto.id, b.producto.id, c.producto.id] {
            assert!(seen.remove(&id.to_string()));
        }
        assert!(page2[0].id.to_string() > page1[1].id.to_string());
    }

    #[tokio::test]
    async fn catalog_list_unknown_category_is_empty() {
        let (service, seeds) = seed_active_catalog().await;
        service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let filtro = FiltroProducto {
            categoria: Some(Id::generate()),
            marca: None,
            activo: true,
        };
        let rows = service.list(filtro, None, 50).await.expect("list");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn catalog_deactivate_sets_inactive() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let deactivated = service
            .deactivate(created.producto.id.clone())
            .await
            .expect("deactivate");
        assert!(!deactivated.producto.activo);
    }

    #[tokio::test]
    async fn catalog_reactivate_sets_active() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        service
            .deactivate(created.producto.id.clone())
            .await
            .expect("deactivate");
        let reactivated = service
            .reactivate(created.producto.id)
            .await
            .expect("reactivate");
        assert!(reactivated.producto.activo);
    }

    #[tokio::test]
    async fn catalog_deactivate_idempotent_when_already_inactive() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let id = created.producto.id.clone();
        service.deactivate(id.clone()).await.expect("deactivate");
        let again = service.deactivate(id).await.expect("deactivate again");
        assert!(!again.producto.activo);
    }

    #[tokio::test]
    async fn catalog_get_inactive_product_returns_ficha() {
        let (service, seeds) = seed_active_catalog().await;
        let created = service
            .create(perno_new_producto(&seeds))
            .await
            .expect("create");
        let id = created.producto.id.clone();
        service.deactivate(id.clone()).await.expect("deactivate");
        let got = service.get(id).await.expect("get");
        assert!(!got.producto.activo);
    }

    #[tokio::test]
    async fn catalog_deactivate_unknown_id_is_not_found() {
        let (service, _) = seed_active_catalog().await;
        let err = service
            .deactivate(Id::generate())
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_reactivate_unknown_id_is_not_found() {
        let (service, _) = seed_active_catalog().await;
        let err = service
            .reactivate(Id::generate())
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_create_forwards_terminal_to_repo() {
        let categories = Arc::new(InMemoryCategoryRepo::new());
        let categoria = categories
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("categoria");
        let brands = Arc::new(InMemoryBrandRepo::new());
        let attrs = Arc::new(InMemoryAttributeDefRepo::new());
        let cat_shared = SharedCategoryRepo(Arc::clone(&categories));
        let brand_shared = SharedBrandRepo(Arc::clone(&brands));
        let attr_shared = SharedAttributeDefRepo(Arc::clone(&attrs));
        let products = RecordingProductRepo::new(InMemoryProductRepo::new(
            cat_shared.clone(),
            brand_shared.clone(),
            attr_shared.clone(),
        ));
        let service = ProductService::new(products, cat_shared, brand_shared, attr_shared);
        let input = NewProducto {
            sku: "SKU-TERM".into(),
            nombre: "Producto".into(),
            tipo: "Perno".into(),
            descripcion: None,
            marca: None,
            categoria: categoria.id,
            atributos: vec![],
            codigos: vec![],
            unidad_medida: UnidadMedida::Unidad,
            precio: "1".parse().expect("precio"),
            ubicacion: None,
            stock_inicial: "0".parse().expect("stock"),
            terminal: Some("caja-1".into()),
        };
        service.create(input).await.expect("create");
        assert_eq!(
            service.products.last_create().and_then(|n| n.terminal),
            Some("caja-1".into())
        );
    }
}
