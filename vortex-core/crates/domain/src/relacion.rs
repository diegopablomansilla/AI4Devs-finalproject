//! Product relation entity, type rules and `RelationRepo` port (T-04-02).

use std::future::Future;

use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::id::Id;
use crate::producto::Producto;
use crate::timestamp::Timestamp;

const MAX_NOTA_LEN: usize = 500;

/// Shared ULID for a bidirectional pair (marker until edges are persisted in T-04-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrupoBidireccional;

/// Relation type (model 4.6.1 — Spanish wire / table names).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoRelacion {
    SimilarA,
    SustituiblePor,
    Complementa,
}

impl TipoRelacion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SimilarA => "similar_a",
            Self::SustituiblePor => "sustituible_por",
            Self::Complementa => "complementa",
        }
    }

    pub fn parse_wire(s: &str) -> Result<Self, DomainError> {
        match s {
            "similar_a" => Ok(Self::SimilarA),
            "sustituible_por" => Ok(Self::SustituiblePor),
            "complementa" => Ok(Self::Complementa),
            _ => Err(field_invalid_input("tipo", "invalid tipo relacion")),
        }
    }
}

pub fn default_bidireccional(tipo: TipoRelacion) -> bool {
    matches!(tipo, TipoRelacion::SimilarA)
}

