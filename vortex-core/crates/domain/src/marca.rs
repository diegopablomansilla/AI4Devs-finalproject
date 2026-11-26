//! Brand entity and `BrandRepo` port (T-02-08).

use std::future::Future;

use crate::categoria::Slug;
use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::id::Id;
use crate::timestamp::Timestamp;

const MAX_NOMBRE_LEN: usize = 100;

/// Snapshot used to construct a validated `Marca` (e.g. hydration from storage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarcaDraft {
    pub id: Id<Marca>,
    pub nombre: String,
    pub slug: Slug,
    pub activo: bool,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

/// Catalog brand (model 4.4.2 — field names in Spanish).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marca {
    pub id: Id<Marca>,
    pub nombre: String,
    pub slug: Slug,
    pub activo: bool,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

impl Marca {
    pub fn try_new(draft: MarcaDraft) -> Result<Self, DomainError> {
        let MarcaDraft {
            id,
            nombre,
            slug,
            activo,
            creado_en,
            actualizado_en,
        } = draft;
        let nombre_trimmed = nombre.trim();
        if nombre_trimmed.is_empty() {
            return Err(field_invalid_input("nombre", "nombre must not be empty"));
        }
        if nombre_trimmed.len() > MAX_NOMBRE_LEN {
            return Err(field_invalid_input(
                "nombre",
                "nombre must be at most 100 characters",
            ));
        }
        Ok(Self {
            id,
            nombre: nombre_trimmed.to_string(),
            slug,
            activo,
            creado_en,
            actualizado_en,
        })
    }
}

/// Input for `BrandRepo::create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMarca {
    pub nombre: String,
    pub slug: Option<Slug>,
}

/// Input for `BrandRepo::update`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMarca {
    pub id: Id<Marca>,
    pub nombre: String,
    pub slug: Option<Slug>,
}

pub trait BrandRepo {
    fn create(&self, input: NewMarca) -> impl Future<Output = Result<Marca, DomainError>> + Send;

    fn get(&self, id: Id<Marca>) -> impl Future<Output = Result<Marca, DomainError>> + Send;

    fn list_all(&self) -> impl Future<Output = Result<Vec<Marca>, DomainError>> + Send;

    fn update(&self, input: UpdateMarca)
    -> impl Future<Output = Result<Marca, DomainError>> + Send;

    fn set_activo(
        &self,
        id: Id<Marca>,
        activo: bool,
    ) -> impl Future<Output = Result<Marca, DomainError>> + Send;
}

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DomainCode, UnavailableCode};

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    #[test]
    fn marca_constructs_fixser() {
        let id = Id::<Marca>::generate();
        let slug = Slug::from_nombre("Fixser").unwrap();
        let marca = Marca::try_new(MarcaDraft {
            id: id.clone(),
            nombre: "Fixser".into(),
            slug,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("marca");
        assert!(!id.to_string().contains(':'));
        assert_eq!(marca.nombre, "Fixser");
        assert!(marca.activo);
    }

    #[test]
    fn marca_rejects_blank_nombre() {
        let id = Id::<Marca>::generate();
        let slug = Slug::from_nombre("x").unwrap();
        let err = Marca::try_new(MarcaDraft {
            id,
            nombre: "   ".into(),
            slug,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect_err("blank");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn marca_rejects_nombre_too_long() {
        let id = Id::<Marca>::generate();
        let slug = Slug::from_nombre("x").unwrap();
        let long = "a".repeat(101);
        let err = Marca::try_new(MarcaDraft {
            id,
            nombre: long,
            slug,
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect_err("too long");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    struct DummyBrandRepo;

    impl BrandRepo for DummyBrandRepo {
        fn create(
            &self,
            _input: NewMarca,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn get(&self, _id: Id<Marca>) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            async { Err(DomainError::not_found()) }
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<Marca>, DomainError>> + Send {
            async { Ok(Vec::new()) }
        }

        fn update(
            &self,
            _input: UpdateMarca,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn set_activo(
            &self,
            _id: Id<Marca>,
            _activo: bool,
        ) -> impl Future<Output = Result<Marca, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }
    }

    #[test]
    fn brand_repo_trait_exposes_create_get_list_all_update_set_activo() {
        fn assert_repo<R: BrandRepo>(r: &R) {
            let _f1 = r.create(NewMarca {
                nombre: "x".into(),
                slug: None,
            });
            let _f2 = r.get(Id::generate());
            let _f3 = r.list_all();
            let _f4 = r.update(UpdateMarca {
                id: Id::generate(),
                nombre: "y".into(),
                slug: None,
            });
            let _f5 = r.set_activo(Id::generate(), false);
        }
        assert_repo(&DummyBrandRepo);
    }
}
