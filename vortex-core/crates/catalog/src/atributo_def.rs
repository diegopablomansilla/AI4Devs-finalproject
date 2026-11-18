//! Attribute definition use cases (T-02-12).

use std::future::Future;
use std::pin::Pin;

use domain::{
    AtributoDef, AttributeDefRepo, AttributeUsage, ConflictCode, DomainError, Id, NewAtributoDef,
    UpdateAtributoDef, valor_compatible_con,
};

/// No products use any attribute (tests and default until occupancy is wired).
#[derive(Debug, Default, Clone, Copy)]
pub struct EmptyAttributeUsage;

impl AttributeUsage for EmptyAttributeUsage {
    async fn valores(
        &self,
        _id: Id<AtributoDef>,
    ) -> Result<Vec<domain::ValorAtributo>, DomainError> {
        Ok(Vec::new())
    }
}

pub struct AttributeDefService<R, U> {
    pub repo: R,
    pub usage: U,
}

impl<R, U> AttributeDefService<R, U>
where
    R: AttributeDefRepo,
    U: AttributeUsage,
{
    pub fn new(repo: R, usage: U) -> Self {
        Self { repo, usage }
    }

    pub async fn create(&self, input: NewAtributoDef) -> Result<AtributoDef, DomainError> {
        self.repo.create(input).await
    }

    pub async fn get(&self, id: Id<AtributoDef>) -> Result<AtributoDef, DomainError> {
        self.repo.get(id).await
    }

    pub async fn list_all(&self) -> Result<Vec<AtributoDef>, DomainError> {
        self.repo.list_all().await
    }

    pub async fn update(&self, input: UpdateAtributoDef) -> Result<AtributoDef, DomainError> {
        input.validate()?;
        let current = self.repo.get(input.id.clone()).await?;
        let valores = self.usage.valores(input.id.clone()).await?;
        if !valores.is_empty() {
            if input.clave != current.clave {
                return Err(DomainError::conflict(ConflictCode::AttributeInUse));
            }
            let tipo_changed = input.tipo != current.tipo;
            let opciones_changed = input.opciones != current.opciones;
            if tipo_changed || opciones_changed {
                let compatible = valores
                    .iter()
                    .all(|v| valor_compatible_con(input.tipo, &input.opciones, v));
                if !compatible {
                    return Err(DomainError::conflict(ConflictCode::AttributeInUse));
                }
            }
        }
        self.repo.update(input).await
    }

    pub async fn deactivate(&self, id: Id<AtributoDef>) -> Result<AtributoDef, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        if !current.activo {
            return Ok(current);
        }
        self.repo.set_activo(id, false).await
    }

    pub async fn reactivate(&self, id: Id<AtributoDef>) -> Result<AtributoDef, DomainError> {
        let current = self.repo.get(id.clone()).await?;
        if current.activo {
            return Ok(current);
        }
        self.repo.set_activo(id, true).await
    }
}

