//! Category entity, slug rules, tree depth/cycle checks and `CategoryRepo` port (T-02-02).

use std::fmt;
use std::future::Future;
use std::str::FromStr;

use crate::error::{DomainError, FieldError, InvalidInputCode};
use crate::id::Id;
use crate::timestamp::Timestamp;

/// Maximum category tree depth (root = 1), aligned with Meilisearch `categoria.lvl0`…`lvl3`.
pub const MAX_DEPTH: usize = 4;

const MAX_NOMBRE_LEN: usize = 100;

/// URL-safe category slug (`^[a-z0-9]+(-[a-z0-9]+)*$`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Accepts only a string that already matches the OpenAPI pattern (no transliteration).
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        if matches_wire_pattern(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(slug_field_error("slug", "invalid slug format"))
        }
    }

    /// Generates a slug from a display name (Spanish transliteration, OpenAPI pattern).
    pub fn from_nombre(nombre: &str) -> Result<Self, DomainError> {
        let trimmed = nombre.trim();
        if trimmed.is_empty() {
            return Err(slug_field_error("nombre", "nombre must not be empty"));
        }
        let mut out = String::new();
        let mut last_hyphen = false;
        for ch in trimmed.chars() {
            if let Some(lower) = map_spanish_char(ch) {
                out.push(lower);
                last_hyphen = false;
            } else if !last_hyphen && !out.is_empty() {
                out.push('-');
                last_hyphen = true;
            }
        }
        while out.ends_with('-') {
            out.pop();
        }
        if out.is_empty() || !matches_wire_pattern(&out) {
            return Err(slug_field_error("nombre", "cannot derive slug from nombre"));
        }
        Ok(Self(out))
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Slug {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Snapshot used to construct a validated `Categoria` (e.g. hydration from storage).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoriaDraft {
    pub id: Id<Categoria>,
    pub nombre: String,
    pub slug: Slug,
    pub padre: Option<Id<Categoria>>,
    pub ancestros: Vec<Id<Categoria>>,
    pub ruta: Vec<String>,
    pub activo: bool,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

/// Catalog category (model 4.4.1 — field names in Spanish).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Categoria {
    pub id: Id<Categoria>,
    pub nombre: String,
    pub slug: Slug,
    pub padre: Option<Id<Categoria>>,
    pub ancestros: Vec<Id<Categoria>>,
    pub ruta: Vec<String>,
    pub activo: bool,
    pub creado_en: Timestamp,
    pub actualizado_en: Timestamp,
}

impl Categoria {
    pub fn try_new(draft: CategoriaDraft) -> Result<Self, DomainError> {
        let CategoriaDraft {
            id,
            nombre,
            slug,
            padre,
            ancestros,
            ruta,
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
        let depth = category_depth(&ancestros);
        if depth > MAX_DEPTH {
            return Err(field_invalid_input(
                "padre",
                "category tree exceeds maximum depth",
            ));
        }
        if ruta.len() != depth {
            return Err(field_invalid_input(
                "ruta",
                "ruta length must match tree depth",
            ));
        }
        Ok(Self {
            id,
            nombre: nombre_trimmed.to_string(),
            slug,
            padre,
            ancestros,
            ruta,
            activo,
            creado_en,
            actualizado_en,
        })
    }

    pub fn depth(&self) -> usize {
        category_depth(&self.ancestros)
    }
}

pub fn category_depth(ancestros: &[Id<Categoria>]) -> usize {
    ancestros.len() + 1
}

pub fn would_form_cycle(
    node: &Id<Categoria>,
    new_parent: Option<&Id<Categoria>>,
    new_parent_ancestors: &[Id<Categoria>],
) -> bool {
    let Some(parent) = new_parent else {
        return false;
    };
    if parent == node {
        return true;
    }
    new_parent_ancestors.iter().any(|a| a == node)
}

pub fn reject_cycle(
    node: &Id<Categoria>,
    new_parent: Option<&Id<Categoria>>,
    new_parent_ancestors: &[Id<Categoria>],
) -> Result<(), DomainError> {
    if would_form_cycle(node, new_parent, new_parent_ancestors) {
        Err(DomainError::invalid_input(
            InvalidInputCode::CategoryCycle,
            vec![FieldError::try_new("padre", "category would form a cycle").expect("field")],
        ))
    } else {
        Ok(())
    }
}

/// Input for `CategoryRepo::create` (ancestros/ruta computed by the adapter in T-02-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCategoria {
    pub nombre: String,
    pub slug: Option<Slug>,
    pub padre: Option<Id<Categoria>>,
}

pub trait CategoryRepo {
    fn create(
        &self,
        input: NewCategoria,
    ) -> impl Future<Output = Result<Categoria, DomainError>> + Send;

    fn get(&self, id: Id<Categoria>)
    -> impl Future<Output = Result<Categoria, DomainError>> + Send;

    fn list_all(&self) -> impl Future<Output = Result<Vec<Categoria>, DomainError>> + Send;
}

fn map_spanish_char(ch: char) -> Option<char> {
    let lower = match ch {
        'Á' => 'a',
        'É' => 'e',
        'Í' => 'i',
        'Ó' => 'o',
        'Ú' => 'u',
        'Ü' => 'u',
        'Ñ' => 'n',
        'á' => 'a',
        'é' => 'e',
        'í' => 'i',
        'ó' => 'o',
        'ú' => 'u',
        'ü' => 'u',
        'ñ' => 'n',
        c if c.is_ascii_alphabetic() => c.to_ascii_lowercase(),
        c if c.is_ascii_digit() => c,
        _ => return None,
    };
    Some(lower)
}

fn matches_wire_pattern(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let bytes = s.as_bytes();
    if !bytes[0].is_ascii_digit() && !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let mut prev_hyphen = false;
    for &b in bytes.iter().skip(1) {
        if b == b'-' {
            if prev_hyphen {
                return false;
            }
            prev_hyphen = true;
            continue;
        }
        prev_hyphen = false;
        if !b.is_ascii_digit() && !b.is_ascii_lowercase() {
            return false;
        }
    }
    !bytes.ends_with(b"-")
}

fn field_invalid_input(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

fn slug_field_error(field: &str, message: &str) -> DomainError {
    DomainError::invalid_input(
        InvalidInputCode::InvalidInput,
        vec![FieldError::try_new(field, message).expect("field")],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DomainCode, InvalidInputCode, UnavailableCode};

    const TS: &str = "2026-09-24T21:33:00Z";

    fn ts() -> Timestamp {
        TS.parse().expect("timestamp")
    }

    #[test]
    fn slug_from_nombre_ascii() {
        let slug = Slug::from_nombre("Fijaciones").expect("slug");
        assert_eq!(slug.as_str(), "fijaciones");
    }

    #[test]
    fn slug_from_nombre_spanish_accents() {
        assert_eq!(
            Slug::from_nombre("Tornillería").expect("slug").as_str(),
            "tornilleria"
        );
        assert_eq!(Slug::from_nombre("Caños").expect("slug").as_str(), "canos");
    }

    #[test]
    fn slug_from_nombre_spaces_and_code() {
        let slug = Slug::from_nombre("  Pernos M8 x 40  ").expect("slug");
        assert_eq!(slug.as_str(), "pernos-m8-x-40");
    }

    #[test]
    fn slug_from_nombre_rejects_empty_or_punctuation() {
        for name in ["", "   ", "???"] {
            let err = Slug::from_nombre(name).expect_err("invalid");
            assert_eq!(err.code(), DomainCode::InvalidInput);
        }
    }

    #[test]
    fn slug_rejects_explicit_off_pattern() {
        for bad in ["Tornillería", "Pernos_M8", "-abc-"] {
            let err = Slug::parse(bad).expect_err("invalid");
            assert_eq!(err.code(), DomainCode::InvalidInput);
        }
    }

    #[test]
    fn categoria_root_constructs() {
        let id = Id::<Categoria>::generate();
        let slug = Slug::from_nombre("Fijaciones").unwrap();
        let cat = Categoria::try_new(CategoriaDraft {
            id,
            nombre: "Fijaciones".into(),
            slug,
            padre: None,
            ancestros: vec![],
            ruta: vec!["Fijaciones".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("categoria");
        assert!(!cat.id.to_string().contains(':'));
        assert_eq!(cat.nombre, "Fijaciones");
        assert!(cat.padre.is_none());
        assert!(cat.ancestros.is_empty());
    }

    #[test]
    fn categoria_child_constructs_with_depth_2() {
        let parent = Id::<Categoria>::generate();
        let id = Id::<Categoria>::generate();
        let slug = Slug::from_nombre("Pernos").unwrap();
        let cat = Categoria::try_new(CategoriaDraft {
            id,
            nombre: "Pernos".into(),
            slug,
            padre: Some(parent.clone()),
            ancestros: vec![parent],
            ruta: vec!["Fijaciones".into(), "Pernos".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect("categoria");
        assert_eq!(cat.depth(), 2);
    }

    #[test]
    fn categoria_rejects_blank_nombre() {
        let id = Id::<Categoria>::generate();
        let slug = Slug::from_nombre("x").unwrap();
        let err = Categoria::try_new(CategoriaDraft {
            id,
            nombre: "   ".into(),
            slug,
            padre: None,
            ancestros: vec![],
            ruta: vec!["x".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect_err("blank");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn categoria_root_depth_is_one() {
        assert_eq!(category_depth(&[]), 1);
    }

    #[test]
    fn categoria_allows_depth_four() {
        let a = Id::<Categoria>::generate();
        let b = Id::<Categoria>::generate();
        let c = Id::<Categoria>::generate();
        let id = Id::<Categoria>::generate();
        let slug = Slug::from_nombre("n4").unwrap();
        let cat = Categoria::try_new(CategoriaDraft {
            id,
            nombre: "n4".into(),
            slug,
            padre: Some(c.clone()),
            ancestros: vec![a, b, c],
            ruta: vec!["l0".into(), "l1".into(), "l2".into(), "l4".into()],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        });
        assert!(cat.is_ok());
        assert_eq!(cat.unwrap().depth(), 4);
    }

    #[test]
    fn categoria_rejects_depth_five() {
        let ids: Vec<_> = (0..4).map(|_| Id::<Categoria>::generate()).collect();
        let id = Id::<Categoria>::generate();
        let slug = Slug::from_nombre("deep").unwrap();
        let err = Categoria::try_new(CategoriaDraft {
            id,
            nombre: "deep".into(),
            slug,
            padre: Some(ids.last().unwrap().clone()),
            ancestros: ids,
            ruta: vec!["a".into(); 5],
            activo: true,
            creado_en: ts(),
            actualizado_en: ts(),
        })
        .expect_err("depth");
        assert_eq!(err.code(), DomainCode::InvalidInput);
    }

    #[test]
    fn cycle_valid_parent_is_false() {
        let node = Id::<Categoria>::generate();
        let parent = Id::<Categoria>::generate();
        assert!(!would_form_cycle(&node, Some(&parent), &[]));
    }

    #[test]
    fn cycle_move_to_root_is_false() {
        let node = Id::<Categoria>::generate();
        let other = Id::<Categoria>::generate();
        assert!(!would_form_cycle(&node, None, &[other]));
    }

    #[test]
    fn cycle_self_parent_is_category_cycle() {
        let node = Id::<Categoria>::generate();
        assert!(would_form_cycle(&node, Some(&node), &[]));
        let err = reject_cycle(&node, Some(&node), &[]).expect_err("cycle");
        match err {
            DomainError::InvalidInput {
                code: InvalidInputCode::CategoryCycle,
                ..
            } => {}
            _ => panic!("expected category_cycle"),
        }
        assert_eq!(err.code(), DomainCode::CategoryCycle);
    }

    #[test]
    fn cycle_move_under_descendant_is_category_cycle() {
        let fijaciones = Id::<Categoria>::generate();
        let pernos = Id::<Categoria>::generate();
        let ancestros = vec![fijaciones.clone()];
        assert!(would_form_cycle(&fijaciones, Some(&pernos), &ancestros));
        let err = reject_cycle(&fijaciones, Some(&pernos), &ancestros).expect_err("cycle");
        assert_eq!(err.code(), DomainCode::CategoryCycle);
    }

    struct DummyCategoryRepo;

    impl CategoryRepo for DummyCategoryRepo {
        fn create(
            &self,
            _input: NewCategoria,
        ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
            async { Err(DomainError::unavailable(UnavailableCode::Unavailable)) }
        }

        fn get(
            &self,
            _id: Id<Categoria>,
        ) -> impl Future<Output = Result<Categoria, DomainError>> + Send {
            async { Err(DomainError::not_found()) }
        }

        fn list_all(&self) -> impl Future<Output = Result<Vec<Categoria>, DomainError>> + Send {
            async { Ok(Vec::new()) }
        }
    }

    #[test]
    fn category_repo_trait_exposes_create_get_list_all() {
        fn assert_repo<R: CategoryRepo>(_r: &R) {}
        assert_repo(&DummyCategoryRepo);
    }
}
