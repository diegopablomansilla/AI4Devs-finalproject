//! In-memory `CategoryRepo` for HTTP integration tests (not the `catalog` crate test module).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use catalog::{
    AttributeDefGateway, AttributeDefService, BrandGateway, BrandService, CategoryGateway,
    CategoryService, ProductionOccupancy,
};
use domain::{
    AtributoDef, AtributoDefDraft, AttributeDefRepo, BrandRepo, Categoria, CategoriaDraft,
    CategoryRepo, ConflictCode, DomainError, Id, InvalidInputCode, MAX_DEPTH, Marca, MarcaDraft,
    NewAtributoDef, NewCategoria, NewMarca, Slug, Timestamp, UpdateAtributoDef, UpdateCategoria,
    UpdateMarca, reject_cycle,
};

const TS: &str = "2026-09-24T21:33:00Z";

fn ts() -> Timestamp {
    TS.parse().expect("timestamp")
}

pub struct FakeCategoryRepo {
    inner: Mutex<HashMap<String, Categoria>>,
}

impl FakeCategoryRepo {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
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

pub struct FakeBrandRepo {
    inner: Mutex<HashMap<String, Marca>>,
}

impl FakeBrandRepo {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
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

pub struct FakeAttributeDefRepo {
    inner: Mutex<HashMap<String, AtributoDef>>,
}

impl FakeAttributeDefRepo {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
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

pub fn test_attribute_def_gateway() -> Arc<dyn AttributeDefGateway> {
    Arc::new(AttributeDefService::new(FakeAttributeDefRepo::new()))
}
