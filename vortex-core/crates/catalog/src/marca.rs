//! Brand use cases (T-02-09).

use std::future::Future;
use std::pin::Pin;

use domain::{BrandRepo, DomainError, Id, Marca, NewMarca, Slug, UpdateMarca};

pub struct BrandService<R> {
    pub repo: R,
}

impl<R> BrandService<R>
where
    R: BrandRepo,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(&self, input: NewMarca) -> Result<Marca, DomainError> {
        self.repo.create(input).await
    }

    pub async fn get(&self, id: Id<Marca>) -> Result<Marca, DomainError> {
        self.repo.get(id).await
    }

    pub async fn list_all(&self) -> Result<Vec<Marca>, DomainError> {
        self.repo.list_all().await
    }

    pub async fn rename(&self, id: Id<Marca>, nombre: String) -> Result<Marca, DomainError> {
        let _ = self.repo.get(id.clone()).await?;
        self.repo
            .update(UpdateMarca {
                id,
                nombre,
                slug: None,
            })
            .await
    }

    pub async fn update(&self, input: UpdateMarca) -> Result<Marca, DomainError> {
        self.repo.update(input).await
    }

    pub async fn update_slug(&self, id: Id<Marca>, slug: Slug) -> Result<Marca, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        self.repo
            .update(UpdateMarca {
                id,
                nombre: current.nombre,
                slug: Some(slug),
            })
            .await
    }

    pub async fn deactivate(&self, id: Id<Marca>) -> Result<Marca, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        if !current.activo {
            return Ok(current);
        }
        self.repo.set_activo(id, false).await
    }

    pub async fn reactivate(&self, id: Id<Marca>) -> Result<Marca, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        if current.activo {
            return Ok(current);
        }
        self.repo.set_activo(id, true).await
    }
}