/// Object-safe facade for HTTP injection (`Arc<dyn AttributeDefGateway>`).
pub trait AttributeDefGateway: Send + Sync {
    fn create(
        &self,
        input: NewAtributoDef,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>>;

    fn get(
        &self,
        id: Id<AtributoDef>,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>>;

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send + '_>>;

    fn update(
        &self,
        input: UpdateAtributoDef,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>>;

    fn deactivate(
        &self,
        id: Id<AtributoDef>,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>>;

    fn reactivate(
        &self,
        id: Id<AtributoDef>,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>>;
}

impl<R, U> AttributeDefGateway for AttributeDefService<R, U>
where
    R: AttributeDefRepo + Send + Sync,
    U: AttributeUsage + Send + Sync,
{
    fn create(
        &self,
        input: NewAtributoDef,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>> {
        Box::pin(AttributeDefService::create(self, input))
    }

    fn get(
        &self,
        id: Id<AtributoDef>,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>> {
        Box::pin(AttributeDefService::get(self, id))
    }

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send + '_>> {
        Box::pin(AttributeDefService::list_all(self))
    }

    fn update(
        &self,
        input: UpdateAtributoDef,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>> {
        Box::pin(AttributeDefService::update(self, input))
    }

    fn deactivate(
        &self,
        id: Id<AtributoDef>,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>> {
        Box::pin(AttributeDefService::deactivate(self, id))
    }

    fn reactivate(
        &self,
        id: Id<AtributoDef>,
    ) -> Pin<Box<dyn Future<Output = Result<AtributoDef, DomainError>> + Send + '_>> {
        Box::pin(AttributeDefService::reactivate(self, id))
    }
}

#[cfg(test)]
pub mod testing {
    use super::*;
    use domain::{AtributoDefDraft, ConflictCode, DomainError, Timestamp};
    use std::collections::HashMap;
    use std::sync::Mutex;

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    pub struct InMemoryAttributeDefRepo {
        inner: Mutex<HashMap<String, AtributoDef>>,
    }

    impl InMemoryAttributeDefRepo {
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

    impl AttributeDefRepo for InMemoryAttributeDefRepo {
        fn create(
            &self,
            input: NewAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
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
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
            async move {
                self.inner
                    .lock()
                    .expect("lock")
                    .get(&id.to_string())
                    .cloned()
                    .ok_or(DomainError::not_found())
            }
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<AtributoDef>, DomainError>> + Send {
            async move { Ok(self.inner.lock().expect("lock").values().cloned().collect()) }
        }

        fn update(
            &self,
            input: UpdateAtributoDef,
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
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
        ) -> impl Future<Output = Result<AtributoDef, DomainError>> + Send {
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

    pub struct InMemoryAttributeUsage {
        inner: Mutex<HashMap<String, Vec<domain::ValorAtributo>>>,
    }

    impl InMemoryAttributeUsage {
        pub fn new() -> Self {
            Self {
                inner: Mutex::new(HashMap::new()),
            }
        }

        pub fn set(&self, id: Id<AtributoDef>, valores: Vec<domain::ValorAtributo>) {
            self.inner
                .lock()
                .expect("lock")
                .insert(id.to_string(), valores);
        }
    }

    impl AttributeUsage for InMemoryAttributeUsage {
        fn valores(
            &self,
            id: Id<AtributoDef>,
        ) -> impl Future<Output = Result<Vec<domain::ValorAtributo>, DomainError>> + Send {
            async move {
                Ok(self
                    .inner
                    .lock()
                    .expect("lock")
                    .get(&id.to_string())
                    .cloned()
                    .unwrap_or_default())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{InMemoryAttributeDefRepo, InMemoryAttributeUsage};
    use super::*;
    use domain::{ConflictCode, DomainCode, DomainError, TipoAtributo, ValorAtributo};

    fn svc(
        repo: InMemoryAttributeDefRepo,
    ) -> AttributeDefService<InMemoryAttributeDefRepo, EmptyAttributeUsage> {
        AttributeDefService::new(repo, EmptyAttributeUsage)
    }

    fn svc_with_usage(
        repo: InMemoryAttributeDefRepo,
        usage: InMemoryAttributeUsage,
    ) -> AttributeDefService<InMemoryAttributeDefRepo, InMemoryAttributeUsage> {
        AttributeDefService::new(repo, usage)
    }

    fn new_grado() -> NewAtributoDef {
        NewAtributoDef {
            clave: "grado".into(),
            etiqueta: "Grado".into(),
            tipo: TipoAtributo::Enum,
            unidad: None,
            opciones: Some(vec!["2".into(), "5".into(), "8".into()]),
            facetable: true,
            buscable: true,
            orden: 0,
        }
    }

    #[tokio::test]
    async fn catalog_create_grado_enum_sets_options() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let def = service.create(new_grado()).await.expect("grado");
        assert_eq!(def.clave, "grado");
        assert_eq!(def.tipo, TipoAtributo::Enum);
        assert_eq!(def.opciones, Some(vec!["2".into(), "5".into(), "8".into()]));
        assert!(def.facetable);
        assert!(def.activo);
    }

    #[tokio::test]
    async fn catalog_create_medida_texto() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let def = service
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
        assert_eq!(def.tipo, TipoAtributo::Texto);
        assert_eq!(def.unidad.as_deref(), Some("mm"));
    }

    #[tokio::test]
    async fn catalog_update_etiqueta() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let grado = service.create(new_grado()).await.expect("grado");
        let current = service.get(grado.id.clone()).await.expect("get");
        let updated = service
            .update(UpdateAtributoDef {
                id: current.id.clone(),
                clave: current.clave.clone(),
                etiqueta: "Grado SAE".into(),
                tipo: current.tipo,
                unidad: current.unidad.clone(),
                opciones: current.opciones.clone(),
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
            })
            .await
            .expect("update");
        assert_eq!(updated.etiqueta, "Grado SAE");
    }

    #[tokio::test]
    async fn catalog_update_clave_unused_succeeds() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let medida = service
            .create(NewAtributoDef {
                clave: "medida".into(),
                etiqueta: "Medida".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
            })
            .await
            .expect("medida");
        let current = service.get(medida.id.clone()).await.expect("get");
        let updated = service
            .update(UpdateAtributoDef {
                id: current.id.clone(),
                clave: "medida_mm".into(),
                etiqueta: current.etiqueta.clone(),
                tipo: current.tipo,
                unidad: current.unidad.clone(),
                opciones: current.opciones.clone(),
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
            })
            .await
            .expect("update clave");
        assert_eq!(updated.clave, "medida_mm");
    }

    #[tokio::test]
    async fn catalog_update_clave_in_use_is_attribute_in_use() {
        let repo = InMemoryAttributeDefRepo::new();
        let usage = InMemoryAttributeUsage::new();
        let service = svc_with_usage(repo, usage);
        let medida = service
            .create(NewAtributoDef {
                clave: "medida".into(),
                etiqueta: "Medida".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
            })
            .await
            .expect("medida");
        service
            .usage
            .set(medida.id.clone(), vec![ValorAtributo::Texto("M8".into())]);
        let current = service.get(medida.id.clone()).await.expect("get");
        let err = service
            .update(UpdateAtributoDef {
                id: current.id.clone(),
                clave: "medida_mm".into(),
                etiqueta: current.etiqueta.clone(),
                tipo: current.tipo,
                unidad: current.unidad.clone(),
                opciones: current.opciones.clone(),
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
            })
            .await
            .expect_err("clave en uso");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::AttributeInUse
            }
        ));
    }

    #[tokio::test]
    async fn catalog_update_tipo_incompatible_in_use_is_attribute_in_use() {
        let repo = InMemoryAttributeDefRepo::new();
        let usage = InMemoryAttributeUsage::new();
        let service = svc_with_usage(repo, usage);
        let medida = service
            .create(NewAtributoDef {
                clave: "medida".into(),
                etiqueta: "Medida".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
            })
            .await
            .expect("medida");
        service
            .usage
            .set(medida.id.clone(), vec![ValorAtributo::Texto("M8".into())]);
        let current = service.get(medida.id.clone()).await.expect("get");
        let err = service
            .update(UpdateAtributoDef {
                id: current.id.clone(),
                clave: current.clave.clone(),
                etiqueta: current.etiqueta.clone(),
                tipo: TipoAtributo::Numero,
                unidad: current.unidad.clone(),
                opciones: None,
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
            })
            .await
            .expect_err("tipo incompatible");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::AttributeInUse
            }
        ));
    }

    #[tokio::test]
    async fn catalog_update_opciones_incompatible_in_use_is_attribute_in_use() {
        let repo = InMemoryAttributeDefRepo::new();
        let usage = InMemoryAttributeUsage::new();
        let service = svc_with_usage(repo, usage);
        let grado = service.create(new_grado()).await.expect("grado");
        service
            .usage
            .set(grado.id.clone(), vec![ValorAtributo::Texto("5".into())]);
        let current = service.get(grado.id.clone()).await.expect("get");
        let err = service
            .update(UpdateAtributoDef {
                id: current.id.clone(),
                clave: current.clave.clone(),
                etiqueta: current.etiqueta.clone(),
                tipo: current.tipo,
                unidad: current.unidad.clone(),
                opciones: Some(vec!["2".into(), "8".into()]),
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
            })
            .await
            .expect_err("opcion incompatible");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::AttributeInUse
            }
        ));
    }

    #[tokio::test]
    async fn catalog_update_etiqueta_in_use_succeeds() {
        let repo = InMemoryAttributeDefRepo::new();
        let usage = InMemoryAttributeUsage::new();
        let service = svc_with_usage(repo, usage);
        let medida = service
            .create(NewAtributoDef {
                clave: "medida".into(),
                etiqueta: "Medida".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
            })
            .await
            .expect("medida");
        service
            .usage
            .set(medida.id.clone(), vec![ValorAtributo::Texto("M8".into())]);
        let current = service.get(medida.id.clone()).await.expect("get");
        let updated = service
            .update(UpdateAtributoDef {
                id: current.id.clone(),
                clave: current.clave.clone(),
                etiqueta: "Medida métrica".into(),
                tipo: current.tipo,
                unidad: current.unidad.clone(),
                opciones: current.opciones.clone(),
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
            })
            .await
            .expect("etiqueta");
        assert_eq!(updated.etiqueta, "Medida métrica");
    }

    #[tokio::test]
    async fn catalog_update_enum_to_texto_in_use_succeeds() {
        let repo = InMemoryAttributeDefRepo::new();
        let usage = InMemoryAttributeUsage::new();
        let service = svc_with_usage(repo, usage);
        let grado = service.create(new_grado()).await.expect("grado");
        service
            .usage
            .set(grado.id.clone(), vec![ValorAtributo::Texto("5".into())]);
        let current = service.get(grado.id.clone()).await.expect("get");
        let updated = service
            .update(UpdateAtributoDef {
                id: current.id.clone(),
                clave: current.clave.clone(),
                etiqueta: current.etiqueta.clone(),
                tipo: TipoAtributo::Texto,
                unidad: current.unidad.clone(),
                opciones: None,
                facetable: current.facetable,
                buscable: current.buscable,
                orden: current.orden,
            })
            .await
            .expect("enum a texto");
        assert_eq!(updated.tipo, TipoAtributo::Texto);
    }

    #[tokio::test]
    async fn catalog_deactivate_in_use_succeeds() {
        let repo = InMemoryAttributeDefRepo::new();
        let usage = InMemoryAttributeUsage::new();
        let service = svc_with_usage(repo, usage);
        let medida = service
            .create(NewAtributoDef {
                clave: "medida".into(),
                etiqueta: "Medida".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
            })
            .await
            .expect("medida");
        service
            .usage
            .set(medida.id.clone(), vec![ValorAtributo::Texto("M8".into())]);
        let updated = service
            .deactivate(medida.id.clone())
            .await
            .expect("deactivate in use");
        assert!(!updated.activo);
    }

    #[tokio::test]
    async fn catalog_deactivate_sets_inactive() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let grado = service.create(new_grado()).await.expect("grado");
        let updated = service
            .deactivate(grado.id.clone())
            .await
            .expect("deactivate");
        assert!(!updated.activo);
    }

