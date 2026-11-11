//! Category use cases (T-02-05).

use std::future::Future;

use domain::{
    Categoria, CategoryRepo, ConflictCode, DomainError, Id, NewCategoria, UpdateCategoria,
};

/// Whether a category has active products (real impl when `producto` exists — T-02-18).
pub trait CategoryOccupancy {
    fn has_active_products(
        &self,
        id: Id<Categoria>,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;
}

pub struct CategoryService<R, O> {
    pub repo: R,
    pub occupancy: O,
}

impl<R, O> CategoryService<R, O>
where
    R: CategoryRepo,
    O: CategoryOccupancy,
{
    pub fn new(repo: R, occupancy: O) -> Self {
        Self { repo, occupancy }
    }

    pub async fn create(&self, input: NewCategoria) -> Result<Categoria, DomainError> {
        self.repo.create(input).await
    }

    pub async fn get(&self, id: Id<Categoria>) -> Result<Categoria, DomainError> {
        self.repo.get(id).await
    }

    pub async fn list_all(&self) -> Result<Vec<Categoria>, DomainError> {
        self.repo.list_all().await
    }

    pub async fn rename(
        &self,
        id: Id<Categoria>,
        nombre: String,
    ) -> Result<Categoria, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        self.repo
            .update(UpdateCategoria {
                id,
                nombre,
                padre: current.padre,
            })
            .await
    }