/// Object-safe facade for HTTP injection (`Arc<dyn BrandGateway>`).
pub trait BrandGateway: Send + Sync {
    fn create(
        &self,
        input: NewMarca,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>>;

    fn get(
        &self,
        id: Id<Marca>,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>>;

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Marca>, DomainError>> + Send + '_>>;

    fn update(
        &self,
        input: UpdateMarca,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>>;

    fn rename(
        &self,
        id: Id<Marca>,
        nombre: String,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>>;

    fn deactivate(
        &self,
        id: Id<Marca>,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>>;

    fn reactivate(
        &self,
        id: Id<Marca>,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>>;
}

impl<R> BrandGateway for BrandService<R>
where
    R: BrandRepo + Send + Sync,
{
    fn create(
        &self,
        input: NewMarca,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>> {
        Box::pin(BrandService::create(self, input))
    }

    fn get(
        &self,
        id: Id<Marca>,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>> {
        Box::pin(BrandService::get(self, id))
    }

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Marca>, DomainError>> + Send + '_>> {
        Box::pin(BrandService::list_all(self))
    }

    fn update(
        &self,
        input: UpdateMarca,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>> {
        Box::pin(BrandService::update(self, input))
    }

    fn rename(
        &self,
        id: Id<Marca>,
        nombre: String,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>> {
        Box::pin(BrandService::rename(self, id, nombre))
    }

    fn deactivate(
        &self,
        id: Id<Marca>,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>> {
        Box::pin(BrandService::deactivate(self, id))
    }

    fn reactivate(
        &self,
        id: Id<Marca>,
    ) -> Pin<Box<dyn Future<Output = Result<Marca, DomainError>> + Send + '_>> {
        Box::pin(BrandService::reactivate(self, id))
    }
}

#[cfg(test)]
pub mod testing {
    use super::*;
    use domain::{BrandRepo, ConflictCode, DomainError, InvalidInputCode, MarcaDraft, Timestamp};
    use std::collections::HashMap;
    use std::sync::Mutex;

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    pub struct InMemoryBrandRepo {
        inner: Mutex<HashMap<String, Marca>>,
    }

    impl InMemoryBrandRepo {
        pub fn new() -> Self {
            Self {
                inner: Mutex::new(HashMap::new()),
            }
        }

        fn slug_taken(
            map: &HashMap<String, Marca>,
            slug: &str,
            except: Option<&Id<Marca>>,
        ) -> bool {
            map.values().any(|m| {
                if except.is_some_and(|e| e == &m.id) {
                    return false;
                }
                m.slug.as_str() == slug
            })
        }
    }

    impl BrandRepo for InMemoryBrandRepo {
        fn create(
            &self,
            input: NewMarca,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
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

        fn get(&self, id: Id<Marca>) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            async move {
                self.inner
                    .lock()
                    .expect("lock")
                    .get(&id.to_string())
                    .cloned()
                    .ok_or(DomainError::not_found())
            }
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<Marca>, DomainError>> + Send {
            async move { Ok(self.inner.lock().expect("lock").values().cloned().collect()) }
        }

        fn update(
            &self,
            input: UpdateMarca,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
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
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
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
}

#[cfg(test)]
mod tests {
    use super::testing::InMemoryBrandRepo;
    use super::*;
    use domain::{ConflictCode, DomainCode, DomainError, Slug};

    fn svc(repo: InMemoryBrandRepo) -> BrandService<InMemoryBrandRepo> {
        BrandService::new(repo)
    }

    #[tokio::test]
    async fn catalog_create_fixser_sets_slug() {
        let service = svc(InMemoryBrandRepo::new());
        let marca = service
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("fixser");
        assert_eq!(marca.nombre, "Fixser");
        assert_eq!(marca.slug.as_str(), "fixser");
        assert!(marca.activo);
    }

    #[tokio::test]
    async fn catalog_rename_updates_nombre() {
        let service = svc(InMemoryBrandRepo::new());
        let fixser = service
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("fixser");
        let updated = service
            .rename(fixser.id.clone(), "Fixser Pro".into())
            .await
            .expect("rename");
        assert_eq!(updated.nombre, "Fixser Pro");
        assert_eq!(updated.slug.as_str(), "fixser-pro");
    }

    #[tokio::test]
    async fn catalog_update_slug() {
        let service = svc(InMemoryBrandRepo::new());
        let fixser = service
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("fixser");
        let slug = Slug::parse("fixser-pro").expect("slug");
        let updated = service
            .update_slug(fixser.id.clone(), slug)
            .await
            .expect("slug update");
        assert_eq!(updated.nombre, "Fixser");
        assert_eq!(updated.slug.as_str(), "fixser-pro");
    }

    #[tokio::test]
    async fn catalog_deactivate_sets_inactive() {
        let service = svc(InMemoryBrandRepo::new());
        let fixser = service
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("fixser");
        let updated = service
            .deactivate(fixser.id.clone())
            .await
            .expect("deactivate");
        assert!(!updated.activo);
    }

    #[tokio::test]
    async fn catalog_deactivate_with_associated_products_succeeds() {
        let service = svc(InMemoryBrandRepo::new());
        let fixser = service
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("fixser");
        let updated = service.deactivate(fixser.id).await.expect("deactivate");
        assert!(!updated.activo);
    }

    #[tokio::test]
    async fn catalog_reactivate_sets_active() {
        let service = svc(InMemoryBrandRepo::new());
        let fixser = service
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("fixser");
        service
            .deactivate(fixser.id.clone())
            .await
            .expect("deactivate");
        let updated = service
            .reactivate(fixser.id.clone())
            .await
            .expect("reactivate");
        assert!(updated.activo);
    }

    #[tokio::test]
    async fn catalog_create_duplicate_slug_is_duplicate_slug() {
        let service = svc(InMemoryBrandRepo::new());
        let slug = Slug::parse("fixser").expect("slug");
        service
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: Some(slug.clone()),
            })
            .await
            .expect("first");
        let err = service
            .create(NewMarca {
                nombre: "Otra".into(),
                slug: Some(slug),
            })
            .await
            .expect_err("duplicate");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::DuplicateSlug
            }
        ));
    }

    #[tokio::test]
    async fn catalog_create_blank_nombre_is_invalid_input() {
        let service = svc(InMemoryBrandRepo::new());
        let err = service
            .create(NewMarca {
                nombre: "   ".into(),
                slug: None,
            })
            .await
            .expect_err("blank");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[tokio::test]
    async fn catalog_rename_unknown_id_is_not_found() {
        let service = svc(InMemoryBrandRepo::new());
        let err = service
            .rename(Id::generate(), "X".into())
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_deactivate_unknown_id_is_not_found() {
        let service = svc(InMemoryBrandRepo::new());
        let err = service
            .deactivate(Id::generate())
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn brand_gateway_is_object_safe() {
        let gateway: std::sync::Arc<dyn BrandGateway> =
            std::sync::Arc::new(svc(InMemoryBrandRepo::new()));
        let marca = gateway
            .create(NewMarca {
                nombre: "Fixser".into(),
                slug: None,
            })
            .await
            .expect("create");
        assert_eq!(marca.nombre, "Fixser");
    }
}