pub fn resolve_bidireccional(tipo: TipoRelacion, flag: Option<bool>) -> bool {
    flag.unwrap_or_else(|| default_bidireccional(tipo))
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelacionDraft {
    pub id: Id<Relacion>,
    pub origen: Id<Producto>,
    pub destino: Id<Producto>,
    pub tipo: TipoRelacion,
    pub prioridad: i32,
    pub nota: Option<String>,
    pub grupo_bidireccional: Option<Id<GrupoBidireccional>>,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Relacion {
    pub id: Id<Relacion>,
    pub origen: Id<Producto>,
    pub destino: Id<Producto>,
    pub tipo: TipoRelacion,
    pub prioridad: i32,
    pub nota: Option<String>,
    pub grupo_bidireccional: Option<Id<GrupoBidireccional>>,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

impl Relacion {
    pub fn try_new(draft: RelacionDraft) -> Result<Self, DomainError> {
        let RelacionDraft {
            id,
            origen,
            destino,
            tipo,
            prioridad,
            nota,
            grupo_bidireccional,
            creado_en,
            actualizado_en,
        } = draft;

        if origen == destino {
            return Err(DomainError::invalid_input(
                InvalidInputCode::SelfRelation,
                vec![
                    FieldError::try_new("destino", "product cannot relate to itself")
                        .expect("field"),
                ],
            ));
        }
        validate_prioridad(prioridad)?;
        let nota = normalize_nota(nota)?;

        Ok(Self {
            id,
            origen,
            destino,
            tipo,
            prioridad,
            nota,
            grupo_bidireccional,
            creado_en,
            actualizado_en,
        })
    }
}

/// Input for `RelationRepo::create`.
#[derive(Debug, Clone, PartialEq)]
pub struct NewRelacion {
    pub origen: Id<Producto>,
    pub destino: Id<Producto>,
    pub tipo: TipoRelacion,
    pub prioridad: i32,
    pub nota: Option<String>,
    pub bidireccional: Option<bool>,
}

impl NewRelacion {
    pub fn resolved_bidireccional(&self) -> bool {
        resolve_bidireccional(self.tipo, self.bidireccional)
    }
}

/// Input for `RelationRepo::update`.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateRelacion {
    pub id: Id<Relacion>,
    pub tipo: TipoRelacion,
    pub prioridad: Option<i32>,
    pub nota: Option<Option<String>>,
}

pub trait RelationRepo {
    fn create(
        &self,
        input: NewRelacion,
    ) -> impl Future<Output = Result<Relacion, DomainError>> + Send;

    fn update(
        &self,
        input: UpdateRelacion,
    ) -> impl Future<Output = Result<Relacion, DomainError>> + Send;

    fn delete(
        &self,
        tipo: TipoRelacion,
        id: Id<Relacion>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn list_for_product(
        &self,
        producto: Id<Producto>,
    ) -> impl Future<Output = Result<Vec<Relacion>, DomainError>> + Send;
}

fn validate_prioridad(prioridad: i32) -> Result<(), DomainError> {
    if prioridad < 1 {
        return Err(field_invalid_input(
            "prioridad",
            "prioridad must be at least 1",
        ));
    }
    Ok(())
}

fn normalize_nota(nota: Option<String>) -> Result<Option<String>, DomainError> {
    match nota {
        None => Ok(None),
        Some(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else if trimmed.len() > MAX_NOTA_LEN {
                Err(field_invalid_input(
                    "nota",
                    "nota must be at most 500 characters",
                ))
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
    }
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

    fn relacion_draft(
        origen: Id<Producto>,
        destino: Id<Producto>,
        tipo: TipoRelacion,
        prioridad: i32,
        nota: Option<&str>,
        grupo: Option<Id<GrupoBidireccional>>,
    ) -> RelacionDraft {
        RelacionDraft {
            id: Id::generate(),
            origen,
            destino,
            tipo,
            prioridad,
            nota: nota.map(str::to_string),
            grupo_bidireccional: grupo,
            creado_en: ts(),
            actualizado_en: ts(),
        }
    }

    fn invalid_fields(err: &DomainError) -> &[FieldError] {
        match err {
            DomainError::InvalidInput { errors, .. } => errors,
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn relacion_constructs_complement_perno_to_arandela() {
        let origen = Id::<Producto>::generate();
        let destino = Id::<Producto>::generate();
        let rel = Relacion::try_new(relacion_draft(
            origen,
            destino,
            TipoRelacion::Complementa,
            1,
            Some("Arandela plana M8"),
            None,
        ))
        .expect("complement");
        assert!(!rel.id.to_string().contains(':'));
        assert_eq!(rel.tipo, TipoRelacion::Complementa);
        assert_eq!(rel.prioridad, 1);
        assert_eq!(rel.nota.as_deref(), Some("Arandela plana M8"));
        assert!(rel.grupo_bidireccional.is_none());
    }

    #[test]
    fn relacion_constructs_similar_with_bidirectional_group() {
        let origen = Id::<Producto>::generate();
        let destino = Id::<Producto>::generate();
        let grupo = Id::<GrupoBidireccional>::generate();
        let rel = Relacion::try_new(relacion_draft(
            origen,
            destino,
            TipoRelacion::SimilarA,
            1,
            None,
            Some(grupo),
        ))
        .expect("similar");
        assert_eq!(rel.tipo, TipoRelacion::SimilarA);
        assert_eq!(rel.grupo_bidireccional, Some(grupo));
        assert!(!grupo.to_string().contains(':'));
    }

    #[test]
    fn default_bidireccional_similar_a_is_true() {
        assert!(default_bidireccional(TipoRelacion::SimilarA));
    }

    #[test]
    fn default_bidireccional_substitute_and_complement_are_false() {
        assert!(!default_bidireccional(TipoRelacion::SustituiblePor));
        assert!(!default_bidireccional(TipoRelacion::Complementa));
    }

    #[test]
    fn resolve_bidireccional_explicit_flag_overrides_default() {
        assert!(!resolve_bidireccional(TipoRelacion::SimilarA, Some(false)));
        assert!(resolve_bidireccional(TipoRelacion::Complementa, Some(true)));
    }

    #[test]
    fn tipo_relacion_rejects_english_or_unknown() {
        assert!(TipoRelacion::parse_wire("similar").is_err());
        assert!(TipoRelacion::parse_wire("substitute").is_err());
        assert!(TipoRelacion::parse_wire("complement").is_err());
        let err = TipoRelacion::parse_wire("similar").unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn relacion_rejects_self_relation() {
        let pid = Id::<Producto>::generate();
        let err = Relacion::try_new(relacion_draft(
            pid.clone(),
            pid,
            TipoRelacion::Complementa,
            1,
            None,
            None,
        ))
        .unwrap_err();
        assert_eq!(err.code(), DomainCode::SelfRelation);
        assert_eq!(invalid_fields(&err)[0].field, "destino");
    }

    #[test]
    fn relacion_accepts_distinct_origen_destino() {
        let origen = Id::<Producto>::generate();
        let destino = Id::<Producto>::generate();
        let rel = Relacion::try_new(relacion_draft(
            origen.clone(),
            destino.clone(),
            TipoRelacion::SustituiblePor,
            2,
            None,
            None,
        ))
        .expect("distinct");
        assert_eq!(rel.origen, origen);
        assert_eq!(rel.destino, destino);
    }

    #[test]
    fn relacion_accepts_prioridad_one_and_nota_500() {
        let nota = "a".repeat(500);
        let rel = Relacion::try_new(relacion_draft(
            Id::generate(),
            Id::generate(),
            TipoRelacion::Complementa,
            1,
            Some(&nota),
            None,
        ))
        .expect("nota 500");
        assert_eq!(rel.prioridad, 1);
        assert_eq!(rel.nota.as_ref().map(String::len), Some(500));
    }

    #[test]
    fn relacion_rejects_prioridad_below_one() {
        let err = Relacion::try_new(relacion_draft(
            Id::generate(),
            Id::generate(),
            TipoRelacion::Complementa,
            0,
            None,
            None,
        ))
        .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "prioridad");
    }

    #[test]
    fn relacion_rejects_nota_over_500() {
        let nota = "a".repeat(501);
        let err = Relacion::try_new(relacion_draft(
            Id::generate(),
            Id::generate(),
            TipoRelacion::Complementa,
            1,
            Some(&nota),
            None,
        ))
        .unwrap_err();
        assert_eq!(err.code(), DomainCode::InvalidInput);
        assert_eq!(invalid_fields(&err)[0].field, "nota");
    }

    struct DummyRelationRepo;

    impl RelationRepo for DummyRelationRepo {
        fn create(
            &self,
            _input: NewRelacion,
        ) -> impl Future<Output = Result<Relacion, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn update(
            &self,
            _input: UpdateRelacion,
        ) -> impl Future<Output = Result<Relacion, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn delete(
            &self,
            _tipo: TipoRelacion,
            _id: Id<Relacion>,
        ) -> impl Future<Output = Result<(), DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn list_for_product(
            &self,
            _producto: Id<Producto>,
        ) -> impl Future<Output = Result<Vec<Relacion>, DomainError>> + Send {
            async { Ok(vec![]) }
        }
    }

    #[test]
    fn relation_repo_trait_exposes_crud() {
        fn assert_repo<R: RelationRepo>(r: &R) {
            let _f1 = r.create(NewRelacion {
                origen: Id::generate(),
                destino: Id::generate(),
                tipo: TipoRelacion::SimilarA,
                prioridad: 1,
                nota: None,
                bidireccional: None,
            });
            let _f2 = r.update(UpdateRelacion {
                id: Id::generate(),
                tipo: TipoRelacion::SimilarA,
                prioridad: Some(2),
                nota: None,
            });
            let _f3 = r.delete(TipoRelacion::SimilarA, Id::generate());
            let _f4 = r.list_for_product(Id::generate());
        }
        assert_repo(&DummyRelationRepo);
    }

    #[test]
    fn new_relacion_omitted_bidireccional_uses_type_default() {
        let similar = NewRelacion {
            origen: Id::generate(),
            destino: Id::generate(),
            tipo: TipoRelacion::SimilarA,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        };
        assert!(similar.resolved_bidireccional());

        let complement = NewRelacion {
            origen: Id::generate(),
            destino: Id::generate(),
            tipo: TipoRelacion::Complementa,
            prioridad: 1,
            nota: None,
            bidireccional: None,
        };
        assert!(!complement.resolved_bidireccional());
    }

    #[test]
    fn relation_repo_not_declared_in_catalog() {
        let src = include_str!("../../catalog/src/lib.rs");
        assert!(!src.contains("RelationRepo"));
    }
}