    pub async fn move_category(
        &self,
        id: Id<Categoria>,
        padre: Option<Id<Categoria>>,
    ) -> Result<Categoria, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        self.repo
            .update(UpdateCategoria {
                id,
                nombre: current.nombre,
                padre,
            })
            .await
    }

    pub async fn deactivate(&self, id: Id<Categoria>) -> Result<Categoria, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        if !current.activo {
            return Ok(current);
        }
        let all = self.repo.list_all().await?;
        let has_active_child = all
            .iter()
            .any(|c| c.activo && c.padre.as_ref() == Some(&id));
        if has_active_child {
            return Err(DomainError::conflict(ConflictCode::Conflict));
        }
        if self.occupancy.has_active_products(id.clone()).await? {
            return Err(DomainError::conflict(ConflictCode::Conflict));
        }
        self.repo.set_activo(id, false).await
    }

    pub async fn reactivate(&self, id: Id<Categoria>) -> Result<Categoria, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        if current.activo {
            return Ok(current);
        }
        self.repo.set_activo(id, true).await
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use domain::{
        CategoriaDraft, ConflictCode, InvalidInputCode, MAX_DEPTH, Slug, Timestamp, reject_cycle,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    pub struct FakeOccupancy {
        active: Mutex<HashMap<String, bool>>,
    }

    impl FakeOccupancy {
        pub fn new() -> Self {
            Self {
                active: Mutex::new(HashMap::new()),
            }
        }

        pub fn set_active_products(&self, id: Id<Categoria>, has: bool) {
            self.active
                .lock()
                .expect("lock")
                .insert(id.to_string(), has);
        }
    }

    impl CategoryOccupancy for FakeOccupancy {
        async fn has_active_products(&self, id: Id<Categoria>) -> Result<bool, DomainError> {
            Ok(self
                .active
                .lock()
                .expect("lock")
                .get(&id.to_string())
                .copied()
                .unwrap_or(false))
        }
    }

    pub struct InMemoryCategoryRepo {
        inner: Mutex<HashMap<String, Categoria>>,
    }

    impl InMemoryCategoryRepo {
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

        fn subtree_ids(
            map: &HashMap<String, Categoria>,
            root: &Id<Categoria>,
        ) -> Vec<Id<Categoria>> {
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

    impl CategoryRepo for InMemoryCategoryRepo {
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
                if Self::sibling_slug_taken(
                    &map,
                    input.padre.as_ref(),
                    slug.as_str(),
                    Some(&input.id),
                ) {
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

        async fn set_activo(
            &self,
            id: Id<Categoria>,
            activo: bool,
        ) -> Result<Categoria, DomainError> {
            let mut map = self.inner.lock().expect("lock");
            let mut cat = map
                .get(&id.to_string())
                .cloned()
                .ok_or(DomainError::not_found())?;
            cat.activo = activo;
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
}

#[cfg(test)]
mod tests {
    use super::testing::{FakeOccupancy, InMemoryCategoryRepo};
    use super::*;
    use domain::{ConflictCode, DomainCode, DomainError, Slug};

    fn svc(
        repo: InMemoryCategoryRepo,
        occupancy: FakeOccupancy,
    ) -> CategoryService<InMemoryCategoryRepo, FakeOccupancy> {
        CategoryService::new(repo, occupancy)
    }

    #[tokio::test]
    async fn catalog_create_child_sets_ruta() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let pernos = service
            .create(NewCategoria {
                nombre: "Pernos".into(),
                slug: None,
                padre: Some(fijaciones.id),
            })
            .await
            .expect("pernos");
        assert_eq!(pernos.ruta, vec!["Fijaciones", "Pernos"]);
    }

    #[tokio::test]
    async fn catalog_rename_keeps_padre() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let updated = service
            .rename(fijaciones.id.clone(), "Tornillería".into())
            .await
            .expect("rename");
        assert_eq!(updated.nombre, "Tornillería");
        assert_eq!(updated.padre, None);
    }

    #[tokio::test]
    async fn catalog_move_updates_padre() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let herramientas = service
            .create(NewCategoria {
                nombre: "Herramientas".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("herramientas");
        let pernos = service
            .create(NewCategoria {
                nombre: "Pernos".into(),
                slug: None,
                padre: Some(fijaciones.id),
            })
            .await
            .expect("pernos");
        let moved = service
            .move_category(pernos.id, Some(herramientas.id.clone()))
            .await
            .expect("move");
        assert_eq!(moved.padre.as_ref(), Some(&herramientas.id));
    }

    #[tokio::test]
    async fn catalog_deactivate_leaf_sets_inactive() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let pernos = service
            .create(NewCategoria {
                nombre: "Pernos".into(),
                slug: None,
                padre: Some(fijaciones.id),
            })
            .await
            .expect("pernos");
        let deactivated = service.deactivate(pernos.id).await.expect("deactivate");
        assert!(!deactivated.activo);
    }

    #[tokio::test]
    async fn catalog_reactivate_sets_active() {
        let repo = InMemoryCategoryRepo::new();
        let service = svc(repo, FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let pernos = service
            .create(NewCategoria {
                nombre: "Pernos".into(),
                slug: None,
                padre: Some(fijaciones.id),
            })
            .await
            .expect("pernos");
        let pernos_id = pernos.id.clone();
        service
            .deactivate(pernos_id.clone())
            .await
            .expect("deactivate");
        let reactivated = service.reactivate(pernos_id).await.expect("reactivate");
        assert!(reactivated.activo);
    }

    #[tokio::test]
    async fn catalog_create_duplicate_slug_is_duplicate_slug() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let err = service
            .create(NewCategoria {
                nombre: "Otra".into(),
                slug: Some(Slug::parse("fijaciones").expect("slug")),
                padre: None,
            })
            .await
            .expect_err("duplicate");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::DuplicateSlug
            }
        ));
        let _ = fijaciones;
    }

    #[tokio::test]
    async fn catalog_move_under_descendant_is_category_cycle() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let pernos = service
            .create(NewCategoria {
                nombre: "Pernos".into(),
                slug: None,
                padre: Some(fijaciones.id.clone()),
            })
            .await
            .expect("pernos");
        let err = service
            .move_category(fijaciones.id, Some(pernos.id))
            .await
            .expect_err("cycle");
        assert_eq!(err.code(), DomainCode::CategoryCycle);
    }

    #[tokio::test]
    async fn catalog_rename_unknown_id_is_not_found() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let err = service
            .rename(Id::generate(), "X".into())
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_deactivate_with_active_child_is_conflict() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let _pernos = service
            .create(NewCategoria {
                nombre: "Pernos".into(),
                slug: None,
                padre: Some(fijaciones.id.clone()),
            })
            .await
            .expect("pernos");
        let err = service
            .deactivate(fijaciones.id.clone())
            .await
            .expect_err("conflict");
        assert_eq!(err.code(), DomainCode::Conflict);
        let still = service.get(fijaciones.id).await.expect("get");
        assert!(still.activo);
    }

    #[tokio::test]
    async fn catalog_deactivate_with_active_product_is_conflict() {
        let repo = InMemoryCategoryRepo::new();
        let occupancy = FakeOccupancy::new();
        let service = svc(repo, occupancy);
        let fijaciones = service
            .create(NewCategoria {
                nombre: "Fijaciones".into(),
                slug: None,
                padre: None,
            })
            .await
            .expect("fijaciones");
        let pernos = service
            .create(NewCategoria {
                nombre: "Pernos".into(),
                slug: None,
                padre: Some(fijaciones.id),
            })
            .await
            .expect("pernos");
        service
            .occupancy
            .set_active_products(pernos.id.clone(), true);
        let err = service
            .deactivate(pernos.id.clone())
            .await
            .expect_err("conflict");
        assert_eq!(err.code(), DomainCode::Conflict);
        let still = service.get(pernos.id).await.expect("get");
        assert!(still.activo);
    }

    #[tokio::test]
    async fn catalog_deactivate_unknown_id_is_not_found() {
        let service = svc(InMemoryCategoryRepo::new(), FakeOccupancy::new());
        let err = service
            .deactivate(Id::generate())
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }
}