    #[tokio::test]
    async fn catalog_reactivate_sets_active() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let grado = service.create(new_grado()).await.expect("grado");
        service
            .deactivate(grado.id.clone())
            .await
            .expect("deactivate");
        let updated = service
            .reactivate(grado.id.clone())
            .await
            .expect("reactivate");
        assert!(updated.activo);
    }

    #[tokio::test]
    async fn catalog_create_duplicate_clave_is_conflict() {
        let service = svc(InMemoryAttributeDefRepo::new());
        service.create(new_grado()).await.expect("first");
        let err = service.create(new_grado()).await.expect_err("duplicate");
        assert!(matches!(
            err,
            DomainError::Conflict {
                code: ConflictCode::Conflict
            }
        ));
    }

    #[tokio::test]
    async fn catalog_create_enum_without_options_is_invalid_input() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let err = service
            .create(NewAtributoDef {
                clave: "grado".into(),
                etiqueta: "Grado".into(),
                tipo: TipoAtributo::Enum,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
            })
            .await
            .expect_err("enum sin opciones");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[tokio::test]
    async fn catalog_update_unknown_id_is_not_found() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let err = service
            .update(UpdateAtributoDef {
                id: Id::generate(),
                clave: "x".into(),
                etiqueta: "X".into(),
                tipo: TipoAtributo::Texto,
                unidad: None,
                opciones: None,
                facetable: false,
                buscable: true,
                orden: 0,
            })
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn catalog_deactivate_unknown_id_is_not_found() {
        let service = svc(InMemoryAttributeDefRepo::new());
        let err = service
            .deactivate(Id::generate())
            .await
            .expect_err("not found");
        assert_eq!(err.code(), DomainCode::NotFound);
    }

    #[tokio::test]
    async fn attribute_def_gateway_is_object_safe() {
        let gateway: std::sync::Arc<dyn AttributeDefGateway> = std::sync::Arc::new(
            AttributeDefService::new(InMemoryAttributeDefRepo::new(), EmptyAttributeUsage),
        );
        let def = gateway.create(new_grado()).await.expect("create");
        assert_eq!(def.clave, "grado");
    }
}
